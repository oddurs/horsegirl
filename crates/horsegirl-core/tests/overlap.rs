//! The overlap detector on scenes that are written by hand, one for each kind
//! of collision and one for each case that is not a collision.

use horsegirl_core::overlap::{Kind, Overlap, overlaps};
use horsegirl_core::scene::{Point, Rect, Role, Scene, Shape};

fn shape(role: Role, of: &[&str], rect: (f64, f64, f64, f64), points: &[(f64, f64)]) -> Shape {
    Shape {
        role,
        of: of.iter().map(|&name| name.to_owned()).collect(),
        rect: Rect {
            x: rect.0,
            y: rect.1,
            width: rect.2,
            height: rect.3,
        },
        points: points.iter().map(|&(x, y)| Point { x, y }).collect(),
        text: String::new(),
        facts: Vec::new(),
    }
}

fn thing(name: &str, rect: (f64, f64, f64, f64)) -> Shape {
    shape(Role::Thing, &[name], rect, &[])
}

fn group(name: &str, rect: (f64, f64, f64, f64)) -> Shape {
    shape(Role::Group, &[name], rect, &[])
}

fn link(from: &str, to: &str, points: &[(f64, f64)]) -> Shape {
    shape(Role::Link, &[from, to], (0.0, 0.0, 0.0, 0.0), points)
}

fn label(of: &[&str], rect: (f64, f64, f64, f64)) -> Shape {
    shape(Role::Label, of, rect, &[])
}

/// The kind and the two shapes of each collision.
fn found(shapes: Vec<Shape>) -> Vec<(Kind, usize, usize)> {
    let scene = Scene {
        width: 400.0,
        height: 400.0,
        font_size: 12.0,
        colour: None,
        shapes,
    };
    let of = |overlap: &Overlap| (overlap.kind, overlap.shapes[0], overlap.shapes[1]);
    overlaps(&scene).iter().map(of).collect()
}

const A: (f64, f64, f64, f64) = (0.0, 0.0, 60.0, 36.0);
const B: (f64, f64, f64, f64) = (200.0, 0.0, 60.0, 36.0);

#[test]
fn two_boxes_that_share_room_overlap() {
    let b = (50.0, 30.0, 60.0, 36.0);
    assert_eq!(
        found(vec![thing("a", A), thing("b", b)]),
        [(Kind::Boxes, 0, 1)]
    );
}

#[test]
fn two_boxes_that_share_a_border_do_not_overlap() {
    let b = (60.0, 0.0, 60.0, 36.0);
    assert_eq!(found(vec![thing("a", A), thing("b", b)]), []);
}

#[test]
fn a_group_holds_a_box_and_a_group_and_overlaps_what_it_holds_in_part() {
    let outer = group("outer", (0.0, 0.0, 300.0, 200.0));
    let inner = group("inner", (10.0, 30.0, 100.0, 100.0));
    let held = thing("held", (20.0, 60.0, 60.0, 36.0));
    assert_eq!(found(vec![outer.clone(), inner, held]), []);
    let half = thing("half", (280.0, 60.0, 60.0, 36.0));
    assert_eq!(found(vec![outer, half]), [(Kind::Boxes, 0, 1)]);
}

#[test]
fn a_thing_never_holds_a_thing() {
    let inner = thing("inner", (10.0, 10.0, 20.0, 10.0));
    assert_eq!(found(vec![thing("a", A), inner]), [(Kind::Boxes, 0, 1)]);
}

