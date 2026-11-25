//! The second part of the order phase: keep each link out of the groups
//! that hold neither of its ends.
//!
//! An upright line is in the corridor between two ranks. A group that has
//! both ranks has its box across that corridor. So a link passes through the
//! group when it is above the group in one rank and below it in the next. A
//! sweep sorts one rank at a time and cannot see that, and no sweep moves
//! every item of one link at once or changes the order of two groups. This
//! part makes those moves. It keeps a move when the passes fall, or when the
//! passes stay and the crossings fall, so it ends.
//!
//! It tries one move at a time, so a pass can stay that two moves together
//! would remove. Some passes no order removes: two groups under one parent
//! can each have a link to a third group that only goes through the other.
//!
//! A move that changes a group counts the whole net again. That is the
//! number of moves times the number of steps for each round, which is small
//! for a diagram that a person reads.

use super::order::{arrange, crossed, positions};
use super::{Group, Item, Net, real, shared};

/// A change to the order of the ranks. Each one keeps the two rules of the
/// order phase: a group is in one piece in each rank, and the groups under
/// one parent have one order in every rank.
#[derive(Clone, Copy)]
enum Move {
    /// A group takes another place among the groups under its parent, in
    /// every rank.
    Shift { group: usize, to: usize },
    /// Every item of a link that is beside a group goes to one side of it:
    /// above it or below it.
    Side {
        link: usize,
        group: usize,
        above: bool,
    },
    /// The entry that ends before a position of a rank and the entry that
    /// starts there exchange their places. An entry is an item, or every
    /// item of one group.
    Transpose { rank: usize, at: usize },
}

struct Search<'a> {
    net: &'a mut Net,
    groups: &'a [Group],
    /// The groups around each group, outermost first, ending with the group.
    chains: Vec<Vec<usize>>,
    /// Each link with each group that holds neither of its ends and that has
    /// two or more of the ranks of the link.
    pairs: Vec<(usize, usize)>,
    /// The steps that leave each rank.
    gaps: Vec<Vec<(usize, usize)>>,
    /// The place of each group among the groups under its parent.
    places: Vec<f64>,
}

/// Changes the order of the ranks until no move removes a pass or a
/// crossing.
///
/// `keys` are the numbers that gave the groups of `net.layers` their order.
/// A net with no pass keeps its order.
pub(super) fn keep_out(net: &mut Net, groups: &[Group], keys: &[f64]) {
    let mut pairs = Vec::new();
    for (link, path) in net.paths.iter().enumerate() {
        let (Some(&first), Some(&last)) = (path.first(), path.last()) else {
            continue;
        };
        let (from, to) = (&net.items[first], &net.items[last]);
        for (group, span) in net.spans.iter().enumerate() {
            let around = from.chain.contains(&group) || to.chain.contains(&group);
            if let Some((low, high)) = *span
                && !around
                && low.max(from.rank) < high.min(to.rank)
            {
                pairs.push((link, group));
            }
        }
    }
    let mut gaps = vec![Vec::new(); net.layers.len()];
    for &(a, b) in &net.steps {
        gaps[net.items[a].rank].push((a, b));
    }
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by(|&a, &b| keys[a].total_cmp(&keys[b]).then(a.cmp(&b)));
    let mut places = vec![0.0; groups.len()];
    for (place, &group) in order.iter().enumerate() {
        places[group] = real(place);
    }
    let mut search = Search {
        net,
        groups,
        chains: super::chains(groups),
        pairs,
        gaps,
        places,
    };
    let mut cost = search.cost();
    if cost.0 == 0 {
        return;
    }
    loop {
        let before = cost;
        for change in search.moves() {
            let earlier = (search.net.layers.clone(), search.places.clone());
            let now = search.apply(change, cost);
            debug_assert_eq!(
                now,
                search.cost(),
                "the cost of one rank is not the whole cost"
            );
            if now < cost {
                cost = now;
            } else {
                (search.net.layers, search.places) = earlier;
            }
        }
        if cost == before {
            return;
        }
    }
}

