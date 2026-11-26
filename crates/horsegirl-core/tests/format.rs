//! The formatter keeps the model, settles on one form, and reads its own
//! output.

use horsegirl_core::format::{format, unquotable_name};
use horsegirl_core::model::{Model, lower};
use horsegirl_core::syntax::{Element, Node, TokenKind, parse};

/// What a file states, with every choice of layout removed. Source bytes are
/// cleared, and the facts of a thing are in the order of their keys, because
/// the formatter sorts them. The sort is stable, as in the formatter. Two files
/// with one projection state the same things.
fn model(source: &str) -> Model {
    let mut model = lower(&parse(source).tree);
    for thing in &mut model.things {
        thing.mentions.clear();
        thing.facts.sort_by(|a, b| a.key.cmp(&b.key));
    }
    for link in &mut model.links {
        link.span = 0..0;
    }
    for rule in &mut model.rules {
        rule.span = 0..0;
    }
    for show in &mut model.shows {
        show.span = 0..0;
    }
    for misplaced in &mut model.misplaced {
        misplaced.span = 0..0;
        misplaced.removable = misplaced.removable.as_ref().map(|_| 0..0);
    }
    model
}

fn comments(node: &Node<'_>, found: &mut Vec<String>) {
    for child in &node.children {
        match child {
            Element::Node(inner) => comments(inner, found),
            Element::Token(token) if token.kind == TokenKind::Comment => {
                found.push(token.text.trim_end().to_owned());
            }
            Element::Token(_) => {}
        }
    }
}

/// Checks the three promises on one source that has no error. Returns the
/// formatted text.
fn promises(source: &str) -> String {
    let once = format(source).unwrap_or_else(|e| panic!("{e:?} in {source:?}"));
    let before = parse(source);
    let after = parse(&once);
    assert!(
        after.diagnostics.iter().all(|d| !d.code.starts_with('E')),
        "the output has an error: {:?} in {once:?} from {source:?}",
        after.diagnostics
    );
    assert_eq!(
        model(source),
        model(&once),
        "the model changed in {source:?} to {once:?}"
    );
    let (mut kept, mut now) = (Vec::new(), Vec::new());
    comments(&before.tree, &mut kept);
    comments(&after.tree, &mut now);
    assert_eq!(kept, now, "a comment changed");
    assert_eq!(
        format(&once).as_deref(),
        Ok(once.as_str()),
        "not idempotent on {source:?}"
    );
    // A file with no canonical form stays as it is, with its own line breaks.
    // The layout rules hold for every other file.
    if unquotable_name(source).is_some() {
        assert_eq!(once, source, "a file with no canonical form changed");
    } else {
        assert!(once.is_empty() || once.ends_with('\n') && !once.ends_with("\n\n"));
        assert!(!once.contains('\r'));
    }
    once
}

#[test]
fn a_formatted_file_does_not_change() {
    let source = include_str!("../../../examples/checkout.horse");
    assert_eq!(format(source).unwrap(), source);
}

#[test]
fn the_layout_settles_on_one_form() {
    let cases = [
        ("A->B", "A -> B\n"),
        ("A   ->   B :  pay  now", "A -> B: pay now\n"),
        ("A,B,C  -x  D", "A, B, C -x D\n"),
        ("A -x B: reason", "A -x B: reason\n"),
        ("A -> B ?", "A -> B ?\n"),
        ("  Web   app  \r\n\r\n\r\n\r\nB\r\n", "Web app\n\nB\n"),
        ("\n\nA\n\n\n", "A\n"),
        ("", ""),
        ("A {\n  owner: X\n}", "A { owner: X }\n"),
        ("A{owner:X,  kind:y}", "A { kind: y, owner: X }\n"),
        ("A { z: 1, a: 2, z: 0 }", "A { a: 2, z: 1, z: 0 }\n"),
        ("A {\n  b: 1\n  a: 2\n}", "A {\n  b: 1\n  a: 2\n}\n"),
        ("G { A, B }", "G { A, B }\n"),
        ("G {\n\n  A\n\n\n  B\n\n}", "G {\n  A\n\n  B\n}\n"),
        ("G { A -> B }", "G {\n  A -> B\n}\n"),
        ("G { H { A } }", "G {\n  H { A }\n}\n"),
        ("G {\n# why\nA\n}", "G {\n  # why\n  A\n}\n"),
        ("A {}", "A {}\n"),
        ("G {# x\nB\n}", "G {\n  \"# x\"\n  B\n}\n"),
        ("G {#a\"b\nB\n}\n", "G {#a\"b\nB\n}\n"),
        ("A { k: x\"y ?, a: b }\n", "A { k: x\"y ?, a: b }\n"),
        ("A -> B   ?   @   src/a.ts:17", "A -> B ? @ src/a.ts:17\n"),
        ("A ? ", "A ?\n"),
        ("A -> B  : save", "A -> B: save\n"),
        ("A -> B:  x  \"a   b\"  y", "A -> B: x \"a   b\" y\n"),
        ("story s {\nA -> B\n}", "story s {\n  A -> B\n}\n"),
        ("show   sequence   of   s", "show sequence of s\n"),
        ("show flow", "show flow\n"),
        ("  # keep\t\nA", "# keep\nA\n"),
    ];
    for (source, expected) in cases {
        assert_eq!(promises(source), expected, "{source:?}");
    }
}

