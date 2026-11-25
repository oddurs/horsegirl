//! Every diagnostic code, and what it means.
//!
//! A code is stable: it keeps its meaning in every release, and a code that
//! goes out of use is not given to a new mistake. The catalogue in
//! `docs/diagnostics.md` is made from this list, and a test fails when the
//! source uses a code that the list does not hold.

/// One code and what it means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Code {
    /// `E`, `W` or `N` and four digits.
    pub code: &'static str,
    /// What the code reports, in one sentence.
    pub summary: &'static str,
}

/// Every code, sorted.
pub const CATALOGUE: &[Code] = &[
    Code {
        code: "E0001",
        summary: "A token stands where the line cannot hold it: a second arrow, a `@` that is not a citation, or a word after the end of a fact.",
    },
    Code {
        code: "E0002",
        summary: "A `{` never closes.",
    },
    Code {
        code: "E0003",
        summary: "A `}` closes nothing.",
    },
    Code {
        code: "E0004",
        summary: "A name, a value or a label holds a character that only a text in double quotes can hold. The fix adds the quotes.",
    },
    Code {
        code: "E0005",
        summary: "A double quote opens and does not close on its line.",
    },
    Code {
        code: "E0006",
        summary: "A name or a value is missing. At the end of a line, the span is empty.",
    },
    Code {
        code: "E0007",
        summary: "Braces nest deeper than the parser reads.",
    },
    Code {
        code: "E0008",
        summary: "A `show` line is not `show` and a lens, with `of`, `from` or `by` and a name after it.",
    },
    Code {
        code: "E0009",
        summary: "A fact stands outside braces, or a line mixes names and facts.",
    },
    Code {
        code: "W0010",
        summary: "A story is parsed, and this version does not order its links.",
    },
];

/// The catalogue as a Markdown table, as `docs/diagnostics.md` holds it.
#[must_use]
pub fn catalogue_markdown() -> String {
    let rows = CATALOGUE.iter().map(|entry| {
        let severity = crate::diagnostic::Severity::of(entry.code)
            .map_or("unknown", crate::diagnostic::Severity::word);
        format!("| `{}` | {severity} | {} |\n", entry.code, entry.summary)
    });
    let header = "| Code | Severity | Meaning |\n| --- | --- | --- |\n".to_owned();
    std::iter::once(header).chain(rows).collect()
}
