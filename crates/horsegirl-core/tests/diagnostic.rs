//! The two forms of a diagnostic, held by snapshots, and the catalogue of codes.

use horsegirl_core::check::check;
use horsegirl_core::codes::{CATALOGUE, catalogue_markdown};
use horsegirl_core::diagnostic::{
    Confidence, Diagnostic, Edit, Fix, LineIndex, Severity, apply_safe,
};
use horsegirl_core::syntax::parse;

const MISTAKES: &str = include_str!("diagnostics/mistakes.horse");
const MISTAKES_JSON: &str = include_str!("diagnostics/mistakes.jsonl");
const MISTAKES_TEXT: &str = include_str!("diagnostics/mistakes.txt");
const DOC: &str = include_str!("../../../docs/diagnostics.md");

fn json_lines(source: &str) -> String {
    let lines = LineIndex::new(source);
    let parse = parse(source);
    let json = parse.diagnostics.iter().map(|d| d.to_json(&lines) + "\n");
    json.collect()
}

fn text(source: &str, name: &str) -> String {
    let lines = LineIndex::new(source);
    let parse = parse(source);
    let text = parse.diagnostics.iter().map(|d| d.render(&lines, name));
    text.collect::<Vec<_>>().join("\n")
}

/// Compares `got` with a snapshot. `HORSEGIRL_BLESS=1` writes `got` to the
/// snapshot instead, and the diff in review shows what moved.
fn snapshot(name: &str, expected: &str, got: &str) {
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        let path = format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(path, got).unwrap();
        return;
    }
    assert!(
        got == expected,
        "{name} moved; run with HORSEGIRL_BLESS=1 to accept it. It is now:\n{got}"
    );
}

#[test]
fn the_json_form_is_held_by_a_snapshot() {
    snapshot(
        "tests/diagnostics/mistakes.jsonl",
        MISTAKES_JSON,
        &json_lines(MISTAKES),
    );
}

#[test]
fn the_text_form_is_held_by_a_snapshot() {
    snapshot(
        "tests/diagnostics/mistakes.txt",
        MISTAKES_TEXT,
        &text(MISTAKES, "mistakes.horse"),
    );
}

#[test]
fn the_documented_examples_are_what_the_code_prints() {
    let source = "Web app -> API@v2\n";
    let line = json_lines(source);
    assert!(
        DOC.contains(line.trim_end()),
        "docs/diagnostics.md does not show the line:\n{line}"
    );
    let text = text(source, "example.horse");
    assert!(
        DOC.contains(&text),
        "docs/diagnostics.md does not show the text:\n{text}"
    );
}

#[test]
fn the_text_form_marks_the_span_under_the_source_line() {
    let lines = LineIndex::new("Café -> API@v2\n");
    let diagnostic = &parse("Café -> API@v2\n").diagnostics[0];
    let text = diagnostic.render(&lines, "a.horse");
    // The column counts characters, so `é` before the span is one column and
    // one space of padding.
    assert!(text.contains("--> a.horse:1:9\n"), "{text}");
    assert!(
        text.contains("1 | Café -> API@v2\n  |         ^^^^^^\n"),
        "{text}"
    );
}

#[test]
fn a_character_of_two_bytes_inside_the_span_gets_one_mark() {
    let source = "Web -> Café@v2\n";
    let diagnostic = &parse(source).diagnostics[0];
    let text = diagnostic.render(&LineIndex::new(source), "a.horse");
    // `Café@v2` is seven characters in eight bytes.
    assert!(
        text.contains("1 | Web -> Café@v2\n  |        ^^^^^^^\n"),
        "{text}"
    );
}

#[test]
fn a_tab_before_the_span_stays_a_tab_under_it() {
    let source = "G {\n\tA -> B@c\n}\n";
    let diagnostic = &parse(source).diagnostics[0];
    let text = diagnostic.render(&LineIndex::new(source), "a.horse");
    assert!(text.contains("2 | \tA -> B@c\n  | \t     ^^^\n"), "{text}");
}

// The span of a missing name at the end of a line is empty. It is the place
// where the name goes, and not the line break after it.
#[test]
fn a_mistake_at_the_end_of_a_line_has_an_empty_span_and_one_mark() {
    for source in ["A ->\nB\n", "A ->\r\nB\r\n", "A ->"] {
        let diagnostic = &parse(source).diagnostics[0];
        assert_eq!(diagnostic.code, "E0006");
        assert_eq!(diagnostic.span, 4..4, "{source:?}");
        let text = diagnostic.render(&LineIndex::new(source), "a.horse");
        assert!(text.contains("1 | A ->\n  |     ^\n"), "{source:?}: {text}");
    }
}

// From the review of item 23: two fixes of one parse overlapped, and a tool
// that applied both corrupted the line.
#[test]
fn when_two_fixes_overlap_the_outer_one_is_kept() {
    let source = "A { B @ x } -> C\n";
    let diagnostics = parse(source).diagnostics;
    let codes: Vec<_> = diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(codes, ["E0004", "E0004"]);
    let fix = diagnostics[0].fix.as_ref().expect("the outer fix is kept");
    assert_eq!(fix.apply(source), "\"A { B @ x }\" -> C\n");
    assert_eq!(diagnostics[1].fix, None);
    assert_eq!(parse(&fix.apply(source)).diagnostics, []);
}

