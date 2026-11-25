//! The flow lens: its example draws with no overlap, missing facts give a
//! diagnostic that names them, the language reference lists its dials, and
//! the picture follows the dials and nothing else.

use horsegirl_core::check::check;
use horsegirl_core::layout::Graph;
use horsegirl_core::lens::{self, Channel, Dial, FLOW, Field, Lens, Need, Solver};
use horsegirl_core::model::{Model, lower};
use horsegirl_core::overlap::overlaps;
use horsegirl_core::scene::{Role, Scene};
use horsegirl_core::svg;
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;

const STYLE: Style = Style { size: 14.0 };

/// The example of the flow lens in its backlog item and in `docs/lenses.md`.
const EXAMPLE: &str = "Web app, Mobile app -> API gateway\nAPI gateway -> Orders\nshow flow\n";
const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const REFERENCE: &str = include_str!("../../../docs/lenses.md");

fn model(source: &str) -> Model {
    lower(&parse(source).tree)
}

fn collisions(scene: &Scene) -> Vec<String> {
    let found = overlaps(scene);
    found
        .iter()
        .map(|overlap| overlap.describe(scene))
        .collect()
}

fn names(scene: &Scene, role: Role) -> Vec<String> {
    let shapes = scene.shapes.iter().filter(|shape| shape.role == role);
    shapes.map(|shape| shape.of.join(" -> ")).collect()
}

#[test]
fn the_example_draws_with_no_overlap() {
    let scene = lens::draw(&model(EXAMPLE), &FLOW, STYLE);
    assert_eq!(collisions(&scene), Vec::<String>::new());
    assert_eq!(
        names(&scene, Role::Thing),
        ["Web app", "Mobile app", "API gateway", "Orders"]
    );
    assert_eq!(
        scene.shapes.iter().filter(|s| s.role == Role::Link).count(),
        3
    );
    assert!(check(EXAMPLE).is_empty(), "{:?}", check(EXAMPLE));
}

#[test]
fn the_example_reads_from_left_to_right_by_depth() {
    let scene = lens::draw(&model(EXAMPLE), &FLOW, STYLE);
    // Each rank centres its boxes, so a column is a centre.
    let x = |name: &str| {
        let shape = scene
            .shapes
            .iter()
            .find(|s| s.role == Role::Thing && s.of == [name]);
        let rect = shape.unwrap().rect;
        rect.x + rect.width / 2.0
    };
    assert_eq!(x("Web app"), x("Mobile app"));
    assert!(x("Web app") < x("API gateway"));
    assert!(x("API gateway") < x("Orders"));
}

#[test]
fn the_reference_file_draws_with_no_overlap() {
    let scene = lens::draw(&model(CHECKOUT), &FLOW, STYLE);
    assert_eq!(collisions(&scene), Vec::<String>::new());
    assert_eq!(names(&scene, Role::Group), ["Services"]);
}

#[test]
fn a_file_with_no_link_gives_an_error_that_names_links() {
    let source = "Web app, Mobile app\nshow flow\n";
    let found = check(source);
    let missing: Vec<_> = found.iter().filter(|d| d.code == "E0301").collect();
    assert_eq!(missing.len(), 1, "{found:?}");
    assert_eq!(&source[missing[0].span.clone()], "flow");
    assert!(
        missing[0].message.contains("no link"),
        "{}",
        missing[0].message
    );
    assert!(missing[0].message.contains("->"), "{}", missing[0].message);
    assert_eq!(FLOW.missing(&model(source)), [Need::Links]);
}

#[test]
fn a_rule_is_not_a_link_for_the_flow_lens() {
    let found = check("A -x B\nshow flow\n");
    assert!(found.iter().any(|d| d.code == "E0301"), "{found:?}");
}

#[test]
fn a_file_with_a_link_or_with_no_show_line_needs_nothing() {
    assert!(!check(EXAMPLE).iter().any(|d| d.code == "E0301"));
    assert!(!check("A, B\n").iter().any(|d| d.code == "E0301"));
    assert!(
        !check("A, B\nshow sequence of S\n")
            .iter()
            .any(|d| d.code == "E0301")
    );
}

