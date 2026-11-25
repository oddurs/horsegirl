//! The parser against the reference file, the corpus and bad input.

use horsegirl_core::syntax::{Diagnostic, Element, Fix, Node, NodeKind, parse};

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
    fix.apply(source)
}

/// The bytes that the edits of a fix write.
fn fix_len(fix: &Fix) -> usize {
    fix.edits.iter().map(|edit| edit.text.len()).sum()
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
        // `@` and a brace end a bare name too. An arrow after them is the
        // first arrow of its line.
        (
            "Renderer PixiJS / @pixi/react",
            "\"Renderer PixiJS / @pixi/react\"",
        ),
        (
            "Renderer PixiJS / @pixi/react -> B",
            "\"Renderer PixiJS / @pixi/react\" -> B",
        ),
        (
            "A -> Renderer PixiJS / @pixi/react",
            "A -> \"Renderer PixiJS / @pixi/react\"",
        ),
        ("@pixi/react, A -> B", "\"@pixi/react\", A -> B"),
        (
            "A -> B: email bob@example.com",
            "A -> B: \"email bob@example.com\"",
        ),
        (
            "A -> B: mail bob@example.com ? @ a.rs:3",
            "A -> B: \"mail bob@example.com\" ? @ a.rs:3",
        ),
        (
            "A { mail: bob@example.com, owner: X }",
            "A { mail: \"bob@example.com\", owner: X }",
        ),
        // A citation is `@ path:line` after a space. A path with no line
        // number is not one.
        ("A -> B @ src/app.ts", "A -> \"B @ src/app.ts\""),
        ("A @ a.rs:3 -> B", "\"A @ a.rs:3\" -> B"),
        ("A {x} -> B", "\"A {x}\" -> B"),
        ("A, GET /users/{id} -x B", "A, \"GET /users/{id}\" -x B"),
        // A block is not a part of a name. The quotes end before a `{` that
        // does not close on its line, and before a group or a block of facts.
        (
            "Renderer @pixi/react {\n  owner: Web\n}",
            "\"Renderer @pixi/react\" {\n  owner: Web\n}",
        ),
        ("Team @pixi {\n  A\n}", "\"Team @pixi\" {\n  A\n}"),
        ("Team@x { A, B }", "\"Team@x\" { A, B }"),
        ("@pixi/react { A, B }", "\"@pixi/react\" { A, B }"),
        ("Team@x { A -> B }", "\"Team@x\" { A -> B }"),
        ("Team: x {\n  A\n}", "\"Team: x\" {\n  A\n}"),
        // A group with one part is a group too. The name needs its quotes in
        // any case, so the block stays outside them.
        ("Team@x { A }", "\"Team@x\" { A }"),
        ("Team @pixi/react { Web }", "\"Team @pixi/react\" { Web }"),
        ("A, B@x { C }", "A, \"B@x\" { C }"),
        // Braces with no space before them are characters of the name.
        ("GET@v1 /users/{id} -> B", "\"GET@v1 /users/{id}\" -> B"),
        ("A {x} @y -> B", "\"A {x} @y\" -> B"),
        // A quote that touches `-x` would be a character of that word.
        ("A -x@pixi/react", "A -x \"@pixi/react\""),
    ] {
        assert_eq!(only(source).code, "E0004", "{source}");
        assert_eq!(fixed(source), repaired);
        assert_eq!(read(repaired).1, [""; 0], "{repaired}");
    }
    // The keyword of a story stays outside the quotes.
    let story = "story s@x {\n  A -> B\n}";
    assert_eq!(fixed(story), "story \"s@x\" {\n  A -> B\n}");
    assert_eq!(read(&fixed(story)).1, ["W0010"]);
    let story = "story s@x { A -> B }";
    assert_eq!(fixed(story), "story \"s@x\" { A -> B }");
    assert_eq!(read(&fixed(story)).1, ["W0010"]);
    let story = "story s@x { A }";
    assert_eq!(fixed(story), "story \"s@x\" { A }");
    assert_eq!(read(&fixed(story)).1, ["W0010"]);
    // A colon in the name of a story takes the same fix.
    let story = "story s: x {\n  A -> B\n}";
    assert_eq!(fixed(story), "story \"s: x\" {\n  A -> B\n}");
    assert_eq!(read(&fixed(story)).1, ["W0010"]);
    let story = "story checkout: card { A -> B }";
    assert_eq!(fixed(story), "story \"checkout: card\" { A -> B }");
    assert_eq!(read(&fixed(story)).1, ["W0010"]);
    assert_eq!(fixed("story: x {\n  A\n}"), "\"story: x\" {\n  A\n}");
    // Without a block, `story` is a word of the name.
    assert_eq!(fixed("story s: x -> B"), "\"story s: x\" -> B");
    assert_eq!(fixed("story s@x -> B"), "\"story s@x\" -> B");
}

