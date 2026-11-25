//! The place phase: the position of each item along its rank.
//!
//! The order of each rank and the gaps are difference constraints: one
//! position is at least a gap below another. The first solution packs
//! everything toward the top of the page. Each round then moves every item
//! toward the place it wants, as far as the constraints allow. A constraint
//! holds after every move, so two neighbours of one rank are never closer
//! than their gap.

use super::{BEND_GAP, GROUP_PAD, GROUP_TITLE, Group, Kind, NODE_GAP, Net, real, shared};

const ROUNDS: usize = 80;

/// `position[below] - position[above] >= gap`, stored at both ends.
struct Limits {
    above: Vec<Vec<(usize, f64)>>,
    below: Vec<Vec<(usize, f64)>>,
}

impl Limits {
    fn add(&mut self, above: usize, below: usize, gap: f64) {
        self.above[below].push((above, gap));
        self.below[above].push((below, gap));
    }

    /// The highest position on the page that `v` can have.
    fn top(&self, v: usize, at: &[f64]) -> Option<f64> {
        self.above[v]
            .iter()
            .map(|&(a, gap)| at[a] + gap)
            .reduce(f64::max)
    }

    /// The lowest position on the page that `v` can have.
    fn bottom(&self, v: usize, at: &[f64]) -> Option<f64> {
        self.below[v]
            .iter()
            .map(|&(b, gap)| at[b] - gap)
            .reduce(f64::min)
    }
}

/// The variables: the centre of each item, then the top and the bottom edge
/// of each group.
struct Vars {
    items: usize,
}

impl Vars {
    fn top(&self, group: usize) -> usize {
        self.items + 2 * group
    }

    fn bottom(&self, group: usize) -> usize {
        self.items + 2 * group + 1
    }
}

/// Returns the centre of each item, and the top and the bottom of each group.
///
/// A group that holds no node has no constraint, and its two numbers mean
/// nothing.
pub(super) fn place(
    net: &Net,
    groups: &[Group],
    chains: &[Vec<usize>],
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let vars = Vars {
        items: net.items.len(),
    };
    let limits = limits(net, groups, &vars);
    let mut at = pack(&limits);
    let wishes = wishes(net);
    // Groups with a node, outermost first.
    let mut outer_first: Vec<usize> = (0..groups.len())
        .filter(|&g| net.spans[g].is_some())
        .collect();
    outer_first.sort_by_key(|&g| chains[g].len());

    for round in 0..ROUNDS {
        // Open each group as far as what is around it allows, so that its
        // items can move. The side that opens first alternates, so that two
        // groups share the room between them.
        for side in [round % 2, 1 - round % 2] {
            for &g in &outer_first {
                if side == 0 {
                    let top = limits.top(vars.top(g), &at);
                    at[vars.top(g)] = top.unwrap_or(f64::NEG_INFINITY);
                } else {
                    let bottom = limits.bottom(vars.bottom(g), &at);
                    at[vars.bottom(g)] = bottom.unwrap_or(f64::INFINITY);
                }
            }
        }
        let mut sequence: Vec<usize> = net.layers.iter().flatten().copied().collect();
        if round % 2 == 1 {
            sequence.reverse();
        }
        for i in sequence {
            let wanted = match &wishes[i] {
                Wish::Stay => at[i],
                Wish::Middle(of) => {
                    let mut ys: Vec<f64> = of.iter().map(|&j| at[j]).collect();
                    ys.sort_by(f64::total_cmp);
                    f64::midpoint(ys[(ys.len() - 1) / 2], ys[ys.len() / 2])
                }
                Wish::Mean(of) => of.iter().map(|&j| at[j]).sum::<f64>() / real(of.len()),
            };
            let top = limits.top(i, &at).unwrap_or(f64::NEG_INFINITY);
            let bottom = limits.bottom(i, &at).unwrap_or(f64::INFINITY);
            // A wish is a whole pixel. A middle of two middles would halve the
            // distance for ever, and the last bits of a position would be
            // noise that keeps a straight link from being exactly straight.
            at[i] = wanted.round().max(top).min(bottom);
        }
        // Close each group around what it holds again, innermost first.
        for &g in outer_first.iter().rev() {
            if let Some(top) = limits.bottom(vars.top(g), &at) {
                at[vars.top(g)] = top;
            }
            if let Some(bottom) = limits.top(vars.bottom(g), &at) {
                at[vars.bottom(g)] = bottom;
            }
        }
    }
    let edges = (0..groups.len())
        .map(|g| (at[vars.top(g)], at[vars.bottom(g)]))
        .collect();
    at.truncate(vars.items);
    (at, edges)
}

