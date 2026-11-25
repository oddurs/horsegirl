//! Tokens to the tree.
//!
//! Every function appends what it reads to a list of children before it can
//! fail, so an error never drops a token and the tree stays lossless.

use std::ops::Range;

use super::lexer::lex;
use super::{Diagnostic, Element, Fix, Node, NodeKind, Parse, Token, TokenKind};

/// Braces nest this deep and no deeper. The parser calls itself once for each
/// level, so a file of ten thousand `{` must not reach the end of the stack.
const MAX_DEPTH: usize = 64;

type Children<'s> = Vec<Element<'s>>;

pub(super) fn parse(source: &str) -> Parse<'_> {
    let mut parser = Parser {
        source,
        tokens: lex(source),
        pos: 0,
        diagnostics: Vec::new(),
    };
    let mut children = Vec::new();
    loop {
        parser.lines(0, &mut children);
        if parser.kind().is_none() {
            break;
        }
        // `lines` stops at a `}`. Outside every block, that brace closes nothing.
        parser.diagnostics.push(diagnostic(
            "E0003",
            parser.span_at(parser.pos),
            "This `}` closes nothing. Remove it, or add the `{` that it closes.",
        ));
        let from = children.len();
        parser.bump(&mut children);
        wrap(&mut children, from, NodeKind::Error);
    }
    // An unclosed brace is reported at the end of the file. Sorting puts it
    // back where the brace is.
    parser.diagnostics.sort_by_key(|d| d.span.start);
    Parse {
        tree: Node {
            kind: NodeKind::File,
            children,
        },
        diagnostics: parser.diagnostics,
    }
}

fn diagnostic(code: &'static str, span: Range<usize>, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        span,
        message: message.into(),
        fix: None,
    }
}

/// Moves the children from `from` to the end into one new node.
fn wrap(children: &mut Children<'_>, from: usize, kind: NodeKind) {
    let inside = children.split_off(from);
    children.push(Element::Node(Node {
        kind,
        children: inside,
    }));
}

/// Takes the half-built nodes of a bad line apart. Blocks stay whole, because
/// the lines inside them were read on their own.
fn flatten<'s>(children: Children<'s>, flat: &mut Children<'s>) {
    for child in children {
        match child {
            Element::Node(node) if node.kind != NodeKind::Block => flatten(node.children, flat),
            other => flat.push(other),
        }
    }
}

/// One name or one fact in a list.
struct Entry {
    /// The index of the first token.
    start: usize,
    /// The index of the first token of the value, when the entry is a fact.
    value: Option<usize>,
    /// The byte offset where the entry ends.
    end: usize,
}

