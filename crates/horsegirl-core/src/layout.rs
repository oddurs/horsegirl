//! The layered layout: rank, order, place and route.
//!
//! A flow reads from left to right. The rank phase gives each node a column,
//! so that a link goes from a lower rank to a higher one. The order phase
//! sorts each rank to cross few links, and keeps each link on one side of a
//! group that holds neither of its ends. The place phase sets the position
//! of each node along its rank. A group stays in one piece: its box holds
//! its nodes and no other node. The route phase draws each link as a path at
//! right angles that crosses no box of a node. A path can still pass through
//! a group that holds neither end of its link, where the order phase finds
//! no order that keeps it out.
//!
//! A link can start or end at a group. The rank phase reads it as a link to
//! each node that the group holds, so the group ranks as one unit after a
//! node that links to it. The link then runs to a port: a point of the group
//! in its rank that is nearest to the other end, outside every group inside
//! it. The path ends where it meets the border of the group, so it enters no
//! box inside the group.
//!
//! The phases use sums, differences, halves and means only, and they iterate
//! no hash map, so the same graph gives the same bits on every machine.

mod order;
mod place;
mod rank;
mod route;
mod through;

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
/// The room between a border of a group and a label that the layout moved
/// off that border.
pub const LABEL_CLEAR: f64 = 3.0;
/// The room at the top of a group for its name.
pub const GROUP_TITLE: f64 = 30.0;

/// What the layout places: boxes, the groups that hold them, and links.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub groups: Vec<Group>,
    /// Each link, as the end it starts at and the end it ends at.
    pub links: Vec<(End, End)>,
    /// The size of the label of each link, in the order of `links`. A link
    /// with `None` or with no entry has no label.
    pub labels: Vec<Option<Size>>,
}

/// One end of a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// The index of a node.
    Node(usize),
    /// The index of a group. The link meets the border of the group box. A
    /// link cannot join a group to itself or to a node or a group inside it,
    /// and the group holds at least one node.
    Group(usize),
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Group {
    /// The index of the group that holds this group. No group holds itself,
    /// directly or through other groups.
    pub parent: Option<usize>,
    /// The width of the name of the group. The box is at least this wide
    /// plus [`GROUP_PAD`] on each side, so the name stays inside it.
    pub width: f64,
}

/// Where everything is. Each list has the order of its list in the [`Graph`].
///
/// The top left corner of the picture is at `0, 0`: no box and no bend has a
/// negative position. The box of a label can pass that corner, when the
/// label is on the topmost line of the picture.
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
    /// the box of its first end, ends on the border of the box of its
    /// second end, and crosses no box of a node. It can pass through the box
    /// of a group that holds neither of its two nodes, where the order phase
    /// finds no order of the ranks that keeps it out. Some graphs have no
    /// such order: two groups under one parent can each have a link to a
    /// third group that only goes through the other.
    pub paths: Vec<Vec<Point>>,
    /// The centre of the label of each link that has one. It is on the path
    /// of the link, and a label of the size in the [`Graph`] overlaps no box
    /// of a node there. It lies across no border of a group when its upright
    /// line has a height that is clear of every border. It covers no other
    /// label between the same two ranks, unless it moved off a border and
    /// every height off the borders covers one.
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
/// When a link names a node or a group that is not in the graph, when a node
/// or a group names a group that is not in the graph, or when a link joins a
/// group to itself or to a node or a group inside it, or ends at a group that
/// holds no node.
#[must_use]
pub fn layout(graph: &Graph, _seed: Option<&Layout>) -> Layout {
    lay_out(graph).0
}

