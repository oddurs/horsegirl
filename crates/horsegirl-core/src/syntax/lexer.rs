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
        let (kind, len) = token(source, start, first, line_start);
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
fn token(source: &str, start: usize, first: char, line_start: bool) -> (TokenKind, usize) {
    let rest = &source[start..];
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
        '-' if arrow(rest).is_some() => (TokenKind::Arrow, arrow(rest).unwrap_or(2)),
        ';' if ends_line(&rest[1..]) && !follows_semi(&source[..start]) => (TokenKind::Semi, 1),
        // The scan stops at the closing quote. A scan to the end of the line
        // for every quote makes a long line of quoted names quadratic.
        '"' => match rest[1..].find(['"', '\n']) {
            Some(close) if rest[close + 1..].starts_with('"') => (TokenKind::Quoted, close + 2),
            _ => (TokenKind::Unclosed, line(rest)),
        },
        _ => {
            let len = run(rest, |tail, c| {
                is_space(c)
                    || RESERVED.contains(&c)
                    || arrow(tail).is_some()
                    || (c == ';'
                        && ends_line(&tail[1..])
                        && !follows_semi(&source[..source.len() - tail.len()]))
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

/// Whether a `;` is the last mark before the end of `before`, spaces aside. A
/// space is any `is_space` character except a line feed. `;;` and `; ;` are not marks: removing one `;` would leave the other
/// at the line end and change the name.
fn follows_semi(before: &str) -> bool {
    before
        .trim_end_matches(|c| is_space(c) && c != '\n')
        .ends_with(';')
}

/// The length of the arrow at the start of `rest`. `->` is the arrow. Mermaid
/// writes `-->`, and the reader takes a run of dashes before the `>` as one.
fn arrow(rest: &str) -> Option<usize> {
    let dashes = rest.bytes().take_while(|&byte| byte == b'-').count();
    (dashes > 0 && rest[dashes..].starts_with('>')).then_some(dashes + 1)
}

/// Whether only spaces are left on the line, or a `}` closes the block. A lone
/// carriage return, U+2028 and U+0085 are spaces. The `\r` of `\r\n` is one too.
fn ends_line(rest: &str) -> bool {
    let rest = rest.trim_start_matches(|c| is_space(c) && c != '\n');
    rest.is_empty() || rest.starts_with(['\n', '}'])
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
