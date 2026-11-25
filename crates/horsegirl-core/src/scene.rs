//! The scene: positioned shapes that every backend draws.
//!
//! A backend reads the scene and nothing else, so the SVG and the text backend
//! draw the same picture. The scene holds each box, line and label with its
//! position, its size, its role and the facts that a theme reads, such as
//! `owner`. It holds owned text and no reference into the model.
//!
//! The layout decides where things go and gives a [`Placement`]. [`build`]
//! joins the placement with the model: it names each shape, copies its facts
//! and measures each label.

use std::fmt::Write as _;

use crate::model::{Fact, Model};
use crate::text::{self, Style};

/// The room between the border of a group and the top-left corner of its
/// name. The layout must leave this room inside each group.
pub const GROUP_INSET: f64 = 8.0;

/// What a shape is a picture of. A theme styles a shape by its role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A thing that holds nothing: a box.
    Thing,
    /// A thing that holds other things: a box behind its parts.
    Group,
    /// A link: a line from its first point to its last, with an arrow head at
    /// the last.
    Link,
    /// The text of a thing, a group or a link.
    Label,
}

impl Role {
    /// The role as one lower-case word, as the JSON and a CSS class write it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Thing => "thing",
            Role::Group => "group",
            Role::Link => "link",
            Role::Label => "label",
        }
    }
}

/// A position in pixels. `x` grows to the right and `y` grows down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A box in pixels: its top-left corner and its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// One shape of the picture.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub role: Role,
    /// The names of what the shape draws: one name for a thing, a group or
    /// its label, and the two ends of a link for the link and its label. A
    /// backend can tie a shape to the file with these names.
    pub of: Vec<String>,
    /// The box that holds the shape. For a link, the box around its points.
    /// For a label, the box that its text fills: the text is centred in it.
    pub rect: Rect,
    /// The points of a link, from the start to the end, with each bend
    /// between. Empty for every other role.
    pub points: Vec<Point>,
    /// The text of a label. Empty for every other role.
    pub text: String,
    /// The facts of what the shape draws, in the order of the model.
    pub facts: Vec<Fact>,
}

/// The whole picture.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// The size of the picture. Every shape lies between the origin and this
    /// corner.
    pub width: f64,
    pub height: f64,
    /// The font size of every label, in pixels.
    pub font_size: f64,
    /// Every shape in the order to draw it: a later shape covers an earlier
    /// one. Groups come first, outer before inner, then links, then things,
    /// then labels.
    pub shapes: Vec<Shape>,
}

/// Where the layout put each part of a model.
///
/// Each list has one entry for each thing or link of the model, at the same
/// index. `None`, or a link with fewer than two points, is not drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placement {
    /// The box of each thing, groups included.
    pub things: Vec<Option<Rect>>,
    /// The points of each link, from its start to its end.
    pub links: Vec<Vec<Point>>,
    /// The centre of the label of each link. A link without a label needs none.
    pub labels: Vec<Option<Point>>,
}

/// Joins a placement with the model that it places.
///
/// The scene is moved so that its top-left shape touches the origin.
#[must_use]
pub fn build(model: &Model, placement: &Placement, style: Style) -> Scene {
    let groups = model.groups();
    let leaves: Vec<usize> = (0..model.things.len())
        .filter(|id| !groups.contains(id))
        .collect();
    let boxes = |id: usize| placement.things.get(id).copied().flatten();

    let mut shapes = Vec::new();
    // A group is drawn before its parts, so an outer group comes before an
    // inner one. Depth orders them; the sort is stable, so ties keep the
    // order of first mention.
    let mut ordered = groups.clone();
    ordered.sort_by_key(|&id| depth(model, id));
    for &id in &ordered {
        if let Some(rect) = boxes(id) {
            shapes.push(thing(model, id, Role::Group, rect));
        }
    }
    for index in 0..model.links.len() {
        let Some(points) = placement.links.get(index).filter(|p| p.len() >= 2) else {
            continue;
        };
        shapes.push(Shape {
            role: Role::Link,
            of: ends(model, index),
            rect: around(points),
            points: points.clone(),
            text: String::new(),
            facts: Vec::new(),
        });
    }
    for &id in &leaves {
        if let Some(rect) = boxes(id) {
            shapes.push(thing(model, id, Role::Thing, rect));
        }
    }

    for &id in &ordered {
        if let Some(rect) = boxes(id) {
            let size = text::measure(&model.things[id].name, style);
            let at = Point {
                x: rect.x + GROUP_INSET + size.width / 2.0,
                y: rect.y + GROUP_INSET + size.height / 2.0,
            };
            shapes.push(label(model, id, at, style));
        }
    }
    for &id in &leaves {
        if let Some(rect) = boxes(id) {
            let at = Point {
                x: rect.x + rect.width / 2.0,
                y: rect.y + rect.height / 2.0,
            };
            shapes.push(label(model, id, at, style));
        }
    }
    for (index, link) in model.links.iter().enumerate() {
        let drawn = placement.links.get(index).is_some_and(|p| p.len() >= 2);
        let at = placement.labels.get(index).copied().flatten();
        let Some(at) = at.filter(|_| drawn && !link.labels.is_empty()) else {
            continue;
        };
        // A link stated with two labels shows both.
        let text = link.labels.join(", ");
        shapes.push(Shape {
            role: Role::Label,
            of: ends(model, index),
            rect: centred(&text, at, style),
            points: Vec::new(),
            text,
            facts: Vec::new(),
        });
    }

    origin(Scene {
        width: 0.0,
        height: 0.0,
        font_size: style.size,
        shapes,
    })
}

