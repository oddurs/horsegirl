//! The formatter: a file to its one canonical form.
//!
//! The formatter reads the lossless tree and prints it again. It changes the
//! layout and the spelling of a name, and nothing that the file states:
//!
//! - One fact on each line, indented two spaces for each level of braces.
//! - One space around an arrow, none before a colon, one after it.
//! - A comma and a space between names and between facts.
//! - The facts of one line in the order of their keys.
//! - A block that holds one line of names or facts stays on the line of its
//!   name. Any other block has a line for each part.
//! - A name or a value in quotes loses them when a bare one reads the same
//!   (ADR-119). A name that needs quotes keeps them.
//! - A run of white space inside a name or a label is one space. A comment
//!   stays as it is, and a run of blank lines is one blank line.
//! - Line breaks are `\n`, and the file ends with one.
//!
//! A file with an error has no canonical form, so the formatter refuses it.

use crate::syntax::{Diagnostic, Element, Node, NodeKind, Token, TokenKind, parse};

/// Prints `source` in its canonical form.
///
/// # Errors
///
/// Returns the errors of the file when it has any. The warnings do not stop
/// the formatter.
pub fn format(source: &str) -> Result<String, Vec<Diagnostic>> {
    let parse = parse(source);
    let errors: Vec<_> = parse
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.code.starts_with('E'))
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    // A bare name that starts with `#` reads as a name after a `{`, and as a
    // comment at the start of a line. The formatter can move it to the start of
    // a line, and quotes cannot hold a name with a `"`. No form is both
    // canonical and the same file, so the file stays as it is.
    if hides_a_comment(&parse.tree) {
        return Ok(source.to_owned());
    }
    let mut out = String::new();
    lines(&parse.tree.children, 0, &mut out);
    Ok(out)
}

/// Whether a bare name holds a `"` and needs quotes: it starts with `#`, or it
/// is a fact value that ends in ` ?`. Quotes cannot hold a `"`.
fn hides_a_comment(node: &Node<'_>) -> bool {
    let named = matches!(
        node.kind,
        NodeKind::Name | NodeKind::Key | NodeKind::Value | NodeKind::Arg
    );
    if named {
        let text = squeeze(node);
        let needs_quotes =
            text.starts_with('#') || (node.kind == NodeKind::Value && text.ends_with(" ?"));
        return needs_quotes && text.contains('"');
    }
    node.children.iter().any(|child| match child {
        Element::Node(inner) => hides_a_comment(inner),
        Element::Token(_) => false,
    })
}