fn limits(net: &Net, groups: &[Group], vars: &Vars) -> Limits {
    let count = vars.items + 2 * groups.len();
    let mut limits = Limits {
        above: vec![Vec::new(); count],
        below: vec![Vec::new(); count],
    };
    for (i, item) in net.items.iter().enumerate() {
        if let Some(&g) = item.chain.last() {
            limits.add(vars.top(g), i, GROUP_TITLE + item.height / 2.0);
            limits.add(i, vars.bottom(g), GROUP_PAD + item.height / 2.0);
        }
    }
    for (g, group) in groups.iter().enumerate() {
        if let Some(parent) = group.parent
            && net.spans[g].is_some()
        {
            limits.add(vars.top(parent), vars.top(g), GROUP_TITLE);
            limits.add(vars.bottom(g), vars.bottom(parent), GROUP_PAD);
        }
    }
    for layer in &net.layers {
        for pair in layer.windows(2) {
            let (a, b) = (&net.items[pair[0]], &net.items[pair[1]]);
            // What to keep apart: for each of the two, the outermost group
            // that the other is not in, or the item when no such group is.
            let shared = shared(&a.chain, &b.chain);
            let (upper, upper_half) = match a.chain.get(shared) {
                Some(&g) => (vars.bottom(g), 0.0),
                None => (pair[0], a.height / 2.0),
            };
            let (lower, lower_half) = match b.chain.get(shared) {
                Some(&g) => (vars.top(g), 0.0),
                None => (pair[1], b.height / 2.0),
            };
            // A marker is the only item of its group in its rank, so one side
            // of a pair with a marker is the edge of a group, and the pair
            // gets the node gap.
            let two_items = shared == a.chain.len() && shared == b.chain.len();
            let gap = if two_items && (a.kind == Kind::Bend || b.kind == Kind::Bend) {
                BEND_GAP
            } else {
                NODE_GAP
            };
            limits.add(upper, lower, gap + upper_half + lower_half);
        }
    }
    limits
}

/// The first solution: every variable as high on the page as the constraints
/// allow, with the first at 0.
///
/// # Panics
///
/// When the constraints form a loop. The order phase gives the groups under
/// one parent one order in every rank, so no loop exists.
fn pack(limits: &Limits) -> Vec<f64> {
    let count = limits.above.len();
    let mut at = vec![0.0; count];
    let mut waiting: Vec<usize> = limits.above.iter().map(Vec::len).collect();
    let mut ready: Vec<usize> = (0..count).filter(|&v| waiting[v] == 0).collect();
    let mut solved = 0;
    while let Some(v) = ready.pop() {
        solved += 1;
        for &(w, gap) in &limits.below[v] {
            at[w] = f64::max(at[w], at[v] + gap);
            waiting[w] -= 1;
            if waiting[w] == 0 {
                ready.push(w);
            }
        }
    }
    assert_eq!(
        solved, count,
        "two ranks hold two groups in a different order"
    );
    at
}

/// Where an item wants to be.
enum Wish {
    Stay,
    /// The middle of these items: a node wants the middle of the nodes that
    /// it links to, so that a link can run straight. A bend wants the height
    /// of the node that its link starts at, so that the link turns once.
    Middle(Vec<usize>),
    /// The mean of these items: a marker wants the middle of its group.
    Mean(Vec<usize>),
}

fn wishes(net: &Net) -> Vec<Wish> {
    let mut linked: Vec<Vec<usize>> = vec![Vec::new(); net.items.len()];
    for path in &net.paths {
        if let (Some(&first), Some(&last)) = (path.first(), path.last()) {
            linked[first].push(last);
            linked[last].push(first);
            for &bend in &path[1..path.len() - 1] {
                linked[bend].push(first);
            }
        }
    }
    let nodes_of = |group: usize| -> Vec<usize> {
        let inside =
            |i: &usize| net.items[*i].kind == Kind::Node && net.items[*i].chain.contains(&group);
        (0..net.items.len()).filter(inside).collect()
    };
    net.items
        .iter()
        .zip(linked)
        .map(|(item, linked)| match (item.kind, item.chain.last()) {
            (Kind::Marker, Some(&group)) => Wish::Mean(nodes_of(group)),
            _ if linked.is_empty() => Wish::Stay,
            _ => Wish::Middle(linked),
        })
        .collect()
}