impl Search<'_> {
    /// The passes, then the crossings.
    fn cost(&self) -> (usize, usize) {
        let at = positions(&self.net.layers, self.net.items.len());
        let crossings = (0..self.gaps.len()).map(|gap| self.crossed(gap, &at));
        (self.passes(&at).len(), crossings.sum())
    }

    /// How many pairs of steps cross after one rank.
    fn crossed(&self, gap: usize, at: &[f64]) -> usize {
        let steps = self.gaps[gap].iter();
        crossed(&steps.map(|&(a, b)| (at[a], at[b])).collect::<Vec<_>>())
    }

    /// Each link with each group that it passes through.
    fn passes(&self, at: &[f64]) -> Vec<(usize, usize)> {
        let net = &*self.net;
        // The first position of each group in each rank.
        let ranks = net.layers.len();
        let mut top = vec![f64::INFINITY; self.groups.len() * ranks];
        for (i, item) in net.items.iter().enumerate() {
            for &g in &item.chain {
                let slot = &mut top[g * ranks + item.rank];
                *slot = slot.min(at[i]);
            }
        }
        let above = |i: usize, group: usize| at[i] < top[group * ranks + net.items[i].rank];
        let passes = |&(link, group): &(usize, usize)| {
            let (low, high) = net.spans[group].unwrap_or((0, 0));
            net.paths[link].windows(2).any(|step| {
                let inside = low <= net.items[step[0]].rank && net.items[step[1]].rank <= high;
                inside && above(step[0], group) != above(step[1], group)
            })
        };
        self.pairs.iter().copied().filter(passes).collect()
    }

    /// The groups with a node under the parent of `group`, in their order.
    fn family(&self, group: usize) -> Vec<usize> {
        let parent = self.groups[group].parent;
        let member = |&g: &usize| self.groups[g].parent == parent && self.net.spans[g].is_some();
        let mut family: Vec<usize> = (0..self.groups.len()).filter(member).collect();
        family.sort_by(|&a, &b| self.places[a].total_cmp(&self.places[b]));
        family
    }

    /// Every move to try in one round, in a fixed order.
    fn moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for group in (0..self.groups.len()).filter(|&g| self.net.spans[g].is_some()) {
            let family = self.family(group);
            let places = (0..family.len()).filter(|&to| family[to] != group);
            moves.extend(places.map(|to| Move::Shift { group, to }));
        }
        let at = positions(&self.net.layers, self.net.items.len());
        for (link, group) in self.passes(&at) {
            moves.extend([true, false].map(|above| Move::Side { link, group, above }));
        }
        for (rank, layer) in self.net.layers.iter().enumerate() {
            moves.extend((1..layer.len()).map(|at| Move::Transpose { rank, at }));
        }
        moves
    }

    /// Makes a move and returns the cost after it. `cost` is the cost before
    /// the move.
    fn apply(&mut self, change: Move, cost: (usize, usize)) -> (usize, usize) {
        let items = &self.net.items;
        match change {
            Move::Shift { group, to } => {
                let mut family = self.family(group);
                let mut values: Vec<f64> = family.iter().map(|&g| self.places[g]).collect();
                values.sort_by(f64::total_cmp);
                family.retain(|&g| g != group);
                family.insert(to, group);
                for (g, value) in family.into_iter().zip(values) {
                    self.places[g] = value;
                }
                for layer in &mut self.net.layers {
                    let value: Vec<f64> = (0..layer.len()).map(real).collect();
                    arrange(layer, &value, items, self.groups, &self.places);
                }
            }
            Move::Side { link, group, above } => {
                for &i in &self.net.paths[link] {
                    let layer = &mut self.net.layers[items[i].rank];
                    beside(layer, items, i, &self.chains[group], above);
                }
            }
            Move::Transpose { rank, at } => {
                // Two items that exchange their places change the crossings
                // beside their rank and nothing else.
                let beside = |search: &Self| {
                    let at = positions(&search.net.layers, search.net.items.len());
                    let gaps = rank.saturating_sub(1)..=rank;
                    gaps.map(|gap| search.crossed(gap, &at)).sum::<usize>()
                };
                let before = beside(self);
                if !transpose(&mut self.net.layers[rank], items, at) {
                    return (cost.0, cost.1 + beside(self) - before);
                }
            }
        }
        self.cost()
    }
}

/// Puts item `i` next to a group, above it or below it. `chain` is the chain
/// of the group.
///
/// The item moves when the group has the rank and the item is beside the
/// group or beside a group around it: when the innermost group that holds
/// both holds the item with no other group between. An item in another group
/// stays, because its group has its place.
fn beside(layer: &mut Vec<usize>, items: &[Item], i: usize, chain: &[usize], above: bool) {
    let depth = shared(&items[i].chain, chain);
    if depth < items[i].chain.len() || depth == chain.len() {
        return;
    }
    let holds = |&j: &usize| items[j].chain.get(depth) == Some(&chain[depth]);
    let Some(first) = layer.iter().position(holds) else {
        return;
    };
    let last = layer.iter().rposition(holds).unwrap_or(first);
    let Some(at) = layer.iter().position(|&j| j == i) else {
        return;
    };
    if above && at > last {
        layer.remove(at);
        layer.insert(first, i);
    } else if !above && at < first {
        layer.remove(at);
        layer.insert(last, i);
    }
}

/// Exchanges the entry that ends before `at` and the entry that starts at
/// `at`, when one of the two is an item. Two groups keep their order: every
/// rank holds it. Returns whether a group moved.
fn transpose(layer: &mut [usize], items: &[Item], at: usize) -> bool {
    let (upper, lower) = (&items[layer[at - 1]].chain, &items[layer[at]].chain);
    let depth = shared(upper, lower);
    if upper.len() > depth && lower.len() > depth {
        return false;
    }
    let same = |j: usize, chain: &[usize]| {
        chain.len() > depth && items[j].chain.get(depth) == chain.get(depth)
    };
    let mut start = at - 1;
    while start > 0 && same(layer[start - 1], upper) {
        start -= 1;
    }
    let mut end = at + 1;
    while end < layer.len() && same(layer[end], lower) {
        end += 1;
    }
    layer[start..end].rotate_left(at - start);
    upper.len() > depth || lower.len() > depth
}
