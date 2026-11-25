//! The SVG backend, held by a snapshot of the reference flow.
//!
//! The placement comes from the layered layout, through the flow lens. To
//! accept a change to `checkout.svg`, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test svg
//! ```
//!
//! Then open the file in a browser and look at it before you commit it.

use horsegirl_core::lens::{self, FLOW};
use horsegirl_core::model::{Model, lower};
use horsegirl_core::scene::{Role, Scene};
use horsegirl_core::svg::draw;
use horsegirl_core::syntax::parse;
use horsegirl_core::text::{Style, measure};

const STYLE: Style = Style { size: 12.0 };

fn model(source: &str) -> Model {
    lower(&parse(source).tree)
}

fn scene(source: &str) -> Scene {
    lens::draw(&model(source), &FLOW, STYLE)
}

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const CHECKOUT_SVG: &str = include_str!("checkout.svg");

fn checkout() -> String {
    draw(&scene(CHECKOUT), "checkout")
}

#[test]
fn the_reference_flow_draws_to_its_svg() {
    let svg = checkout();
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/checkout.svg");
        std::fs::write(path, &svg).unwrap();
        return;
    }
    assert_eq!(
        svg, CHECKOUT_SVG,
        "the SVG changed; read the diff, then run HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test svg"
    );
}

#[test]
fn the_svg_is_well_formed_xml() {
    well_formed(&checkout()).unwrap();
    let source = "<a> & 'b\" -> C: x < y\nC { owner: \"Platform team\" }\n";
    let svg = draw(&scene(source), "a & b");
    well_formed(&svg).unwrap_or_else(|e| panic!("{e}\n{svg}"));
    assert!(
        svg.contains(">&lt;a&gt; &amp; &apos;b&quot;</text>"),
        "{svg}"
    );
    assert!(svg.contains(">x &lt; y</text>"), "{svg}");
    assert!(
        svg.contains("class=\"thing slot-1 owner-platform-team\""),
        "{svg}"
    );
    assert!(svg.contains("<title>a &amp; b</title>"), "{svg}");
}

#[test]
fn a_character_xml_cannot_hold_is_drawn_as_a_replacement() {
    let svg = draw(&scene("\"a\u{1}\u{fffe}\u{ffff}b\" -> B\n"), "a\u{ffff}");
    well_formed(&svg).unwrap_or_else(|e| panic!("{e}\n{svg}"));
    assert!(svg.contains(">a\u{fffd}\u{fffd}\u{fffd}b</text>"), "{svg}");
    assert!(svg.contains("<title>a\u{fffd}</title>"), "{svg}");
}

#[test]
fn the_svg_has_no_script_and_loads_nothing() {
    let svg = checkout().to_lowercase();
    for banned in [
        "<script",
        "javascript:",
        "href",
        "@import",
        "<image",
        "<foreignobject",
    ] {
        assert!(!svg.contains(banned), "{banned}");
    }
    // An attribute such as onload runs script.
    for (at, _) in svg.match_indices(" on") {
        let name: String = svg[at + 1..]
            .chars()
            .take_while(char::is_ascii_alphabetic)
            .collect();
        assert!(!svg[at + 1 + name.len()..].starts_with('='), "{name}");
    }
    // The namespace is a name, not a resource. Every url() points into the
    // document itself.
    assert_eq!(svg.matches("http").count(), 1);
    assert_eq!(svg.matches("url(").count(), svg.matches("url(#").count());
}

#[test]
fn every_colour_and_the_font_is_a_css_variable_with_a_fallback() {
    let svg = checkout();
    let style = &svg[svg.find("<style>").unwrap()..svg.find("</style>").unwrap()];
    for declaration in style.split(['{', ';', '}']).filter(|d| d.contains(':')) {
        let (property, value) = declaration.split_once(':').unwrap();
        if value == "none" || value == "pre" {
            continue;
        }
        assert!(
            value.starts_with("var(--hg-") && value.contains(','),
            "{property}: {value}"
        );
    }
    assert!(style.contains("--hg-accent"));
    assert!(style.contains("var(--hg-font,Arial,\"Liberation Sans\",Helvetica,sans-serif)"));
}

