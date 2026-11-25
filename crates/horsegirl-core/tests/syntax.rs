//! The parser against the reference file, the corpus and bad input.

use horsegirl_core::syntax::{Diagnostic, Element, Node, NodeKind, parse};

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const CHECKOUT_TREE: &str = include_str!("checkout.tree");

/// The tree of `source` as text, and the code of each diagnostic.
fn read(source: &str) -> (String, Vec<&'static str>) {
    let parse = parse(source);
    assert_eq!(parse.tree.to_string(), source, "the tree lost a byte");
    let codes = parse.diagnostics.iter().map(|d| d.code).collect();
    (format!("{:?}", parse.tree), codes)
}

/// The one diagnostic of `source`.
fn only(source: &str) -> Diagnostic {
    let mut diagnostics = parse(source).diagnostics;
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    diagnostics.remove(0)
}

/// The source after the fix of its one diagnostic.
fn fixed(source: &str) -> String {
    let fix = only(source).fix.expect("the diagnostic carries a fix");
    format!(
        "{}{}{}",
        &source[..fix.span.start],
        fix.text,
        &source[fix.span.end..]
    )
}

fn kinds(node: &Node<'_>) -> Vec<NodeKind> {
    let nodes = node.children.iter().filter_map(|child| match child {
        Element::Node(node) => Some(node.kind),
        Element::Token(_) => None,
    });
    nodes.collect()
}

#[test]
fn the_reference_file_parses_into_the_expected_tree() {
    let (tree, codes) = read(CHECKOUT);
    assert!(tree == CHECKOUT_TREE, "the tree moved; it is now:\n{tree}");
    // The story is rule 4, which arrives in v0.2. Nothing else is wrong.
    assert_eq!(codes, ["W0010"]);
}

#[test]
fn a_story_is_reported_and_the_file_still_parses() {
    let story = only("story checkout {\n  A -> B: pay\n}\nshow flow\n");
    assert_eq!(story.code, "W0010");
    assert!(story.message.contains("v0.2"), "{}", story.message);
    assert!(story.message.contains("`checkout`"), "{}", story.message);
}

// The corpus is 239 files, and the WebAssembly tests get no directory to
// read. The reference file above covers that target.
#[cfg(not(target_family = "wasm"))]
#[test]
fn every_file_in_the_corpus_prints_back_byte_for_byte() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let mut flowcharts = 0;
    let mut horse = 0;
    for directory in [
        "corpus/flowcharts",
        "examples",
        "spikes/layered-layout/corpus",
    ] {
        let mut paths: Vec<_> = std::fs::read_dir(format!("{root}/{directory}"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        paths.sort();
        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            let parse = parse(&source);
            assert!(
                parse.tree.to_string() == source,
                "{} changed",
                path.display()
            );
            match path.extension().and_then(|e| e.to_str()) {
                Some("mmd") => flowcharts += 1,
                // A horsegirl file in this repository has no error. A Mermaid
                // file has many, and it still prints back.
                Some("horse") => {
                    let errors: Vec<_> = parse
                        .diagnostics
                        .iter()
                        .filter(|d| d.code.starts_with('E'))
                        .collect();
                    assert!(errors.is_empty(), "{}: {errors:?}", path.display());
                    horse += 1;
                }
                _ => {}
            }
        }
    }
    assert_eq!((flowcharts, horse), (239, 11));
}

#[test]
fn three_bad_lines_give_three_errors_and_the_good_lines_still_parse() {
    let source = "\
Web app -> API gateway
owner: Commerce
API gateway -> Orders: create
Orders -> -> Payments
Services { Orders, Payments }
Payments -> Notify -> Events
show flow
";
    let parse = parse(source);
    let found: Vec<_> = parse
        .diagnostics
        .iter()
        .map(|d| (d.code, source[..d.span.start].matches('\n').count() + 1))
        .collect();
    // Each diagnostic is paired with its line, counted from 1.
    assert_eq!(found, [("E0009", 2), ("E0006", 4), ("E0001", 6)]);
    assert_eq!(
        kinds(&parse.tree),
        [
            NodeKind::Link,
            NodeKind::Error,
            NodeKind::Link,
            NodeKind::Error,
            NodeKind::Things,
            NodeKind::Error,
            NodeKind::Show,
        ]
    );
    let (tree, _) = read(source);
    assert!(tree.contains("Label \"create\""), "{tree}");
    assert!(tree.contains("Lens \"flow\""), "{tree}");
}

#[test]
fn a_quoted_name_holds_reserved_characters() {
    let (tree, codes) =
        read("\"Sensors: wind\", \"a -> b\" -> \"Hub, main\": \"GET /users/{id}\"\n");
    assert_eq!(codes, [""; 0]);
    assert_eq!(
        tree,
        "\
File
  Link
    Names
      Name \"\\\"Sensors: wind\\\"\"
      Comma \",\"
      Name \"\\\"a -> b\\\"\"
    Arrow \"->\"
    Names
      Name \"\\\"Hub, main\\\"\"
    Colon \":\"
    Label \"\\\"GET /users/{id}\\\"\"
"
    );
}

