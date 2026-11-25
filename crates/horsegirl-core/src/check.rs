//! The checks: what the model says about a file that the parser cannot see.
//!
//! A thing exists from its first mention, so a typo makes a new thing with no
//! error from the parser. The checks read the model and report a typo, a thing
//! with no link, a link that breaks a `-x` rule, a guess and an unknown lens.

use crate::diagnostic::{Confidence, Diagnostic, Edit, Fix, LineIndex};
use crate::model::{Model, Thing, ThingId, lower};
use crate::syntax::parse;

/// Every lens that a `show` line can pick, as the language card lists them.
pub const LENSES: &[&str] = &["flow", "sequence", "lanes", "matrix", "rings", "seams"];

/// Every diagnostic for a file: the parser's and the checks', sorted by the
/// start of the span and then by code.
#[must_use]
pub fn check(source: &str) -> Vec<Diagnostic> {
    let parse = parse(source);
    let mut diagnostics = parse.diagnostics;
    diagnostics.extend(of_model(source, &lower(&parse.tree)));
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
    diagnostics.extend(broken_rules(source, model));
    diagnostics.extend(guesses(model));
    diagnostics.extend(lenses(model));
    sort(&mut diagnostics);
    diagnostics
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
            .filter(|known| known.mentions.len() > 1 && distance(&thing.name, &known.name) == 1)
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