#[test]
fn each_shape_has_a_class_for_its_role_and_for_its_facts() {
    let svg = checkout();
    let model = model(CHECKOUT);
    for owner in model.owners() {
        let class = format!(" owner-{}", owner.to_lowercase());
        assert!(svg.contains(&format!("{class}\" x=")), "{class}");
        assert!(svg.contains(&format!("class=\"label{class}\"")), "{class}");
    }
    assert_eq!(svg.matches("class=\"group").count(), 1);
    assert_eq!(svg.matches("class=\"link").count(), 9);
    for thing in &model.things {
        assert!(
            svg.contains(&format!(">{}</text>", thing.name)),
            "{}",
            thing.name
        );
    }
}

#[test]
fn each_label_is_held_to_its_measured_width() {
    let svg = checkout();
    let labels: Vec<&str> = svg.lines().filter(|l| l.starts_with("<text ")).collect();
    let shapes = scene(CHECKOUT).shapes;
    let texts: Vec<&str> = shapes
        .iter()
        .filter(|s| s.role == Role::Label)
        .map(|s| s.text.as_str())
        .collect();
    assert_eq!(labels.len(), texts.len());
    for (line, text) in labels.iter().zip(texts) {
        let (_, rest) = line.split_once(" textLength=\"").expect(line);
        let (length, rest) = rest.split_once('"').unwrap();
        assert!(
            rest.starts_with(" lengthAdjust=\"spacingAndGlyphs\""),
            "{line}"
        );
        let length: f64 = length.parse().unwrap();
        // The SVG writes two decimals.
        let width = measure(text, STYLE).width;
        assert!((length - width).abs() <= 0.005, "{line}: {width}");
    }
}

// The table measures an unknown character as one em. Held to that width, an
// Arabic label draws at more than twice its natural width.
#[test]
fn a_label_outside_the_metrics_table_keeps_its_natural_width() {
    let svg = draw(&scene("\"مرحبا بالعالم\" -> B\n"), "arabic");
    let arabic = svg
        .lines()
        .find(|l| l.ends_with(">مرحبا بالعالم</text>"))
        .expect(&svg);
    assert!(!arabic.contains("textLength"), "{arabic}");
    assert!(!arabic.contains("lengthAdjust"), "{arabic}");
    let latin = svg.lines().find(|l| l.ends_with(">B</text>")).expect(&svg);
    assert!(latin.contains(" textLength=\""), "{latin}");
}

// SVG collapses leading, trailing and repeated spaces unless the label says
// to keep them, and `textLength` would then stretch the text (BUG-158).
#[test]
fn a_label_keeps_the_white_space_that_its_width_counts() {
    let svg = draw(
        &scene("\"  Wide name  \" -> E\n\"Wide  name\" -> F\n"),
        "spaces",
    );
    assert!(svg.contains(".horsegirl .label{fill:"), "{svg}");
    let rule = svg
        .lines()
        .find(|l| l.contains(".horsegirl .label{"))
        .expect(&svg);
    assert!(rule.contains(";white-space:pre}"), "{rule}");
    for text in ["  Wide name  ", "Wide  name"] {
        let line = svg
            .lines()
            .find(|l| l.ends_with(&format!(">{text}</text>")))
            .expect(&svg);
        let (_, rest) = line.split_once(" textLength=\"").expect(line);
        let length: f64 = rest.split_once('"').unwrap().0.parse().unwrap();
        assert!((length - measure(text, STYLE).width).abs() < 0.01, "{line}");
    }
}

// Zero is not a width a renderer can stretch text to.
#[test]
fn an_empty_label_has_no_text_length() {
    let svg = draw(&scene("\"\" -> B\n"), "empty");
    let empty = svg.lines().find(|l| l.ends_with("></text>")).expect(&svg);
    assert!(!empty.contains("textLength"), "{empty}");
}

#[test]
fn the_svg_has_a_title_and_a_desc() {
    let svg = checkout();
    assert!(svg.contains("<title>checkout</title>"));
    assert!(svg.contains("<desc>9 things, 1 group and 9 links.</desc>"));
}

#[test]
fn the_same_scene_gives_the_same_bytes() {
    let first = checkout();
    for _ in 0..3 {
        assert_eq!(checkout(), first);
    }
}