#[test]
fn a_link_that_crosses_a_thing_is_found_and_a_link_that_ends_at_one_is_not() {
    let between = thing("c", (100.0, 0.0, 60.0, 36.0));
    let straight = link("a", "b", &[(60.0, 18.0), (200.0, 18.0)]);
    assert_eq!(
        found(vec![
            thing("a", A),
            thing("b", B),
            between.clone(),
            straight
        ]),
        [(Kind::LinkOnThing, 3, 2)]
    );
    let around = [
        (60.0, 18.0),
        (80.0, 18.0),
        (80.0, 50.0),
        (180.0, 50.0),
        (180.0, 18.0),
        (200.0, 18.0),
    ];
    let shapes = vec![
        thing("a", A),
        thing("b", B),
        between.clone(),
        link("a", "b", &around),
    ];
    assert_eq!(found(shapes), []);
    // A slanted line that cuts one corner of the box.
    let corner = link("a", "b", &[(90.0, 20.0), (120.0, -20.0)]);
    assert_eq!(
        found(vec![between.clone(), corner]),
        [(Kind::LinkOnThing, 1, 0)]
    );
    // A line along the border of the box.
    let border = link("a", "b", &[(90.0, 36.0), (170.0, 36.0)]);
    assert_eq!(found(vec![between, border]), []);
}

#[test]
fn a_link_out_of_a_group_crosses_its_border_once() {
    let g = group("g", (0.0, 0.0, 120.0, 100.0));
    let a = thing("a", (20.0, 30.0, 60.0, 36.0));
    let b = thing("b", (200.0, 30.0, 60.0, 36.0));
    let once = link("a", "b", &[(80.0, 48.0), (200.0, 48.0)]);
    assert_eq!(found(vec![g.clone(), a.clone(), b.clone(), once]), []);
    let points = [
        (80.0, 48.0),
        (150.0, 48.0),
        (150.0, 150.0),
        (100.0, 150.0),
        (100.0, 80.0),
        (190.0, 80.0),
        (190.0, 48.0),
        (200.0, 48.0),
    ];
    let thrice = link("a", "b", &points);
    assert_eq!(found(vec![g, a, b, thrice]), [(Kind::LinkOnGroup, 3, 0)]);
}

#[test]
fn a_link_between_two_things_of_a_group_stays_in_the_group() {
    let g = group("g", (0.0, 0.0, 300.0, 100.0));
    let a = thing("a", (20.0, 30.0, 60.0, 36.0));
    let b = thing("b", (200.0, 30.0, 60.0, 36.0));
    let inside = link("a", "b", &[(80.0, 48.0), (200.0, 48.0)]);
    assert_eq!(found(vec![g.clone(), a.clone(), b.clone(), inside]), []);
    let points = [
        (80.0, 48.0),
        (140.0, 48.0),
        (140.0, 150.0),
        (180.0, 150.0),
        (180.0, 48.0),
        (200.0, 48.0),
    ];
    let outside = link("a", "b", &points);
    assert_eq!(found(vec![g, a, b, outside]), [(Kind::LinkOnGroup, 3, 0)]);
}

#[test]
fn a_link_through_a_group_that_holds_neither_end_is_found() {
    let g = group("g", (100.0, -50.0, 60.0, 200.0));
    let through = link("a", "b", &[(60.0, 18.0), (200.0, 18.0)]);
    assert_eq!(
        found(vec![g.clone(), thing("a", A), thing("b", B), through]),
        [(Kind::LinkThroughGroup, 3, 0)]
    );
    let around = [
        (60.0, 18.0),
        (80.0, 18.0),
        (80.0, 170.0),
        (180.0, 170.0),
        (180.0, 18.0),
        (200.0, 18.0),
    ];
    let shapes = vec![g, thing("a", A), thing("b", B), link("a", "b", &around)];
    assert_eq!(found(shapes), []);
}

#[test]
fn a_label_on_a_thing_that_it_does_not_name_is_found() {
    let own = label(&["a"], (10.0, 10.0, 40.0, 16.0));
    assert_eq!(found(vec![thing("a", A), own]), []);
    let other = label(&["b"], (10.0, 10.0, 40.0, 16.0));
    assert_eq!(
        found(vec![thing("a", A), other]),
        [(Kind::LabelOnThing, 1, 0)]
    );
}