struct Parser<'s> {
    source: &'s str,
    tokens: Vec<Token<'s>>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'s> Parser<'s> {
    fn kind_at(&self, index: usize) -> Option<TokenKind> {
        self.tokens.get(index).map(|token| token.kind)
    }

    fn kind(&self) -> Option<TokenKind> {
        self.kind_at(self.pos)
    }

    /// The index of the first token at or after `index` that is not a space.
    fn skip(&self, mut index: usize) -> usize {
        while self.kind_at(index) == Some(TokenKind::Space) {
            index += 1;
        }
        index
    }

    /// The kind of the next token that is not a space.
    fn next(&self) -> Option<TokenKind> {
        self.kind_at(self.skip(self.pos))
    }

    /// The bytes of the token at `index`, or the end of the source.
    fn span_at(&self, index: usize) -> Range<usize> {
        self.tokens
            .get(index)
            .map_or(self.source.len()..self.source.len(), Token::span)
    }

    /// The byte offset where the token before the current one ends.
    fn here(&self) -> usize {
        self.span_at(self.pos).start
    }

    fn is_word(&self, index: usize, text: &str) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| token.kind == TokenKind::Word && token.text == text)
    }

    /// A `?` is a guess mark only when it ends the fact. Anywhere else it is a
    /// word of a name or of a label.
    fn is_guess(&self, index: usize) -> bool {
        self.is_word(index, "?")
            && matches!(
                self.kind_at(self.skip(index + 1)),
                None | Some(TokenKind::Newline | TokenKind::At | TokenKind::Close)
            )
    }

    fn at_line_end(&self) -> bool {
        matches!(
            self.next(),
            None | Some(TokenKind::Newline | TokenKind::Close)
        )
    }

    fn bump(&mut self, into: &mut Children<'s>) {
        self.bump_as(None, into);
    }

    fn bump_as(&mut self, kind: Option<TokenKind>, into: &mut Children<'s>) {
        if let Some(&token) = self.tokens.get(self.pos) {
            into.push(Element::Token(Token {
                kind: kind.unwrap_or(token.kind),
                ..token
            }));
            self.pos += 1;
        }
    }

    fn spaces(&mut self, into: &mut Children<'s>) {
        while self.kind() == Some(TokenKind::Space) {
            self.bump(into);
        }
    }

    /// Reads lines until the end of the file or a `}`.
    fn lines(&mut self, depth: usize, into: &mut Children<'s>) {
        loop {
            match self.kind() {
                None | Some(TokenKind::Close) => return,
                Some(TokenKind::Space | TokenKind::Newline | TokenKind::Comment) => self.bump(into),
                Some(_) => self.statement(depth, into),
            }
        }
    }

    /// Reads one fact. A fact that cannot be read becomes an `Error` node that
    /// runs to the end of its line.
    fn statement(&mut self, depth: usize, into: &mut Children<'s>) {
        let mut children = Vec::new();
        let kind = match self.fact(depth, &mut children) {
            Ok(kind) => kind,
            Err(error) => {
                self.diagnostics.push(error);
                loop {
                    match self.kind() {
                        None | Some(TokenKind::Newline | TokenKind::Close) => break,
                        Some(TokenKind::Open) if depth < MAX_DEPTH => {
                            self.block(depth, &mut children);
                        }
                        Some(_) => self.bump(&mut children),
                    }
                }
                let mut flat = Vec::new();
                flatten(children, &mut flat);
                children = flat;
                NodeKind::Error
            }
        };
        into.push(Element::Node(Node { kind, children }));
    }

    fn fact(&mut self, depth: usize, into: &mut Children<'s>) -> Result<NodeKind, Diagnostic> {
        let kind = if self.is_word(self.pos, "show") && self.is_show_line() {
            self.show(into)?
        } else if self.is_word(self.pos, "story") && self.is_story() {
            self.story(depth, into)?
        } else {
            let kind = self.names_or_link(depth, into)?;
            if self.is_guess(self.skip(self.pos)) {
                self.spaces(into);
                self.bump_as(Some(TokenKind::Guess), into);
            }
            if self.next() == Some(TokenKind::At) {
                self.spaces(into);
                self.cite(into)?;
            }
            kind
        };
        if self.at_line_end() {
            Ok(kind)
        } else {
            Err(self.unexpected(self.skip(self.pos)))
        }
    }

    fn names_or_link(
        &mut self,
        depth: usize,
        into: &mut Children<'s>,
    ) -> Result<NodeKind, Diagnostic> {
        // Outside braces a colon can only start a label, so `key: value` is a
        // fact only inside a block.
        let entries = self.list(depth > 0, into)?;
        if entries.iter().any(|entry| entry.value.is_some()) {
            self.facts(&entries)?;
            return Ok(NodeKind::Facts);
        }
        match self.next() {
            Some(arrow @ (TokenKind::Arrow | TokenKind::Cross)) => {
                wrap(into, 0, NodeKind::Names);
                self.spaces(into);
                self.bump(into);
                self.spaces(into);
                let from = into.len();
                self.list(false, into)?;
                wrap(into, from, NodeKind::Names);
                if self.next() == Some(TokenKind::Colon) {
                    self.spaces(into);
                    self.bump(into);
                    self.label(into)?;
                }
                Ok(if arrow == TokenKind::Arrow {
                    NodeKind::Link
                } else {
                    NodeKind::Rule
                })
            }
            Some(TokenKind::Open) => {
                wrap(into, 0, NodeKind::Names);
                self.spaces(into);
                self.open(depth, into)?;
                Ok(NodeKind::Things)
            }
            Some(TokenKind::Colon) => Err(self.colon_outside_braces(&entries)),
            _ => {
                wrap(into, 0, NodeKind::Names);
                Ok(NodeKind::Things)
            }
        }
    }

    /// Reads names with commas between them. With `facts`, an entry can be
    /// `key: value` instead.
    fn list(&mut self, facts: bool, into: &mut Children<'s>) -> Result<Vec<Entry>, Diagnostic> {
        let mut entries = Vec::new();
        loop {
            let start = self.pos;
            let from = into.len();
            if !self.name(NodeKind::Name, into) {
                return Err(self.missing("name"));
            }
            let mut value = None;
            if facts && self.next() == Some(TokenKind::Colon) {
                if let Some(Element::Node(key)) = into.last_mut() {
                    key.kind = NodeKind::Key;
                }
                self.spaces(into);
                self.bump(into);
                self.spaces(into);
                value = Some(self.pos);
                if !self.name(NodeKind::Value, into) {
                    return Err(self.missing("value"));
                }
                wrap(into, from, NodeKind::Fact);
            }
            entries.push(Entry {
                start,
                value,
                end: self.here(),
            });
            if self.next() != Some(TokenKind::Comma) {
                return Ok(entries);
            }
            self.spaces(into);
            self.bump(into);
            self.spaces(into);
        }
    }

    /// Reads one quoted name or one bare name, and says whether one was there.
    fn name(&mut self, kind: NodeKind, into: &mut Children<'s>) -> bool {
        let from = into.len();
        match self.kind() {
            Some(TokenKind::Quoted) => self.bump(into),
            Some(TokenKind::Word) if !self.is_guess(self.pos) => {
                self.bump(into);
                // A bare name has spaces, so it runs on while words follow.
                loop {
                    let word = self.skip(self.pos);
                    if self.kind_at(word) != Some(TokenKind::Word) || self.is_guess(word) {
                        break;
                    }
                    self.spaces(into);
                    self.bump(into);
                }
            }
            _ => return false,
        }
        wrap(into, from, kind);
        true
    }

    /// Checks a list that holds a fact. The list ends its line, and it holds
    /// facts only.
    fn facts(&self, entries: &[Entry]) -> Result<(), Diagnostic> {
        let last_value = entries.last().and_then(|entry| entry.value);
        if let Some(value) = last_value
            && !self.at_line_end()
            && self.next() != Some(TokenKind::At)
            && !self.is_guess(self.skip(self.pos))
        {
            let span = self.span_at(value).start..self.extent(value, true);
            return Err(self.needs_quotes(span, "value"));
        }
        let Some(name) = entries.iter().position(|entry| entry.value.is_none()) else {
            return Ok(());
        };
        // `note: install, list` reads as a fact and a name. The author meant
        // one value with a comma in it.
        let value = name.checked_sub(1).and_then(|fact| entries[fact].value);
        let Some(value) = value else {
            return Err(diagnostic(
                "E0009",
                self.span_at(entries[name].start).start..self.here(),
                "This line mixes names and facts. Write the names on one line and the facts on another line.",
            ));
        };
        let names = entries[name..]
            .iter()
            .take_while(|entry| entry.value.is_none());
        let end = names.last().map_or(self.here(), |entry| entry.end);
        Err(self.needs_quotes(self.span_at(value).start..end, "value"))
    }

    /// The error for `key: value` outside braces. The colon is in a name when
    /// an arrow or a brace follows it, and the line is a stray fact otherwise.
    fn colon_outside_braces(&self, entries: &[Entry]) -> Diagnostic {
        let start = entries.last().map_or(self.pos, |entry| entry.start);
        let start = self.span_at(start).start;
        let mut end = self.here();
        let mut index = self.pos;
        loop {
            match self.kind_at(index) {
                Some(TokenKind::Arrow | TokenKind::Cross | TokenKind::Open) => {
                    return self.needs_quotes(start..end, "name");
                }
                None
                | Some(TokenKind::Newline | TokenKind::Comma | TokenKind::Close | TokenKind::At) => {
                    break;
                }
                Some(TokenKind::Space) => {}
                Some(_) => end = self.span_at(index).end,
            }
            index += 1;
        }
        diagnostic(
            "E0009",
            start..end,
            "A fact goes inside braces after a name, as in `Payments { owner: Commerce }`. A name that holds `:` is written in double quotes.",
        )
    }

    /// Reads the words after the colon of a link. A label holds commas, colons
    /// and arrows as they are, because nothing follows a label on its line.
    fn label(&mut self, into: &mut Children<'s>) -> Result<(), Diagnostic> {
        let first = self.skip(self.pos);
        let mut end = first;
        let mut index = first;
        loop {
            match self.kind_at(index) {
                None
                | Some(
                    TokenKind::Newline
                    | TokenKind::At
                    | TokenKind::Open
                    | TokenKind::Close
                    // An unclosed quote is a mistake in a label too.
                    | TokenKind::Unclosed,
                ) => break,
                Some(TokenKind::Space) => {}
                Some(_) if self.is_guess(index) => break,
                Some(_) => end = index + 1,
            }
            index += 1;
        }
        if end > first {
            self.spaces(into);
            let from = into.len();
            while self.pos < end {
                self.bump(into);
            }
            wrap(into, from, NodeKind::Label);
        }
        // A brace would end a one-line block, so a label holds one in quotes only.
        if self.next() == Some(TokenKind::Open) {
            let span = self.span_at(first).start..self.extent(first, false);
            return Err(self.needs_quotes(span, "label"));
        }
        Ok(())
    }

    /// Reads `@` and the path after it. The path has no spaces.
    fn cite(&mut self, into: &mut Children<'s>) -> Result<(), Diagnostic> {
        let from = into.len();
        let at = self.span_at(self.pos);
        self.bump(into);
        self.spaces(into);
        let path = self.pos;
        while !matches!(
            self.kind(),
            None | Some(TokenKind::Space | TokenKind::Newline | TokenKind::Open | TokenKind::Close)
        ) {
            self.bump(into);
        }
        if self.pos == path {
            return Err(diagnostic(
                "E0001",
                at,
                "`@` cites evidence and nothing follows it. Write a path and a line after it, as in `@ src/app.ts:17`. A name that holds `@` is written in double quotes.",
            ));
        }
        wrap(into, from, NodeKind::Cite);
        Ok(())
    }

    /// A line that starts with `show` is a `show` line unless it holds an
    /// arrow, a comma, a colon or a brace. Then `show` is a word of a name.
    fn is_show_line(&self) -> bool {
        let mut index = self.pos + 1;
        loop {
            match self.kind_at(index) {
                None | Some(TokenKind::Newline | TokenKind::Close) => return true,
                Some(
                    TokenKind::Arrow
                    | TokenKind::Cross
                    | TokenKind::Comma
                    | TokenKind::Colon
                    | TokenKind::Open,
                ) => return false,
                Some(_) => index += 1,
            }
        }
    }

    fn show(&mut self, into: &mut Children<'s>) -> Result<NodeKind, Diagnostic> {
        let start = self.pos;
        if self.show_parts(into) {
            return Ok(NodeKind::Show);
        }
        Err(diagnostic(
            "E0008",
            self.span_at(start).start..self.extent(start, false),
            "A `show` line is `show` and a lens, as in `show flow`, or with `of`, `from` or `by` and a name, as in `show sequence of checkout`. A thing with the name `show` is written `\"show\"`.",
        ))
    }

    fn show_parts(&mut self, into: &mut Children<'s>) -> bool {
        self.bump_as(Some(TokenKind::Keyword), into);
        self.spaces(into);
        if self.kind() != Some(TokenKind::Word) {
            return false;
        }
        let from = into.len();
        self.bump(into);
        wrap(into, from, NodeKind::Lens);
        if self.at_line_end() {
            return true;
        }
        self.spaces(into);
        if !["of", "from", "by"]
            .iter()
            .any(|word| self.is_word(self.pos, word))
        {
            return false;
        }
        self.bump_as(Some(TokenKind::Keyword), into);
        self.spaces(into);
        self.name(NodeKind::Arg, into) && self.at_line_end()
    }

    /// `story`, a name and `{` start a story. Without the brace, `story` is a
    /// word of a name.
    fn is_story(&self) -> bool {
        let mut index = self.pos + 1;
        if self.kind_at(index) != Some(TokenKind::Space) {
            return false;
        }
        index = self.skip(index);
        match self.kind_at(index) {
            Some(TokenKind::Quoted) => index += 1,
            Some(TokenKind::Word) => {
                while matches!(
                    self.kind_at(index),
                    Some(TokenKind::Word | TokenKind::Space)
                ) {
                    index += 1;
                }
            }
            _ => return false,
        }
        self.kind_at(self.skip(index)) == Some(TokenKind::Open)
    }

    fn story(&mut self, depth: usize, into: &mut Children<'s>) -> Result<NodeKind, Diagnostic> {
        let start = self.span_at(self.pos).start;
        self.bump_as(Some(TokenKind::Keyword), into);
        self.spaces(into);
        let name = self.span_at(self.pos).start;
        if !self.name(NodeKind::Name, into) {
            return Err(self.missing("name"));
        }
        let end = self.here();
        self.spaces(into);
        self.open(depth, into)?;
        self.diagnostics.push(diagnostic(
            "W0010",
            start..end,
            format!(
                "Stories arrive in v0.2. This version parses the story `{}` and does not order its links. You do not need to change the file.",
                &self.source[name..end]
            ),
        ));
        Ok(NodeKind::Story)
    }

    /// Reads a block, unless the braces are too deep.
    fn open(&mut self, depth: usize, into: &mut Children<'s>) -> Result<(), Diagnostic> {
        if depth >= MAX_DEPTH {
            return Err(diagnostic(
                "E0007",
                self.span_at(self.pos),
                format!(
                    "Braces nest more than {MAX_DEPTH} deep here. Write the inner groups as their own lines."
                ),
            ));
        }
        self.block(depth, into);
        Ok(())
    }

    /// Reads `{`, the lines inside, and `}`.
    fn block(&mut self, depth: usize, into: &mut Children<'s>) {
        let open = self.span_at(self.pos);
        let from = into.len();
        self.bump(into);
        self.lines(depth + 1, into);
        if self.kind() == Some(TokenKind::Close) {
            self.bump(into);
        } else {
            self.diagnostics.push(diagnostic(
                "E0002",
                open,
                "This `{` never closes. Add a `}` on the line after the last part.",
            ));
        }
        wrap(into, from, NodeKind::Block);
    }

    /// The byte offset where the text that starts at token `from` ends: at the
    /// end of the line, at `@`, or at the `}` that closes the block around it.
    fn extent(&self, from: usize, stop_at_comma: bool) -> usize {
        let mut end = self.span_at(from).start;
        let mut open = 0_usize;
        let mut index = from;
        loop {
            match self.kind_at(index) {
                None | Some(TokenKind::Newline | TokenKind::At) => return end,
                Some(TokenKind::Comma) if stop_at_comma => return end,
                Some(TokenKind::Space) => {}
                Some(kind) => {
                    if kind == TokenKind::Open {
                        open += 1;
                    } else if kind == TokenKind::Close {
                        if open == 0 {
                            return end;
                        }
                        open -= 1;
                    }
                    end = self.span_at(index).end;
                }
            }
            index += 1;
        }
    }

    fn needs_quotes(&self, span: Range<usize>, what: &str) -> Diagnostic {
        let text = &self.source[span.clone()];
        // A quoted name cannot hold a double quote, so no fix exists then.
        let fix = (!text.contains('"')).then(|| Fix {
            span: span.clone(),
            text: format!("\"{text}\""),
        });
        Diagnostic {
            fix,
            ..diagnostic(
                "E0004",
                span,
                format!(
                    "`{text}` holds a character that a bare {what} cannot hold. Write the {what} in double quotes."
                ),
            )
        }
    }

    fn missing(&self, what: &str) -> Diagnostic {
        let index = self.skip(self.pos);
        let span = self.span_at(index);
        let place = match self.kind_at(index) {
            Some(TokenKind::Unclosed) => return self.unexpected(index),
            None | Some(TokenKind::Newline) => "at the end of the line".to_owned(),
            Some(_) => format!("before `{}`", &self.source[span.clone()]),
        };
        diagnostic(
            "E0006",
            span,
            format!("A {what} is missing {place}. Write the {what} there."),
        )
    }

    fn unexpected(&self, index: usize) -> Diagnostic {
        let span = self.span_at(index);
        let text = &self.source[span.clone()];
        match self.kind_at(index) {
            Some(TokenKind::Unclosed) => diagnostic(
                "E0005",
                span,
                "A double quote opens here and does not close. Add the closing `\"` on the same line.",
            ),
            Some(TokenKind::Arrow | TokenKind::Cross) => diagnostic(
                "E0001",
                span,
                format!(
                    "A line holds one arrow, and this `{text}` is a second one. Write each link on its own line. A name that holds `{text}` is written in double quotes."
                ),
            ),
            _ => diagnostic(
                "E0001",
                span,
                format!(
                    "Cannot read `{text}` here. Write one fact on a line: a name, a link with `->`, or a name with braces."
                ),
            ),
        }
    }
}