/// The layout, and the work of the pass search.
fn lay_out(graph: &Graph) -> (Layout, usize) {
    let chains = chains(&graph.groups);
    let held = held(graph, &chains);
    let sets: Vec<(Vec<usize>, Vec<usize>)> = graph
        .links
        .iter()
        .map(|&(from, to)| {
            let (from, to) = (nodes_of(&held, from), nodes_of(&held, to));
            assert!(
                !from.is_empty() && !to.is_empty(),
                "a link ends at a group that holds no node"
            );
            let alone = from.len() == 1 && from == to;
            assert!(
                alone || !from.iter().any(|node| to.contains(node)),
                "a link joins a group to itself or to what it holds"
            );
            (from, to)
        })
        .collect();
    let (reversed, apart) = orient(graph.nodes.len(), &sets);
    let ranks = rank::rank(graph.nodes.len(), &apart);
    let forward_ends: Vec<(End, End)> = graph
        .links
        .iter()
        .zip(&reversed)
        .map(|(&(from, to), &back)| if back { (to, from) } else { (from, to) })
        .collect();
    let forward: Vec<(usize, usize)> = sets
        .iter()
        .zip(&reversed)
        .map(|((from, to), &back)| {
            let set = if back {
                (&to[..], &from[..])
            } else {
                (&from[..], &to[..])
            };
            stand_ins(set, &ranks)
        })
        .collect();
    let loops = loops(graph);
    let mut net = build(graph, &chains, &forward, &forward_ends, &ranks, &loops);
    let work = order::order(&mut net, &graph.groups);
    let (centres, edges) = place::place(&net, &graph.groups, &chains);
    let routes = route::route(&net, &centres, &label_sizes(graph, &net));
    let columns = columns(&net, &graph.groups, &chains, &routes.gaps);
    let (nodes, groups) = boxes(&net, &centres, &edges, &columns, &loops);
    let (paths, mut labels) = paths(
        graph,
        &net,
        (&nodes, &groups),
        &columns,
        &routes,
        (&loops, &forward),
    );
    for link in 0..labels.len() {
        if let (Some(centre), Some(size)) = (labels[link], label(graph, link)) {
            let group_of = |end: End| match end {
                End::Node(node) => graph.nodes[node].group,
                End::Group(group) => Some(group),
            };
            let (from, to) = graph.links[link];
            let ends = [group_of(from), group_of(to)];
            let around: Vec<usize> = ends
                .iter()
                .flatten()
                .flat_map(|&g| &chains[g])
                .copied()
                .collect();
            let other: Vec<Rect> = groups
                .iter()
                .enumerate()
                .filter(|(g, _)| !around.contains(g))
                .filter_map(|(_, group)| *group)
                .collect();
            let others: Vec<Rect> = (0..labels.len())
                .filter(|&other| other != link)
                .filter_map(|other| Some(around_label(labels[other]?, label(graph, other)?)))
                .collect();
            let y = off_borders(centre, size, &paths[link], &groups, &other, &others);
            labels[link] = Some(Point { x: centre.x, y });
        }
    }
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
    (picture, work)
}

/// The nodes that each group holds, at any depth, in the order of the graph.
fn held(graph: &Graph, chains: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut held = vec![Vec::new(); graph.groups.len()];
    for (node, at) in graph.nodes.iter().enumerate() {
        for &g in at.group.map_or(&[][..], |g| &chains[g]) {
            held[g].push(node);
        }
    }
    held
}

/// The nodes that the rank phase reads for one end of a link.
fn nodes_of(held: &[Vec<usize>], end: End) -> Vec<usize> {
    match end {
        End::Node(node) => vec![node],
        End::Group(group) => held[group].clone(),
    }
}

/// The node at each end of a link that the order, place and route phases
/// read: the last node of the first end, and the first node of the second
/// end after it. When every node of the second end is after every node of
/// the first end, these are the nodes of the two ends that are nearest to
/// each other.
fn stand_ins((from, to): (&[usize], &[usize]), ranks: &[usize]) -> (usize, usize) {
    if let ([a], [b]) = (from, to)
        && a == b
    {
        return (*a, *b);
    }
    // `orient` gives each link one pair of nodes that it ranks in order, so
    // a node with a node of the second end after it exists. The first of the
    // highest rank is the stand-in, and so on, so the choice is the same on
    // every machine.
    let before = |a: usize| to.iter().any(|&b| ranks[a] < ranks[b]);
    let first = from.iter().copied().filter(|&a| before(a));
    let a = first.fold(None, |best: Option<usize>, a| match best {
        Some(b) if ranks[b] >= ranks[a] => Some(b),
        _ => Some(a),
    });
    let a = a.expect("a link has one pair of nodes in rank order");
    let last = to.iter().copied().filter(|&b| ranks[a] < ranks[b]);
    let b = last.fold(None, |best: Option<usize>, b| match best {
        Some(c) if ranks[c] <= ranks[b] => Some(c),
        _ => Some(b),
    });
    (
        a,
        b.expect("the stand-in of the first end has a node after it"),
    )
}