#[test]
fn a_label_across_the_border_of_a_group_is_found() {
    let g = group("g", (0.0, 0.0, 120.0, 100.0));
    let inside = label(&["a"], (40.0, 60.0, 40.0, 16.0));
    assert_eq!(found(vec![g.clone(), inside]), []);
    let outside = label(&["a"], (40.0, 100.0, 40.0, 16.0));
    assert_eq!(found(vec![g.clone(), outside]), []);
    let bottom = label(&["a"], (40.0, 92.0, 40.0, 16.0));
    assert_eq!(found(vec![g.clone(), bottom]), [(Kind::LabelOnGroup, 1, 0)]);
    let side = label(&["a"], (100.0, 40.0, 40.0, 16.0));
    assert_eq!(found(vec![g.clone(), side]), [(Kind::LabelOnGroup, 1, 0)]);
    // The name of a group can be wider than its box.
    let name = label(&["g"], (10.0, 10.0, 200.0, 16.0));
    assert_eq!(found(vec![g, name]), []);
}

#[test]
fn the_label_of_a_link_has_its_centre_on_the_link() {
    let line = link(
        "a",
        "b",
        &[(60.0, 18.0), (130.0, 18.0), (130.0, 80.0), (200.0, 80.0)],
    );
    let on = label(&["a", "b"], (110.0, 40.0, 40.0, 16.0));
    assert_eq!(found(vec![line.clone(), on]), []);
    let off = label(&["a", "b"], (112.0, 40.0, 40.0, 16.0));
    assert_eq!(
        found(vec![line.clone(), off.clone()]),
        [(Kind::LabelOffLink, 1, 0)]
    );
    // A link the other way round is another link.
    let back = link("b", "a", &[(132.0, 0.0), (132.0, 100.0)]);
    assert_eq!(
        found(vec![line, back, off.clone()]),
        [(Kind::LabelOffLink, 2, 0)]
    );
    assert_eq!(found(vec![off]), [(Kind::LabelOffLink, 0, 0)]);
}

#[test]
fn two_labels_that_share_room_overlap() {
    let first = label(&["a"], (0.0, 0.0, 40.0, 16.0));
    let second = label(&["b"], (30.0, 10.0, 40.0, 16.0));
    assert_eq!(found(vec![first.clone(), second]), [(Kind::Labels, 0, 1)]);
    let beside = label(&["b"], (40.0, 0.0, 40.0, 16.0));
    assert_eq!(found(vec![first, beside]), []);
}

#[test]
fn two_links_with_no_end_in_common_do_not_lie_on_each_other() {
    let first = link("a", "b", &[(0.0, 10.0), (100.0, 10.0)]);
    let on = link("c", "d", &[(150.0, 10.0), (60.0, 10.0), (60.0, 90.0)]);
    assert_eq!(found(vec![first.clone(), on]), [(Kind::Links, 0, 1)]);
    // Two links from one thing share the line that leaves it.
    let shared = link("a", "d", &[(0.0, 10.0), (60.0, 10.0), (60.0, 90.0)]);
    assert_eq!(found(vec![first.clone(), shared]), []);
    let crossing = link("c", "d", &[(60.0, -40.0), (60.0, 90.0)]);
    assert_eq!(found(vec![first.clone(), crossing]), []);
    let after = link("c", "d", &[(100.0, 10.0), (180.0, 10.0)]);
    assert_eq!(found(vec![first, after]), []);
}

#[test]
fn a_collision_reads_as_a_sentence_with_both_names() {
    let scene = Scene {
        width: 300.0,
        height: 100.0,
        font_size: 12.0,
        colour: None,
        shapes: vec![
            thing("c", (100.0, 0.0, 60.0, 36.0)),
            link("a", "b", &[(60.0, 18.0), (200.0, 18.0)]),
        ],
    };
    let found = overlaps(&scene);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].describe(&scene), "link `a -> b` crosses thing `c`");
    let scene = Scene {
        shapes: vec![
            group("g", (0.0, 0.0, 120.0, 100.0)),
            label(&["a"], (40.0, 92.0, 40.0, 16.0)),
        ],
        ..scene
    };
    assert_eq!(
        overlaps(&scene)[0].describe(&scene),
        "label `a` lies across the border of group `g`"
    );
}
