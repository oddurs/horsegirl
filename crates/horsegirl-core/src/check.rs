//! The checks: what the model says about a file that the parser cannot see.
//!
//! A thing exists from its first mention, so a typo makes a new thing with no
//! error from the parser. The checks read the model and report a typo, a thing
//! with no link, a part that the model leaves out of a group, a link that
//! breaks a `-x` rule, a guess, an unknown lens and a lens whose facts the
//! file does not state.

use crate::diagnostic::{Confidence, Diagnostic, Edit, Fix, Fixed, LineIndex, apply_safe};
use crate::lens::Need;
use crate::model::{Model, Thing, ThingId, Why, lower};
use crate::syntax::{Element, Node, NodeKind, TokenKind, parse};

/// Every lens that a `show` line can pick, as the language card lists them.
pub const LENSES: &[&str] = &["flow", "sequence", "lanes", "matrix", "rings", "seams"];

/// Every diagnostic for a file: the parser's and the checks', sorted by the
/// start of the span and then by code.
#[must_use]
pub fn check(source: &str) -> Vec<Diagnostic> {
    let parse = parse(source);
    let mut diagnostics = parse.diagnostics;
    diagnostics.extend(of_model(source, &lower(&parse.tree)));
    diagnostics.extend(paths(&parse.tree));
    diagnostics.extend(label_links(source, &parse.tree));
    sort(&mut diagnostics);
    diagnostics
}

/// The diagnostics of the checks alone, for a model lowered from `source`,
/// sorted as [`check`] sorts them.
#[must_use]
pub fn of_model(source: &str, model: &Model) -> Vec<Diagnostic> {
    let typos = typos(source, model);
    let mut diagnostics = orphans(model, &typos);
    diagnostics.extend(typos.into_iter().map(|(_, diagnostic)| diagnostic));
    diagnostics.extend(misplaced(model));
    diagnostics.extend(broken_rules(source, model));
    diagnostics.extend(guesses(model));
    diagnostics.extend(lenses(model));
    diagnostics.extend(needs(model));
    sort(&mut diagnostics);
    diagnostics
}

/// The most passes of `check` and `apply_safe` that `fix` makes. Each pass
/// that applies a fix removes a diagnostic, so a real file stops long before.
const FIX_PASSES: usize = 16;

/// `source` with every safe fix made, as `horsegirl fix` makes them.
///
/// One fix can reveal another: a fix that overlaps an earlier one is left out
/// of a pass, so this checks again until a pass applies none.
#[must_use]
pub fn fix_all(source: &str) -> Fixed {
    let mut fixed = Fixed {
        text: source.to_owned(),
        applied: 0,
    };
    for _ in 0..FIX_PASSES {
        let pass = apply_safe(&fixed.text, &check(&fixed.text));
        if pass.applied == 0 {
            break;
        }
        fixed.text = pass.text;
        fixed.applied += pass.applied;
    }
    fixed
}

fn sort(diagnostics: &mut [Diagnostic]) {
    // A stable sort, and every check pushes in model order, so equal keys
    // keep one order on every run.
    diagnostics.sort_by(|a, b| (a.span.start, a.code).cmp(&(b.span.start, b.code)));
}

/// E0412: a name written once that is one edit from a name written more
/// often. The name written more often is taken as the one the file means.
/// Two names written once each give nothing, because neither is more likely.
/// When more than one name written more often is one edit away, the check
/// cannot choose, so the error names each of them and carries no fix: a tool
/// applies a safe fix with no question.
fn typos(source: &str, model: &Model) -> Vec<(ThingId, Diagnostic)> {
    let mut found = Vec::new();
    for (id, thing) in model.things.iter().enumerate() {
        let [mention] = thing.mentions.as_slice() else {
            continue;
        };
        let near: Vec<&Thing> = model
            .things
            .iter()
            .filter(|known| {
                known.mentions.len() > 1
                    && distance(&thing.name, &known.name) == 1
                    && !siblings(&thing.name, &known.name)
            })
            .collect();
        let diagnostic = match near.as_slice() {
            [] => continue,
            [known] => {
                // The first mention is a way the file already writes the name,
                // so it parses, with its quotes where the name needs them.
                let written = &source[known.mentions[0].clone()];
                Diagnostic {
                    code: "E0412",
                    span: mention.clone(),
                    message: format!(
                        "`{}` is written once and is one edit from `{}`. A thing exists from its first mention, so this makes a new thing. Write `{}` if you mean that thing; if not, choose a name that differs by more than one character.",
                        thing.name, known.name, known.name
                    ),
                    fix: Some(Fix {
                        confidence: Confidence::Safe,
                        description: format!("Replace `{}` with `{}`.", thing.name, known.name),
                        edits: vec![Edit {
                            span: mention.clone(),
                            text: written.to_owned(),
                        }],
                    }),
                }
            }
            many => {
                let names: Vec<String> = many
                    .iter()
                    .map(|known| format!("`{}`", known.name))
                    .collect();
                Diagnostic {
                    code: "E0412",
                    span: mention.clone(),
                    message: format!(
                        "`{}` is written once and is one edit from each of {}. A thing exists from its first mention, so this makes a new thing. Write the one of those names that you mean; if you mean none of them, choose a name that differs by more than one character.",
                        thing.name,
                        names.join(" and ")
                    ),
                    fix: None,
                }
            }
        };
        found.push((id, diagnostic));
    }
    found
}

