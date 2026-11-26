//! The overlap detector: what collides in a scene.
//!
//! A picture is wrong when one shape covers another. The detector reads a
//! scene, as a backend does, and reports each pair of shapes that collide.
//! It knows no layout, so it holds every lens and every solver to one rule.

use std::collections::BTreeMap;

use crate::scene::{Point, Rect, Role, Scene, Shape};

/// Two shapes collide when they share more than this many pixels. A path
/// ends on the border of a box, and a sum of sizes can lose its last bits, so
/// a border is not a collision.
const SLACK: f64 = 0.01;

/// How two shapes collide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Two boxes overlap, and neither is a group that holds the other.
    Boxes,
    /// A link crosses the box of a thing.
    LinkOnThing,
    /// A link crosses a group that holds neither of its ends.
    LinkThroughGroup,
    /// A link crosses the border of a group that holds one of its ends more
    /// than once, or leaves a group that holds both of its ends.
    LinkOnGroup,
    /// Two links with no end in common lie on each other, so that the picture
    /// shows a link that the file does not state.
    Links,
    /// A label overlaps the box of a thing that it does not name.
    LabelOnThing,
    /// A label lies across the border of a group. A label inside a group is
    /// no collision, and the name of a group is not held to its own border.
    LabelOnGroup,
    /// The centre of the label of a link is not on its link.
    LabelOffLink,
    /// Two labels overlap.
    Labels,
    /// A link crosses the label of another link. A part of the link that
    /// lies on the link of the label is no collision: two links from one
    /// thing share the line that leaves it.
    LinkOnLabel,
}

/// One collision in a scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overlap {
    pub kind: Kind,
    /// The index in [`Scene::shapes`] of each of the two shapes. For
    /// [`Kind::LabelOffLink`] they are the label and its link, and a label
    /// with no link is there twice.
    pub shapes: [usize; 2],
}

impl Overlap {
    /// The collision in words, with the names of both shapes.
    #[must_use]
    pub fn describe(&self, scene: &Scene) -> String {
        let name = |index: usize| {
            let shape = &scene.shapes[index];
            format!("{} `{}`", shape.role.as_str(), shape.of.join(" -> "))
        };
        let [first, second] = self.shapes.map(name);
        match self.kind {
            Kind::Boxes => format!("{first} overlaps {second}"),
            Kind::LinkOnThing | Kind::LinkOnLabel => format!("{first} crosses {second}"),
            Kind::LinkThroughGroup => format!("{first} passes through {second}"),
            Kind::LinkOnGroup => format!("{first} crosses the border of {second} too often"),
            Kind::Links => format!("{first} lies on {second}"),
            Kind::LabelOnThing | Kind::Labels => format!("{first} covers {second}"),
            Kind::LabelOnGroup => format!("{first} lies across the border of {second}"),
            Kind::LabelOffLink => format!("{first} is not on {second}"),
        }
    }
}

