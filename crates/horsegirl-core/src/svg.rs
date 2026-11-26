//! The SVG backend: a scene drawn as one self-contained SVG document.
//!
//! Every colour and the font are CSS custom properties with a fallback, so a
//! page that holds the SVG inline restyles it with no rebuild: setting
//! `--hg-accent` on the page changes the arrow heads. The fallbacks draw a
//! plain picture where no property is set, such as in an image viewer.
//!
//! Each shape has a class for its role (`thing`, `group`, `link`, `label`) and
//! one for each of its facts, such as `owner-commerce`, so a theme can colour
//! by meaning. The document holds no script and loads nothing: the style is
//! inline and the arrow head is a marker in the same document.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::overlap::{self, Meet};
use crate::scene::{HEAD_LENGTH, LABEL_CUT, Point, Rect, Role, Scene, Shape, number};
use crate::text::{self, Style};
use crate::theme;

/// The room around the scene, so a stroke on its edge is not cut in half.
const MARGIN: f64 = 1.0;

/// The style of every picture. Each rule is scoped to the `horsegirl` class,
/// so a page that holds the SVG inline keeps its own `.label` and `.link`.
///
/// A label draws every space that `text::measure` counts: SVG would otherwise
/// collapse leading, trailing and repeated spaces, and `textLength` would
/// stretch the text to a width that it does not have.
///
/// The first three values of the colour fact take the palette slots 1 to 3,
/// and a fourth value takes slot 1 again. `color-scheme` lets `light-dark()` follow the reader's
/// mode; a page sets `--hg-scheme` to `light` or `dark` to force one.
fn style() -> String {
    let v = theme::var;
    let mut css = format!(
        ".horsegirl{{color-scheme:var(--hg-scheme,light dark);font-family:var(--hg-font,{})}}\n\
         .horsegirl .page{{fill:{}}}\n\
         .horsegirl .group{{fill:{};stroke:{}}}\n\
         .horsegirl .thing{{fill:{};stroke:{}}}\n\
         .horsegirl .link{{fill:none;stroke:{}}}\n\
         .horsegirl .head{{fill:{}}}\n\
         .horsegirl .label{{fill:{};white-space:pre}}\n",
        theme::FONT,
        v("paper"),
        v("c0"),
        v("line"),
        v("paper"),
        v("muted"),
        v("ink"),
        v("accent"),
        v("ink"),
    );
    for (slot, stroke) in [(1, "accent"), (2, "s2"), (3, "s3")] {
        writeln!(
            css,
            ".horsegirl .thing.slot-{slot}{{fill:{};stroke:{}}}",
            v(&format!("c{slot}")),
            v(stroke)
        )
        .expect("a String takes any write");
    }
    css
}

/// Draws `scene` as an SVG document.
///
/// `title` names the picture: the document's `title` holds it, and a screen
/// reader reads it as the name of the image. The `desc` counts what the
/// picture shows.
#[must_use]
pub fn draw(scene: &Scene, title: &str) -> String {
    let width = number(scene.width + 2.0 * MARGIN);
    let height = number(scene.height + 2.0 * MARGIN);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"horsegirl\" role=\"img\" \
         width=\"{width}\" height=\"{height}\" viewBox=\"{} {} {width} {height}\" font-size=\"{}\">\n",
        number(-MARGIN),
        number(-MARGIN),
        number(scene.font_size),
    );
    writeln!(out, "<title>{}</title>", escape(title)).expect("a String takes any write");
    writeln!(out, "<desc>{}</desc>", describe(scene)).expect("a String takes any write");
    writeln!(out, "<style>\n{}</style>", style()).expect("a String takes any write");
    let (n, half) = (number(HEAD_LENGTH), number(HEAD_LENGTH / 2.0));
    writeln!(
        out,
        "<defs>\n<marker id=\"hg-head\" viewBox=\"0 0 {n} {n}\" refX=\"{n}\" refY=\"{half}\" \
         markerWidth=\"{n}\" markerHeight=\"{n}\" markerUnits=\"userSpaceOnUse\" orient=\"auto\">\
         <path class=\"head\" d=\"M0,0 L{n},{half} L0,{n} z\"/></marker>\n</defs>"
    )
    .expect("a String takes any write");
    // The page colour is drawn, not assumed: link strokes and labels sit
    // outside every box, and on a host page of the other mode they would
    // vanish.
    writeln!(
        out,
        "<rect class=\"page\" x=\"{}\" y=\"{}\" width=\"{width}\" height=\"{height}\"/>",
        number(-MARGIN),
        number(-MARGIN),
    )
    .expect("a String takes any write");
    let style = Style {
        size: scene.font_size,
    };
    let owners = owners(scene);
    let colour = scene.colour.as_deref();
    let by_ends = overlap::parts_by_ends(scene.shapes.iter().filter(|s| s.role == Role::Link));
    for shape in &scene.shapes {
        out.push_str(&element(scene, shape, style, (colour, &owners), &by_ends));
    }
    out.push_str("</svg>\n");
    out
}

