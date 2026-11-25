//! The reader: source text to a lossless tree.
//!
//! The tree holds every byte of the source, so printing it gives the source
//! back and a formatter can work on it. A bad line becomes one `Error` node and
//! one diagnostic, and the lines around it still parse.
//!
//! The tree is text only. It does not know what a thing or a link is.

mod lexer;
mod parser;

use std::fmt;
use std::ops::Range;

pub use crate::diagnostic::{Confidence, Diagnostic, Edit, Fix};

/// What a token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// A run of characters with no space and no reserved character.
    Word,
    /// Text in double quotes on one line, with both quotes.
    Quoted,
    /// A double quote that does not close, with the rest of its line.
    Unclosed,
    /// `->`
    Arrow,
    /// `-x` as a whole word.
    Cross,
    Comma,
    Colon,
    /// `{`
    Open,
    /// `}`
    Close,
    /// `@`
    At,
    /// `show`, `story`, `of`, `from` or `by` where the grammar reads it.
    Keyword,
    /// A final `?`.
    Guess,
    /// Spaces and tabs.
    Space,
    Newline,
    /// A line that starts with `#`.
    Comment,
}

/// What a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    File,
    /// `Names -> Names: Label`
    Link,
    /// `Names -x Names: Label`
    Rule,
    /// `Names`, or `Names Block`.
    Things,
    /// One or more `Fact`, inside a block.
    Facts,
    /// `show Lens`, or `show Lens of Arg`.
    Show,
    /// `story Name Block`
    Story,
    /// `{`, the lines inside, and `}`.
    Block,
    /// One or more `Name` with commas between them.
    Names,
    Name,
    /// `Key: Value`
    Fact,
    Key,
    Value,
    Label,
    Lens,
    /// The name after `of`, `from` or `by` on a `show` line.
    Arg,
    /// `@` and the path after it.
    Cite,
    /// A line that the parser cannot read. It holds tokens and blocks only.
    Error,
}

/// One piece of the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<'s> {
    pub kind: TokenKind,
    pub text: &'s str,
    /// The byte offset of the token in the source.
    pub start: usize,
}

impl Token<'_> {
    /// The bytes of the source that the token fills.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        self.start..self.start + self.text.len()
    }
}

/// A child of a node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element<'s> {
    Node(Node<'s>),
    Token(Token<'s>),
}

/// A node of the tree.
///
/// `Display` prints the source text of the node. `Debug` prints the shape of
/// the tree without spaces and line breaks, one node on each line.
#[derive(Clone, PartialEq, Eq)]
pub struct Node<'s> {
    pub kind: NodeKind,
    pub children: Vec<Element<'s>>,
}

impl Node<'_> {
    fn dump(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = depth * 2;
        write!(f, "{:indent$}{:?}", "", self.kind)?;
        let leaf = |child: &Element<'_>| matches!(child, Element::Token(_));
        if self.children.iter().all(leaf) {
            return writeln!(f, " {:?}", self.to_string());
        }
        writeln!(f)?;
        for child in &self.children {
            match child {
                Element::Node(node) => node.dump(f, depth + 1)?,
                Element::Token(token) => {
                    if !matches!(token.kind, TokenKind::Space | TokenKind::Newline) {
                        writeln!(f, "{:indent$}  {:?} {:?}", "", token.kind, token.text)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            match child {
                Element::Node(node) => node.fmt(f)?,
                Element::Token(token) => f.write_str(token.text)?,
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(f, 0)
    }
}

/// The result of reading one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse<'s> {
    /// The `File` node. It prints back to the source byte for byte.
    pub tree: Node<'s>,
    /// Every diagnostic, sorted by the start of its span and then by code.
    /// No two fixes overlap, so a tool can apply them all.
    pub diagnostics: Vec<Diagnostic>,
}

/// Reads a horsegirl file.
///
/// The parser never fails. A line that it cannot read becomes an `Error` node
/// with one diagnostic, and the other lines parse as if the bad line were
/// absent.
#[must_use]
pub fn parse(source: &str) -> Parse<'_> {
    parser::parse(source)
}