/// Every collision in `scene`, in the order of [`Kind`] and then of the
/// shapes.
#[must_use]
pub fn overlaps(scene: &Scene) -> Vec<Overlap> {
    let of_role = |role: Role| {
        let shapes = scene.shapes.iter().enumerate();
        shapes.filter(move |(_, shape)| shape.role == role)
    };
    let boxes: Vec<(usize, &Shape)> = of_role(Role::Group).chain(of_role(Role::Thing)).collect();
    let things: Vec<(usize, &Shape)> = of_role(Role::Thing).collect();
    let links: Vec<(usize, &Shape)> = of_role(Role::Link).collect();
    let labels: Vec<(usize, &Shape)> = of_role(Role::Label).collect();

    let mut found = Vec::new();
    let mut report = |kind: Kind, first: usize, second: usize| {
        found.push(Overlap {
            kind,
            shapes: [first, second],
        });
    };
    for (index, &(i, a)) in boxes.iter().enumerate() {
        for &(j, b) in &boxes[index + 1..] {
            let nested = a.role == Role::Group && holds(&a.rect, &b.rect)
                || b.role == Role::Group && holds(&b.rect, &a.rect);
            if share(&a.rect, &b.rect) && !nested {
                report(Kind::Boxes, i, j);
            }
        }
    }
    for &(i, link) in &links {
        for &(j, thing) in &things {
            if parts(&link.points).any(|(a, b)| inside(a, b, &thing.rect).is_some()) {
                report(Kind::LinkOnThing, i, j);
            }
        }
    }
    for kind in [Kind::LinkThroughGroup, Kind::LinkOnGroup] {
        for &(i, link) in &links {
            for (j, group) in of_role(Role::Group) {
                let (ends, crossings) = border(&link.points, &group.rect);
                // A link with one end inside crosses the border once.
                let through = ends == 0 && crossings > 0;
                let often = ends > 0 && crossings > ends % 2;
                if (kind == Kind::LinkThroughGroup && through)
                    || (kind == Kind::LinkOnGroup && often)
                {
                    report(kind, i, j);
                }
            }
        }
    }
    for (index, &(i, a)) in links.iter().enumerate() {
        for &(j, b) in &links[index + 1..] {
            let apart = a.of.iter().all(|name| !b.of.contains(name));
            let on = |(p, q): (Point, Point)| parts(&b.points).any(|part| lie_on((p, q), part));
            if apart && parts(&a.points).any(on) {
                report(Kind::Links, i, j);
            }
        }
    }
    for &(i, label) in &labels {
        for &(j, thing) in &things {
            if label.of != thing.of && share(&label.rect, &thing.rect) {
                report(Kind::LabelOnThing, i, j);
            }
        }
    }
    for &(i, label) in &labels {
        for (j, group) in of_role(Role::Group) {
            let across = share(&label.rect, &group.rect) && !holds(&group.rect, &label.rect);
            if label.of != group.of && across {
                report(Kind::LabelOnGroup, i, j);
            }
        }
    }
    // A label with two names is the label of the link between them.
    for &(i, label) in labels.iter().filter(|(_, label)| label.of.len() == 2) {
        let own: Vec<(usize, &Shape)> = links
            .iter()
            .copied()
            .filter(|(_, link)| link.of == label.of)
            .collect();
        let centre = Point {
            x: label.rect.x + label.rect.width / 2.0,
            y: label.rect.y + label.rect.height / 2.0,
        };
        let on = |link: &Shape| parts(&link.points).any(|(a, b)| distance(centre, a, b) <= SLACK);
        if !own.iter().any(|(_, link)| on(link)) {
            report(Kind::LabelOffLink, i, own.first().map_or(i, |&(j, _)| j));
        }
    }
    for (index, &(i, a)) in labels.iter().enumerate() {
        for &(j, b) in &labels[index + 1..] {
            if share(&a.rect, &b.rect) {
                report(Kind::Labels, i, j);
            }
        }
    }
    for (i, j) in links_on_labels(&links, &labels) {
        report(Kind::LinkOnLabel, i, j);
    }
    found
}

/// Each link and label of another link that the link crosses, as two
/// indices in [`Scene::shapes`].
fn links_on_labels(links: &[(usize, &Shape)], labels: &[(usize, &Shape)]) -> Vec<(usize, usize)> {
    let by_ends = parts_by_ends(links.iter().map(|&(_, link)| link));
    let mut found = Vec::new();
    for &(i, link) in links {
        for &(j, label) in labels.iter().filter(|(_, label)| label.of.len() == 2) {
            if label.of != link.of && meet(link, label, &by_ends) == Meet::Across {
                found.push((i, j));
            }
        }
    }
    found
}

/// How a link meets the label of another link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Meet {
    /// No part of the link is inside the box of the label.
    Apart,
    /// Each part of the link inside the box lies on the link that the label
    /// names. Two links from one thing share the line that leaves it, and a
    /// label on that line is on both links. That is no collision, and the
    /// SVG backend stops the line of both links at the label.
    Along,
    /// A part of the link inside the box does not lie on the link that the
    /// label names.
    Across,
}

/// How `link` meets the box of `label`. `by_ends` holds the parts of the
/// links of the scene, from [`parts_by_ends`]. The detector and the SVG
/// backend both decide with this one function, so a line that the backend
/// draws through the text of a label is always a collision.
pub(crate) fn meet(
    link: &Shape,
    label: &Shape,
    by_ends: &BTreeMap<&[String], Vec<(Point, Point)>>,
) -> Meet {
    let mut meet = Meet::Apart;
    let mut own: Option<&[(Point, Point)]> = None;
    for (a, b) in parts(&link.points) {
        let Some((first, last)) = inside(a, b, &label.rect) else {
            continue;
        };
        // Most links miss most labels, so the link of the label is looked up
        // only for a link that enters the box.
        let own = *own.get_or_insert_with(|| {
            by_ends
                .get(label.of.as_slice())
                .map_or(&[][..], Vec::as_slice)
        });
        let at = |t: f64| Point {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        };
        let (p, q) = (at(first), at(last));
        let on =
            |&(c, d): &(Point, Point)| distance(p, c, d) <= SLACK && distance(q, c, d) <= SLACK;
        if !own.iter().any(on) {
            return Meet::Across;
        }
        meet = Meet::Along;
    }
    meet
}