// ADR-119: quotes come off a name that reads the same without them.
#[test]
fn quotes_stay_only_where_a_name_needs_them() {
    let cases = [
        ("\"Web app\" -> \"B\"", "Web app -> B\n"),
        ("\"Sensors: wind\" -> B", "\"Sensors: wind\" -> B\n"),
        ("A { note: \"plain\" }", "A { note: plain }\n"),
        (
            "A { note: \"install, list\" }",
            "A { note: \"install, list\" }\n",
        ),
        ("\"a -> b\"", "\"a -> b\"\n"),
        ("\"\"", "\"\"\n"),
        ("\"#tag\" -> B", "\"#tag\" -> B\n"),
        ("\"show me\"", "\"show me\"\n"),
        ("\"story x\" -> B", "\"story x\" -> B\n"),
        ("A -> \"-x\"", "A -> \"-x\"\n"),
        ("A -> \"B ?\"", "A -> \"B ?\"\n"),
        ("A -x \"B\"", "A -x B\n"),
        (
            "A { owner: X ?, kind: y }",
            "A { kind: y, owner: \"X ?\" }\n",
        ),
        ("\"  a   b \"", "\"  a   b \"\n"),
        ("\"a  b\" -> \"a b\"", "\"a  b\" -> a b\n"),
        ("\"Web   app\" -> B", "\"Web   app\" -> B\n"),
        ("\" Web\" -> B", "\" Web\" -> B\n"),
        ("A -> B: \"a   b\"", "A -> B: \"a   b\"\n"),
        ("A { note: \"x   y\" }", "A { note: \"x   y\" }\n"),
        (
            "A -> B: \"GET /users/{id}\"",
            "A -> B: \"GET /users/{id}\"\n",
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(promises(source), expected, "{source:?}");
    }
}

#[test]
fn a_file_with_an_error_is_refused() {
    let errors = format("A -> -> B\n").unwrap_err();
    assert_eq!(errors[0].code, "E0006");
}

#[test]
fn a_warning_does_not_stop_the_formatter() {
    assert!(format("story s {\n}\n").is_ok());
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn every_horse_file_in_the_repository_keeps_its_model() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let mut seen = 0;
    for directory in ["examples", "spikes/layered-layout/corpus"] {
        for entry in std::fs::read_dir(format!("{root}/{directory}")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "horse") {
                promises(&std::fs::read_to_string(&path).unwrap());
                seen += 1;
            }
        }
    }
    assert_eq!(seen, 11);
}

/// A seeded generator of files made of the forms of the language, so a
/// failure repeats. Many of the files have errors, and the test skips those.
#[test]
fn random_files_keep_their_model_and_settle() {
    const PIECES: [&str; 39] = [
        "owner: X ?, kind: y",
        "k: x\"y ?, a: b",
        "A",
        "B",
        "Web app",
        "  ",
        " ",
        "\t",
        "\n",
        "\n\n",
        "\r\n",
        ",",
        ", ",
        ":",
        ": ",
        "{",
        "}",
        " { ",
        " }",
        "->",
        " -> ",
        " -x ",
        "?",
        " ?",
        " @ src/a.rs:3",
        "# note\n",
        "show ",
        "story ",
        "of ",
        "owner",
        "owner: X",
        "\"a b\"",
        "\"a, b\"",
        "\"#x\"",
        "\"\"",
        "\u{feff}",
        "\"show x\"",
        "\"a   b\"",
        "\" a\"",
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move |limit: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        usize::try_from(state % limit as u64).unwrap()
    };
    let mut formatted = 0;
    // A long run: HORSEGIRL_FUZZ_ROUNDS=3000000 cargo test --release -p horsegirl-core --test format
    let rounds = std::env::var("HORSEGIRL_FUZZ_ROUNDS")
        .ok()
        .and_then(|rounds| rounds.parse().ok())
        .unwrap_or(60_000);
    for _ in 0..rounds {
        let source: String = (0..next(24)).map(|_| PIECES[next(PIECES.len())]).collect();
        if format(&source).is_ok() {
            promises(&source);
            formatted += 1;
        }
    }
    // The test must reach the formatter often enough to mean something.
    assert!(formatted > 3_000, "only {formatted} files had no error");
}