/// Two names that differ only in a final word that is a number or one letter
/// (`Service A` and `Service B`, `Deploy #1` and `Deploy #2`), or that are both
/// shorter than three characters, are a numbered or lettered series, not a typo.
fn siblings(a: &str, b: &str) -> bool {
    if a.chars().count() < 3 && b.chars().count() < 3 {
        return true;
    }
    let is_marker = |word: &str| {
        let digits = word.strip_prefix('#').unwrap_or(word);
        (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
            || word.chars().count() == 1
    };
    match (a.rsplit_once(' '), b.rsplit_once(' ')) {
        (Some((head_a, last_a)), Some((head_b, last_b))) => {
            head_a == head_b && is_marker(last_a) && is_marker(last_b)
        }
        _ => false,
    }
}

/// W0107: a thing that no link and no rule names. A group needs no link of
/// its own, because its parts carry them. A typo is reported once, as E0412.
fn orphans(model: &Model, typos: &[(ThingId, Diagnostic)]) -> Vec<Diagnostic> {
    let named = |id: ThingId| {
        let links = model.links.iter().map(|link| (link.from, link.to));
        let rules = model.rules.iter().map(|rule| (rule.from, rule.to));
        links.chain(rules).any(|(from, to)| from == id || to == id)
    };
    (0..model.things.len())
        .filter(|&id| !named(id))
        .filter(|&id| model.parts(id).is_empty())
        .filter(|&id| !typos.iter().any(|(typo, _)| *typo == id))
        .map(|id| {
            let thing = &model.things[id];
            Diagnostic {
                code: "W0107",
                span: thing.mentions[0].clone(),
                message: format!(
                    "`{}` has no link, so it stands alone in every picture. Link it to a thing with `->`, or remove it.",
                    thing.name
                ),
                fix: None,
            }
        })
        .collect()
}

/// E0420: a name in braces that the model does not place in their group: a
/// thing in a second group, a group inside itself, or a part in the braces
/// after several names. The fix removes the name where the line only names
/// things. It is suggested and never safe: the error is the only sign that
/// the writer meant two things or one, and removing a name can leave another
/// name written once, which E0412 then reads as a typo. Two such names next
/// to each other on one line would remove the same comma, so the second keeps
/// no fix.
fn misplaced(model: &Model) -> Vec<Diagnostic> {
    let name = |id: ThingId| &model.things[id].name;
    let mut removed_to = 0;
    let mut found = Vec::new();
    for misplaced in &model.misplaced {
        let thing = name(misplaced.thing);
        let group = name(misplaced.group);
        let message = match &misplaced.why {
            Why::Held(holder) => format!(
                "`{thing}` is in the group `{}` already, and a thing is in one group, so the braces of `{group}` do not place it. Remove `{thing}` from one of the two groups. To link `{thing}` from inside `{group}`, write the link outside the braces.",
                name(*holder)
            ),
            Why::Loop if misplaced.thing == misplaced.group => format!(
                "`{thing}` is in its own braces, and a group would hold itself. Remove `{thing}` from these braces."
            ),
            Why::Loop => format!(
                "`{group}` is inside `{thing}`, so placing `{thing}` in `{group}` would make the group `{thing}` hold itself. Remove `{thing}` from the braces of `{group}`."
            ),
            Why::Shared(groups) => {
                let groups: Vec<String> =
                    groups.iter().map(|&g| format!("`{}`", name(g))).collect();
                // The parent at the end of lowering, so that the message
                // states where the thing is and not where these braces aimed.
                let placed = match model.things[misplaced.thing].parent {
                    Some(parent) if parent == misplaced.group => {
                        format!("`{thing}` is in `{group}` only")
                    }
                    Some(parent) => format!("`{thing}` is in `{}`", name(parent)),
                    None => format!("`{thing}` is in no group"),
                };
                format!(
                    "`{thing}` is in the braces after {}, and a part belongs to one group, so {placed}. Write the part in the braces after one name. Braces after several names hold facts only.",
                    groups.join(" and ")
                )
            }
        };
        let fix = misplaced
            .removable
            .clone()
            .filter(|span| span.start >= removed_to)
            .map(|span| {
                removed_to = span.end;
                Fix {
                    confidence: Confidence::Suggested,
                    description: format!("Remove `{thing}` from the braces of `{group}`."),
                    edits: vec![Edit {
                        span,
                        text: String::new(),
                    }],
                }
            });
        found.push(Diagnostic {
            code: "E0420",
            span: misplaced.span.clone(),
            message,
            fix,
        });
    }
    found
}

/// E0500: a link that a `-x` rule forbids, in the same direction.
fn broken_rules(source: &str, model: &Model) -> Vec<Diagnostic> {
    let lines = LineIndex::new(source);
    let mut found = Vec::new();
    for link in &model.links {
        for rule in &model.rules {
            if (rule.from, rule.to) != (link.from, link.to) {
                continue;
            }
            let from = &model.things[link.from].name;
            let to = &model.things[link.to].name;
            let reason = rule
                .reason
                .as_ref()
                .map_or_else(String::new, |reason| format!(": {reason}"));
            found.push(Diagnostic {
                code: "E0500",
                span: link.span.clone(),
                message: format!(
                    "`{from} -> {to}` breaks the rule `{from} -x {to}{reason}` on line {}. Remove the link, or remove the rule if it no longer holds.",
                    lines.position(rule.span.start).line
                ),
                fix: None,
            });
        }
    }
    found
}

/// W0108: a name that ends in `/` followed by braces that hold one bare name,
/// as in `GET /users/{id}`. The parser reads a group `GET /users/` with the
/// part `id`, and cannot tell it from a path. The fix writes the whole text in
/// double quotes. It is only suggested, because the writer may mean a group.
/// This reads the tree, because the model keeps no span for the braces.
fn paths(tree: &Node<'_>) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    path_lines(tree, &mut found);
    found
}

fn path_lines(node: &Node<'_>, found: &mut Vec<Diagnostic>) {
    for inner in node.children.iter().filter_map(|element| match element {
        Element::Node(inner) => Some(inner),
        Element::Token(_) => None,
    }) {
        if inner.kind == NodeKind::Things
            && let Some(diagnostic) = path(inner)
        {
            found.push(diagnostic);
        }
        path_lines(inner, found);
    }
}

fn path(things: &Node<'_>) -> Option<Diagnostic> {
    let names = child_of(things, NodeKind::Names)?;
    let block = child_of(things, NodeKind::Block)?;
    let [Element::Node(name)] = names.children.as_slice() else {
        return None;
    };
    let name_text = name.to_string();
    if !name_text.ends_with('/') || name_text.starts_with('"') {
        return None;
    }
    // The braces hold one line, and that line is one bare name: no link, no
    // fact, no block and no second name.
    let lines: Vec<&Node<'_>> = block
        .children
        .iter()
        .filter_map(|element| match element {
            Element::Node(node) => Some(node),
            Element::Token(_) => None,
        })
        .collect();
    let [line] = lines.as_slice() else {
        return None;
    };
    let part = match (line.kind, child_of(line, NodeKind::Names)) {
        (NodeKind::Things, Some(names)) if child_of(line, NodeKind::Block).is_none() => {
            match names.children.as_slice() {
                [Element::Node(part)] if part.kind == NodeKind::Name => part,
                _ => return None,
            }
        }
        _ => return None,
    };
    if part.to_string().starts_with('"') {
        return None;
    }
    let part = part.to_string();
    let text = things.to_string();
    if text.contains('\n') || text.contains('"') {
        return None;
    }
    let start = first_start(things)?;
    let span = start..start + text.len();
    Some(Diagnostic {
        code: "W0108",
        span: span.clone(),
        message: format!(
            "`{text}` reads as the group `{name_text}` with the part `{part}`, and not as one name. If you mean one name, write it in double quotes: `\"{text}\"`. If you mean a group, write a name that does not end in `/`."
        ),
        fix: Some(Fix {
            confidence: Confidence::Suggested,
            description: format!("Write `{text}` in double quotes."),
            edits: vec![Edit {
                span,
                text: format!("\"{text}\""),
            }],
        }),
    })
}

/// W0109: a label that holds an arrow, a name and a colon, as in
/// `A -> B: pay -> C: ship`. The parser keeps the whole text as one label, so
/// the file passes and draws one link where the writer meant two. A label
/// with an arrow and no colon after it, as in `calls -> returns`, is common
/// and stays as it is. The fix is only suggested: it cannot know that the
/// writer meant a chain. This reads the tree, because the label is one node.
fn label_links(source: &str, tree: &Node<'_>) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    label_link_lines(source, tree, &mut found);
    found
}

fn label_link_lines(source: &str, node: &Node<'_>, found: &mut Vec<Diagnostic>) {
    if node.kind == NodeKind::Link
        && let Some(diagnostic) = label_link(source, node)
    {
        found.push(diagnostic);
    }
    for inner in node.children.iter().filter_map(|element| match element {
        Element::Node(inner) => Some(inner),
        Element::Token(_) => None,
    }) {
        label_link_lines(source, inner, found);
    }
}

fn label_link(source: &str, link: &Node<'_>) -> Option<Diagnostic> {
    let label = child_of(link, NodeKind::Label)?;
    let tokens: Vec<_> = label
        .children
        .iter()
        .filter_map(|element| match element {
            Element::Token(token) => Some(token),
            Element::Node(_) => None,
        })
        .collect();
    // An arrow, then names, then a colon. A quoted label is one token, so a
    // quoted `-> C: x` never matches.
    let is_name = |kind: TokenKind| {
        matches!(
            kind,
            TokenKind::Word | TokenKind::Quoted | TokenKind::Comma | TokenKind::Space
        )
    };
    let arrow = (0..tokens.len()).find(|&index| {
        if tokens[index].kind != TokenKind::Arrow {
            return false;
        }
        let after = &tokens[index + 1..];
        let names = after.iter().take_while(|t| is_name(t.kind)).count();
        after[..names]
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Word | TokenKind::Quoted))
            && after.get(names).map(|t| t.kind) == Some(TokenKind::Colon)
    })?;
    let arrow_start = tokens[arrow].start;
    let end = tokens.last().map(|t| t.span().end)?;
    let span = arrow_start..end;
    let tail = &source[span.clone()];
    let first = tokens.first()?.start;
    let before = source[first..arrow_start].trim_end();
    let first_label = first..first + before.len();
    let target = link
        .children
        .iter()
        .filter_map(|element| match element {
            Element::Node(names) if names.kind == NodeKind::Names => Some(names),
            _ => None,
        })
        .nth(1)?;
    let target = target.to_string();
    let target = target.trim();
    let line_start = source[..first_start(link)?]
        .rfind('\n')
        .map_or(0, |n| n + 1);
    let indent = &source[line_start..first_start(link)?];
    // A final `?` or a citation stays after the label. The edit would hand it
    // to the second link, so a link that carries one gets no fix.
    let marked = link.children.iter().any(|element| match element {
        Element::Token(token) => token.kind == TokenKind::Guess,
        Element::Node(inner) => inner.kind == NodeKind::Cite,
    });
    let fix = (!marked && indent.chars().all(|c| c == ' ' || c == '\t')).then(|| Fix {
        confidence: Confidence::Suggested,
        description: format!("Write the second link on its own line, as `{target} {tail}`."),
        edits: vec![Edit {
            span: first_label.end..end,
            text: format!("\n{indent}{target} {tail}"),
        }],
    });
    Some(Diagnostic {
        code: "W0109",
        span,
        message: format!(
            "The label holds `{tail}`, which reads as a second link. A label keeps arrows and colons as text, so this is one link with a long label. If you mean two links, write each on its own line, the second as `{target} {tail}`. If you mean one label, write it in double quotes."
        ),
        fix,
    })
}

