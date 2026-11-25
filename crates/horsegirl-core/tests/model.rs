//! The model that the tree lowers into.

use std::fmt::Write as _;

use horsegirl_core::model::{Fact, Model, Rule, Show, lower};
use horsegirl_core::syntax::parse;

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const CHECKOUT_MODEL: &str = include_str!("checkout.model");

fn model(source: &str) -> Model {
    lower(&parse(source).tree)
}

fn names(model: &Model) -> Vec<&str> {
    model.things.iter().map(|t| t.name.as_str()).collect()
}

/// Each link as `from -> to: labels`.
fn links(model: &Model) -> Vec<String> {
    model
        .links
        .iter()
        .map(|link| {
            format!(
                "{} -> {}: {}",
                model.things[link.from].name,
                model.things[link.to].name,
                link.labels.join(" | ")
            )
        })
        .collect()
}

#[test]
fn the_reference_file_lowers_to_its_counts() {
    let model = model(CHECKOUT);
    let groups = model.groups();
    // A group is a thing too, so the nine things are the ones that hold nothing.
    assert_eq!(model.things.len() - groups.len(), 9, "{:?}", names(&model));
    assert_eq!(model.links.len(), 9, "{:#?}", links(&model));
    assert_eq!(groups.len(), 1);
    assert_eq!(
        model.owners(),
        ["Apps", "Platform", "Commerce", "Growth", "External"]
    );
}

#[test]
fn the_reference_file_lowers_to_its_model() {
    assert_eq!(format!("{:#?}\n", model(CHECKOUT)), CHECKOUT_MODEL);
}

#[test]
fn the_reference_file_keeps_order_groups_and_facts() {
    let model = model(CHECKOUT);
    assert_eq!(
        names(&model),
        [
            "Web app",
            "Mobile app",
            "API gateway",
            "Payments",
            "Postgres",
            "Services",
            "Orders",
            "Notify",
            "Events",
            "Email provider",
        ]
    );
    let services = model.find("Services").unwrap();
    let parts: Vec<&str> = model
        .parts(services)
        .into_iter()
        .map(|id| model.things[id].name.as_str())
        .collect();
    assert_eq!(parts, ["Payments", "Orders", "Notify"]);
    let orders = &model.things[model.find("Orders").unwrap()];
    assert_eq!(
        orders.facts,
        [Fact {
            key: "owner".into(),
            value: "Commerce".into()
        }]
    );
    assert_eq!(
        links(&model)[0],
        "Web app -> API gateway: POST /checkout",
        "the story states this link again with a label"
    );
    assert_eq!(
        model.shows,
        [Show {
            lens: "flow".into(),
            arg: None
        }]
    );
}

#[test]
fn a_link_stated_twice_is_one_link() {
    let model = model("A -> B\nA -> B\n\"A\" -> B: x\nA  ->  B: x\nA -> B: y\n");
    assert_eq!(links(&model), ["A -> B: x | y"]);
    assert_eq!(model.links[0].span, 0..6);
    assert_eq!(model.things[0].mentions.len(), 5);
}

#[test]
fn the_opposite_direction_is_another_link() {
    assert_eq!(links(&model("A -> B\nB -> A\n")), ["A -> B: ", "B -> A: "]);
}

#[test]
fn commas_fan_out() {
    assert_eq!(
        links(&model("A, B -> C, D: go\n")),
        ["A -> C: go", "A -> D: go", "B -> C: go", "B -> D: go"]
    );
}

#[test]
fn a_quoted_name_is_the_bare_name() {
    let model = model("\"Web app\" -> API\nWeb   app -> API\n\"Sensors: wind\" -> API\n");
    assert_eq!(names(&model), ["Web app", "API", "Sensors: wind"]);
}

#[test]
fn a_rule_mentions_things_and_makes_no_link() {
    let model = model("A -x B: never\n");
    assert_eq!(names(&model), ["A", "B"]);
    assert_eq!(model.links, []);
}

#[test]
fn a_rule_is_stored_with_its_reason() {
    let model = model("A -x B: the web app never reads the table\nA -x C\n");
    assert_eq!(
        model.rules,
        [
            Rule {
                from: 0,
                to: 1,
                reason: Some("the web app never reads the table".into()),
                span: 0..41,
            },
            Rule {
                from: 0,
                to: 2,
                reason: None,
                span: 42..48,
            },
        ]
    );
}

#[test]
fn a_rule_fans_out_and_is_stored_once() {
    let model = model("A, B -x C: no\nA -x C: no\nA -x C: later\n");
    let rules: Vec<_> = model
        .rules
        .iter()
        .map(|r| (r.from, r.to, r.reason.as_deref()))
        .collect();
    assert_eq!(
        rules,
        [
            (0, 2, Some("no")),
            (1, 2, Some("no")),
            (0, 2, Some("later"))
        ]
    );
}

#[test]
fn a_final_question_mark_marks_a_guess() {
    let model = model("A -> B ?\nC -> D: maybe ?\nE -> F ? @ a.rs:3\nG -> H\n");
    let guesses: Vec<bool> = model.links.iter().map(|l| l.guess).collect();
    assert_eq!(guesses, [true, true, true, false]);
    assert_eq!(model.links[1].labels, ["maybe"]);
}