#[test]
fn the_check_for_well_formed_xml_rejects_what_xml_does_not_allow() {
    assert!(well_formed("<a><b/></a>").is_ok());
    for bad in [
        "<a><b></a>",
        "<a>",
        "<a/><a/>",
        "<a>x & y</a>",
        "<a b=\"1\" b=\"2\"/>",
        "<a b=1/>",
        "x<a/>",
        "<a>\u{ffff}</a>",
        "<a b=\"\u{fffe}\"/>",
        "<a>\u{1}</a>",
    ] {
        assert!(well_formed(bad).is_err(), "{bad}");
    }
}

/// Checks that `xml` is one element with balanced tags, quoted attributes and
/// only the five predefined entities, and that every character is one the
/// XML `Char` production allows. It reads the subset of XML that the backend
/// writes: no comments, no CDATA and no declarations.
fn well_formed(xml: &str) -> Result<(), String> {
    // A Rust char is never a surrogate, so these are the only gaps in `Char`.
    if let Some(c) = xml.chars().find(|&c| {
        matches!(c, '\u{fffe}' | '\u{ffff}')
            || (u32::from(c) < 0x20 && !matches!(c, '\t' | '\n' | '\r'))
    }) {
        return Err(format!("{c:?} is not an XML character"));
    }
    let mut open: Vec<&str> = Vec::new();
    let mut roots = 0;
    let mut rest = xml;
    while !rest.is_empty() {
        let Some(start) = rest.find('<') else {
            text(rest)?;
            break;
        };
        text(&rest[..start])?;
        if open.is_empty() && !rest[..start].trim().is_empty() {
            return Err("text outside the root".to_owned());
        }
        let end = rest[start..].find('>').ok_or("a tag is not closed")? + start;
        let tag = &rest[start + 1..end];
        rest = &rest[end + 1..];
        if let Some(name) = tag.strip_prefix('/') {
            match open.pop() {
                Some(top) if top == name => {}
                top => return Err(format!("</{name}> closes {top:?}")),
            }
            continue;
        }
        let (body, empty) = match tag.strip_suffix('/') {
            Some(body) => (body, true),
            None => (tag, false),
        };
        let name = body.split_whitespace().next().ok_or("a tag has no name")?;
        attributes(&body[name.len()..])?;
        if open.is_empty() {
            roots += 1;
        }
        if !empty {
            open.push(name);
        }
    }
    match (open.as_slice(), roots) {
        ([], 1) => Ok(()),
        ([], n) => Err(format!("{n} root elements")),
        (open, _) => Err(format!("{open:?} not closed")),
    }
}

fn attributes(mut rest: &str) -> Result<(), String> {
    let mut names = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return Ok(());
        }
        let (name, after) = rest
            .split_once("=\"")
            .ok_or(format!("bad attribute in {rest}"))?;
        if name.is_empty() || name.contains(char::is_whitespace) || names.contains(&name) {
            return Err(format!("bad attribute name {name:?}"));
        }
        names.push(name);
        let (value, after) = after.split_once('"').ok_or("an attribute is not closed")?;
        if value.contains('<') {
            return Err(format!("< in the value of {name}"));
        }
        text(value)?;
        rest = after;
    }
}

fn text(text: &str) -> Result<(), String> {
    let mut parts = text.split('&');
    parts.next();
    for part in parts {
        let known = ["amp;", "lt;", "gt;", "quot;", "apos;"];
        if !known.iter().any(|k| part.starts_with(k)) {
            return Err(format!("a bare & before {part:?}"));
        }
    }
    Ok(())
}

#[test]
fn the_picture_draws_its_own_paper_before_any_shape() {
    let svg = checkout();
    let page = svg
        .find("<rect class=\"page\" x=\"-1\" y=\"-1\"")
        .expect("a page rect");
    assert!(page < svg.find("class=\"thing").expect("a thing"));
    assert!(svg.contains(".horsegirl .page{fill:var(--hg-paper,"));
}

#[test]
fn only_a_thing_takes_a_palette_slot_for_its_owner() {
    let source = "Team { owner: Ghost }\nTeam { A, B }\nA -> B\nA { owner: X }\nB { owner: Y }\n";
    let svg = draw(&scene(source), "owners");
    assert!(svg.contains("class=\"thing slot-1 owner-x\""), "{svg}");
    assert!(svg.contains("class=\"thing slot-2 owner-y\""), "{svg}");
}
