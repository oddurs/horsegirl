//! Source text to tokens. Every byte of the source lands in one token.

use super::{Token, TokenKind};

/// The characters that end a bare word. `->` ends one too.
const RESERVED: [char; 5] = [',', ':', '{', '}', '@'];

pub(super) fn lex(source: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start = 0;
    // A comment starts a line, so the lexer knows whether a line has begun.
    let mut line_start = true;
    while let Some(first) = source[start..].chars().next() {
        let rest = &source[start..];
        let (kind, len) = token(rest, first, line_start);
        line_start = match kind {
            TokenKind::Newline => true,
            TokenKind::Space => line_start,
            _ => false,
        };
        tokens.push(Token {
            kind,
            text: &rest[..len],
            start,
        });
        start += len;
    }
    tokens
}

/// The kind and the length in bytes of the token at the start of `rest`.
fn token(rest: &str, first: char, line_start: bool) -> (TokenKind, usize) {
    if let Some(len) = newline(rest) {
        return (TokenKind::Newline, len);
    }
    if is_space(first) {
        return (
            TokenKind::Space,
            run(rest, |tail, c| !is_space(c) || newline(tail).is_some()),
        );
    }
    match first {
        '#' if line_start => (TokenKind::Comment, line(rest)),
        ',' => (TokenKind::Comma, 1),
        ':' => (TokenKind::Colon, 1),
        '{' => (TokenKind::Open, 1),
        '}' => (TokenKind::Close, 1),
        '@' => (TokenKind::At, 1),
        '-' if rest.starts_with("->") => (TokenKind::Arrow, 2),
        '"' => {
            let line = line(rest);
            match rest[1..line].find('"') {
                Some(close) => (TokenKind::Quoted, close + 2),
                None => (TokenKind::Unclosed, line),
            }
        }
        _ => {
            let len = run(rest, |tail, c| {
                is_space(c) || RESERVED.contains(&c) || tail.starts_with("->")
            });
            let kind = if &rest[..len] == "-x" {
                TokenKind::Cross
            } else {
                TokenKind::Word
            };
            (kind, len)
        }
    }
}

/// The length of the line break at the start of `rest`, if one is there.
fn newline(rest: &str) -> Option<usize> {
    if rest.starts_with("\r\n") {
        Some(2)
    } else if rest.starts_with('\n') {
        Some(1)
    } else {
        None
    }
}

// A byte order mark is not white space in Unicode. An editor can write one at
// the start of a file, and it must not become part of the first name.
fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// The number of bytes before the first character where `stop` holds.
fn run(rest: &str, stop: impl Fn(&str, char) -> bool) -> usize {
    rest.char_indices()
        .find(|&(at, c)| stop(&rest[at..], c))
        .map_or(rest.len(), |(at, _)| at)
}

/// The number of bytes before the line break that ends this line.
fn line(rest: &str) -> usize {
    run(rest, |tail, _| newline(tail).is_some())
}
