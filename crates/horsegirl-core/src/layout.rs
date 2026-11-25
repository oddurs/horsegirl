//! The layered layout: rank, order, place and route.
//!
//! A flow reads from left to right. The rank phase gives each node a column,
//! so that a link goes from a lower rank to a higher one. The order phase
//! sorts each rank to cross few links. The place phase sets the position of
//! each node along its rank. A group stays in one piece: its box holds its
//! nodes and no other node. The route phase draws each link as a path at
//! right angles that crosses no box.
//!
//! The phases use sums, differences, halves and means only, and they iterate
//! no hash map, so the same graph gives the same bits on every machine.

mod order;
mod place;
mod rank;
mod route;

use crate::scene::{Point, Rect};
use crate::text::Size;
use route::{LABEL_PAD, Routes};

/// The least distance between two boxes of one rank.
pub const NODE_GAP: f64 = 24.0;
/// The least distance between a link that passes a rank and what is beside
/// it, and between two upright lines of links that are beside each other.
pub const BEND_GAP: f64 = 12.0;
/// The least distance between the boxes of two neighbouring ranks. Many
/// upright lines or a wide label between two ranks make the distance larger.
pub const RANK_GAP: f64 = 64.0;
/// The distance from the border of a group to what it holds.
pub const GROUP_PAD: f64 = 14.0;
/// The room at the top of a group for its name.
pub const GROUP_TITLE: f64 = 30.0;

/// What the layout places: boxes, the groups that hold them, and links.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub groups: Vec<Group>,
    /// Each link, as the index of the node it starts at and of the node it
    /// ends at.
    pub links: Vec<(usize, usize)>,
    /// The size of the label of each link, in the order of `links`. A link
    /// with `None` or with no entry has no label.
    pub labels: Vec<Option<Size>>,
}

/// A box of a known size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub width: f64,
    pub height: f64,
    /// The index of the group that holds the node.
    pub group: Option<usize>,
}

/// A box around nodes and around other groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    /// The index of the group that holds this group. No group holds itself,
    /// directly or through other groups.
    pub parent: Option<usize>,
}

/// Where everything is. Each list has the order of its list in the [`Graph`].
///
/// The top left corner of the picture is at `0, 0`: no box and no bend has a
/// negative position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub nodes: Vec<Rect>,
    /// `None` for a group that holds no node.
    pub groups: Vec<Option<Rect>>,
    /// The rank of each node. Rank 0 is the leftmost column.
    pub ranks: Vec<usize>,
    /// For each link, the point where it passes each rank between its two
    /// ends, from its start to its end. A link between neighbouring ranks has
    /// none, and a link from a node to itself has none. The path of the link
    /// passes the rank at this height, or up to half of [`BEND_GAP`] from it.
    pub bends: Vec<Vec<Point>>,
    /// The path of each link: its points from its start to its end. Each
    /// part of a path is level or upright. A path starts on the border of
    /// the box of its first node, ends on the border of the box of its
    /// second node, and crosses no box.
    pub paths: Vec<Vec<Point>>,
    /// The centre of the label of each link that has one. It is on the path
    /// of the link, and a label of the size in the [`Graph`] overlaps no box
    /// of a node there.
    pub labels: Vec<Option<Point>>,
}

/// One entry of a rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Node,
    /// The point where a link passes a rank between its two ends.
    Bend,
    /// Holds the place of a group in a rank where the group has nothing else,
    /// so that the order of the rank keeps other nodes out of the group box.
    Marker,
}

#[derive(Debug)]
struct Item {
    kind: Kind,
    width: f64,
    height: f64,
    rank: usize,
    /// The groups around the item, outermost first.
    chain: Vec<usize>,
}

/// The graph as the order phase and the place phase read it.
#[derive(Debug)]
struct Net {
    /// The nodes of the graph first, in their order, then bends and markers.
    items: Vec<Item>,
    /// The items of each rank, from the top of the page to the bottom.
    layers: Vec<Vec<usize>>,
    /// Each part of a link between two neighbouring ranks, lower rank first.
    steps: Vec<(usize, usize)>,
    /// The items of each link, lower rank first. Empty for a link from a node
    /// to itself.
    paths: Vec<Vec<usize>>,
    /// The index in `steps` of the first step of each link.
    first_steps: Vec<usize>,
    /// The first and the last rank of each group that holds a node.
    spans: Vec<Option<(usize, usize)>>,
}

