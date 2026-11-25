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

use std::fmt::Write as _;

use crate::scene::{Role, Scene, Shape, number};
use crate::text::{self, Style};

/// The room around the scene, so a stroke on its edge is not cut in half.
const MARGIN: f64 = 1.0;

/// The style of every picture. Each rule is scoped to the `horsegirl` class,
/// so a page that holds the SVG inline keeps its own `.label` and `.link`.
const STYLE: &str = "\
.horsegirl{font-family:var(--hg-font,Arial,\"Liberation Sans\",Helvetica,sans-serif)}
.horsegirl .group{fill:var(--hg-group,#F1EAF1);stroke:var(--hg-line,#6E5F72)}
.horsegirl .thing{fill:var(--hg-surface,#FFFFFF);stroke:var(--hg-line,#6E5F72)}
.horsegirl .link{fill:none;stroke:var(--hg-line,#6E5F72)}
.horsegirl .head{fill:var(--hg-accent,#D4246F)}
.horsegirl .label{fill:var(--hg-ink,#241627)}
";

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
    writeln!(out, "<style>\n{STYLE}</style>").expect("a String takes any write");
    out.push_str(
        "<defs>\n<marker id=\"hg-head\" viewBox=\"0 0 8 8\" refX=\"8\" refY=\"4\" \
         markerWidth=\"8\" markerHeight=\"8\" markerUnits=\"userSpaceOnUse\" orient=\"auto\">\
         <path class=\"head\" d=\"M0,0 L8,4 L0,8 z\"/></marker>\n</defs>\n",
    );
    let style = Style {
        size: scene.font_size,
    };
    for shape in &scene.shapes {
        out.push_str(&element(shape, style));
    }
    out.push_str("</svg>\n");
    out
}

fn element(shape: &Shape, style: Style) -> String {
    let class = class(shape);
    let r = shape.rect;
    match shape.role {
        Role::Group | Role::Thing => format!(
            "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\"/>\n",
            number(r.x),
            number(r.y),
            number(r.width),
            number(r.height)
        ),
        Role::Link => {
            let points: Vec<String> = shape
                .points
                .iter()
                .map(|p| format!("{},{}", number(p.x), number(p.y)))
                .collect();
            format!(
                "<polyline class=\"{class}\" points=\"{}\" marker-end=\"url(#hg-head)\"/>\n",
                points.join(" ")
            )
        }
        // The baseline comes from the measured face, not from
        // `dominant-baseline`, which renderers treat differently.
        Role::Label => format!(
            "<text class=\"{class}\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>\n",
            number(r.x + r.width / 2.0),
            number(r.y + text::ascent(style)),
            escape(&shape.text)
        ),
    }
}

/// The role of a shape, then `key-value` for each of its facts.
fn class(shape: &Shape) -> String {
    let mut class = shape.role.as_str().to_owned();
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