#[test]
fn json_strings_are_escaped_as_serde_json_escapes_them() {
    let diagnostic = Diagnostic {
        code: "E0001",
        span: 0..0,
        message: "\"\\\n\r\t\u{8}\u{c}\u{1}\u{7f}é/".to_owned(),
        fix: None,
    };
    let json = diagnostic.to_json(&LineIndex::new(""));
    assert!(
        json.contains(r#""message":"\"\\\n\r\t\b\f\u0001"#),
        "{json}"
    );
    assert!(json.contains("\u{7f}é/\""), "{json}");
}

#[test]
fn every_code_in_the_catalogue_is_well_formed_and_listed_once() {
    let mut last = "";
    for entry in CATALOGUE {
        let (letter, digits) = entry.code.split_at(1);
        assert!(Severity::of(letter).is_some(), "{}", entry.code);
        assert!(
            digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()),
            "{}",
            entry.code
        );
        assert!(entry.code > last, "{} is out of order or twice", entry.code);
        last = entry.code;
    }
}

#[test]
fn the_catalogue_in_the_docs_is_generated_from_the_code() {
    let start = "<!-- catalogue: start -->\n";
    let end = "<!-- catalogue: end -->";
    let from = DOC.find(start).expect("the start marker") + start.len();
    let to = DOC.find(end).expect("the end marker");
    let got = catalogue_markdown();
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/diagnostics.md");
        std::fs::write(path, format!("{}{got}{}", &DOC[..from], &DOC[to..])).unwrap();
        return;
    }
    assert!(
        DOC[from..to] == got,
        "the catalogue in docs/diagnostics.md moved; run with HORSEGIRL_BLESS=1. It is now:\n{got}"
    );
}

/// Each string literal in `text` that has the form of a code.
fn codes_in(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut codes = Vec::new();
    for (index, window) in bytes.windows(7).enumerate() {
        let quoted = window[0] == b'"' && window[6] == b'"';
        let letter = matches!(window[1], b'E' | b'W' | b'N');
        if quoted && letter && window[2..6].iter().all(u8::is_ascii_digit) {
            codes.push(&text[index + 1..index + 6]);
        }
    }
    codes
}

#[cfg(not(target_family = "wasm"))]
fn sources(directory: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

// The WebAssembly tests get no directory to read. The scan is the same on
// every target, so the native run is enough.
#[cfg(not(target_family = "wasm"))]
#[test]
fn every_code_in_the_source_is_in_the_catalogue() {
    let crates = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(crates).unwrap() {
        sources(&entry.unwrap().path().join("src"), &mut paths);
    }
    let mut seen = 0;
    // The catalogue holds every code by definition, so it proves nothing.
    for path in paths.iter().filter(|path| !path.ends_with("src/codes.rs")) {
        let text = std::fs::read_to_string(path).unwrap();
        for code in codes_in(&text) {
            seen += 1;
            assert!(
                CATALOGUE.iter().any(|entry| entry.code == code),
                "{} uses {code}, and the catalogue in src/codes.rs does not list it",
                path.display()
            );
        }
    }
    // The scan found the codes of the parser, so it read the right files.
    assert!(seen > 0, "the scan found no code");
}

#[test]
fn the_scan_finds_a_code_in_a_string_and_nothing_else() {
    let text = r#"diagnostic("E0042", span) "W1234" E0001 "E001" "X0001" "N0003""#;
    assert_eq!(codes_in(text), ["E0042", "W1234", "N0003"]);
}

/// A diagnostic at `span` whose fix writes `text` there.
fn fixable(span: std::ops::Range<usize>, text: &str, confidence: Confidence) -> Diagnostic {
    Diagnostic {
        code: "E0412",
        span: span.clone(),
        message: String::new(),
        fix: Some(Fix {
            confidence,
            description: String::new(),
            edits: vec![Edit {
                span,
                text: text.to_owned(),
            }],
        }),
    }
}

#[test]
fn apply_safe_makes_every_safe_fix_and_changes_nothing_else() {
    // A typo, a Mermaid habit, a lens one edit off and a name that needs
    // quotes, each with a safe fix, among lines that need none.
    let source = "# keep this\nWeb app -> Payments\nPayments -> Bank\nOrders -> Paymnts\nA --> B\nC -> API@v2\nshow flwo\n";
    let diagnostics = check(source);
    let safe = diagnostics
        .iter()
        .filter(|d| {
            d.fix
                .as_ref()
                .is_some_and(|f| f.confidence == Confidence::Safe)
        })
        .count();
    assert_eq!(safe, 4, "{diagnostics:#?}");
    let fixed = apply_safe(source, &diagnostics);
    assert_eq!(fixed.applied, 4);
    assert_eq!(
        fixed.text,
        "# keep this\nWeb app -> Payments\nPayments -> Bank\nOrders -> Payments\nA -> B\nC -> \"API@v2\"\nshow flow\n"
    );
    let left = check(&fixed.text);
    assert!(left.iter().all(|d| d.fix.is_none()), "{left:#?}");
}

#[test]
fn apply_safe_leaves_a_suggested_fix_and_an_overlapping_fix() {
    let source = "abcdef";
    let diagnostics = [
        fixable(0..2, "X", Confidence::Safe),
        fixable(1..3, "Y", Confidence::Safe),
        fixable(4..5, "Z", Confidence::Suggested),
        fixable(5..6, "W", Confidence::Safe),
    ];
    let fixed = apply_safe(source, &diagnostics);
    assert_eq!(fixed.text, "XcdeW");
    assert_eq!(fixed.applied, 2);
}

#[test]
fn apply_safe_with_no_fix_returns_the_source() {
    let source = "A -> B\nC\n";
    let fixed = apply_safe(source, &check(source));
    assert_eq!(fixed.text, source);
    assert_eq!(fixed.applied, 0);
}