#[test]
fn the_reference_lists_each_lens_with_its_dials() {
    let start = "<!-- lenses: start -->\n";
    let end = "<!-- lenses: end -->";
    let from = REFERENCE
        .find(start)
        .expect("docs/lenses.md has the start marker")
        + start.len();
    let to = REFERENCE
        .find(end)
        .expect("docs/lenses.md has the end marker");
    assert_eq!(
        &REFERENCE[from..to],
        lens::reference_markdown(),
        "docs/lenses.md is out of date: paste lens::reference_markdown() between the markers"
    );
    assert!(REFERENCE.contains("`x: depth`, `inside: group`, `colour: owner`"));
    assert!(
        REFERENCE.contains(EXAMPLE),
        "docs/lenses.md shows the example"
    );
}

#[test]
fn the_flow_lens_is_declared_as_dials() {
    assert_eq!(FLOW.name, "flow");
    assert_eq!(FLOW.solver, Solver::Layered);
    let dials: Vec<String> = FLOW.dials.iter().map(ToString::to_string).collect();
    assert_eq!(dials, ["x: depth", "inside: group", "colour: owner"]);
    assert_eq!(lens::find("flow"), Some(&FLOW));
    assert_eq!(lens::find("sequence"), None);
}

#[test]
fn every_lens_the_check_knows_and_this_version_draws_binds() {
    for lens in lens::DRAWN {
        assert!(
            horsegirl_core::check::LENSES.contains(&lens.name),
            "{}",
            lens.name
        );
        assert_eq!(lens.unshown(), None, "{}", lens.name);
        let bound = lens::bind(&model(CHECKOUT), lens, STYLE);
        assert_ne!(bound.graph, Graph::default(), "{}", lens.name);
    }
}

/// A lens that differs from flow in its dials alone. The picture changes with
/// the dials, so the dials, not the solver, decide what it shows.
const fn with(dials: &'static [Dial]) -> Lens {
    Lens { dials, ..FLOW }
}

#[test]
fn without_the_inside_dial_the_solver_gets_no_group() {
    const FLAT: Lens = with(&[Dial {
        channel: Channel::X,
        field: Field::Depth,
    }]);
    let checkout = model(CHECKOUT);
    let flow = lens::bind(&checkout, &FLOW, STYLE);
    assert_eq!(flow.groups, [checkout.find("Services").unwrap()]);
    let flat = lens::bind(&checkout, &FLAT, STYLE);
    assert_eq!(flat.groups, Vec::<usize>::new());
    assert!(flat.graph.nodes.iter().all(|node| node.group.is_none()));
    assert_eq!(flat.nodes.len(), checkout.things.len());
    let scene = lens::draw(&checkout, &FLAT, STYLE);
    assert_eq!(scene.colour, None);
    assert!(!svg::draw(&scene, "flat").contains("class=\"thing slot-"));
}

#[test]
fn the_colour_dial_picks_the_fact_that_colours_a_thing() {
    const BY_TEAM: Lens = with(&[
        Dial {
            channel: Channel::X,
            field: Field::Depth,
        },
        Dial {
            channel: Channel::Colour,
            field: Field::Fact("team"),
        },
    ]);
    let source = "A -> B\nA { owner: X, team: Red }\nB { owner: Y, team: Blue }\n";
    let by_owner = svg::draw(&lens::draw(&model(source), &FLOW, STYLE), "owner");
    assert!(
        by_owner.contains("class=\"thing slot-1 owner-x team-red\""),
        "{by_owner}"
    );
    let scene = lens::draw(&model(source), &BY_TEAM, STYLE);
    assert_eq!(scene.colour.as_deref(), Some("team"));
    let by_team = svg::draw(&scene, "team");
    assert!(
        by_team.contains("class=\"thing slot-2 owner-y team-blue\""),
        "{by_team}"
    );
}

#[test]
fn the_layered_solver_shows_every_dial_of_flow_and_refuses_x_by_group() {
    const WRONG: Lens = with(&[Dial {
        channel: Channel::X,
        field: Field::Group,
    }]);
    assert_eq!(FLOW.unshown(), None);
    assert_eq!(WRONG.unshown(), Some(WRONG.dials[0]));
    assert_eq!(WRONG.unshown().unwrap().to_string(), "x: group");
}