fn label(graph: &Graph, link: usize) -> Option<Size> {
    graph.labels.get(link).copied().flatten()
}

/// The box of a label of `size` centred at `centre`.
fn around_label(centre: Point, size: Size) -> Rect {
    Rect {
        x: centre.x - size.width / 2.0,
        y: centre.y - size.height / 2.0,
        width: size.width,
        height: size.height,
    }
}

/// The room that each node keeps free at each side for its links to itself.
///
/// The link is a loop at the right side of the box, and its label is centred
/// on the upright line of the loop. The room is at both sides, so that the
/// box stays in the middle of its rank.
fn loops(graph: &Graph) -> Vec<f64> {
    let mut rooms: Vec<f64> = vec![0.0; graph.nodes.len()];
    for (link, &ends) in graph.links.iter().enumerate() {
        if let (End::Node(from), End::Node(to)) = ends
            && from == to
        {
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

/// For each step, the size of the label that sits on it.
fn label_sizes(graph: &Graph, net: &Net) -> Vec<Option<Size>> {
    let mut sizes = vec![None; net.steps.len()];
    for link in 0..graph.links.len() {
        if let Some(step) = label_step(net, link) {
            sizes[step] = label(graph, link);
        }
    }
    sizes
}

/// A count as a number to average. A rank holds far fewer than 2^52 items,
/// so the number is exact.
#[expect(clippy::cast_precision_loss, reason = "a count of items is small")]
fn real(count: usize) -> f64 {
    count as f64
}

/// Marks the links to turn round so that no loop remains, and gives the
/// pairs of nodes that the rank phase puts in order.
///
/// Each end of a link is the nodes it stands for: one node, or every node
/// of a group. The links are read in the order of the file. A link that
/// closes a loop with the links before it is the one that turns, so the
/// first statements of a file decide which way the picture reads. A link
/// that turns can still close a loop through a group, when a node of the
/// group is before the other end and a node is after it. Each pair of nodes
/// that would close a loop is left out, so the group then ranks on both
/// sides of the other end.
fn orient(count: usize, links: &[(Vec<usize>, Vec<usize>)]) -> (Vec<bool>, Vec<(usize, usize)>) {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut pairs = Vec::new();
    let reversed = links
        .iter()
        .map(|(from, to)| {
            if from == to {
                return false;
            }
            let back = to
                .iter()
                .any(|&b| from.iter().any(|&a| reaches(&out, b, a)));
            let (from, to) = if back { (to, from) } else { (from, to) };
            for &a in from {
                for &b in to {
                    if !reaches(&out, b, a) {
                        out[a].push(b);
                        pairs.push((a, b));
                    }
                }
            }
            back
        })
        .collect();
    (reversed, pairs)
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
    ends: &[(End, End)],
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
    for (&(a, b), &(from, to)) in links.iter().zip(ends) {
        first_steps.push(steps.len());
        if a == b {
            paths.push(Vec::new());
            continue;
        }
        // A link that starts or ends at a group has a port there: a point of
        // the group in the rank of the node that stands in for the group,
        // outside every group inside it. The port of a link that starts at a
        // group is in the last rank of the group, so the link leaves through
        // the border and crosses no box inside the group. A link that ends at
        // a group arrives at a port in its first rank. A group with nodes on
        // both sides of the other end has a port in a middle rank, and the
        // link starts or ends at the port, inside the group box.
        let mut port = |end: End, node: usize| match end {
            End::Group(g) => {
                items.push(Item {
                    kind: Kind::Bend,
                    width: 0.0,
                    height: 0.0,
                    rank: ranks[node],
                    chain: chains[g].clone(),
                });
                items.len() - 1
            }
            End::Node(_) => node,
        };
        let (a, b) = (port(from, a), port(to, b));
        // A bend lies in every group that holds both ends of its link. It
        // also lies in a group of one end while that group has the rank of
        // the bend: the link enters or leaves the group at its side, and
        // crosses no group inside it on the way to its end.
        let shared = shared(&items[a].chain, &items[b].chain);
        let mut path = vec![a];
        for rank in items[a].rank + 1..items[b].rank {
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
    for &g in &inner_first {
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
    let mut width: Vec<f64> = net
        .layers
        .iter()
        .map(|layer| {
            layer
                .iter()
                .map(|&i| net.items[i].width)
                .fold(0.0, f64::max)
        })
        .collect();
    // A group with a name wider than its box gets the missing width in its
    // first rank. Only the first rank grows, so a group whose name fits
    // keeps its place. An inner group goes first: its growth can make the
    // box of the group around it wide enough.
    for &g in &inner_first {
        let Some((low, high)) = net.spans[g] else {
            continue;
        };
        let inside: f64 = (low..=high).map(|rank| width[rank]).sum();
        let between: f64 = (low..high)
            .map(|rank| gaps[rank] + before[rank + 1] + after[rank])
            .sum();
        let missing = groups[g].width + 2.0 * GROUP_PAD - (inside + between + 2.0 * border[g]);
        if missing > 0.0 {
            width[low] += missing;
        }
    }
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
    (nodes, groups): (&[Rect], &[Option<Rect>]),
    columns: &Columns,
    routes: &Routes,
    (loops, stand_ins): (&[f64], &[(usize, usize)]),
) -> (Vec<Vec<Point>>, Vec<Option<Point>>) {
    // The left and the right side of the box of a node. A port has the
    // border of its group on the side where the group ends in its rank, and
    // its own place on the other side.
    let sides = |i: usize| {
        if i < nodes.len() {
            return (nodes[i].x, nodes[i].x + nodes[i].width);
        }
        let item = &net.items[i];
        let at = columns.x(item);
        let g = *item.chain.last().expect("a port is in its group");
        match (net.spans[g], groups[g]) {
            (Some((low, high)), Some(r)) => (
                if item.rank == low { r.x } else { at },
                if item.rank == high { r.x + r.width } else { at },
            ),
            _ => (at, at),
        }
    };
    let middle = |rank: usize| columns.corridor[rank] + routes.gaps[rank] / 2.0;
    let mut paths = Vec::new();
    let mut labels = Vec::new();
    for (link, path) in net.paths.iter().enumerate() {
        let (Some(&first), Some(&last)) = (path.first(), path.last()) else {
            let node = stand_ins[link].0;
            let (points, centre) = turn_back(&nodes[node], loops[node]);
            paths.push(points);
            labels.push(label(graph, link).map(|_| centre));
            continue;
        };
        let start = net.first_steps[link];
        let mut points = vec![Point {
            x: sides(first).1,
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
            x: sides(last).0,
            y: routes.into[start + path.len() - 2],
        });
        paths.push(points);
        let centre = label_step(net, link).and_then(|step| {
            let rank = net.items[net.steps[step].0].rank;
            routes.labels[step].map(|at| Point {
                x: middle(rank) + at.x,
                y: at.y,
            })
        });
        labels.push(centre);
    }
    (paths, labels)
}

/// The height of a label that lies across no border of a group.
///
/// A label is at the middle of an upright line, and a line that leaves a
/// group through its top or its bottom has that border somewhere on it. The
/// name of a group is drawn in the [`GROUP_TITLE`] band under its top
/// border, so the top border is that whole band here. A label that lies
/// across a border moves along its upright line to the nearest height where
/// it is [`LABEL_CLEAR`] from every border that it would lie across. It
/// stays where it is when the line has no such height.
///
/// `other` has the groups that hold neither end of the link. A height
/// outside all of them comes before a nearer height inside one: a label
/// inside such a group reads as a label of that group, and the overlap
/// detector does not see it there. `labels` has the boxes of the other
/// labels. The route phase put the label where it covers none of them. A
/// height that covers none of them comes before every other height, but a
/// label across a border is worse than a label on a label, so the label
/// still moves when each height off the borders covers one.
fn off_borders(
    centre: Point,
    size: Size,
    path: &[Point],
    groups: &[Option<Rect>],
    other: &[Rect],
    labels: &[Rect],
) -> f64 {
    let half = size.width / 2.0;
    let beside =
        |group: &&Rect| group.x < centre.x + half && centre.x - half < group.x + group.width;
    let other: Vec<(f64, f64)> = other
        .iter()
        .filter(beside)
        .map(|group| (group.y, group.y + group.height))
        .collect();
    let borders: Vec<(f64, f64)> = groups
        .iter()
        .flatten()
        .filter(beside)
        .flat_map(|group| {
            let bottom = group.y + group.height;
            [(group.y, group.y + GROUP_TITLE), (bottom, bottom)]
        })
        .collect();
    let labels: Vec<(f64, f64)> = labels
        .iter()
        .filter(beside)
        .map(|label| (label.y, label.y + label.height))
        .collect();
    let half = size.height / 2.0;
    let across = |y: f64| {
        let mut borders = borders.iter();
        borders.any(|&(top, bottom)| top - half < y && y < bottom + half)
    };
    if !across(centre.y) {
        return centre.y;
    }
    let covers = |y: f64| {
        let mut labels = labels.iter();
        labels.any(|&(top, bottom)| top - half < y && y < bottom + half)
    };
    // The upright line that holds the label.
    let line = path.windows(2).find_map(|pair| {
        let (top, bottom) = (pair[0].y.min(pair[1].y), pair[0].y.max(pair[1].y));
        let holds = pair[0].x - centre.x == 0.0 && pair[1].x - centre.x == 0.0;
        (holds && top <= centre.y && centre.y <= bottom).then_some((top, bottom))
    });
    let Some((top, bottom)) = line else {
        return centre.y;
    };
    let reach = half + LABEL_CLEAR;
    let inside = |y: f64| {
        let mut other = other.iter();
        other.any(|&(top, bottom)| top < y + half && y - half < bottom)
    };
    borders
        .iter()
        .chain(&labels)
        .flat_map(|&(top, bottom)| [top - reach, bottom + reach])
        .filter(|&y| top <= y && y <= bottom && !across(y))
        .min_by(|a, b| {
            let far = (a - centre.y).abs().total_cmp(&(b - centre.y).abs());
            let worse = |y: f64| (covers(y), inside(y));
            worse(*a).cmp(&worse(*b)).then(far)
        })
        .unwrap_or(centre.y)
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
    use super::{End, Graph, Group, Node, lay_out, orient};

    // A net of 100 nodes in 10 groups with 390 links has hundreds of passes
    // that no order removes, so the pass search must stop at its limit. The
    // test reads the search, not a clock, so a slow runner cannot fail it.
    #[test]
    fn a_dense_graph_stops_the_pass_search_at_its_limit() {
        let mut state: u32 = 9;
        let mut next = |below: usize| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 16) as usize % below
        };
        let graph = Graph {
            nodes: (0..100)
                .map(|i| Node {
                    width: 50.0,
                    height: 36.0,
                    group: Some(i % 10),
                })
                .collect(),
            groups: vec![
                Group {
                    parent: None,
                    width: 0.0
                };
                10
            ],
            labels: Vec::new(),
            links: (0..390)
                .map(|_| (End::Node(next(100)), End::Node(next(100))))
                .collect(),
        };
        let work = lay_out(&graph).1;
        // The numbers are literals, so a change to `LIMIT` fails this test
        // and the person who makes it must read it. The search checks the
        // limit before each move, so it can end a little past 1 000 000.
        assert!(work > 1_000_000, "the search ended early, after {work}");
        assert!(work < 2_000_000, "the search passed its limit: {work}");
    }

    fn turned(count: usize, links: &[(usize, usize)]) -> Vec<bool> {
        let sets: Vec<(Vec<usize>, Vec<usize>)> =
            links.iter().map(|&(a, b)| (vec![a], vec![b])).collect();
        orient(count, &sets).0
    }

    #[test]
    fn the_link_that_closes_a_loop_is_the_one_that_turns() {
        assert_eq!(turned(3, &[(0, 1), (1, 2), (2, 0)]), [false, false, true]);
        assert_eq!(turned(3, &[(2, 0), (0, 1), (1, 2)]), [false, false, true]);
    }

    #[test]
    fn a_link_from_a_node_to_itself_does_not_turn() {
        assert_eq!(turned(2, &[(0, 0), (0, 1), (1, 0)]), [false, false, true]);
    }

    // Node 1 is before node 0 and node 2 after it, so the link from node 0
    // to the group of nodes 1 and 2 turns, and only the pair that closes no
    // loop orders the ranks.
    #[test]
    fn a_link_to_a_group_on_both_sides_of_its_start_orders_one_pair() {
        let sets = vec![
            (vec![1], vec![0]),
            (vec![0], vec![2]),
            (vec![0], vec![1, 2]),
        ];
        let (reversed, pairs) = orient(3, &sets);
        assert_eq!(reversed, [false, false, true]);
        assert_eq!(pairs, [(1, 0), (0, 2), (1, 0)]);
    }
}