// ADR-119: a name or a value that needs quotes and lacks them is an error
// whose fix adds the quotes.
#[test]
fn a_name_or_a_value_that_needs_quotes_gets_them_from_the_fix() {
    for (source, repaired) in [
        ("Sensors: wind -> Hub", "\"Sensors: wind\" -> Hub"),
        (
            "A, up:Disease { owner: X }",
            "A, \"up:Disease\" { owner: X }",
        ),
        (
            "wally { note: install, list }",
            "wally { note: \"install, list\" }",
        ),
        (
            "api { note: django :8000 }",
            "api { note: \"django :8000\" }",
        ),
        ("api {\n  note: a -> b\n}", "api {\n  note: \"a -> b\"\n}"),
        ("A -> B: returns {id}", "A -> B: \"returns {id}\""),
    ] {
        assert_eq!(only(source).code, "E0004", "{source}");
        assert_eq!(fixed(source), repaired);
        assert_eq!(read(repaired).1, [""; 0], "{repaired}");
    }
}

#[test]
fn a_fact_in_quotes_is_one_fact() {
    let (tree, codes) = read("wally { note: \"install, list\", owner: X }");
    assert_eq!(codes, [""; 0]);
    assert_eq!(tree.matches("Fact\n").count(), 2, "{tree}");
    assert!(tree.contains("Value \"\\\"install, list\\\"\""), "{tree}");
}

#[test]
fn a_final_question_mark_is_a_guess_and_any_other_is_text() {
    let (tree, _) = read("A -> B ?\nA -> B: is it ok ?\nIs valid? -> B: sure?\nA ? B\n");
    assert_eq!(tree.matches("Guess").count(), 2, "{tree}");
    assert!(tree.contains("Label \"is it ok\""), "{tree}");
    assert!(tree.contains("Name \"Is valid?\""), "{tree}");
    assert!(tree.contains("Label \"sure?\""), "{tree}");
    assert!(tree.contains("Name \"A ? B\""), "{tree}");
}

#[test]
fn a_citation_ends_a_line() {
    let (tree, codes) =
        read("Orders -> Postgres: save  @ src/repo.ts:17\nA { owner: X @ a.rs:3 }\n");
    assert_eq!(codes, [""; 0]);
    assert!(tree.contains("Label \"save\""), "{tree}");
    assert!(tree.contains("Cite \"@ src/repo.ts:17\""), "{tree}");
    assert!(tree.contains("Cite \"@ a.rs:3\""), "{tree}");
    assert_eq!(only("A -> B @ a.rs:3 more").code, "E0001");
    assert_eq!(only("A -> B @").code, "E0001");
}

#[test]
fn a_crossed_arrow_is_a_whole_word() {
    let (tree, codes) = read("Dash-x, -xray -x B\n");
    assert_eq!(codes, [""; 0]);
    assert!(tree.contains("Rule"), "{tree}");
    assert!(tree.contains("Name \"Dash-x\""), "{tree}");
    assert!(tree.contains("Name \"-xray\""), "{tree}");
}

#[test]
fn a_show_line_names_a_lens_and_an_argument() {
    let (tree, codes) =
        read("show sequence of checkout\nshow rings from API gateway\nshow seams by owner\n");
    assert_eq!(codes, [""; 0]);
    assert_eq!(tree.matches("Show").count(), 3, "{tree}");
    assert!(tree.contains("Arg \"API gateway\""), "{tree}");
    for bad in [
        "show",
        "show flow fo checkout",
        "show flow of",
        "show flow of a b ?",
    ] {
        assert_eq!(only(bad).code, "E0008", "{bad}");
    }
    // `show` is a word of a name when the line is a link.
    assert_eq!(read("show room -> Hall\n").1, [""; 0]);
}

#[test]
fn each_mistake_has_its_own_code() {
    for (source, code) in [
        ("A -> B -> C", "E0001"),
        ("A {\n  B\n", "E0002"),
        ("A }", "E0003"),
        ("A -> \"B", "E0005"),
        ("A -> B: \"oops", "E0005"),
        ("A, -> B", "E0006"),
        ("A ->", "E0006"),
        ("A { owner: }", "E0006"),
        ("A { B, owner: X }", "E0009"),
    ] {
        assert_eq!(only(source).code, code, "{source}");
    }
}

#[test]
fn a_comment_a_blank_line_and_a_windows_line_end_are_kept() {
    let source = "\u{feff}# the edge\r\n\r\n  # indented\r\nDeploy EC2 #1 -> B\r\n";
    let (tree, codes) = read(source);
    assert_eq!(codes, [""; 0]);
    assert_eq!(tree.matches("Comment").count(), 2, "{tree}");
    assert!(tree.contains("Name \"Deploy EC2 #1\""), "{tree}");
}

#[test]
fn braces_that_nest_too_deep_are_an_error_and_not_a_crash() {
    let source = "A {".repeat(10_000);
    let parse = parse(&source);
    assert_eq!(parse.tree.to_string(), source);
    assert!(parse.diagnostics.iter().any(|d| d.code == "E0007"));
}