fn child_of<'a, 's>(node: &'a Node<'s>, kind: NodeKind) -> Option<&'a Node<'s>> {
    node.children.iter().find_map(|element| match element {
        Element::Node(inner) if inner.kind == kind => Some(inner),
        _ => None,
    })
}

/// The byte offset of the first token under a node.
fn first_start(node: &Node<'_>) -> Option<usize> {
    node.children.iter().find_map(|element| match element {
        Element::Token(token) => Some(token.start),
        Element::Node(inner) => first_start(inner),
    })
}

/// N0200: a link or a rule that every statement marks with a final `?`.
fn guesses(model: &Model) -> Vec<Diagnostic> {
    let name = |id: ThingId| &model.things[id].name;
    let links = model.links.iter().filter(|link| link.guess).map(|link| {
        (
            format!("{} -> {}", name(link.from), name(link.to)),
            link.span.clone(),
        )
    });
    let rules = model.rules.iter().filter(|rule| rule.guess).map(|rule| {
        (
            format!("{} -x {}", name(rule.from), name(rule.to)),
            rule.span.clone(),
        )
    });
    links
        .chain(rules)
        .map(|(statement, span)| Diagnostic {
            code: "N0200",
            span,
            message: format!(
                "`{statement}` is a guess. Confirm it against the code or the system, then remove the final `?`."
            ),
            fix: None,
        })
        .collect()
}

