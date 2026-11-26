//! Tokens to the tree.
//!
//! Every function appends what it reads to a list of children before it can
//! fail, so an error never drops a token and the tree stays lossless.

use std::ops::Range;

use super::lexer::lex;
use super::{Confidence, Diagnostic, Edit, Element, Fix, Node, NodeKind, Parse, Token, TokenKind};
use crate::check::LENSES;

/// Braces nest this deep and no deeper. The parser calls itself once for each
/// level, so a file of ten thousand `{` must not reach the end of the stack.
const MAX_DEPTH: usize = 64;

/// A message quotes this many characters of the source and no more. A line of
/// a megabyte with an error in each fact must not give a megabyte for each.
const MAX_QUOTE: usize = 60;

/// A fix quotes a text of this many bytes and no more. A longer text gets the
/// message and no fix, so the cost of one diagnostic does not grow with its
/// line. Braces that do not pair the way the parser reads them can make one
/// text of most of a line, and each fix once held a copy of it.
const MAX_FIX: usize = 200;

type Children<'s> = Vec<Element<'s>>;

pub(super) fn parse(source: &str) -> Parse<'_> {
    let tokens = lex(source);
    let mut parser = Parser {
        source,
        closes: closes(&tokens),
        tokens,
        pos: 0,
        last: None,
        habits: Vec::new(),
        story: None,
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
    parser.diagnostics.sort_by_key(|d| (d.span.start, d.code));
    drop_overlapping_fixes(&mut parser.diagnostics);
    Parse {
        tree: Node {
            kind: NodeKind::File,
            children,
        },
        diagnostics: parser.diagnostics,
    }
}

/// Takes the fix off each diagnostic whose fix overlaps the fix of an earlier
/// one, so that a tool can apply every fix of one parse.
///
/// The earlier fix is the outer one: in `A { note: x {B@y} }` the value and
/// the block inside it each need quotes. Quotes around the whole value repair
/// both, and the inner fix would cut into them.
fn drop_overlapping_fixes(diagnostics: &mut [Diagnostic]) {
    let mut end = 0;
    for diagnostic in diagnostics {
        let Some(fix) = &diagnostic.fix else {
            continue;
        };
        let span = fix.span();
        if span.start < end {
            diagnostic.fix = None;
        } else {
            end = span.end;
        }
    }
}

/// For each `{`, the index of the `}` that closes it on the same line. The
/// braces of a line are matched once here, so no later question about a brace
/// reads the rest of its line.
fn closes(tokens: &[Token<'_>]) -> Vec<Option<usize>> {
    let mut closes = vec![None; tokens.len()];
    let mut open = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Newline => open.clear(),
            TokenKind::Open => open.push(index),
            TokenKind::Close => {
                if let Some(start) = open.pop() {
                    closes[start] = Some(index);
                }
            }
            _ => {}
        }
    }
    closes
}

/// The start of `text` for a message, with `...` in place of the rest when
/// the text is long.
fn quote(text: &str) -> String {
    match text.char_indices().nth(MAX_QUOTE) {
        Some((cut, _)) => format!("{}...", &text[..cut]),
        None => text.to_owned(),
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
}

/// A text that double quotes can hold.
#[derive(Clone, Copy, PartialEq)]
enum Text {
    Name,
    Value,
    Label,
}

impl Text {
    fn word(self) -> &'static str {
        match self {
            Text::Name => "name",
            Text::Value => "value",
            Text::Label => "label",
        }
    }
}

