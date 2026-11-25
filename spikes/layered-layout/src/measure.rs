//! Counts what is wrong with a picture. Every candidate is measured by this one code.

use crate::model::{Graph, Picture, Rect};
use crate::svg::bounds;

pub struct Score {
    pub width: f64,
    pub height: f64,
    /// Pairs of boxes that overlap.
    pub overlaps: usize,
    /// Things outside their group or inside a group that is not theirs, and pairs of groups
    /// that overlap without one being inside the other.
    pub group_faults: usize,
    /// Links whose path enters a box that is not one of their two ends.
    pub through_box: usize,
    /// Links without a path.
    pub unrouted: usize,
    pub crossings: usize,
    pub bends: usize,
    pub length: f64,
}

type Point = (f64, f64);

fn overlap(a: &Rect, b: &Rect) -> bool {
    let slack = 0.5;
    a.x + slack < b.x + b.w
        && b.x + slack < a.x + a.w
        && a.y + slack < b.y + b.h
        && b.y + slack < a.y + a.h
}

fn inside(inner: &Rect, outer: &Rect) -> bool {
    let slack = 0.5;
    inner.x + slack >= outer.x
        && inner.y + slack >= outer.y
        && inner.x + inner.w <= outer.x + outer.w + slack
        && inner.y + inner.h <= outer.y + outer.h + slack
}

fn side(a: Point, b: Point, c: Point) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// True when the two segments cross at one point inside both.
fn cross(p: (Point, Point), q: (Point, Point)) -> bool {
    let tiny = 1e-6;
    let (d1, d2) = (side(q.0, q.1, p.0), side(q.0, q.1, p.1));
    let (d3, d4) = (side(p.0, p.1, q.0), side(p.0, p.1, q.1));
    d1 * d2 < -tiny && d3 * d4 < -tiny
}

/// True when the segment passes through the inside of the box.
fn enters(segment: (Point, Point), r: &Rect) -> bool {
    let inner = Rect {
        x: r.x + 1.0,
        y: r.y + 1.0,
        w: r.w - 2.0,
        h: r.h - 2.0,
    };
    let within = |p: Point| {
        p.0 > inner.x && p.0 < inner.x + inner.w && p.1 > inner.y && p.1 < inner.y + inner.h
    };
    let corners = [
        (inner.x, inner.y),
        (inner.x + inner.w, inner.y),
        (inner.x + inner.w, inner.y + inner.h),
        (inner.x, inner.y + inner.h),
    ];
    within(segment.0)
        || within(segment.1)
        || (0..4).any(|i| cross(segment, (corners[i], corners[(i + 1) % 4])))
}

fn segments(path: &[Point]) -> impl Iterator<Item = (Point, Point)> {
    path.windows(2).map(|pair| (pair[0], pair[1]))
}

pub fn score(graph: &Graph, picture: &Picture) -> Score {
    let b = bounds(picture);
    let nodes = &picture.nodes;
    let mut overlaps = 0;
    for (i, a) in nodes.iter().enumerate() {
        overlaps += nodes[i + 1..].iter().filter(|b| overlap(a, b)).count();
    }

    let mut group_faults = 0;
    for (node, r) in graph.nodes.iter().zip(nodes) {
        let chain = graph.chain(node.group);
        for (g, group) in picture.groups.iter().enumerate() {
            let member = chain.contains(&g);
            if (member && !inside(r, group)) || (!member && overlap(r, group)) {
                group_faults += 1;
            }
        }
    }
    for (g, a) in picture.groups.iter().enumerate() {
        for (offset, b) in picture.groups[g + 1..].iter().enumerate() {
            let h = g + 1 + offset;
            let nested = graph.chain(Some(h)).contains(&g) || graph.chain(Some(g)).contains(&h);
            if !nested && overlap(a, b) {
                group_faults += 1;
            }
        }
    }

    let mut through_box = 0;
    let mut unrouted = 0;
    let mut bends = 0;
    let mut length = 0.0;
    for (path, &(from, to)) in picture.edges.iter().zip(&graph.edges) {
        if path.len() < 2 {
            unrouted += 1;
            continue;
        }
        let others = || {
            nodes
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != from && *i != to)
        };
        if others().any(|(_, r)| segments(path).any(|s| enters(s, r))) {
            through_box += 1;
        }
        for (p, q) in segments(path) {
            length += (q.0 - p.0).hypot(q.1 - p.1);
        }
        bends += path
            .windows(3)
            .filter(|w| side(w[0], w[1], w[2]).abs() > 1e-6)
            .count();
    }

    let mut crossings = 0;
    for (i, a) in picture.edges.iter().enumerate() {
        for b in &picture.edges[i + 1..] {
            crossings += segments(a)
                .flat_map(|p| segments(b).map(move |q| (p, q)))
                .filter(|&(p, q)| cross(p, q))
                .count();
        }
    }
    Score {
        width: b.w,
        height: b.h,
        overlaps,
        group_faults,
        through_box,
        unrouted,
        crossings,
        bends,
        length,
    }
}
