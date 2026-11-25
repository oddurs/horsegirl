//! Draws a picture as SVG. Every candidate uses this one renderer, so the pictures differ
//! only in where things are.

use std::fmt::Write;

use crate::model::{Graph, Picture, Rect};

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn bounds(picture: &Picture) -> Rect {
    let rects = picture.nodes.iter().chain(&picture.groups);
    let corners = rects.flat_map(|r| [(r.x, r.y), (r.x + r.w, r.y + r.h)]);
    let points: Vec<(f64, f64)> = corners
        .chain(picture.edges.iter().flatten().copied())
        .collect();
    let min = |f: fn(&(f64, f64)) -> f64| points.iter().map(f).fold(f64::INFINITY, f64::min);
    let max = |f: fn(&(f64, f64)) -> f64| points.iter().map(f).fold(f64::NEG_INFINITY, f64::max);
    let (x, y) = (min(|p| p.0), min(|p| p.1));
    Rect {
        x,
        y,
        w: max(|p| p.0) - x,
        h: max(|p| p.1) - y,
    }
}

pub fn draw(graph: &Graph, picture: &Picture) -> String {
    let margin = 16.0;
    let b = bounds(picture);
    let (w, h) = (b.w + 2.0 * margin, b.h + 2.0 * margin);
    let mut out = String::new();
    // Writing to a String cannot fail.
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.1} {:.1} {w:.1} {h:.1}\" \
         width=\"{w:.0}\" height=\"{h:.0}\" font-family=\"Helvetica, Arial, sans-serif\" \
         font-size=\"13\">",
        b.x - margin,
        b.y - margin,
    );
    let _ = writeln!(
        out,
        "<defs><marker id=\"tip\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" \
         markerHeight=\"7\" orient=\"auto\"><path d=\"M0 0L10 5L0 10z\" fill=\"#444\"/></marker>\
         </defs>\n<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"#fff\"/>",
        b.x - margin,
        b.y - margin,
    );
    for (group, r) in graph.groups.iter().zip(&picture.groups) {
        let _ = writeln!(
            out,
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"6\" \
             fill=\"#000\" fill-opacity=\"0.04\" stroke=\"#999\"/>\n\
             <text x=\"{:.1}\" y=\"{:.1}\" fill=\"#555\">{}</text>",
            r.x,
            r.y,
            r.w,
            r.h,
            r.x + 10.0,
            r.y + 19.0,
            escape(&group.name),
        );
    }
    for path in picture.edges.iter().filter(|path| path.len() > 1) {
        let points: Vec<String> = path.iter().map(|(x, y)| format!("{x:.1},{y:.1}")).collect();
        let _ = writeln!(
            out,
            "<polyline points=\"{}\" fill=\"none\" stroke=\"#444\" stroke-width=\"1.2\" \
             marker-end=\"url(#tip)\"/>",
            points.join(" "),
        );
    }
    for (node, r) in graph.nodes.iter().zip(&picture.nodes) {
        let (cx, cy) = r.centre();
        let _ = writeln!(
            out,
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"4\" \
             fill=\"#fff\" stroke=\"#222\"/>\n<text x=\"{cx:.1}\" y=\"{cy:.1}\" \
             text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"#111\">{}</text>",
            r.x,
            r.y,
            r.w,
            r.h,
            escape(&node.name),
        );
    }
    out.push_str("</svg>\n");
    out
}