fn element(
    scene: &Scene,
    shape: &Shape,
    style: Style,
    (colour, owners): (Option<&str>, &[&str]),
    by_ends: &BTreeMap<&[String], Vec<(Point, Point)>>,
) -> String {
    let class = class(shape, colour, owners);
    let r = shape.rect;
    match shape.role {
        Role::Group | Role::Thing => format!(
            "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\"/>\n",
            number(r.x),
            number(r.y),
            number(r.width),
            number(r.height)
        ),
        // The layout puts a link label on its own line, and the text has no
        // background: a colour behind it would have to match the page or the
        // group that it sits on. So the line stops at the label instead, and
        // the label stays where the layout put it. A label of another link
        // between the same two things cuts the line too: both name the pair.
        // So does a label on a line that this link shares with the link of
        // the label, such as the line that two links from one thing leave
        // on: that line stops at the label in both links, or it would pass
        // through the text. The overlap detector decides with the same
        // function, so any other line through a label is a collision.
        Role::Link => {
            let along = |label: &Shape| overlap::meet(shape, label, by_ends) == Meet::Along;
            let holes: Vec<Rect> = scene
                .shapes
                .iter()
                .filter(|s| s.role == Role::Label && (s.of == shape.of || along(s)))
                .map(|s| pad(s.rect))
                .collect();
            let mut d = String::new();
            for run in cut(&shape.points, &holes) {
                for (i, p) in run.iter().enumerate() {
                    let pen = if i == 0 { 'M' } else { 'L' };
                    write!(
                        d,
                        "{}{pen}{},{}",
                        if d.is_empty() { "" } else { " " },
                        number(p.x),
                        number(p.y)
                    )
                    .expect("a String takes any write");
                }
            }
            format!("<path class=\"{class}\" d=\"{d}\" marker-end=\"url(#hg-head)\"/>\n")
        }
        // The baseline comes from the measured face, not from
        // `dominant-baseline`, which renderers treat differently.
        //
        // The width of a label box is its measured width. `textLength` holds
        // the drawn text to that width, so a viewer that falls back to a
        // wider or narrower face squeezes or spreads the label to fit its box
        // instead of overflowing it. In a face with Arial metrics the change
        // is too small to see (SPIKE-19).
        //
        // A character outside the metrics table measures as one em, an
        // over-estimate. Held to that width, an Arabic label draws at twice
        // its natural width, so such a label keeps its natural width.
        Role::Label => {
            let length = if text::holds(&shape.text) && r.width > 0.0 {
                format!(
                    " textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\"",
                    number(r.width)
                )
            } else {
                String::new()
            };
            format!(
                "<text class=\"{class}\" x=\"{}\" y=\"{}\" text-anchor=\"middle\"{length}>{}</text>\n",
                number(r.x + r.width / 2.0),
                number(r.y + text::ascent(style)),
                escape(&shape.text)
            )
        }
    }
}

fn pad(r: Rect) -> Rect {
    Rect {
        x: r.x - LABEL_CUT,
        y: r.y - LABEL_CUT,
        width: r.width + 2.0 * LABEL_CUT,
        height: r.height + 2.0 * LABEL_CUT,
    }
}

/// The parts of the line through `points` that lie outside every hole, each
/// as a run of points from its start to its end.
fn cut(points: &[Point], holes: &[Rect]) -> Vec<Vec<Point>> {
    let mut runs: Vec<Vec<Point>> = Vec::new();
    let mut down = false;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let at = |t: f64| Point {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        };
        let mut hidden: Vec<(f64, f64)> = holes.iter().filter_map(|h| inside(a, b, *h)).collect();
        hidden.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut shown = Vec::new();
        let mut t = 0.0;
        for (t0, t1) in hidden {
            if t0 > t {
                shown.push((t, t0));
            }
            t = f64::max(t, t1);
        }
        if t < 1.0 {
            shown.push((t, 1.0));
        }
        let mut next = false;
        for (start, end) in shown {
            match runs.last_mut() {
                Some(run) if down && start <= 0.0 => run.push(at(end)),
                _ => runs.push(vec![at(start), at(end)]),
            }
            next = end >= 1.0;
        }
        down = next;
    }
    runs
}

