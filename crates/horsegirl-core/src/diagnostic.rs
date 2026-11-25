//! What the tool says about a file, and the two forms it says it in.
//!
//! A diagnostic holds byte offsets into the source. The two forms turn them
//! into lines and columns: a text block for a person at a terminal, and one
//! JSON object on one line for an agent. `docs/diagnostics.md` states the JSON
//! shape, and the catalogue in `codes` lists every code.

use std::ops::Range;

/// How bad a diagnostic is. The first letter of its code says the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// `E`: the file does not mean what it says until it changes.
    Error,
    /// `W`: the file is read, and the tool says what it did with it.
    Warning,
    /// `N`: a remark that asks for no change.
    Note,
}

impl Severity {
    /// The severity of a code, from its first letter.
    #[must_use]
    pub fn of(code: &str) -> Option<Severity> {
        match code.as_bytes().first() {
            Some(b'E') => Some(Severity::Error),
            Some(b'W') => Some(Severity::Warning),
            Some(b'N') => Some(Severity::Note),
            _ => None,
        }
    }

    /// The word for the severity in both forms.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// Whether a tool may apply a fix with no question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    /// `fix` applies it.
    Safe,
    /// An agent decides.
    Suggested,
}

impl Confidence {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Confidence::Safe => "safe",
            Confidence::Suggested => "suggested",
        }
    }
}

/// One replacement of a part of the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// The bytes of the original source to replace.
    pub span: Range<usize>,
    /// The new text. An empty text deletes the span.
    pub text: String,
}

/// The exact change that repairs a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    pub confidence: Confidence,
    /// What the fix does, in one sentence.
    pub description: String,
    /// Sorted by start, and no two overlap.
    pub edits: Vec<Edit>,
}

impl Fix {
    /// The bytes of the source that the fix changes, from the start of its
    /// first edit to the end of its last.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        let start = self.edits.first().map_or(0, |edit| edit.span.start);
        let end = self.edits.last().map_or(0, |edit| edit.span.end);
        start..end
    }

    /// `source` with the edits of this fix made.
    #[must_use]
    pub fn apply(&self, source: &str) -> String {
        let mut out = String::with_capacity(source.len());
        let mut from = 0;
        for edit in &self.edits {
            out.push_str(&source[from..edit.span.start]);
            out.push_str(&edit.text);
            from = edit.span.end;
        }
        out.push_str(&source[from..]);
        out
    }
}

/// Something the tool has to say about the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// A stable code: `E`, `W` or `N` and four digits. `codes::CATALOGUE`
    /// lists each one.
    pub code: &'static str,
    /// The bytes of the source that the diagnostic is about. An empty span is
    /// a place between two characters, such as the end of a line.
    pub span: Range<usize>,
    pub message: String,
    pub fix: Option<Fix>,
}

impl Diagnostic {
    #[must_use]
    pub fn severity(&self) -> Severity {
        // The catalogue test holds every code to one of the three letters.
        Severity::of(self.code).unwrap_or(Severity::Note)
    }

    /// The diagnostic as one JSON object with no line break.
    ///
    /// Keys are sorted by name and nothing is indented, so the bytes match
    /// what `serde_json` prints for the same value. The MCP server (0038)
    /// reads this text into its value and gives the same bytes.
    #[must_use]
    pub fn to_json(&self, lines: &LineIndex<'_>) -> String {
        let fix = self.fix.as_ref().map_or_else(
            || "null".to_owned(),
            |fix| {
                let edits: Vec<String> = fix
                    .edits
                    .iter()
                    .map(|edit| {
                        format!(
                            "{{\"span\":{},\"text\":{}}}",
                            json_span(lines, &edit.span),
                            json_string(&edit.text)
                        )
                    })
                    .collect();
                format!(
                    "{{\"confidence\":{},\"description\":{},\"edits\":[{}]}}",
                    json_string(fix.confidence.word()),
                    json_string(&fix.description),
                    edits.join(",")
                )
            },
        );
        format!(
            "{{\"code\":{},\"fix\":{fix},\"message\":{},\"severity\":{},\"span\":{}}}",
            json_string(self.code),
            json_string(&self.message),
            json_string(self.severity().word()),
            json_span(lines, &self.span)
        )
    }

