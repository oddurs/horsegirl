//! The golden tests: each `.horse` file in `tests/golden` against the files
//! that hold what it must give.
//!
//! A case is one `.horse` file. Beside it, `<case>.tree` holds the tree that
//! the parser reads and `<case>.diagnostics.json` holds the diagnostics. The
//! expectations are text with one fact on each line, so a change to the parser
//! or to a diagnostic shows as a line diff in review.
//!
//! To add a case or accept a change, write the `.horse` file and run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden
//! ```
//!
//! The command writes every expectation again and removes the expectation of
//! a case that is gone. Read the diff before you commit it.

// The WebAssembly tests get no directory to read. The syntax tests hold the
// reference file on that target.
#![cfg(not(target_family = "wasm"))]

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use horsegirl_core::syntax::{Diagnostic, parse};

const CASES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BLESS: &str = "HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden";

/// The extensions of the expectations, in the order that a case writes them.
const OUTPUTS: [&str; 2] = ["tree", "diagnostics.json"];

/// Each expectation of `source`, with the extension of its file.
fn outputs(source: &str) -> [(&'static str, String); 2] {
    let parse = parse(source);
    assert_eq!(parse.tree.to_string(), source, "the tree lost a byte");
    [
        (OUTPUTS[0], format!("{:?}", parse.tree)),
        (OUTPUTS[1], diagnostics_json(source, &parse.diagnostics)),
    ]
}

/// The diagnostics as JSON, one field on each line.
///
/// Each diagnostic gives its position as a line and a column as well as a byte
/// span, and quotes the text that it is about, so a reviewer reads the diff
/// without counting bytes.
fn diagnostics_json(source: &str, diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return "[]\n".to_owned();
    }
    let objects: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let (line, column) = position(source, d.span.start);
            let fix = match &d.fix {
                None => "null".to_owned(),
                Some(fix) => format!(
                    "{{\n      \"span\": [{}, {}],\n      \"text\": {}\n    }}",
                    fix.span.start,
                    fix.span.end,
                    string(&fix.text),
                ),
            };
            [
                format!("\"code\": {}", string(d.code)),
                format!("\"line\": {line}"),
                format!("\"column\": {column}"),
                format!("\"span\": [{}, {}]", d.span.start, d.span.end),
                format!("\"text\": {}", string(&source[d.span.clone()])),
                format!("\"message\": {}", string(&d.message)),
                format!("\"fix\": {fix}"),
            ]
            .join(",\n    ")
        })
        .map(|fields| format!("  {{\n    {fields}\n  }}"))
        .collect();
    format!("[\n{}\n]\n", objects.join(",\n"))
}

/// The line and the column of a byte offset, both from 1. A column counts
/// characters, not bytes.
fn position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.matches('\n').count() + 1;
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, before[start..].chars().count() + 1)
}

/// A JSON string.
fn string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                write!(out, "\\u{:04x}", u32::from(c)).expect("a String takes any write");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The path of the expectation of `case` with `extension`.
fn expectation(case: &Path, extension: &str) -> PathBuf {
    case.with_extension(extension)
}

/// The first line where `expected` and `actual` differ, as a short report.
fn first_difference(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    for number in 1.. {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(e), Some(a)) if e == a => {}
            (None, None) => break,
            (e, a) => {
                return format!(
                    "line {number}\n    expected: {}\n    actual:   {}",
                    e.unwrap_or("(end of file)"),
                    a.unwrap_or("(end of file)"),
                );
            }
        }
    }
    "the line endings differ".to_owned()
}

#[test]
fn every_case_gives_its_expected_tree_and_diagnostics() {
    let bless = std::env::var_os("HORSEGIRL_BLESS").is_some_and(|v| v == "1");
    let mut files: Vec<PathBuf> = fs::read_dir(CASES)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();

    let cases: Vec<&PathBuf> = files
        .iter()
        .filter(|path| path.extension().is_some_and(|e| e == "horse"))
        .collect();
    assert!(!cases.is_empty(), "no case in {CASES}");

    let mut failures = Vec::new();
    let mut expected_files = BTreeSet::new();
    for case in &cases {
        let source = fs::read_to_string(case).unwrap();
        for (extension, actual) in outputs(&source) {
            let path = expectation(case, extension);
            expected_files.insert(path.clone());
            if bless {
                fs::write(&path, &actual).unwrap();
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy();
            match fs::read_to_string(&path) {
                Ok(expected) if expected == actual => {}
                Ok(expected) => failures.push(format!(
                    "{name} changed at {}",
                    first_difference(&expected, &actual)
                )),
                Err(_) => failures.push(format!("{name} is missing")),
            }
        }
    }

    // An expectation with no case is left over from a case that was removed or
    // renamed. It would otherwise sit in the tree and mislead a reader.
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let is_expectation = OUTPUTS.iter().any(|e| name.ends_with(&format!(".{e}")));
        if is_expectation && !expected_files.contains(path) {
            if bless {
                fs::remove_file(path).unwrap();
            } else {
                failures.push(format!("{name} has no case"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} golden expectation(s) differ:\n  {}\n\nIf the change is intended, run:\n  {BLESS}\nand read the diff before you commit it.",
        failures.len(),
        failures.join("\n  "),
    );
}

#[test]
fn a_json_string_escapes_quotes_backslashes_and_control_characters() {
    assert_eq!(
        string("a \"b\" \\ c\n\t\u{1}é"),
        r#""a \"b\" \\ c\n\t\u0001é""#
    );
}

#[test]
fn a_column_counts_characters_from_the_start_of_its_line() {
    assert_eq!(position("ab\néé x", 0), (1, 1));
    assert_eq!(position("ab\néé x", 3), (2, 1));
    assert_eq!(position("ab\néé x", 8), (2, 4));
}

#[test]
fn the_first_difference_names_the_line_and_both_texts() {
    let report = first_difference("a\nb\n", "a\nc\nd\n");
    assert_eq!(report, "line 2\n    expected: b\n    actual:   c");
    let report = first_difference("a\n", "a\nb\n");
    assert!(report.contains("(end of file)"), "{report}");
}
