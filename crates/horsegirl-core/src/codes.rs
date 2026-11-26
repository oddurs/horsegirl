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
        summary: "A token stands where the line cannot hold it: an arrow after a block, a `@` that is not a citation, or a word after the end of a fact.",
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
        summary: "A name, a value or a label holds a character that only a text in double quotes can hold. The fix adds the quotes. A bare text can hold a double quote, but a text in double quotes cannot. So a text that holds a double quote and also needs quotes gets no fix, and the message says to remove the double quote or the character that needs the quotes.",
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
        code: "E0010",
        summary: "A link has a second arrow, as in `A -> B -> C`. The fix writes each link on its own line with the same indent. It is safe when the line ends at the last name, and suggested when the line ends in a label, a `?` or a citation, which the fix gives to each link.",
    },
    Code {
        code: "E0300",
        summary: "A `show` line names a lens that does not exist. The message lists the lenses, and a lens one edit from a known lens gets a safe fix.",
    },
    Code {
        code: "E0301",
        summary: "A `show` line picks a lens that needs facts the file does not state, such as `show flow` in a file with no link. The message names the missing facts.",
    },
    Code {
        code: "E0302",
        summary: "A `show` line names a lens of the language card that this version does not draw, so `draw` gives no picture. The message names the lenses this version draws.",
    },
    Code {
        code: "E0412",
        summary: "A name is written once and is one edit from a name written more often, so it is likely a typo that makes a new thing. The safe fix writes the other name. When more than one such name is one edit away, the message names each and there is no fix. Two names are not a typo when they differ only in a final word that is a number, one letter or a version such as `v2` (`Service A` and `Service B`, `Billing v1` and `Billing v2`), or when both are shorter than three characters.",
    },
    Code {
        code: "E0420",
        summary: "A name in braces is not placed in their group: the thing is in another group already, the group is the thing or inside it, or the braces follow several names. A thing is in one group. The suggested fix removes the name when the line names other things and says nothing else. It is never safe, because only the writer knows whether the two mentions mean one thing.",
    },
    Code {
        code: "E0500",
        summary: "A link breaks a `-x` rule in the same direction.",
    },
    Code {
        code: "N0101",
        summary: "A line is written the way Mermaid writes it: `-->`, quotes that a name does not need, a space before the colon of a label, or a `;` at the end. The fix writes the canonical form.",
    },
    Code {
        code: "N0200",
        summary: "A link or a rule ends in `?` in every statement, so it is a guess.",
    },
    Code {
        code: "W0010",
        summary: "A story is parsed, and this version does not order its links.",
    },
    Code {
        code: "W0107",
        summary: "A thing has no link and no rule, so it stands alone in every picture. A link or a rule that names its group, at any depth, counts as its own. A link from a group to itself or to a part of it does not count, because `W0110` reports it and the picture leaves it out. A group needs no link of its own.",
    },
    Code {
        code: "W0108",
        summary: "A name that ends in `/` is followed by braces that hold one bare name, as in `GET /users/{id}`. The parser reads a group, and the writer likely meant one name. The suggested fix writes the whole text in double quotes.",
    },
    Code {
        code: "W0109",
        summary: "A label holds an arrow, a name and a colon, as in `A -> B: pay -> C: ship`. The parser reads one link with a long label, and the writer likely meant two links. The suggested fix writes the second link on its own line. A label with an arrow and no colon after it, and a label in double quotes, give no warning.",
    },
    Code {
        code: "W0110",
        summary: "A link joins a group to itself or to a thing inside it, at any depth. A picture draws a group as a box around its parts, so the flow lens leaves the link out. A link between a group and a thing outside it is drawn, and meets the border of the group box.",
    },
    Code {
        code: "W0111",
        summary: "A bare name ends the line and ends in `?` with no space, as in `A -> Archive?`. Only a `?` on its own marks a guess, so the line states a firm link to a thing named `Archive?`. The safe fix adds the space and makes the link a guess. A name that must end in `?` is written in double quotes, as in `\"Archive?\"`. When the file writes the same name with its `?` where the `?` cannot be a guess, as in `Archive? -> Store` or `\"Archive?\"`, the name is a name and there is no warning.",
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