/// Places every node and every group of `graph`.
///
/// The seed is an earlier picture of the graph. The layout does not read it
/// yet, so a layout that is seeded with its own picture moves nothing.
///
/// # Panics
///
/// When a link names a node that is not in the graph, or a node or a group
/// names a group that is not in the graph.
#[must_use]
pub fn layout(graph: &Graph, _seed: Option<&Layout>) -> Layout {
    let reversed = orient(graph.nodes.len(), &graph.links);
    let forward: Vec<(usize, usize)> = graph
        .links
        .iter()
        .zip(&reversed)
        .map(|(&(from, to), &back)| if back { (to, from) } else { (from, to) })
        .collect();
    let apart: Vec<(usize, usize)> = forward.iter().copied().filter(|(a, b)| a != b).collect();
    let ranks = rank::rank(graph.nodes.len(), &apart);
    let chains = chains(&graph.groups);
    let loops = loops(graph);
    let mut net = build(graph, &chains, &forward, &ranks, &loops);
    order::order(&mut net, &graph.groups);
    let (centres, edges) = place::place(&net, &graph.groups, &chains);
    let routes = route::route(&net, &centres, &label_widths(graph, &net));
    let columns = columns(&net, &graph.groups, &chains, &routes.gaps);
    let (nodes, groups) = boxes(&net, &centres, &edges, &columns, &loops);
    let (paths, labels) = paths(graph, &net, &nodes, &columns, &routes, &loops);
    let bends = bends(&net, &centres, &columns);
    let mut picture = Layout {
        nodes,
        groups,
        ranks,
        bends,
        paths,
        labels,
    };
    // A link that the layout turned round has its points in the order of the
    // link as the file states it.
    for (link, _) in reversed.iter().enumerate().filter(|(_, back)| **back) {
        picture.bends[link].reverse();
        picture.paths[link].reverse();
    }
    to_origin(&mut picture);
    picture
}

fn label(graph: &Graph, link: usize) -> Option<Size> {
    graph.labels.get(link).copied().flatten()
}

/// The room that each node keeps free at each side for its links to itself.
///
/// The link is a loop at the right side of the box, and its label is centred
/// on the upright line of the loop. The room is at both sides, so that the
/// box stays in the middle of its rank.
fn loops(graph: &Graph) -> Vec<f64> {
    let mut rooms: Vec<f64> = vec![0.0; graph.nodes.len()];
    for (link, &(from, to)) in graph.links.iter().enumerate() {
        if from == to {
            let text = label(graph, link).map_or(0.0, |size| size.width + 2.0 * LABEL_PAD);
            rooms[from] = rooms[from].max(2.0 * BEND_GAP).max(text);
        }
    }
    rooms
}

/// The step of a link that holds its label: the middle step.
fn label_step(net: &Net, link: usize) -> Option<usize> {
    let steps = net.paths[link].len().checked_sub(1)?;
    Some(net.first_steps[link] + (steps - 1) / 2)
}

/// For each step, the width of the label that sits on it, or 0.
fn label_widths(graph: &Graph, net: &Net) -> Vec<f64> {
    let mut widths = vec![0.0; net.steps.len()];
    for link in 0..graph.links.len() {
        if let (Some(size), Some(step)) = (label(graph, link), label_step(net, link)) {
            widths[step] = size.width;
        }
    }
    widths
}

/// A count as a number to average. A rank holds far fewer than 2^52 items,
/// so the number is exact.
#[expect(clippy::cast_precision_loss, reason = "a count of items is small")]
fn real(count: usize) -> f64 {
    count as f64
}

/// Marks the links to turn round so that no loop remains.
///
/// The links are read in the order of the file. A link that closes a loop
/// with the links before it is the one that turns, so the first statements
/// of a file decide which way the picture reads.
fn orient(count: usize, links: &[(usize, usize)]) -> Vec<bool> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); count];
    links
        .iter()
        .map(|&(from, to)| {
            if from == to {
                return false;
            }
            let back = reaches(&out, to, from);
            if back {
                out[to].push(from);
            } else {
                out[from].push(to);
            }
            back
        })
        .collect()
}

