//! The stand-in for the layered layout that the scene and SVG tests share.
//!
//! The layered layout does not exist yet, so `place` stands in for it: it ranks
//! a file by its links and puts each rank in a column. The tests hold what the
//! scene and the backends do with a placement, not the placement.

use horsegirl_core::model::{Model, lower};
use horsegirl_core::scene::{GROUP_INSET, Placement, Point, Rect, Scene, build};
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;

pub const STYLE: Style = Style { size: 12.0 };

const BOX: (f64, f64) = (120.0, 36.0);
const GAP: (f64, f64) = (80.0, 24.0);

pub fn model(source: &str) -> Model {
    lower(&parse(source).tree)
}

/// The scene of `source`, placed by the stand-in.
/// Things take the colour of their owner, as the flow lens binds it.
pub fn scene(source: &str) -> Scene {
    let model = model(source);
    let mut scene = build(&model, &place(&model), STYLE);
    scene.colour = Some("owner".to_owned());
    scene
}

/// A stand-in for the layout: each thing in the column of its longest path
/// from a thing that nothing links to, each group around its parts, each link
/// a line with one bend, and each label at the middle of its line.
pub fn place(model: &Model) -> Placement {
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
