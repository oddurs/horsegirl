//! The layered layout: rank, order and place.
//!
//! A flow reads from left to right. The rank phase gives each node a column,
//! so that a link goes from a lower rank to a higher one. The order phase
//! sorts each rank to cross few links. The place phase sets the position of
//! each node along its rank. A group stays in one piece: its box holds its
//! nodes and no other node.
//!
//! The layout routes no link. It gives the point where a link passes each
//! rank between its two ends, and a router starts from those points.
//!
//! The phases use sums, differences, halves and means only, and they iterate
//! no hash map, so the same graph gives the same bits on every machine.

mod order;
mod place;
mod rank;

/// The least distance between two boxes of one rank.
pub const NODE_GAP: f64 = 24.0;
/// The least distance between a link that passes a rank and what is beside it.
pub const BEND_GAP: f64 = 12.0;
/// The least distance between the boxes of two neighbouring ranks.
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A box by its top left corner. `y` grows down the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    /// none, and a link from a node to itself has none.
    pub bends: Vec<Vec<Point>>,
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
    let mut net = build(graph, &chains, &forward, &ranks);
    order::order(&mut net, &graph.groups);
    let (centres, edges) = place::place(&net, &graph.groups, &chains);
    let columns = columns(&net, &graph.groups, &chains);
    assemble(&net, ranks, &centres, &edges, &columns, &reversed)
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

fn build(graph: &Graph, chains: &[Vec<usize>], links: &[(usize, usize)], ranks: &[usize]) -> Net {
    let mut items: Vec<Item> = graph
        .nodes
        .iter()
        .zip(ranks)
        .map(|(node, &rank)| Item {
            kind: Kind::Node,
            width: node.width,
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
    for &(a, b) in links {
        if a == b {
            paths.push(Vec::new());
            continue;
        }
        // A bend lies in the innermost group that holds both ends of its link.
        let shared = shared(&items[a].chain, &items[b].chain);
        let chain = items[a].chain[..shared].to_vec();
        let mut path = vec![a];
        for rank in ranks[a] + 1..ranks[b] {
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
    /// How far the border of each group is from its first and last rank: one
    /// pad for the group and one for each level of groups inside it.
    border: Vec<f64>,
}

fn columns(net: &Net, groups: &[Group], chains: &[Vec<usize>]) -> Columns {
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
    for rank in 1..rank_count {
        left[rank] = left[rank - 1] + width[rank - 1] + after[rank - 1] + RANK_GAP + before[rank];
    }
    Columns {
        left,
        width,
        border,
    }
}

/// Joins the two axes into boxes and moves the picture to `0, 0`.
fn assemble(
    net: &Net,
    ranks: Vec<usize>,
    centres: &[f64],
    edges: &[(f64, f64)],
    columns: &Columns,
    reversed: &[bool],
) -> Layout {
    let x_of = |i: usize| {
        let item = &net.items[i];
        columns.left[item.rank] + (columns.width[item.rank] - item.width) / 2.0
    };
    let mut nodes: Vec<Rect> = net
        .items
        .iter()
        .enumerate()
        .take_while(|(_, item)| item.kind == Kind::Node)
        .map(|(i, item)| Rect {
            x: x_of(i),
            y: centres[i] - item.height / 2.0,
            width: item.width,
            height: item.height,
        })
        .collect();
    let mut groups: Vec<Option<Rect>> = net
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
    let mut bends: Vec<Vec<Point>> = net
        .paths
        .iter()
        .zip(reversed)
        .map(|(path, &back)| {
            let inner = path.get(1..path.len().saturating_sub(1)).unwrap_or(&[]);
            let mut points: Vec<Point> = inner
                .iter()
                .map(|&i| Point {
                    x: x_of(i),
                    y: centres[i],
                })
                .collect();
            if back {
                points.reverse();
            }
            points
        })
        .collect();

    // A bend is between the ranks of two nodes, so the boxes alone give the
    // left edge of the picture. A bend can be the first item of its rank, so
    // the top edge needs the bends too.
    let boxes = || nodes.iter().chain(groups.iter().flatten());
    let x = boxes().map(|r| r.x).fold(f64::INFINITY, f64::min);
    let tops = boxes().map(|r| r.y);
    let y = tops
        .chain(bends.iter().flatten().map(|point| point.y))
        .fold(f64::INFINITY, f64::min);
    if x.is_finite() && y.is_finite() {
        for r in nodes.iter_mut().chain(groups.iter_mut().flatten()) {
            r.x -= x;
            r.y -= y;
        }
        for point in bends.iter_mut().flatten() {
            point.x -= x;
            point.y -= y;
        }
    }
    Layout {
        nodes,
        groups,
        ranks,
        bends,
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