/// Prints the statements and the comments of a file or of a block, a line for
/// each. Two or more line breaks between two of them keep one blank line.
fn lines(children: &[Element<'_>], depth: usize, out: &mut String) {
    let mut breaks = 0;
    let mut first = true;
    for child in children {
        let text = match child {
            Element::Token(token) => match token.kind {
                TokenKind::Newline => {
                    breaks += 1;
                    continue;
                }
                TokenKind::Comment => token.text.trim_end().to_owned(),
                _ => continue,
            },
            Element::Node(node) => statement(node, depth),
        };
        if !first && breaks > 1 {
            out.push('\n');
        }
        first = false;
        breaks = 0;
        out.push_str(&"  ".repeat(depth));
        out.push_str(&text);
        out.push('\n');
    }
}

/// The text of a statement. A block inside it ends with its own `}`.
fn statement(node: &Node<'_>, depth: usize) -> String {
    let mut text = match node.kind {
        NodeKind::Link | NodeKind::Rule => link(node),
        NodeKind::Things => things(node, depth),
        NodeKind::Facts => facts(node),
        NodeKind::Show => show(node),
        NodeKind::Story => story(node, depth),
        // An error never reaches here, and any other shape keeps its text.
        _ => return node.to_string(),
    };
    if node.kind != NodeKind::Show && node.kind != NodeKind::Story {
        for child in &node.children {
            match child {
                Element::Token(token) if token.kind == TokenKind::Guess => text.push_str(" ?"),
                Element::Node(cite) if cite.kind == NodeKind::Cite => {
                    text.push_str(" @ ");
                    text.push_str(&path(cite));
                }
                _ => {}
            }
        }
    }
    text
}

fn link(node: &Node<'_>) -> String {
    let mut text = String::new();
    for child in &node.children {
        match child {
            Element::Node(inner) if inner.kind == NodeKind::Names => {
                text.push_str(&names(inner));
            }
            // A colon with no label stays. Without it, a name that ends in a
            // `?` would read as a guess.
            Element::Token(token) if token.kind == TokenKind::Colon => text.push(':'),
            Element::Node(inner) if inner.kind == NodeKind::Label => {
                text.push(' ');
                text.push_str(&label(inner));
            }
            Element::Token(token) if token.kind == TokenKind::Arrow => text.push_str(" -> "),
            Element::Token(token) if token.kind == TokenKind::Cross => text.push_str(" -x "),
            _ => {}
        }
    }
    text
}

fn things(node: &Node<'_>, depth: usize) -> String {
    let mut text = String::new();
    for child in &node.children {
        match child {
            Element::Node(inner) if inner.kind == NodeKind::Names => text.push_str(&names(inner)),
            Element::Node(inner) if inner.kind == NodeKind::Block => {
                text.push(' ');
                text.push_str(&block(inner, depth, true));
            }
            _ => {}
        }
    }
    text
}

fn story(node: &Node<'_>, depth: usize) -> String {
    let mut text = String::from("story");
    for child in &node.children {
        match child {
            Element::Node(inner) if inner.kind == NodeKind::Name => {
                text.push(' ');
                text.push_str(&name(inner));
            }
            Element::Node(inner) if inner.kind == NodeKind::Block => {
                text.push(' ');
                text.push_str(&block(inner, depth, false));
            }
            _ => {}
        }
    }
    text
}

fn show(node: &Node<'_>) -> String {
    let mut text = String::from("show");
    for child in &node.children {
        match child {
            Element::Node(inner) if inner.kind == NodeKind::Lens => {
                text.push(' ');
                text.push_str(&inner.to_string());
            }
            Element::Token(token) if token.kind == TokenKind::Keyword && token.text != "show" => {
                text.push(' ');
                text.push_str(token.text);
            }
            Element::Node(inner) if inner.kind == NodeKind::Arg => {
                text.push(' ');
                text.push_str(&name(inner));
            }
            _ => {}
        }
    }
    text
}

/// A block, from its `{` to its `}`. `depth` is the level of the line that
/// holds the `{`. `inline` allows the block to stay on that line.
fn block(node: &Node<'_>, depth: usize, inline: bool) -> String {
    let inside: Vec<_> = node
        .children
        .iter()
        .filter(|child| match child {
            Element::Node(_) => true,
            Element::Token(token) => token.kind == TokenKind::Comment,
        })
        .collect();
    if inside.is_empty() {
        return "{}".to_owned();
    }
    // A comment ends its line, so a block with one cannot be one line.
    if inline && let [Element::Node(only)] = inside[..] {
        let short = match only.kind {
            NodeKind::Facts => true,
            NodeKind::Things => !only
                .children
                .iter()
                .any(|child| matches!(child, Element::Node(n) if n.kind == NodeKind::Block)),
            _ => false,
        };
        if short {
            return format!("{{ {} }}", statement(only, depth));
        }
    }
    let mut out = String::from("{\n");
    lines(&node.children, depth + 1, &mut out);
    out.push_str(&"  ".repeat(depth));
    out.push('}');
    out
}

/// The names of a list, with a comma and a space between them.
fn names(node: &Node<'_>) -> String {
    let parts: Vec<_> = node
        .children
        .iter()
        .filter_map(|child| match child {
            Element::Node(inner) => Some(name(inner)),
            Element::Token(_) => None,
        })
        .collect();
    parts.join(", ")
}

fn facts(node: &Node<'_>) -> String {
    let mut parts: Vec<(String, String)> = node
        .children
        .iter()
        .filter_map(|child| match child {
            Element::Node(fact) if fact.kind == NodeKind::Fact => {
                let mut key = String::new();
                let mut value = String::new();
                for part in &fact.children {
                    match part {
                        Element::Node(inner) if inner.kind == NodeKind::Key => {
                            key = name(inner);
                        }
                        Element::Node(inner) if inner.kind == NodeKind::Value => {
                            value = name(inner);
                        }
                        _ => {}
                    }
                }
                Some((key.clone(), format!("{key}: {value}")))
            }
            _ => None,
        })
        .collect();
    // The sort is stable, so two facts with one key keep their order.
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    let parts: Vec<_> = parts.into_iter().map(|(_, text)| text).collect();
    parts.join(", ")
}

/// A name, a key, a value or an argument.
fn name(node: &Node<'_>) -> String {
    let tokens: Vec<&Token<'_>> = node
        .children
        .iter()
        .filter_map(|child| match child {
            Element::Token(token) => Some(token),
            Element::Node(_) => None,
        })
        .collect();
    if let [token] = tokens[..]
        && token.kind == TokenKind::Quoted
    {
        // The model keeps the text inside quotes exactly, so the formatter
        // drops the quotes only when the bare text is the same text.
        let inside = &token.text[1..token.text.len() - 1];
        return if bare(inside) && inside == squeeze_text(inside) {
            inside.to_owned()
        } else {
            token.text.to_owned()
        };
    }
    // A bare name moves to the start of a line when a block opens out, and `#`
    // starts a comment there. `hides_a_comment` has dealt with a name that
    // quotes cannot hold.
    let bare_name = squeeze(node);
    // A trailing `?` in a fact value would become a guess when the fact sort
    // moves the value to the end of the line.
    if bare_name.starts_with('#') || (node.kind == NodeKind::Value && bare_name.ends_with(" ?")) {
        return format!("\"{bare_name}\"");
    }
    bare_name
}

/// Whether `text` reads back as one name when it has no quotes. It is a
/// one-line text of words with single spaces.
fn bare(text: &str) -> bool {
    let words: Vec<_> = text.split(' ').collect();
    !text.is_empty()
        && !text.contains(['"', ',', ':', '{', '}', '@'])
        && !text.contains("->")
        // `#` starts a comment at the start of a line.
        && !text.starts_with('#')
        && !words.iter().any(|word| word.is_empty() || ["-x", "?"].contains(word))
        // `show` and `story` start a line of their own.
        && !["show", "story"].contains(&words[0])
}

/// A label. A label of one quoted text keeps that text exactly, as the model
/// does. Any other label has one space between words.
fn label(node: &Node<'_>) -> String {
    let mut tokens = Vec::new();
    collect(node, &mut tokens);
    if let [token] = tokens[..]
        && token.kind == TokenKind::Quoted
    {
        return token.text.to_owned();
    }
    squeeze(node)
}

fn collect<'a, 's>(node: &'a Node<'s>, into: &mut Vec<&'a Token<'s>>) {
    for child in &node.children {
        match child {
            Element::Node(inner) => collect(inner, into),
            Element::Token(token) => into.push(token),
        }
    }
}

/// The tokens of a node as text, with each run of spaces as one space.
/// A quoted token is exact text, as in the model, so its spaces stay.
fn squeeze(node: &Node<'_>) -> String {
    let mut tokens = Vec::new();
    collect(node, &mut tokens);
    let mut out = String::new();
    let mut gap = false;
    for token in tokens {
        if token.kind == TokenKind::Quoted {
            if gap && !out.is_empty() {
                out.push(' ');
            }
            gap = false;
            out.push_str(token.text);
            continue;
        }
        for c in token.text.chars() {
            // The lexer reads a byte order mark as a space, so the formatter does too.
            if c.is_whitespace() || c == '\u{feff}' {
                gap = true;
            } else {
                if gap && !out.is_empty() {
                    out.push(' ');
                }
                gap = false;
                out.push(c);
            }
        }
    }
    out
}

fn squeeze_text(text: &str) -> String {
    // The lexer reads a byte order mark as a space, so the formatter does too.
    text.split(|c: char| c.is_whitespace() || c == '\u{feff}')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The path of a citation, without the `@`.
fn path(node: &Node<'_>) -> String {
    node.children
        .iter()
        .filter_map(|child| match child {
            Element::Token(token) if !matches!(token.kind, TokenKind::At | TokenKind::Space) => {
                Some(token.text)
            }
            _ => None,
        })
        .collect()
}