// Quotes around a group would make one thing of it and lose its parts, so a
// group before an arrow has no fix.
#[test]
fn a_group_or_an_open_block_is_never_quoted_into_a_name() {
    for source in [
        "Services { Orders, Payments } -> Postgres: save",
        "A -> B: x {\n}",
        "A -> B: x { a, b }",
        "A {\n  note: b {\n  }\n}",
    ] {
        let stray = only(source);
        assert_eq!(stray.code, "E0001", "{source}");
        assert_eq!(stray.fix, None, "{source}");
    }
    // A block cannot follow a link. The name still gets its quotes, and the
    // fix leaves the brace outside them for the next message.
    for (source, repaired) in [
        ("A -> B @x {\n}", "A -> \"B @x\" {\n}"),
        ("A -> B@x { C }", "A -> \"B@x\" { C }"),
    ] {
        assert_eq!(fixed(source), repaired);
        assert_eq!(only(repaired).code, "E0001");
    }
}

// A name in quotes needs no more quotes. The `@` after it is the mistake.
#[test]
fn a_bad_citation_after_a_quoted_name_is_not_told_to_use_quotes() {
    for source in ["\"A\" @x", "A -> \"B\" @x"] {
        let stray = only(source);
        assert_eq!(stray.code, "E0001", "{source}");
        assert!(
            stray.message.contains("not a citation"),
            "{}",
            stray.message
        );
    }
}

// Only a line that already holds an arrow has a second one.
#[test]
fn an_arrow_after_a_block_is_not_called_a_second_arrow() {
    for source in ["A { owner: X } -> B", "A {\n  B\n} -> C"] {
        let stray = only(source);
        assert_eq!(stray.code, "E0001", "{source}");
        assert!(!stray.message.contains("second"), "{}", stray.message);
        assert_eq!(stray.fix, None, "{source}");
    }
    let second = only("A -> B -> C");
    assert!(second.message.contains("second"), "{}", second.message);
}

// A megabyte of JSON on one line can reach `check` through the MCP tool. The
// lexer once read to the end of the line for every quote: 14 s for 320 KB.
// The WebAssembly target has no clock to hold the bound with.
#[cfg(not(target_family = "wasm"))]
#[test]
fn a_long_line_of_quoted_names_parses_in_linear_time() {
    for piece in [
        "\"\" ",
        "\"a\",",
        "\"a\" @ a:1 ? ",
        "A @b ",
        // Braces that do not close on the line. Each fact once quoted the
        // rest of the line in its message and in its fix: 5.7 GB for 240 KB.
        "{a: b {y} ",
        "A {a: b {x} ",
        "{A -> B: x {y} ",
        "{a: b {y} } ",
        "A @b {x} ",
    ] {
        linear(&piece.repeat(60_000 / piece.len()));
    }
    // Past 64 levels the parser opens no block, so the braces of a line do
    // not pair the way the parser reads them. Each fact once quoted most of
    // the line in its fix: 4.7 GB for 480 KB.
    for piece in ["a@b {}{", "a@b{}{", "a@b {} {"] {
        let count = 60_000 / piece.len();
        linear(&format!("{}{}", piece.repeat(count), "}".repeat(count)));
    }
    linear(&format!(
        "{}{}{}",
        "{ } {a: b ".repeat(8_000),
        "-x:?".repeat(20_000),
        " {x} }  a: b ".repeat(6_000)
    ));
}