/// E0300: a `show` line that names no lens. A lens one edit from a known lens
/// gets a safe fix, because no two lenses are within two edits of each other.
fn lenses(model: &Model) -> Vec<Diagnostic> {
    model
        .shows
        .iter()
        .filter(|show| !LENSES.contains(&show.lens.as_str()))
        .map(|show| {
            let near = LENSES.iter().find(|lens| distance(&show.lens, lens) == 1);
            Diagnostic {
                code: "E0300",
                span: show.span.clone(),
                message: format!(
                    "`{}` is not a lens. Write one of: {}.",
                    show.lens,
                    LENSES.join(", ")
                ),
                fix: near.map(|lens| Fix {
                    confidence: Confidence::Safe,
                    description: format!("Replace `{}` with `{lens}`.", show.lens),
                    edits: vec![Edit {
                        span: show.span.clone(),
                        text: (*lens).to_owned(),
                    }],
                }),
            }
        })
        .collect()
}

/// E0301: a `show` line whose lens needs facts that the file does not state.
/// The message names each missing fact, so the picture is never drawn empty
/// without a reason.
fn needs(model: &Model) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    for show in &model.shows {
        let Some(lens) = crate::lens::find(&show.lens) else {
            continue;
        };
        for need in lens.missing(model) {
            let message = match need {
                Need::Links => format!(
                    "`show {}` orders things by their links, and this file has no link, so there is nothing to order. Add a link with `->`, as in `Web app -> API gateway`.",
                    lens.name
                ),
            };
            found.push(Diagnostic {
                code: "E0301",
                span: show.span.clone(),
                message,
                fix: None,
            });
        }
    }
    found
}

