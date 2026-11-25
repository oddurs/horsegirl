//! The scene: the shapes that a backend reads instead of the model.
//!
//! The layered layout does not exist yet, so `place` below stands in for it: it
//! ranks the reference file by its links and puts each rank in a column. The
//! tests hold what the scene does with a placement, not the placement.
//!
//! To accept a change to `checkout.scene.json`, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test scene
//! ```

use horsegirl_core::model::{Model, lower};
use horsegirl_core::scene::{GROUP_INSET, Placement, Point, Rect, Role, Scene, build};
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const CHECKOUT_SCENE: &str = include_str!("checkout.scene.json");
const STYLE: Style = Style { size: 12.0 };

const BOX: (f64, f64) = (120.0, 36.0);
const GAP: (f64, f64) = (80.0, 24.0);

fn model(source: &str) -> Model {
    lower(&parse(source).tree)
}

/// A stand-in for the layout: each thing in the column of its longest path
/// from a thing that nothing links to, each group around its parts, each link
/// a line with one bend, and each label at the middle of its line.
fn place(model: &Model) -> Placement {
    let groups = model.groups();
    let leaves: Vec<usize> = (0..model.things.len())
        .filter(|id| !groups.contains(id))
        .collect();
    let mut rank = vec![0_usize; model.things.len()];
    for _ in 0..model.things.len() {
        for link in &model.links {
            rank[link.to] = rank[link.to].max(rank[link.from] + 1);
        }
    }
    let mut things = vec![None; model.things.len()];
    let mut filled = vec![0_usize; model.things.len()];
    for &id in &leaves {
        let row = filled[rank[id]];
        filled[rank[id]] += 1;
        things[id] = Some(Rect {
            x: 40.0 + to_f64(rank[id]) * (BOX.0 + GAP.0),
            y: 40.0 + to_f64(row) * (BOX.1 + GAP.1),
            width: BOX.0,
            height: BOX.1,
        });
    }
    for &group in &groups {
        let parts: Vec<Rect> = model
            .parts(group)
            .iter()
            .filter_map(|&p| things[p])
            .collect();
        let left = parts.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
        let top = parts.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
        let right = parts.iter().map(|r| r.x + r.width).fold(0.0, f64::max);
        let bottom = parts.iter().map(|r| r.y + r.height).fold(0.0, f64::max);
        // Room above the parts for the name of the group.
        let title = 2.0 * GROUP_INSET + STYLE.size;
        things[group] = Some(Rect {
            x: left - GROUP_INSET,
            y: top - title,
            width: right - left + 2.0 * GROUP_INSET,
            height: bottom - top + title + GROUP_INSET,
        });
    }
    let mut links = Vec::new();
    let mut labels = Vec::new();
    for link in &model.links {
        let (a, b) = (things[link.from].unwrap(), things[link.to].unwrap());
        let start = Point {
            x: a.x + a.width,
            y: a.y + a.height / 2.0,
        };
        let end = Point {
            x: b.x,
            y: b.y + b.height / 2.0,
        };
        let bend = Point {
            x: f64::midpoint(start.x, end.x),
            y: start.y,
        };
        let turn = Point {
            x: bend.x,
            y: end.y,
        };
        labels.push(Some(Point {
            x: bend.x,
            y: f64::midpoint(start.y, end.y),
        }));
        links.push(vec![start, bend, turn, end]);
    }
    Placement {
        things,
        links,
        labels,
    }
}

fn to_f64(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap())
}

fn scene(source: &str) -> Scene {
    let model = model(source);
    build(&model, &place(&model), STYLE)
}

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
