//! The scene: the shapes that a backend reads instead of the model.
//!
//! The placement comes from the stand-in for the layout in `support`. The
//! tests hold what the scene does with a placement, not the placement.
//!
//! To accept a change to `checkout.scene.json`, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test scene
//! ```

mod support;

use horsegirl_core::scene::{Placement, Point, Rect, Role, Scene, build};
use support::{STYLE, model, scene};

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const CHECKOUT_SCENE: &str = include_str!("checkout.scene.json");

/// A backend in miniature: it draws a scene as SVG and is given nothing else.
/// If the scene lacked a thing that a picture needs, this function could not
/// draw it.
fn draw(scene: &Scene) -> String {
    let mut lines = vec![format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        scene.width, scene.height
    )];
    for shape in &scene.shapes {
        let class: Vec<String> = std::iter::once(shape.role.as_str().to_owned())
            .chain(
                shape
                    .facts
                    .iter()
                    .map(|f| format!("{}-{}", f.key, f.value.to_lowercase())),
            )
            .collect();
        let class = class.join(" ");
        let r = shape.rect;
        lines.push(match shape.role {
            Role::Group | Role::Thing => format!(
                "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>\n",
                r.x, r.y, r.width, r.height
            ),
            Role::Link => {
                let points: Vec<String> = shape
                    .points
                    .iter()
                    .map(|p| format!("{},{}", p.x, p.y))
                    .collect();
                format!(
                    "<polyline class=\"{class}\" points=\"{}\" marker-end=\"url(#arrow)\"/>\n",
                    points.join(" ")
                )
            }
            Role::Label => format!(
                "<text class=\"{class}\" x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
                r.x + r.width / 2.0,
                r.y + r.height / 2.0,
                scene.font_size,
                shape.text
            ),
        });
    }
    lines.push("</svg>\n".to_owned());
    lines.concat()
}

#[test]
fn the_reference_scene_serialises_to_its_json() {
    let json = scene(CHECKOUT).to_json();
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/checkout.scene.json");
        std::fs::write(path, &json).unwrap();
        return;
    }
    assert_eq!(
        json, CHECKOUT_SCENE,
        "the scene changed; read the diff, then run HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test scene"
    );
}

#[test]
fn the_same_input_gives_the_same_json() {
    let first = scene(CHECKOUT).to_json();
    for _ in 0..3 {
        assert_eq!(scene(CHECKOUT).to_json(), first);
    }
}

#[test]
fn the_reference_scene_draws_with_no_access_to_the_model() {
    let scene = scene(CHECKOUT);
    let svg = draw(&scene);
    let count = |role: Role| scene.shapes.iter().filter(|s| s.role == role).count();
    assert_eq!(count(Role::Thing), 9);
    assert_eq!(count(Role::Group), 1);
    assert_eq!(count(Role::Link), 9);
    // A label for each thing, the group, and each of the seven links that the
    // story labels.
    assert_eq!(count(Role::Label), 17);
    assert_eq!(svg.matches("<rect ").count(), 10);
    assert_eq!(svg.matches("<polyline ").count(), 9);
    assert_eq!(svg.matches("<text ").count(), 17);

    let model = model(CHECKOUT);
    for thing in &model.things {
        assert!(
            svg.contains(&format!(">{}</text>", thing.name)),
            "{}",
            thing.name
        );
    }
    for label in model.links.iter().flat_map(|link| &link.labels) {
        assert!(svg.contains(&format!(">{label}</text>")), "{label}");
    }
    for owner in model.owners() {
        let class = format!("owner-{}", owner.to_lowercase());
        assert!(svg.contains(&class), "{class}");
    }
    let group = scene.shapes.iter().find(|s| s.role == Role::Group).unwrap();
    assert_eq!(group.of, ["Services"]);
    assert_eq!(
        svg.find("<rect class=\"group\""),
        svg.find("<rect "),
        "the group is drawn first"
    );
    for link in &model.links {
        let ends = [
            model.things[link.from].name.as_str(),
            model.things[link.to].name.as_str(),
        ];
        let drawn = scene
            .shapes
            .iter()
            .any(|s| s.role == Role::Link && s.of == ends && s.points.len() >= 2);
        assert!(drawn, "{ends:?}");
    }
}

#[test]
fn every_shape_lies_inside_the_scene() {
    let scene = scene(CHECKOUT);
    let inside =
        |x: f64, y: f64| (0.0..=scene.width).contains(&x) && (0.0..=scene.height).contains(&y);
    for shape in &scene.shapes {
        let r = shape.rect;
        assert!(
            inside(r.x, r.y) && inside(r.x + r.width, r.y + r.height),
            "{shape:?}"
        );
        assert!(shape.points.iter().all(|p| inside(p.x, p.y)), "{shape:?}");
    }
    let left = scene
        .shapes
        .iter()
        .map(|s| s.rect.x)
        .fold(f64::INFINITY, f64::min);
    let top = scene
        .shapes
        .iter()
        .map(|s| s.rect.y)
        .fold(f64::INFINITY, f64::min);
    assert_eq!((left, top), (0.0, 0.0));
}

#[test]
fn an_unplaced_thing_or_link_is_not_drawn() {
    let model = model("A -> B: go\nC\n");
    let placement = Placement {
        things: vec![
            Some(Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            None,
        ],
        links: vec![vec![Point { x: 10.0, y: 5.0 }]],
        labels: vec![Some(Point { x: 20.0, y: 5.0 })],
    };
    let scene = build(&model, &placement, STYLE);
    let roles: Vec<Role> = scene.shapes.iter().map(|s| s.role).collect();
    assert_eq!(roles, [Role::Thing, Role::Label]);
    assert_eq!(scene.shapes[1].text, "A");
}

#[test]
fn an_empty_placement_gives_an_empty_scene() {
    let scene = build(&model("A -> B\n"), &Placement::default(), STYLE);
    assert_eq!(
        scene.to_json(),
        "{\n  \"width\": 0,\n  \"height\": 0,\n  \"font_size\": 12,\n  \"shapes\": []\n}\n"
    );
}

#[test]
fn outer_groups_come_before_inner_groups() {
    let model = model("Inner { A }\nOuter { Inner }\n");
    let unit = Rect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };
    let placement = Placement {
        things: vec![Some(unit); model.things.len()],
        ..Placement::default()
    };
    let scene = build(&model, &placement, STYLE);
    let groups: Vec<&str> = scene
        .shapes
        .iter()
        .filter(|s| s.role == Role::Group)
        .map(|s| s.of[0].as_str())
        .collect();
    assert_eq!(groups, ["Outer", "Inner"]);
}

#[test]
fn a_link_stated_with_two_labels_shows_both() {
    let model = model("A -> B: one\nA -> B: two\n");
    assert_eq!(model.links.len(), 1, "the two statements are one link");
    let unit = |x: f64| Rect {
        x,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };
    let placement = Placement {
        things: vec![Some(unit(0.0)), Some(unit(40.0))],
        links: vec![vec![Point { x: 10.0, y: 5.0 }, Point { x: 40.0, y: 5.0 }]],
        labels: vec![Some(Point { x: 25.0, y: 5.0 })],
    };
    let scene = build(&model, &placement, STYLE);
    let label = scene
        .shapes
        .iter()
        .find(|s| s.role == Role::Label && s.of == ["A", "B"])
        .unwrap();
    assert_eq!(label.text, "one, two");
}