    /// The diagnostic as text for a terminal: the message, the place, the
    /// source line with the span marked under it, and the fix.
    ///
    /// `name` is what the place calls the file, such as its path.
    #[must_use]
    pub fn render(&self, lines: &LineIndex<'_>, name: &str) -> String {
        let start = lines.position(self.span.start);
        let number = start.line.to_string();
        let gutter = " ".repeat(number.len());
        let text = lines.line(start.line);
        // The marks stop at the end of the first line: a line break is not
        // drawn, and a span over lines is marked on the line where it starts.
        let line_start = lines.start_of(start.line);
        let from = (lines.floor(self.span.start) - line_start).min(text.len());
        let to = (lines.floor(self.span.end.max(self.span.start)) - line_start).min(text.len());
        let (before, marked) = (&text[..from], &text[from..to]);
        // A tab before the span stays a tab, so the marks line up under it.
        let pad: String = before
            .chars()
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        // An empty span, such as the end of a line, gets one mark.
        let marks = "^".repeat(marked.chars().count().max(1));
        let fix = self.fix.as_ref().map_or_else(String::new, |fix| {
            format!(
                "{gutter} = {} fix: {}\n",
                fix.confidence.word(),
                fix.description
            )
        });
        format!(
            "{}[{}]: {}\n{gutter}--> {name}:{}:{}\n{gutter} |\n{number} | {text}\n{gutter} | {pad}{marks}\n{fix}",
            self.severity().word(),
            self.code,
            self.message,
            start.line,
            start.column,
        )
    }
}

/// A place in the source as a person counts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    /// From 1.
    pub line: usize,
    /// From 1, in Unicode scalar values.
    pub column: usize,
}

/// Where each line of a source starts, so that a byte offset becomes a line
/// and a column without a scan from the start of the file.
#[derive(Clone, Debug)]
pub struct LineIndex<'s> {
    source: &'s str,
    /// The byte offset of the start of each line. Only a line feed ends a line.
    starts: Vec<usize>,
}

impl<'s> LineIndex<'s> {
    #[must_use]
    pub fn new(source: &'s str) -> LineIndex<'s> {
        let breaks = source
            .bytes()
            .enumerate()
            .filter(|&(_, byte)| byte == b'\n')
            .map(|(index, _)| index + 1);
        LineIndex {
            source,
            starts: std::iter::once(0).chain(breaks).collect(),
        }
    }

    /// The line and the column of a byte offset. An offset past the end
    /// counts as the end.
    #[must_use]
    pub fn position(&self, offset: usize) -> Position {
        let offset = self.floor(offset);
        let line = self.starts.partition_point(|&start| start <= offset);
        let column = self.source[self.starts[line - 1]..offset].chars().count() + 1;
        Position { line, column }
    }

    /// The byte offset where line `line` starts, counted from 1.
    fn start_of(&self, line: usize) -> usize {
        self.starts[line - 1]
    }

    /// The text of line `line`, counted from 1, with no line break.
    fn line(&self, line: usize) -> &'s str {
        let start = self.start_of(line);
        let end = self
            .starts
            .get(line)
            .map_or(self.source.len(), |&next| next - 1);
        let text = &self.source[start..end];
        text.strip_suffix('\r').unwrap_or(text)
    }

    /// The offset, moved back to the end of the source or to the start of the
    /// character that holds it.
    fn floor(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }
}

fn json_span(lines: &LineIndex<'_>, span: &Range<usize>) -> String {
    let start = lines.position(span.start);
    let end = lines.position(span.end);
    format!(
        "{{\"end\":{{\"column\":{},\"line\":{}}},\"start\":{{\"column\":{},\"line\":{}}}}}",
        end.column, end.line, start.column, start.line
    )
}

/// A JSON string, escaped as `serde_json` escapes it: the quote, the
/// backslash and the control characters, and nothing else.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < ' ' => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let byte = c as usize;
                out.push_str("\\u00");
                out.push(char::from(HEX[byte >> 4]));
                out.push(char::from(HEX[byte & 0xf]));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