/// The part of the segment from `a` to `b` inside `r`, as the range of `t`
/// in `a + (b - a) * t`. `None` when the segment misses `r` or runs along
/// its border: the layout keeps a line of another link that far from a
/// label, and that line stays whole.
fn inside(a: Point, b: Point, r: Rect) -> Option<(f64, f64)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, a.x - r.x),
        (dx, r.x + r.width - a.x),
        (-dy, a.y - r.y),
        (dy, r.y + r.height - a.y),
    ] {
        if p.abs() < f64::EPSILON {
            if q <= 0.0 {
                return None;
            }
        } else if p < 0.0 {
            t0 = t0.max(q / p);
        } else {
            t1 = t1.min(q / p);
        }
    }
    (t0 < t1).then_some((t0, t1))
}

/// The role of a shape, the palette slot of its colour fact, then
/// `key-value` for each of its facts.
fn class(shape: &Shape, colour: Option<&str>, owners: &[&str]) -> String {
    let mut class = shape.role.as_str().to_owned();
    if shape.role == Role::Thing {
        let owner = shape
            .facts
            .iter()
            .find(|fact| Some(fact.key.as_str()) == colour);
        if let Some(at) = owner.and_then(|fact| owners.iter().position(|o| *o == fact.value)) {
            write!(class, " slot-{}", at % 3 + 1).expect("a String takes any write");
        }
    }
    for fact in &shape.facts {
        write!(class, " {}-{}", token(&fact.key), token(&fact.value))
            .expect("a String takes any write");
    }
    class
}

/// A fact key or value as part of a class name: lower case, with each
/// character that a CSS selector would need to escape replaced by `-`. A quoted
/// value such as `"Platform team"` gives `platform-team`. A letter outside
/// ASCII stays, since a selector takes it as it is.
fn token(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            let c = if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            };
            c.to_lowercase()
        })
        .collect()
}

/// Each distinct value of the colour fact of the scene, in the order of the
/// shapes, so the same file gets the same colours on every run.
fn owners(scene: &Scene) -> Vec<&str> {
    let mut owners: Vec<&str> = Vec::new();
    let Some(key) = &scene.colour else {
        return owners;
    };
    let things = scene
        .shapes
        .iter()
        .filter(|shape| shape.role == Role::Thing);
    let facts = things.flat_map(|shape| &shape.facts);
    for fact in facts.filter(|fact| &fact.key == key) {
        if !owners.contains(&fact.value.as_str()) {
            owners.push(&fact.value);
        }
    }
    owners
}

/// What the picture shows, counted, for the `desc` of the document.
fn describe(scene: &Scene) -> String {
    let count = |role: Role| scene.shapes.iter().filter(|s| s.role == role).count();
    let noun = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    format!(
        "{}, {} and {}.",
        noun(count(Role::Thing), "thing", "things"),
        noun(count(Role::Group), "group", "groups"),
        noun(count(Role::Link), "link", "links"),
    )
}

/// Text as XML character data or as the value of a quoted attribute.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // XML 1.0 has no form for these, not even as a reference.
            '\u{fffe}' | '\u{ffff}' => out.push('\u{fffd}'),
            c if u32::from(c) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => out.push('\u{fffd}'),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{escape, token};

    #[test]
    fn escape_replaces_markup_and_drops_characters_xml_cannot_hold() {
        assert_eq!(
            escape("a<b & \"c\" 'd'>\u{1}\u{fffe}\u{ffff}"),
            "a&lt;b &amp; &quot;c&quot; &apos;d&apos;&gt;\u{fffd}\u{fffd}\u{fffd}"
        );
    }

    #[test]
    fn a_token_is_lower_case_with_no_character_a_selector_must_escape() {
        assert_eq!(token("Commerce"), "commerce");
        assert_eq!(token("Platform team"), "platform-team");
        assert_eq!(token("a.b:c/d"), "a-b-c-d");
        assert_eq!(token("Ünï"), "ünï");
    }
}