#[test]
fn a_question_mark_inside_a_label_is_not_a_guess() {
    let model = model("A -> B: ok?\nC -> D: is it ok ?x\nIs it? -> E\n");
    assert!(model.links.iter().all(|l| !l.guess), "{:?}", model.links);
    assert_eq!(model.links[0].labels, ["ok?"]);
}

#[test]
fn one_firm_statement_makes_a_guess_firm() {
    let model = model("A -> B ?\nA -> B\nC -> D\nC -> D ?\nE -> F ?\nE -> F ?\n");
    let guesses: Vec<bool> = model.links.iter().map(|l| l.guess).collect();
    assert_eq!(guesses, [false, false, true]);
}

#[test]
fn a_bad_line_states_nothing() {
    let parse = parse("A -> B -> C\nD\n");
    assert_eq!(parse.diagnostics.len(), 1, "{:?}", parse.diagnostics);
    assert_eq!(names(&lower(&parse.tree)), ["D"]);
}

#[test]
fn facts_belong_to_every_name_before_the_braces() {
    let model = model("A, B { owner: X, note: \"a, b\" }\nA { owner: X }\n");
    for thing in &model.things {
        assert_eq!(
            thing.facts,
            [
                Fact {
                    key: "owner".into(),
                    value: "X".into()
                },
                Fact {
                    key: "note".into(),
                    value: "a, b".into()
                },
            ]
        );
    }
    assert_eq!(model.groups(), []);
}

#[test]
fn groups_nest_and_hold_their_links() {
    let model = model("Outer {\n  Inner {\n    A -> B\n  }\n  owner: X\n}\nA -> Outer\n");
    let id = |name| model.find(name).unwrap();
    assert_eq!(model.things[id("Inner")].parent, Some(id("Outer")));
    assert_eq!(model.things[id("A")].parent, Some(id("Inner")));
    assert_eq!(model.things[id("B")].parent, Some(id("Inner")));
    assert_eq!(model.groups(), [id("Outer"), id("Inner")]);
    assert_eq!(model.owners(), ["X"]);
    assert_eq!(links(&model), ["A -> B: ", "A -> Outer: "]);
}

#[test]
fn a_thing_keeps_its_first_group() {
    let model = model("G { A }\nH { A }\n");
    let id = |name| model.find(name).unwrap();
    assert_eq!(model.things[id("A")].parent, Some(id("G")));
    assert_eq!(model.parts(id("H")), []);
}

#[test]
fn groups_never_hold_each_other() {
    let model = model("A { B }\nB { A }\nC { C }\n");
    let id = |name| model.find(name).unwrap();
    assert_eq!(model.things[id("B")].parent, Some(id("A")));
    assert_eq!(model.things[id("A")].parent, None);
    assert_eq!(model.things[id("C")].parent, None);
}

#[test]
fn a_story_gives_plain_links_and_no_thing() {
    let model = model("story checkout {\n  A -> B: go\n  owner: X\n}\n");
    assert_eq!(names(&model), ["A", "B"]);
    assert_eq!(links(&model), ["A -> B: go"]);
    assert_eq!(model.owners(), [""; 0]);
}

#[test]
fn a_show_line_keeps_its_argument() {
    let model = model("show sequence of checkout\nshow rings from \"API: v2\"\n");
    assert_eq!(
        model.shows,
        [
            Show {
                lens: "sequence".into(),
                arg: Some(("of".into(), "checkout".into())),
            },
            Show {
                lens: "rings".into(),
                arg: Some(("from".into(), "API: v2".into())),
            },
        ]
    );
}

#[test]
fn two_runs_give_the_same_model() {
    // Many names, so that a map iterated into the output would show its order.
    let mut source = String::new();
    for i in 0..200 {
        let (to, label, group, owner) = ((i * 7) % 200, i % 3, i % 5, i % 4);
        writeln!(source, "N{i} -> N{to}: l{label}").unwrap();
        writeln!(source, "G{group} {{ N{i} }}").unwrap();
        writeln!(source, "N{i} {{ owner: O{owner} }}").unwrap();
    }
    let first = format!("{:?}", model(&source));
    for _ in 0..5 {
        assert_eq!(format!("{:?}", model(&source)), first);
    }
    assert_eq!(
        format!("{:?}", model(CHECKOUT)),
        format!("{:?}", model(CHECKOUT))
    );
}

#[test]
fn a_long_line_of_names_lowers_in_linear_time() {
    // Each link once measured the whole line again: a fan of 20,000 names
    // cost the square of the line, and a cross of n names the cube.
    let fan: Vec<String> = (0..20_000).map(|i| format!("N{i}")).collect();
    let fan = fan.join(",");
    for source in [format!("{fan} -> B: go\n"), format!("A -> {fan}: go\n")] {
        let tree = parse(&source).tree;
        let start = std::time::Instant::now();
        let model = lower(&tree);
        let spent = start.elapsed();
        assert_eq!(model.links.len(), 20_000);
        assert!(spent.as_secs() < 5, "took {spent:?}");
    }
}