struct Parser<'s> {
    source: &'s str,
    tokens: Vec<Token<'s>>,
    /// The `}` that closes each `{` on its line. See `closes`.
    closes: Vec<Option<usize>>,
    pos: usize,
    /// The first token of the last text that the current fact read, and what
    /// the text is. A `@` or an arrow that cannot be read is a part of it.
    last: Option<(usize, Text)>,
    /// The edits that turn the Mermaid habits of the current fact into the
    /// canonical form. See `note_habits`.
    habits: Vec<Edit>,
    /// The W0010 note of the story line that is being read.
    story: Option<Diagnostic>,
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

    /// Whether an arrow or `-x` is on the line of the token at `index`, in the
    /// same block. In `A -> B {x}` it is, and in `X { A -> B } Team {x}` and
    /// `A -> X { Team {x} }` the arrow is in another block than `{x}`.
    fn in_link(&self, index: usize) -> bool {
        let mut start = index;
        while start > 0 && self.kind_at(start - 1) != Some(TokenKind::Newline) {
            start -= 1;
        }
        let step = |depth: usize, kind: TokenKind| match kind {
            TokenKind::Open => depth + 1,
            TokenKind::Close => depth.saturating_sub(1),
            _ => depth,
        };
        let depth = self.tokens[start..index]
            .iter()
            .fold(0, |depth, token| step(depth, token.kind));
        let mut found = false;
        let mut current = 0;
        for (at, token) in self.tokens.iter().enumerate().skip(start) {
            let kind = token.kind;
            if kind == TokenKind::Newline {
                break;
            }
            // The end of the block ends the statement.
            let ends = current < depth;
            if ends && at > index {
                break;
            }
            if ends {
                found = false;
            } else if current == depth && matches!(kind, TokenKind::Arrow | TokenKind::Cross) {
                found = true;
            }
            current = step(current, kind);
        }
        found
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
                None | Some(
                    TokenKind::Newline | TokenKind::At | TokenKind::Close | TokenKind::Semi
                )
            )
    }

    fn at_line_end(&self) -> bool {
        matches!(
            self.next(),
            None | Some(TokenKind::Newline | TokenKind::Close | TokenKind::Semi)
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
        self.last = None;
        let start = self.pos;
        let outer = std::mem::take(&mut self.habits);
        let mut habits = None;
        let fact = self.fact(depth, &mut children);
        // Taken before the recovery below reads lines of its own.
        let story = self.story.take();
        let kind = match fact {
            Ok(kind) => {
                // A story line that fails after its block states no story.
                self.diagnostics.extend(story);
                habits = Some(kind);
                kind
            }
            Err(error) => {
                // `open` stops at the `{` that nests too deep. The `{` pairs
                // with its `}` whatever follows it on the line, so that the
                // `}` does not close the group around it.
                let too_deep = error.code == "E0007";
                self.diagnostics.push(error);
                if too_deep && self.kind() == Some(TokenKind::Open) {
                    self.skip_block(&mut children);
                }
                loop {
                    match self.kind() {
                        None | Some(TokenKind::Newline | TokenKind::Close) => break,
                        // Any other `{` is a stray mark of the bad line, as in
                        // `A -> B: uses {braces`. A block for it would swallow
                        // the good lines after it.
                        Some(TokenKind::Open) if self.opens_block(start) => {
                            if depth < MAX_DEPTH {
                                self.block(depth, &mut children);
                            } else {
                                self.skip_block(&mut children);
                            }
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
        let edits = std::mem::replace(&mut self.habits, outer);
        // A line that failed to read states nothing, so it has no canonical form.
        if let Some(kind) = habits {
            self.note_habits(start, kind, edits);
        }
        into.push(Element::Node(Node { kind, children }));
    }

    fn fact(&mut self, depth: usize, into: &mut Children<'s>) -> Result<NodeKind, Diagnostic> {
        let start = self.pos;
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
                self.cite(into)?;
            }
            kind
        };
        if self.next() == Some(TokenKind::Semi) {
            // Mermaid ends a line with `;`. The mark is no part of a text.
            let mut before = self.pos;
            while before > 0 && self.kind_at(before - 1) == Some(TokenKind::Space) {
                before -= 1;
            }
            let semi = self.span_at(self.skip(self.pos));
            self.habits.push(Edit {
                span: self.span_at(before).start..semi.end,
                text: String::new(),
            });
            self.spaces(into);
            self.bump(into);
        }
        if self.at_line_end() {
            return Ok(kind);
        }
        let index = self.skip(self.pos);
        let arrow = matches!(
            self.kind_at(index),
            Some(TokenKind::Arrow | TokenKind::Cross)
        );
        Err(match kind {
            NodeKind::Link | NodeKind::Rule if arrow => self.chain(start, index),
            // No arrow came before this one, so a `@` or a brace ended the
            // names too early.
            NodeKind::Things if arrow => self.cut_short(index),
            // A link cannot hold a group, so a `{` after the last name is in
            // that name, as in `A -> GET /users/{id}`.
            NodeKind::Link | NodeKind::Rule if self.kind_at(index) == Some(TokenKind::Open) => {
                self.cut_short(index)
            }
            // A double quote after a bare word, as in `A "b"`, is in the text.
            _ if self.kind_at(index) == Some(TokenKind::Quoted) => self.cut_short(index),
            _ => self.unexpected(index),
        })
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
                let at = self.pos;
                self.bump(into);
                if arrow == TokenKind::Arrow && self.tokens[at].text != "->" {
                    self.habits.push(Edit {
                        span: self.tokens[at].span(),
                        text: "->".to_owned(),
                    });
                }
                self.spaces(into);
                let from = into.len();
                self.list(false, into)?;
                wrap(into, from, NodeKind::Names);
                if self.next() == Some(TokenKind::Colon) {
                    if self.kind() == Some(TokenKind::Space) {
                        self.habits.push(Edit {
                            span: self.tokens[self.pos].span(),
                            text: String::new(),
                        });
                    }
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
                // `note: x, @y` is one value with a comma in it, as `facts`
                // reads `note: x, y`.
                if let Some(value) = entries.last().and_then(|entry: &Entry| entry.value)
                    && self.kind() == Some(TokenKind::At)
                {
                    return Err(self.quotes_or_unexpected(value, Text::Value, self.pos));
                }
                return Err(self.missing_text(Text::Name));
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
                    return Err(self.missing_text(Text::Value));
                }
                wrap(into, from, NodeKind::Fact);
            }
            entries.push(Entry { start, value });
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
        let text = if kind == NodeKind::Value {
            Text::Value
        } else {
            Text::Name
        };
        let start = self.pos;
        match self.kind() {
            Some(TokenKind::Quoted) => {
                let token = self.tokens[self.pos];
                let inside = &token.text[1..token.text.len() - 1];
                // A bare name joins what follows it, so the quotes stay unless a
                // space, the line end or a reserved character comes next:
                // `"a-"->B` would become the arrow `-->`.
                let parted = self.source[token.span().end..]
                    .chars()
                    .next()
                    .is_none_or(|c| " \n\r;:,{}".contains(c));
                if kind == NodeKind::Name && parted && bare_is_safe(inside) {
                    self.habits.push(Edit {
                        span: token.span(),
                        text: inside.to_owned(),
                    });
                }
                self.bump(into);
            }
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
        self.last = Some((start, text));
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
            return Err(self.quotes_or_unexpected(value, Text::Value, self.skip(self.pos)));
        }
        let Some(name) = entries.iter().position(|entry| entry.value.is_none()) else {
            return Ok(());
        };
        // `note: install, list` reads as a fact and a name. The author meant
        // one value with a comma in it. `extent` runs the value on to the next
        // fact, as in `note: install, list {x}, owner: X`.
        let value = name.checked_sub(1).and_then(|fact| entries[fact].value);
        let Some(value) = value else {
            return Err(diagnostic(
                "E0009",
                self.span_at(entries[name].start).start..self.here(),
                "This line mixes names and facts. Write the names on one line and the facts on another line.",
            ));
        };
        let end = self.extent(value, Text::Value);
        Err(self.needs_quotes(
            self.span_at(value).start..self.span_at(end - 1).end,
            Text::Value,
        ))
    }

    /// Whether a fact starts at the token at `index`: a name and a colon.
    fn starts_fact(&self, index: usize) -> bool {
        let mut index = self.skip(index);
        match self.kind_at(index) {
            Some(TokenKind::Quoted) => index += 1,
            Some(TokenKind::Word) => {
                while self.kind_at(self.skip(index + 1)) == Some(TokenKind::Word) {
                    index = self.skip(index + 1);
                }
                index += 1;
            }
            _ => return false,
        }
        self.kind_at(self.skip(index)) == Some(TokenKind::Colon)
    }

    /// The error for `key: value` outside braces. The colon is in a name when
    /// an arrow or a brace follows it, and the line is a stray fact otherwise.
    /// The name ends at a comma, and the scan for the arrow goes on past it:
    /// `Sensors: wind, B -> Hub` is a link.
    fn colon_outside_braces(&self, entries: &[Entry]) -> Diagnostic {
        let first = entries.last().map_or(self.pos, |entry| entry.start);
        let start = self.span_at(first).start;
        let mut end = self.here();
        let mut named = false;
        let mut index = self.pos;
        loop {
            match self.kind_at(index) {
                Some(TokenKind::Arrow | TokenKind::Cross) => {
                    return self.needs_quotes(start..end, Text::Name);
                }
                // `a: b{c} -> D`: the braces are characters of the name, so
                // the quotes go around them too.
                Some(TokenKind::Open) if let Some(close) = self.name_braces(index) => {
                    if !named {
                        end = self.span_at(close).end;
                    }
                    index = close;
                }
                Some(TokenKind::Open) => {
                    // `story s: x {` is a story whose name needs quotes. The
                    // keyword stays outside them.
                    let name = self.after_story(first).filter(|name| *name < self.pos);
                    let start = name.map_or(start, |name| self.span_at(name).start);
                    return self.needs_quotes(start..end, Text::Name);
                }
                None | Some(TokenKind::Newline | TokenKind::Close | TokenKind::Semi) => break,
                Some(TokenKind::At) if !named => break,
                Some(TokenKind::Comma) => named = true,
                Some(TokenKind::Space) => {}
                Some(_) if !named => end = self.span_at(index).end,
                Some(_) => {}
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
        self.last = Some((first, Text::Label));
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
                    | TokenKind::Semi
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
            return Err(self.quotes_or_unexpected(first, Text::Label, self.skip(self.pos)));
        }
        if end == first {
            let mut found = self.missing("label");
            // An unclosed quote has its own error, and it has its own advice.
            if found.code == "E0006" {
                found.message.push_str(" Or remove the colon.");
            }
            return Err(found);
        }
        Ok(())
    }

    /// The index after the citation that starts at `index`. A citation is the
    /// form on the card: a space, `@`, and a path that ends in `:` and a line
    /// number. Any other `@` is a character of a name, and needs quotes.
    fn cite_end(&self, index: usize) -> Option<usize> {
        let spaced =
            index.checked_sub(1).and_then(|before| self.kind_at(before)) == Some(TokenKind::Space);
        if self.kind_at(index) != Some(TokenKind::At) || !spaced {
            return None;
        }
        let path = self.skip(index + 1);
        let end = self.path_end(path);
        // A path that meets an unclosed quote is no citation. `cite` says why.
        if self.kind_at(end) == Some(TokenKind::Unclosed) {
            return None;
        }
        let text = &self.source[self.span_at(path).start..self.span_at(end).start];
        // An odd count of quotes means one quote has no partner. Paired quotes
        // stay part of the path. The check reads the path and no more, so it
        // stays linear on a line of many `@`.
        if text.bytes().filter(|&byte| byte == b'"').count() % 2 == 1 {
            return None;
        }
        let (file, line) = text.rsplit_once(':')?;
        let numbered = !line.is_empty() && line.bytes().all(|byte| byte.is_ascii_digit());
        (numbered && !file.is_empty()).then_some(end)
    }

    /// The index where a citation path that starts at `path` stops. The path
    /// stops at a second `@`, so a row of them is read once. It also stops
    /// before an unclosed quote, which holds the rest of the line, so the scan
    /// reads each token once.
    fn path_end(&self, path: usize) -> usize {
        let mut end = path;
        while !matches!(
            self.kind_at(end),
            None | Some(
                TokenKind::Space
                    | TokenKind::Newline
                    | TokenKind::Open
                    | TokenKind::Close
                    | TokenKind::At
                    | TokenKind::Semi
                    | TokenKind::Unclosed
            )
        ) {
            end += 1;
        }
        end
    }

    /// The span from the last double quote inside the path tokens `from..to` to the
    /// end of its line, when the path holds an odd count of double quotes. An even
    /// count means every quote has a partner, so none is unclosed. The lexer reads a quote
    /// inside a word as a character of the word, so no `Unclosed` token marks
    /// it. The scan runs once, on the line that gives the error.
    fn stray_quote(&self, from: usize, to: usize) -> Option<std::ops::Range<usize>> {
        let start = self.span_at(from).start;
        let end = self.span_at(to).start;
        let path = &self.source[start..end];
        if path.matches('"').count().is_multiple_of(2) {
            return None;
        }
        let quote = start + path.rfind('"')?;
        let line = self.source[quote..]
            .find('\n')
            .map_or(self.source.len(), |n| quote + n);
        Some(quote..line)
    }

    /// Whether the fact ends at `index`: only a guess mark and a citation
    /// follow on the line.
    fn ends_fact(&self, index: usize) -> bool {
        let mut index = self.skip(index);
        if self.is_word(index, "?") {
            index = self.skip(index + 1);
        }
        if let Some(end) = self.cite_end(index) {
            index = self.skip(end);
        }
        matches!(
            self.kind_at(index),
            None | Some(TokenKind::Newline | TokenKind::Close | TokenKind::Semi)
        )
    }

    /// Reads the `@` after a fact as a citation, or says what the `@` is.
    fn cite(&mut self, into: &mut Children<'s>) -> Result<(), Diagnostic> {
        let at = self.skip(self.pos);
        let Some(end) = self.cite_end(at) else {
            let stop = self.path_end(self.skip(at + 1));
            if self.kind_at(stop) == Some(TokenKind::Unclosed) {
                return Err(self.unexpected(stop));
            }
            if let Some(span) = self.stray_quote(self.skip(at + 1), stop) {
                return Err(diagnostic("E0005", span, UNCLOSED));
            }
            let alone = matches!(
                self.kind_at(self.skip(at + 1)),
                None | Some(TokenKind::Newline | TokenKind::Close)
            );
            if !alone {
                return Err(self.cut_short(at));
            }
            return Err(diagnostic(
                "E0001",
                self.span_at(at),
                "`@` cites evidence and nothing follows it. Write a path and a line after it, as in `@ src/app.ts:17`. A name that holds `@` is written in double quotes.",
            ));
        };
        self.spaces(into);
        let from = into.len();
        while self.pos < end {
            self.bump(into);
        }
        wrap(into, from, NodeKind::Cite);
        Ok(())
    }

    /// The error for the second arrow of a link, at `second`. The fix writes
    /// each link of the chain from token `start` on its own line.
    fn chain(&self, start: usize, second: usize) -> Diagnostic {
        let span = self.span_at(second);
        let text = quote(&self.source[span.clone()]);
        Diagnostic {
            fix: self.split_chain(start),
            ..diagnostic(
                "E0010",
                span,
                format!(
                    "A line holds one arrow, and this `{text}` is a second one. Write each link on its own line. A name that holds `{text}` is written in double quotes."
                ),
            )
        }
    }

    /// The fix for a chain of links that starts at token `start`, when the
    /// chain is plain enough to split: names, commas and arrows, and at the
    /// end a label, a guess mark or a citation. That end goes on each link.
    /// Each name list must read the same at the start and at the end of a
    /// line.
    fn split_chain(&self, start: usize) -> Option<Fix> {
        let line = self.source[..self.span_at(start).start]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        let indent = &self.source[line..self.span_at(start).start];
        // A chain inside a one-line block cannot get a line of its own.
        if !indent.chars().all(|c| c == ' ' || c == '\t') {
            return None;
        }
        // The first token and the bytes of each name list. An arrow starts
        // the next one.
        let mut lists: Vec<Option<(usize, Range<usize>)>> = vec![None];
        let mut index = start;
        loop {
            match self.kind_at(index) {
                Some(TokenKind::Arrow | TokenKind::Cross) => lists.push(None),
                Some(TokenKind::Word) if self.is_guess(index) => break,
                Some(TokenKind::Word | TokenKind::Quoted | TokenKind::Comma) => {
                    let span = self.span_at(index);
                    if let Some(list) = lists.last_mut() {
                        let (first, from) = list
                            .as_ref()
                            .map_or((index, span.start), |(first, list)| (*first, list.start));
                        *list = Some((first, from..span.end));
                    }
                }
                Some(TokenKind::Space) => {}
                _ => break,
            }
            index += 1;
        }
        let lists: Vec<(usize, Range<usize>)> = lists.into_iter().collect::<Option<_>>()?;
        // A name list in the middle ends one link and starts the next. At the
        // start of a line `#` makes a comment, and `show`, a lens and `of`,
        // `from` or `by` make a `show` line. At the end of a line a `?` is a
        // guess mark and a `;` is a mark. The split would change what such a
        // list means.
        let mut middle = lists.iter().skip(1).take(lists.len().saturating_sub(2));
        if middle.any(|(first, list)| {
            let text = &self.source[list.clone()];
            text.starts_with('#')
                || (self.is_word(*first, "show") && self.show_argument(*first).is_some())
                || text.ends_with(';')
                || text.rsplit([' ', '\t', ',']).next() == Some("?")
        }) {
            return None;
        }
        let names: Vec<Range<usize>> = lists.into_iter().map(|(_, list)| list).collect();
        // What follows the last name: a label, a guess mark and a citation.
        let mut end = index;
        while !matches!(
            self.kind_at(end),
            None | Some(TokenKind::Newline | TokenKind::Semi | TokenKind::Close)
        ) {
            end += 1;
        }
        let rest = &self.tokens[index..end];
        let plain = match self.kind_at(index) {
            Some(TokenKind::Colon) => {
                let label = self.skip(index + 1);
                self.extent(label, Text::Label) > label
                    && !rest
                        .iter()
                        .any(|token| matches!(token.kind, TokenKind::Open | TokenKind::Unclosed))
            }
            _ => self.ends_fact(index),
        };
        if !plain {
            return None;
        }
        let last = names.last()?.end;
        let tail_end = rest
            .iter()
            .rev()
            .find(|token| token.kind != TokenKind::Space)
            .map_or(last, |token| token.span().end);
        let tail = &self.source[last..tail_end];
        let span = self.span_at(start).start..last + tail.len();
        if span.len() > MAX_FIX {
            return None;
        }
        let links: Vec<String> = names
            .windows(2)
            .map(|pair| format!("{}{tail}", &self.source[pair[0].start..pair[1].end]))
            .collect();
        let confidence = if tail.is_empty() {
            Confidence::Safe
        } else {
            Confidence::Suggested
        };
        Some(Fix {
            confidence,
            description: if tail.is_empty() {
                "Write each link on its own line.".to_owned()
            } else {
                "Write each link on its own line, and give each link the end of the chain: its label, guess mark or citation. Remove that end from each link it does not belong to.".to_owned()
            },
            edits: vec![Edit {
                span,
                text: links.join(&format!("\n{indent}")),
            }],
        })
    }

    /// The error for the token at `index`, which ended a bare text too early.
    fn cut_short(&self, index: usize) -> Diagnostic {
        let Some((mut first, text)) = self.last else {
            return self.unexpected(index);
        };
        if self.kind_at(first) == Some(TokenKind::Quoted) {
            // The text has its quotes already, so the token is the mistake.
            if self.kind_at(index) != Some(TokenKind::At) {
                return self.unexpected(index);
            }
            return diagnostic(
                "E0001",
                self.span_at(index),
                "This `@` is not a citation. A citation is a space, `@`, a path and a line number at the end of the line, as in ` @ src/app.ts:17`. A name that holds `@` has the `@` inside its double quotes.",
            );
        }
        // `story s@x {` is a story whose name needs quotes. The keyword stays
        // outside them.
        let end = self.extent(first, text);
        if text == Text::Name
            && let Some(name) = self.after_story(first)
            && name < end
            && self.kind_at(self.skip(end)) == Some(TokenKind::Open)
        {
            first = name;
        }
        let mut diagnostic = self.quotes_or_unexpected(first, text, index);
        if diagnostic.code == "E0004" && self.source[diagnostic.span.clone()].contains('@') {
            diagnostic.message.push_str(
                " A citation is a space, `@`, a path and a line number at the end of the line, as in ` @ src/app.ts:17`.",
            );
        }
        diagnostic
    }

    /// The index of the token after the keyword `story` and its space, when
    /// the token at `first` is that keyword at the start of a fact.
    fn after_story(&self, first: usize) -> Option<usize> {
        let name = self.skip(first + 1);
        (self.is_word(first, "story") && self.starts_line(first) && name > first + 1)
            .then_some(name)
    }

    /// Whether the token at `index` is the first of its fact.
    fn starts_line(&self, index: usize) -> bool {
        let before = self.tokens[..index]
            .iter()
            .rev()
            .find(|token| token.kind != TokenKind::Space);
        before.is_none_or(|token| matches!(token.kind, TokenKind::Newline | TokenKind::Open))
    }

    /// The error for the token at `index`, which stopped the text that starts
    /// at token `first`. Quotes repair the line only when the text holds a
    /// character that a bare text cannot hold. Without one, as in a name
    /// before a group, the token is reported as it is.
    fn quotes_or_unexpected(&self, first: usize, text: Text, index: usize) -> Diagnostic {
        let end = self.extent(first, text);
        let reserved = |token: &Token<'_>| match token.kind {
            TokenKind::At | TokenKind::Open | TokenKind::Close => true,
            // A label holds these as they are.
            TokenKind::Comma | TokenKind::Colon | TokenKind::Arrow | TokenKind::Cross => {
                text != Text::Label
            }
            _ => false,
        };
        let tokens = &self.tokens[first..end];
        // A text in quotes with nothing after it is whole, and the token
        // after it is the mistake.
        if let [token] = tokens
            && token.kind == TokenKind::Quoted
        {
            return self.unexpected(index);
        }
        // A double quote in a longer text is a mistake of its own, which
        // `needs_quotes` names.
        let quoted = |token: &Token<'_>| token.kind == TokenKind::Quoted;
        if !tokens.iter().any(reserved) && !tokens.iter().any(quoted) {
            return self.unexpected(index);
        }
        let span = self.span_at(first).start..self.span_at(end - 1).end;
        self.needs_quotes(span, text)
    }

    /// The error for a name or a value that is not there. A `@` with a word
    /// after it is the start of a name that needs quotes.
    fn missing_text(&mut self, text: Text) -> Diagnostic {
        let word = self.kind_at(self.pos + 1) == Some(TokenKind::Word);
        if self.kind() == Some(TokenKind::At) && word && self.cite_end(self.pos).is_none() {
            self.last = Some((self.pos, text));
            return self.cut_short(self.pos);
        }
        self.missing(text.word())
    }

    /// A line that starts with `show` is a `show` line unless it holds an
    /// arrow, a comma, a colon or a brace. Then `show` is a word of a name.
    /// `show`, a lens and `of`, `from` or `by` always start a `show` line, so
    /// a reserved character after them is a mistake in the argument.
    fn is_show_line(&self) -> bool {
        if self.show_argument(self.pos).is_some() {
            return true;
        }
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

    /// The first token after `show`, a lens and `of`, `from` or `by`, when the
    /// tokens from `show`, at index `at`, start that way.
    fn show_argument(&self, at: usize) -> Option<usize> {
        let lens = at + 2;
        let spaced = |index: usize| self.kind_at(index) == Some(TokenKind::Space);
        let is_lens =
            self.kind_at(lens) == Some(TokenKind::Word) && LENSES.contains(&self.tokens[lens].text);
        if !spaced(at + 1) || !is_lens || !spaced(lens + 1) {
            return None;
        }
        let word = self.skip(lens + 1);
        let joins = ["of", "from", "by"]
            .iter()
            .any(|join| self.is_word(word, join));
        // A word token is whole, so a comma, a colon or a brace straight after
        // the join word is a mistake in the argument, not a longer word.
        joins.then(|| self.skip(word + 1))
    }

    fn show(&mut self, into: &mut Children<'s>) -> Result<NodeKind, Diagnostic> {
        let start = self.pos;
        let argument = self.show_argument(self.pos);
        if self.show_parts(into) {
            return Ok(NodeKind::Show);
        }
        if let Some(first) = argument
            && let Some(error) = self.bad_argument(first)
        {
            return Err(error);
        }
        Err(diagnostic(
            "E0008",
            self.span_at(start).start..self.span_at(self.extent(start, Text::Label) - 1).end,
            "A `show` line is `show` and a lens, as in `show flow`, or with `of`, `from` or `by` and a name, as in `show sequence of checkout`. A thing with the name `show` is written `\"show\"`.",
        ))
    }

    /// The error for an argument of a `show` line that holds a comma, or a
    /// character that only a name in double quotes can hold.
    fn bad_argument(&self, first: usize) -> Option<Diagnostic> {
        // A label runs to the end of the line and keeps the braces that close
        // on it, so the span holds the whole argument.
        let end = self.extent(first, Text::Label);
        if end <= first {
            return None;
        }
        let tokens = &self.tokens[first..end];
        let span = self.span_at(first).start..self.span_at(end - 1).end;
        if tokens.iter().any(|token| token.kind == TokenKind::Comma) {
            return Some(diagnostic(
                "E0008",
                span,
                "A `show` line takes one name after `of`, `from` or `by`. Write one `show` line for each name. A name that holds a comma is written in double quotes.",
            ));
        }
        let reserved = tokens.iter().any(|token| {
            matches!(
                token.kind,
                TokenKind::At
                    | TokenKind::Open
                    | TokenKind::Close
                    | TokenKind::Colon
                    | TokenKind::Arrow
                    | TokenKind::Cross
            )
        });
        // A quoted name with more after it is not cured by more quotes.
        let quoted = self.kind_at(first) == Some(TokenKind::Quoted);
        (reserved && !quoted).then(|| self.needs_quotes(span, Text::Name))
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
        // `statement` reports the note when the whole line reads.
        self.story = Some(diagnostic(
            "W0010",
            start..end,
            format!(
                "Stories arrive in v0.2. This version parses the story `{}` and does not order its links. You do not need to change the file.",
                quote(&self.source[name..end])
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

    /// Whether the `{` at the current token opens a block of the bad line
    /// that starts at token `start`. A label or a value cannot open a group,
    /// so a `{` in one is a stray mark, as in `A -> B: uses {braces`, unless it
    /// closes on its line or ends the line. A `{` in any other place opens a
    /// block, as in `G@x { A`, because quotes repair that line into a group.
    fn opens_block(&self, start: usize) -> bool {
        let stray = matches!(self.last, Some((_, Text::Label | Text::Value)))
            || self.after_link_colon(start);
        !stray
            || self.closes[self.pos].is_some()
            || matches!(
                self.kind_at(self.skip(self.pos + 1)),
                None | Some(TokenKind::Newline)
            )
    }

    /// Whether the line that starts at token `start` has, before the current
    /// token, an arrow and then a colon outside its blocks: the current token
    /// is then in the label of a link.
    fn after_link_colon(&self, start: usize) -> bool {
        let mut depth = 0usize;
        let mut arrow = false;
        for token in &self.tokens[start..self.pos] {
            match token.kind {
                TokenKind::Open => depth += 1,
                TokenKind::Close => depth = depth.saturating_sub(1),
                TokenKind::Arrow | TokenKind::Cross if depth == 0 => arrow = true,
                TokenKind::Colon if depth == 0 && arrow => return true,
                _ => {}
            }
        }
        false
    }

    /// Reads a block that nests too deep as plain tokens, up to the `}` that
    /// closes it. Each `}` inside closes a `{` inside, so the `}` after the
    /// block still closes the group around it. The loop does not call itself,
    /// so any depth fits on the stack.
    fn skip_block(&mut self, into: &mut Children<'s>) {
        let open = self.span_at(self.pos);
        let mut depth = 0;
        while let Some(kind) = self.kind() {
            self.bump(into);
            match kind {
                TokenKind::Open => depth += 1,
                TokenKind::Close => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                return;
            }
        }
        self.diagnostics.push(diagnostic(
            "E0002",
            open,
            "This `{` never closes. Add a `}` on the line after the last part.",
        ));
    }

    /// Reads `{`, the lines inside, and `}`.
    fn block(&mut self, depth: usize, into: &mut Children<'s>) {
        let open = self.span_at(self.pos);
        let from = into.len();
        // The lines inside are facts of their own. The text before the block
        // stays the last text of this fact.
        let last = self.last;
        self.bump(into);
        self.lines(depth + 1, into);
        self.last = last;
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

    /// The `}` of the braces that open at `index`, when those braces are
    /// characters of a name: they hold one word and an arrow is on the line.
    fn name_braces(&self, index: usize) -> Option<usize> {
        let close = self.closes[index]?;
        (close == index + 2
            && self.kind_at(index + 1) == Some(TokenKind::Word)
            && self.in_link(index))
        .then_some(close)
    }

    /// The index after the last token of the text that starts at token
    /// `from`. The text ends at the end of the line, at the `}` that closes
    /// the block around it, or at the guess mark and the citation that end the
    /// fact. A name ends at a comma or an arrow. A value ends at a comma that
    /// a fact follows: in `note: install, list` the comma is in the value.
    ///
    /// A `{` is a character of the text only when it closes on its line. In
    /// a name, the braces are characters only when the name is in a link and
    /// the braces hold one word and no space, as in `A -> GET /users/{id}`.
    /// A link cannot hold a group, so such braces can be nothing else. Any
    /// other braces are a group, as in `Team@x{A}`, `Team@x{ A }` and
    /// `Services { Orders }`, and quotes would make one thing of the group
    /// and lose its parts. So `Services { Orders } -> Postgres` gets no
    /// quotes. A label ends before a `{` that follows a space and holds a
    /// comma, a colon or an arrow.
    fn extent(&self, from: usize, text: Text) -> usize {
        let mut end = from;
        let mut index = from;
        loop {
            let kind = match self.kind_at(index) {
                None | Some(TokenKind::Newline | TokenKind::Close | TokenKind::Semi) => {
                    return end;
                }
                Some(kind) => kind,
            };
            let stops = match kind {
                TokenKind::Comma => match text {
                    Text::Name => true,
                    Text::Value => self.starts_fact(index + 1),
                    Text::Label => false,
                },
                TokenKind::Arrow | TokenKind::Cross => text == Text::Name,
                // After an arrow, a colon starts the label, so it is not a
                // character of the target name. Before an arrow it is, as in
                // `a: b{c} -> D`.
                TokenKind::Colon => text == Text::Name && self.after_arrow(from),
                TokenKind::Open => match self.closes[index] {
                    None => true,
                    Some(close) => {
                        let spaced =
                            index > from && self.kind_at(index - 1) == Some(TokenKind::Space);
                        let block = match text {
                            Text::Value => false,
                            Text::Name => self.name_braces(index).is_none(),
                            Text::Label => {
                                spaced
                                    && self.tokens[index..close].iter().any(|token| {
                                        matches!(
                                            token.kind,
                                            TokenKind::Comma
                                                | TokenKind::Colon
                                                | TokenKind::Arrow
                                                | TokenKind::Cross
                                        )
                                    })
                            }
                        };
                        if !block {
                            // The braces are matched, so the text goes on
                            // after the `}` and reads no token twice.
                            index = close;
                        }
                        block
                    }
                },
                _ => (kind == TokenKind::At || self.is_word(index, "?")) && self.ends_fact(index),
            };
            if stops {
                return end;
            }
            if kind != TokenKind::Space {
                end = index + 1;
            }
            index += 1;
        }
    }

    /// Whether an arrow comes before token `from` on its line, in the same
    /// braces.
    fn after_arrow(&self, from: usize) -> bool {
        self.tokens[..from]
            .iter()
            .rev()
            .take_while(|token| !matches!(token.kind, TokenKind::Newline | TokenKind::Open))
            .any(|token| matches!(token.kind, TokenKind::Arrow | TokenKind::Cross))
    }

    /// Says that a fact is written in a habit of Mermaid, and what the canonical
    /// line is. The note has a safe fix, and the file still reads as it is.
    fn note_habits(&mut self, start: usize, kind: NodeKind, edits: Vec<Edit>) {
        if edits.is_empty() {
            return;
        }
        // A group states its parts on the lines inside its braces. Those lines
        // have their own notes.
        let stop = if kind == NodeKind::Things {
            (start..self.pos)
                .find(|&index| self.kind_at(index) == Some(TokenKind::Open))
                .unwrap_or(self.pos)
        } else {
            self.pos
        };
        let last = (start..stop)
            .rev()
            .find(|&index| {
                !matches!(
                    self.kind_at(index),
                    Some(TokenKind::Space | TokenKind::Newline)
                )
            })
            .unwrap_or(start);
        // An edit can lie after the last token, such as the `;` after a group's `}`.
        let end = edits
            .iter()
            .map(|edit| edit.span.end)
            .fold(self.span_at(last).end, usize::max);
        let span = self.span_at(start).start..end;
        let mut canonical = String::new();
        let mut from = span.start;
        for edit in &edits {
            canonical.push_str(&self.source[from..edit.span.start]);
            canonical.push_str(&edit.text);
            from = edit.span.end;
        }
        canonical.push_str(&self.source[from..span.end]);
        let message = format!(
            "`{}` is written the way Mermaid writes it. horsegirl reads it, and the canonical form is `{}`.",
            quote(&self.source[span.clone()]),
            quote(&canonical)
        );
        self.diagnostics.push(Diagnostic {
            fix: Some(Fix {
                confidence: Confidence::Safe,
                description: "Write the line in the canonical form.".to_owned(),
                edits,
            }),
            ..diagnostic("N0101", span, message)
        });
    }

    fn needs_quotes(&self, span: Range<usize>, text: Text) -> Diagnostic {
        let what = text.word();
        let text = &self.source[span.clone()];
        // A text in double quotes cannot hold a double quote, so advice to add
        // quotes would ask for a form the language does not have. A bare text
        // holds one, so only the reserved character makes it a mistake.
        if text.contains('"') {
            return diagnostic(
                "E0004",
                span,
                format!(
                    "`{}` holds a character that a bare {what} cannot hold, so the {what} needs double quotes, and a text in double quotes cannot hold a double quote. Remove each double quote from the {what}, or remove the character that needs the quotes, and run check again.",
                    quote(text)
                ),
            );
        }
        // A text longer than `MAX_FIX` gets no fix.
        // In `A -x@b` or `show rings from{id}` the quote would touch the word
        // before it and make one word of the two, so a space goes between them.
        let touches = self.source[..span.start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let space = if touches { " " } else { "" };
        let fix = (text.len() <= MAX_FIX).then(|| Fix {
            confidence: Confidence::Safe,
            description: format!("Write the {what} in double quotes."),
            edits: vec![Edit {
                span: span.clone(),
                text: format!("{space}\"{text}\""),
            }],
        });
        let text = quote(text);
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
        let mut span = self.span_at(index);
        let place = match self.kind_at(index) {
            Some(TokenKind::Unclosed) => return self.unexpected(index),
            None | Some(TokenKind::Newline) => {
                // The span is the place where the text goes, and not the line
                // break after it.
                span.end = span.start;
                "at the end of the line".to_owned()
            }
            Some(_) => format!("before `{}`", quote(&self.source[span.clone()])),
        };
        diagnostic(
            "E0006",
            span,
            format!("A {what} is missing {place}. Write the {what} there."),
        )
    }

    fn unexpected(&self, index: usize) -> Diagnostic {
        let span = self.span_at(index);
        let text = quote(&self.source[span.clone()]);
        match self.kind_at(index) {
            Some(TokenKind::Unclosed) => diagnostic("E0005", span, UNCLOSED),
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

const UNCLOSED: &str =
    "A double quote opens here and does not close. Add the closing `\"` on the same line.";

/// Whether a name that holds `text` reads the same without double quotes. The
/// reader gives a bare name back with one space between words, so `text` must
/// already be written that way. It also must not hold a mark that a bare name
/// cannot, or a word that a bare line reads as something else.
fn bare_is_safe(text: &str) -> bool {
    let plain = text.split(' ').all(|word| {
        !word.is_empty()
            && !word
                .contains(|c: char| c.is_whitespace() || c == '\u{feff}' || ",:{}@\";".contains(c))
            && !matches!(word, "?" | "-x")
            && !word.contains("->")
    });
    plain
        && !text.starts_with('#')
        && !text.starts_with("show")
        && !text.starts_with("story")
        && !text.contains("->")
}