fn reaches(out: &[Vec<usize>], from: usize, to: usize) -> bool {
    let mut seen = vec![false; out.len()];
    let mut stack = vec![from];
    while let Some(at) = stack.pop() {
        if at == to {
            return true;
        }
        if !seen[at] {
            seen[at] = true;
            stack.extend(&out[at]);
        }
    }
    false
}

/// The groups around each group, outermost first, ending with the group.
fn chains(groups: &[Group]) -> Vec<Vec<usize>> {
    (0..groups.len())
        .map(|group| {
            let mut chain = vec![group];
            while let Some(parent) = groups[chain[chain.len() - 1]].parent {
                chain.push(parent);
            }
            chain.reverse();
            chain
        })
        .collect()
}

fn build(
    graph: &Graph,
    chains: &[Vec<usize>],
    links: &[(usize, usize)],
    ranks: &[usize],
    loops: &[f64],
) -> Net {
    let mut items: Vec<Item> = graph
        .nodes
        .iter()
        .zip(ranks.iter().zip(loops))
        .map(|(node, (&rank, room))| Item {
            kind: Kind::Node,
            width: node.width + 2.0 * room,
            height: node.height,
            rank,
            chain: node.group.map_or_else(Vec::new, |g| chains[g].clone()),
        })
        .collect();
    let mut spans: Vec<Option<(usize, usize)>> = vec![None; graph.groups.len()];
    for item in &items {
        for &g in &item.chain {
            let (low, high) = spans[g].unwrap_or((item.rank, item.rank));
            spans[g] = Some((low.min(item.rank), high.max(item.rank)));
        }
    }

    let mut steps = Vec::new();
    let mut paths = Vec::new();
    let mut first_steps = Vec::new();
    for &(a, b) in links {
        first_steps.push(steps.len());
        if a == b {
            paths.push(Vec::new());
            continue;
        }
        // A bend lies in every group that holds both ends of its link. It
        // also lies in a group of one end while that group has the rank of
        // the bend: the link enters or leaves the group at its side, and
        // crosses no group inside it on the way to its end.
        let shared = shared(&items[a].chain, &items[b].chain);
        let mut path = vec![a];
        for rank in ranks[a] + 1..ranks[b] {
            let here =
                |g: &&usize| spans[**g].is_some_and(|(low, high)| low <= rank && rank <= high);
            let depth =
                |end: usize| shared + items[end].chain[shared..].iter().take_while(here).count();
            let chain = match (depth(a), depth(b)) {
                (_, to) if to > shared => items[b].chain[..to].to_vec(),
                (from, _) => items[a].chain[..from].to_vec(),
            };
            path.push(items.len());
            items.push(Item {
                kind: Kind::Bend,
                width: 0.0,
                height: 0.0,
                rank,
                chain: chain.clone(),
            });
        }
        path.push(b);
        steps.extend(path.windows(2).map(|pair| (pair[0], pair[1])));
        paths.push(path);
    }

    let rank_count = ranks.iter().map(|rank| rank + 1).max().unwrap_or(0);
    add_markers(&mut items, chains, &spans, rank_count);
    let mut layers = vec![Vec::new(); rank_count];
    for (i, item) in items.iter().enumerate() {
        layers[item.rank].push(i);
    }
    Net {
        items,
        layers,
        steps,
        paths,
        first_steps,
        spans,
    }
}