/// Parses `source` and holds the time and the size of what comes back.
#[cfg(not(target_family = "wasm"))]
fn linear(source: &str) {
    let piece: String = source.chars().take(20).collect();
    let start = std::time::Instant::now();
    let parse = parse(source);
    let spent = start.elapsed();
    assert!(parse.tree.to_string() == source, "lost a byte of {piece:?}");
    assert!(spent.as_secs() < 5, "{piece:?} took {spent:?}");
    let text = parse
        .diagnostics
        .iter()
        .map(|d| d.message.len() + d.fix.as_ref().map_or(0, fix_len));
    let text: usize = text.sum();
    assert!(
        text < 100 * source.len(),
        "{piece:?} gave {text} bytes of diagnostics"
    );
}

#[test]
fn a_message_quotes_the_start_of_a_long_text() {
    let source = format!("A -> B: {} {{x}}", "word ".repeat(100));
    let long = only(&source);
    assert_eq!(long.code, "E0004");
    assert!(long.message.len() < 200, "{}", long.message);
    assert!(long.message.contains("..."), "{}", long.message);
    // A fix would hold the whole text, so a text this long gets none.
    assert_eq!(long.fix, None);
    let source = format!("A -> B: {} {{x}}", "word ".repeat(39));
    assert_eq!(only(&source).span.len(), 199);
    assert_eq!(read(&fixed(&source)).1, [""; 0]);
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

/// Asserts that `source` has no diagnostic.
fn silent(source: &str) {
    let diagnostics = parse(source).diagnostics;
    assert!(diagnostics.is_empty(), "{source:?} gave {diagnostics:?}");
}

const MERMAID_LINE: &str = "\"Web app\" --> \"API gateway\" : HTTPS;\n";

#[test]
fn a_mermaid_link_reads_as_one_link_with_its_label() {
    let model = horsegirl_core::model::lower(&parse(MERMAID_LINE).tree);
    let names: Vec<&str> = model.things.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Web app", "API gateway"]);
    assert_eq!(model.links.len(), 1);
    assert_eq!(model.links[0].labels, ["HTTPS"]);
}

#[test]
fn a_mermaid_link_gets_one_note_whose_fix_rewrites_the_line() {
    let note = only(MERMAID_LINE);
    assert_eq!(note.code, "N0101");
    assert_eq!(note.span, 0..MERMAID_LINE.len() - 1);
    let fix = note.fix.expect("the note carries a fix");
    assert_eq!(fix.confidence, horsegirl_core::syntax::Confidence::Safe);
    assert_eq!(fix.apply(MERMAID_LINE), "Web app -> API gateway: HTTPS\n");
}

#[test]
fn the_canonical_line_has_no_note() {
    let canonical = "Web app -> API gateway: HTTPS\n";
    silent(canonical);
}

#[test]
fn each_habit_alone_gets_its_own_rewrite() {
    for (habit, canonical) in [
        ("A --> B\n", "A -> B\n"),
        ("A ---> B\n", "A -> B\n"),
        ("A-->B\n", "A->B\n"),
        ("A -> B;\n", "A -> B\n"),
        ("A -> B ;\n", "A -> B\n"),
        ("A, B;\n", "A, B\n"),
        ("A -> B: go;  \n", "A -> B: go  \n"),
        ("\"A b\" -> C\n", "A b -> C\n"),
        ("A -> B : go\n", "A -> B: go\n"),
        ("show flow;\n", "show flow\n"),
        ("A -> B ? @ src/a.ts:1;\n", "A -> B ? @ src/a.ts:1\n"),
        ("A --> \"B\";\n", "A -> B\n"),
        ("\"A\" -> \"B\": x\n", "A -> B: x\n"),
        ("\"a\";\n", "a\n"),
        ("\"a b\" ->\"c d\";\n", "a b ->c d\n"),
    ] {
        let note = only(habit);
        assert_eq!(note.code, "N0101", "{habit:?}");
        assert_eq!(note.fix.unwrap().apply(habit), canonical, "{habit:?}");
        silent(canonical);
        assert_eq!(meaning(habit), meaning(canonical), "{habit:?}");
    }
}

/// The model without its spans: a fix moves text, and it must not change meaning.
fn meaning(source: &str) -> String {
    let model = horsegirl_core::model::lower(&parse(source).tree);
    let things: Vec<_> = model
        .things
        .iter()
        .map(|t| format!("{:?} {:?} {:?}", t.name, t.parent, t.facts))
        .collect();
    let links: Vec<_> = model
        .links
        .iter()
        .map(|l| format!("{} {} {:?}", l.from, l.to, l.labels))
        .collect();
    format!("{things:?} {links:?} {:?}", model.shows)
}

#[test]
fn a_double_semicolon_is_not_a_mark() {
    for source in ["A, b;;\n", "A -> B;;;\n", "A -> B: go;;\n"] {
        silent(source);
    }
    let lowered = horsegirl_core::model::lower(&parse("A, b;;\n").tree);
    assert_eq!(lowered.things[1].name, "b;;");
}

#[test]
fn quotes_stay_where_the_name_needs_them() {
    for source in [
        "\"Sensors: wind\" -> Hub\n",
        "\"a, b\" -> C\n",
        "\"show\" -> C\n",
        "\"show it\" -> C\n",
        "\"two  spaces\" -> C\n",
        "\"-x\" -> C\n",
        "\"?\" -> C\n",
        "\"#tag\" -> C\n",
        "\"\" -> C\n",
        "\"a->b\" -> C\n",
        "A { owner: \"Team A\" }\n",
        "\"a-\"->B\n",
        "A, \"b-\"->C\n",
        "\"a-\"-x B\n",
    ] {
        silent(source);
    }
}

#[test]
fn a_fix_never_joins_a_name_to_an_arrow() {
    let habit = "\"x-\"-->B\n";
    let fixed = only(habit).fix.unwrap().apply(habit);
    assert_eq!(fixed, "\"x-\"->B\n");
    let before = horsegirl_core::model::lower(&parse(habit).tree);
    let after = horsegirl_core::model::lower(&parse(&fixed).tree);
    assert_eq!(before.things[0].name, "x-");
    assert_eq!(after.things[0].name, "x-");
}

#[test]
fn a_group_and_its_parts_get_separate_notes() {
    let source = "\"Services\" {\n  \"Orders\" --> Payments;\n}\n";
    let notes = parse(source).diagnostics;
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes.iter().all(|note| note.code == "N0101"));
    let mut edits: Vec<_> = notes
        .iter()
        .flat_map(|note| note.fix.iter().flat_map(|fix| fix.edits.iter()))
        .collect();
    edits.sort_by_key(|edit| edit.span.start);
    let mut out = String::new();
    let mut from = 0;
    for edit in edits {
        out.push_str(&source[from..edit.span.start]);
        out.push_str(&edit.text);
        from = edit.span.end;
    }
    out.push_str(&source[from..]);
    assert_eq!(out, "Services {\n  Orders -> Payments\n}\n");
}

#[test]
fn a_semicolon_inside_a_line_is_a_character_of_the_name() {
    let source = "a;b -> C\n";
    silent(source);
    let model = horsegirl_core::model::lower(&parse(source).tree);
    assert_eq!(model.things[0].name, "a;b");
}

#[test]
fn a_line_with_a_mistake_gets_the_mistake_and_no_note() {
    let diagnostics = parse("\"A\" --> B -> C;\n").diagnostics;
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, "E0001");
}