/// The number of edits between two names: a character added, removed or
/// changed, or two neighbours swapped. A swap is one edit because it is the
/// most common slip of the fingers.
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    // Three rows of the optimal string alignment table.
    let mut before: Vec<usize> = Vec::new();
    let mut last: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut row = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            row[j] = (last[j] + 1).min(row[j - 1] + 1).min(last[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                row[j] = row[j].min(before[j - 2] + 1);
            }
        }
        before = std::mem::replace(&mut last, row);
    }
    last[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_counts_one_edit_of_each_kind() {
        assert_eq!(distance("Payments", "Payments"), 0);
        assert_eq!(distance("Paymnts", "Payments"), 1);
        assert_eq!(distance("Paymentss", "Payments"), 1);
        assert_eq!(distance("Paymants", "Payments"), 1);
        assert_eq!(distance("Paymetns", "Payments"), 1);
        assert_eq!(distance("Paymts", "Payments"), 2);
        assert_eq!(distance("Café", "Cafe"), 1);
    }

    #[test]
    fn no_two_lenses_are_within_two_edits() {
        // The safe fix for a lens rests on this.
        for a in LENSES {
            for b in LENSES.iter().filter(|b| *b != a) {
                assert!(distance(a, b) > 2, "{a} and {b}");
            }
        }
    }
}