/// How many groups two chains have in common from the outside in.
fn shared(a: &[usize], b: &[usize]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// Gives each group a marker in each rank of its span where it has no item.
///
/// A marker is the only item of its group in its rank. So no marker lies
/// between two items of one group, and the gap of a marker never replaces the
/// gap between two nodes.
fn add_markers(
    items: &mut Vec<Item>,
    chains: &[Vec<usize>],
    spans: &[Option<(usize, usize)>],
    rank_count: usize,
) {
    let mut present = vec![vec![false; rank_count]; chains.len()];
    for item in items.iter() {
        for &g in &item.chain {
            present[g][item.rank] = true;
        }
    }
    // Innermost first: the marker of a group is also an item of the groups
    // around it.
    let mut inner_first: Vec<usize> = (0..chains.len()).collect();
    inner_first.sort_by_key(|&g| std::cmp::Reverse(chains[g].len()));
    for g in inner_first {
        let Some((low, high)) = spans[g] else {
            continue;
        };
        for rank in low..=high {
            if !present[g][rank] {
                for &around in &chains[g] {
                    present[around][rank] = true;
                }
                items.push(Item {
                    kind: Kind::Marker,
                    width: 0.0,
                    height: 0.0,
                    rank,
                    chain: chains[g].clone(),
                });
            }
        }
    }
}

/// The left edge and the width of each rank.
struct Columns {
    left: Vec<f64>,
    width: Vec<f64>,
    /// The left edge of the corridor after each rank: the room between two
    /// ranks that holds no box and no border of a group that ends there.
    corridor: Vec<f64>,
    /// How far the border of each group is from its first and last rank: one
    /// pad for the group and one for each level of groups inside it.
    border: Vec<f64>,
}

fn columns(net: &Net, groups: &[Group], chains: &[Vec<usize>], gaps: &[f64]) -> Columns {
    let mut border = vec![GROUP_PAD; groups.len()];
    let mut inner_first: Vec<usize> = (0..groups.len()).collect();
    inner_first.sort_by_key(|&g| std::cmp::Reverse(chains[g].len()));
    for g in inner_first {
        if let Some(parent) = groups[g].parent {
            border[parent] = f64::max(border[parent], border[g] + GROUP_PAD);
        }
    }
    // The room that the group borders need before and after each rank.
    let rank_count = net.layers.len();
    let mut before = vec![0.0; rank_count];
    let mut after = vec![0.0; rank_count];
    for (g, span) in net.spans.iter().enumerate() {
        if let Some((low, high)) = *span {
            before[low] = f64::max(before[low], border[g]);
            after[high] = f64::max(after[high], border[g]);
        }
    }
    let width: Vec<f64> = net
        .layers
        .iter()
        .map(|layer| {
            layer
                .iter()
                .map(|&i| net.items[i].width)
                .fold(0.0, f64::max)
        })
        .collect();
    let mut left = vec![0.0; rank_count];
    let mut corridor = vec![0.0; rank_count];
    for rank in 0..rank_count {
        if rank > 0 {
            left[rank] = corridor[rank - 1] + gaps[rank - 1] + before[rank];
        }
        corridor[rank] = left[rank] + width[rank] + after[rank];
    }
    Columns {
        left,
        width,
        corridor,
        border,
    }
}

impl Columns {
    /// The left edge of an item: the item is in the middle of its rank.
    fn x(&self, item: &Item) -> f64 {
        self.left[item.rank] + (self.width[item.rank] - item.width) / 2.0
    }
}

/// Joins the two axes into the box of each node and of each group.
fn boxes(
    net: &Net,
    centres: &[f64],
    edges: &[(f64, f64)],
    columns: &Columns,
    loops: &[f64],
) -> (Vec<Rect>, Vec<Option<Rect>>) {
    let nodes = net
        .items
        .iter()
        .zip(centres)
        .zip(loops)
        .map(|((item, centre), room)| Rect {
            x: columns.x(item) + room,
            y: centre - item.height / 2.0,
            width: item.width - 2.0 * room,
            height: item.height,
        })
        .collect();
    let groups = net
        .spans
        .iter()
        .enumerate()
        .map(|(g, span)| {
            span.map(|(low, high)| {
                let x = columns.left[low] - columns.border[g];
                let right = columns.left[high] + columns.width[high] + columns.border[g];
                Rect {
                    x,
                    y: edges[g].0,
                    width: right - x,
                    height: edges[g].1 - edges[g].0,
                }
            })
        })
        .collect();
    (nodes, groups)
}

/// The point where each link passes each rank between its two ends, lower
/// rank first.
fn bends(net: &Net, centres: &[f64], columns: &Columns) -> Vec<Vec<Point>> {
    let point = |&i: &usize| Point {
        x: columns.x(&net.items[i]),
        y: centres[i],
    };
    net.paths
        .iter()
        .map(|path| {
            let inner = path.get(1..path.len().saturating_sub(1)).unwrap_or(&[]);
            inner.iter().map(point).collect()
        })
        .collect()
}

/// The path of each link, lower rank first, and the centre of each label.
fn paths(
    graph: &Graph,
    net: &Net,
    nodes: &[Rect],
    columns: &Columns,
    routes: &Routes,
    loops: &[f64],
) -> (Vec<Vec<Point>>, Vec<Option<Point>>) {
    let middle = |rank: usize| columns.corridor[rank] + routes.gaps[rank] / 2.0;
    let mut paths = Vec::new();
    let mut labels = Vec::new();
    for (link, path) in net.paths.iter().enumerate() {
        let (Some(&first), Some(&last)) = (path.first(), path.last()) else {
            let (points, centre) =
                turn_back(&nodes[graph.links[link].0], loops[graph.links[link].0]);
            paths.push(points);
            labels.push(label(graph, link).map(|_| centre));
            continue;
        };
        let start = net.first_steps[link];
        let mut points = vec![Point {
            x: nodes[first].x + nodes[first].width,
            y: routes.from[start],
        }];
        for (step, pair) in (start..).zip(path.windows(2)) {
            if let Some(track) = routes.track[step] {
                let x = middle(net.items[pair[0]].rank) + track;
                points.push(Point {
                    x,
                    y: routes.from[step],
                });
                points.push(Point {
                    x,
                    y: routes.into[step],
                });
            }
        }
        points.push(Point {
            x: nodes[last].x,
            y: routes.into[start + path.len() - 2],
        });
        paths.push(points);
        // A label sits on the upright line of its step, which only its link
        // and the links from the same item use. A step with no upright line
        // has its label in the middle of the corridor.
        let centre = label_step(net, link).map(|step| {
            let rank = net.items[net.steps[step].0].rank;
            Point {
                x: middle(rank) + routes.track[step].unwrap_or(0.0),
                y: f64::midpoint(routes.from[step], routes.into[step]),
            }
        });
        labels.push(centre.filter(|_| label(graph, link).is_some()));
    }
    (paths, labels)
}

/// A link from a node to itself: a loop at the right side of the box, above
/// the middle, where the links to other nodes leave. Returns its points and
/// the centre of its label.
fn turn_back(node: &Rect, room: f64) -> (Vec<Point>, Point) {
    let right = node.x + node.width;
    let out = right + room / 2.0;
    let middle = node.y + node.height / 2.0;
    let (top, bottom) = (middle - BEND_GAP, middle - BEND_GAP / 3.0);
    let points = [(right, top), (out, top), (out, bottom), (right, bottom)];
    let centre = Point {
        x: out,
        y: f64::midpoint(top, bottom),
    };
    (points.map(|(x, y)| Point { x, y }).to_vec(), centre)
}

/// Moves the picture to `0, 0`.
fn to_origin(picture: &mut Layout) {
    // A path is between the ranks of two nodes, so the boxes alone give the
    // left edge of the picture. A bend can be the first item of its rank, and
    // a path can pass it a few pixels higher, so the top edge needs both.
    let boxes = || picture.nodes.iter().chain(picture.groups.iter().flatten());
    let points = || picture.bends.iter().chain(&picture.paths).flatten();
    let x = boxes().map(|r| r.x).fold(f64::INFINITY, f64::min);
    let tops = boxes().map(|r| r.y);
    let y = tops
        .chain(points().map(|point| point.y))
        .fold(f64::INFINITY, f64::min);
    if !x.is_finite() || !y.is_finite() {
        return;
    }
    let boxes = picture.nodes.iter_mut();
    for r in boxes.chain(picture.groups.iter_mut().flatten()) {
        r.x -= x;
        r.y -= y;
    }
    let points = picture.bends.iter_mut().chain(&mut picture.paths).flatten();
    for point in points.chain(picture.labels.iter_mut().flatten()) {
        point.x -= x;
        point.y -= y;
    }
}

#[cfg(test)]
mod tests {
    use super::orient;

    #[test]
    fn the_link_that_closes_a_loop_is_the_one_that_turns() {
        assert_eq!(orient(3, &[(0, 1), (1, 2), (2, 0)]), [false, false, true]);
        assert_eq!(orient(3, &[(2, 0), (0, 1), (1, 2)]), [false, false, true]);
    }

    #[test]
    fn a_link_from_a_node_to_itself_does_not_turn() {
        assert_eq!(orient(2, &[(0, 0), (0, 1), (1, 0)]), [false, false, true]);
    }
}