fn depth(model: &Model, id: usize) -> usize {
    std::iter::successors(model.things[id].parent, |&p| model.things[p].parent).count()
}

fn thing(model: &Model, id: usize, role: Role, rect: Rect) -> Shape {
    Shape {
        role,
        of: vec![model.things[id].name.clone()],
        rect,
        points: Vec::new(),
        text: String::new(),
        facts: model.things[id].facts.clone(),
    }
}

fn label(model: &Model, id: usize, at: Point, style: Style) -> Shape {
    let thing = &model.things[id];
    Shape {
        role: Role::Label,
        of: vec![thing.name.clone()],
        rect: centred(&thing.name, at, style),
        points: Vec::new(),
        text: thing.name.clone(),
        facts: thing.facts.clone(),
    }
}

fn ends(model: &Model, link: usize) -> Vec<String> {
    let link = &model.links[link];
    vec![
        model.things[link.from].name.clone(),
        model.things[link.to].name.clone(),
    ]
}

/// The box of `text` with its centre at `at`.
fn centred(text: &str, at: Point, style: Style) -> Rect {
    let size = text::measure(text, style);
    Rect {
        x: at.x - size.width / 2.0,
        y: at.y - size.height / 2.0,
        width: size.width,
        height: size.height,
    }
}

fn around(points: &[Point]) -> Rect {
    let (mut left, mut top) = (f64::INFINITY, f64::INFINITY);
    let (mut right, mut bottom) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in points {
        left = left.min(p.x);
        top = top.min(p.y);
        right = right.max(p.x);
        bottom = bottom.max(p.y);
    }
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

/// Moves every shape so that the picture starts at the origin, and sets the
/// size of the picture.
fn origin(mut scene: Scene) -> Scene {
    if scene.shapes.is_empty() {
        return scene;
    }
    let corners = scene.shapes.iter().flat_map(|s| {
        let r = s.rect;
        [
            Point { x: r.x, y: r.y },
            Point {
                x: r.x + r.width,
                y: r.y + r.height,
            },
        ]
    });
    let bounds = around(&corners.collect::<Vec<_>>());
    for shape in &mut scene.shapes {
        shape.rect.x -= bounds.x;
        shape.rect.y -= bounds.y;
        for p in &mut shape.points {
            p.x -= bounds.x;
            p.y -= bounds.y;
        }
    }
    scene.width = bounds.width;
    scene.height = bounds.height;
    scene
}

impl Scene {
    /// The scene as JSON, one field on each line, so a change to a picture
    /// shows as a line diff in review.
    ///
    /// The bytes are the same on every machine: fields come in a fixed order,
    /// facts are a list of pairs in the order of the model, and each number
    /// is rounded to two decimals.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = format!(
            "{{\n  \"width\": {},\n  \"height\": {},\n  \"font_size\": {},\n",
            number(self.width),
            number(self.height),
            number(self.font_size)
        );
        if self.shapes.is_empty() {
            out.push_str("  \"shapes\": []\n}\n");
            return out;
        }
        out.push_str("  \"shapes\": [\n");
        let shapes: Vec<String> = self.shapes.iter().map(shape_json).collect();
        out.push_str(&shapes.join(",\n"));
        out.push_str("\n  ]\n}\n");
        out
    }
}

fn shape_json(shape: &Shape) -> String {
    let r = shape.rect;
    let mut fields = vec![
        format!("\"role\": {}", string(shape.role.as_str())),
        format!("\"of\": [{}]", list(shape.of.iter().map(|n| string(n)))),
        format!(
            "\"rect\": [{}, {}, {}, {}]",
            number(r.x),
            number(r.y),
            number(r.width),
            number(r.height)
        ),
    ];
    if shape.role == Role::Link {
        let points = shape
            .points
            .iter()
            .map(|p| format!("[{}, {}]", number(p.x), number(p.y)));
        fields.push(format!("\"points\": [{}]", list(points)));
    }
    if shape.role == Role::Label {
        fields.push(format!("\"text\": {}", string(&shape.text)));
    }
    let facts = shape
        .facts
        .iter()
        .map(|f| format!("[{}, {}]", string(&f.key), string(&f.value)));
    fields.push(format!("\"facts\": [{}]", list(facts)));
    format!("    {{\n      {}\n    }}", fields.join(",\n      "))
}

fn list(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

/// A number rounded to two decimals, with no trailing zero. Two decimals hide
/// the last bits of a sum, which can differ between two orders of the same
/// additions, and are finer than any screen draws.
pub(crate) fn number(value: f64) -> String {
    let fixed = format!("{value:.2}");
    let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// A JSON string.
fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                write!(out, "\\u{:04x}", u32::from(c)).expect("a String takes any write");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::{number, string};

    #[test]
    fn a_number_has_at_most_two_decimals_and_no_trailing_zero() {
        assert_eq!(number(12.0), "12");
        assert_eq!(number(12.5), "12.5");
        assert_eq!(number(0.125_000_1), "0.13");
        assert_eq!(number(-0.001), "0");
        assert_eq!(number(-3.25), "-3.25");
        assert_eq!(number(100.0), "100");
    }

    #[test]
    fn a_string_escapes_quotes_backslashes_and_control_characters() {
        assert_eq!(
            string("a \"b\" \\ c\n\u{1}"),
            "\"a \\\"b\\\" \\\\ c\\n\\u0001\""
        );
    }
}