/// The parts of the links between each two ends, by the names of the ends.
pub(crate) fn parts_by_ends<'a>(
    links: impl Iterator<Item = &'a Shape>,
) -> BTreeMap<&'a [String], Vec<(Point, Point)>> {
    let mut by_ends: BTreeMap<&[String], Vec<(Point, Point)>> = BTreeMap::new();
    for link in links {
        by_ends
            .entry(link.of.as_slice())
            .or_default()
            .extend(parts(&link.points));
    }
    by_ends
}

/// Two boxes share room on both axes.
fn share(a: &Rect, b: &Rect) -> bool {
    let wide = f64::min(a.x + a.width, b.x + b.width) - f64::max(a.x, b.x);
    let high = f64::min(a.y + a.height, b.y + b.height) - f64::max(a.y, b.y);
    wide > SLACK && high > SLACK
}

fn holds(outer: &Rect, inner: &Rect) -> bool {
    outer.x - SLACK <= inner.x
        && outer.y - SLACK <= inner.y
        && inner.x + inner.width <= outer.x + outer.width + SLACK
        && inner.y + inner.height <= outer.y + outer.height + SLACK
}

/// Each straight part of a path.
fn parts(points: &[Point]) -> impl Iterator<Item = (Point, Point)> {
    points.windows(2).map(|pair| (pair[0], pair[1]))
}

/// The part of the line from `a` to `b` that is inside `r`, as two fractions
/// of the line. A line along the border is not inside.
fn inside(a: Point, b: Point, r: &Rect) -> Option<(f64, f64)> {
    let (mut first, mut last) = (0.0_f64, 1.0_f64);
    let axes = [
        (a.x, b.x - a.x, r.x + SLACK, r.x + r.width - SLACK),
        (a.y, b.y - a.y, r.y + SLACK, r.y + r.height - SLACK),
    ];
    for (start, length, low, high) in axes {
        if length == 0.0 {
            if start <= low || start >= high {
                return None;
            }
        } else {
            let (one, other) = ((low - start) / length, (high - start) / length);
            first = first.max(one.min(other));
            last = last.min(one.max(other));
        }
    }
    (first < last).then_some((first, last))
}

/// How many of the two ends of a path are inside a box, and how often the
/// path crosses the border of the box.
fn border(points: &[Point], r: &Rect) -> (usize, usize) {
    let within = |p: &Point| usize::from(inside(*p, *p, r).is_some());
    let ends = points.first().map_or(0, within) + points.last().map_or(0, within);
    let crossings = parts(points)
        .filter_map(|(a, b)| inside(a, b, r))
        .map(|(first, last)| usize::from(first > 0.0) + usize::from(last < 1.0))
        .sum();
    (ends, crossings)
}

/// Two lines run along each other: they are level with each other and share
/// a part of their length.
fn lie_on(p: (Point, Point), q: (Point, Point)) -> bool {
    let (dx, dy) = (p.1.x - p.0.x, p.1.y - p.0.y);
    let length = (dx * dx + dy * dy).sqrt();
    if length == 0.0 {
        return false;
    }
    let along = |point: Point| ((point.x - p.0.x) * dx + (point.y - p.0.y) * dy) / length;
    let beside = |point: Point| ((point.x - p.0.x) * dy - (point.y - p.0.y) * dx).abs() / length;
    let (start, end) = (along(q.0).min(along(q.1)), along(q.0).max(along(q.1)));
    beside(q.0) <= SLACK && beside(q.1) <= SLACK && end.min(length) - start.max(0.0) > SLACK
}

/// The distance from `p` to the line from `a` to `b`.
fn distance(p: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length = dx * dx + dy * dy;
    let along = if length == 0.0 {
        0.0
    } else {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / length).clamp(0.0, 1.0)
    };
    let (off_x, off_y) = (p.x - (a.x + along * dx), p.y - (a.y + along * dy));
    (off_x * off_x + off_y * off_y).sqrt()
}
