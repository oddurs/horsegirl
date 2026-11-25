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
//! A net with one group ends with no pass: one move puts every item that is
//! outside the group above it. The same move does that for a chain of
//! groups, each inside the one before: it clears each group of the chain in
//! every rank of that group.
//!
//! A move changes some ranks, and the search counts again the steps that
//! start or end in those ranks only. A group takes another place only when a
//! link passes through it or through a group under the same parent. So a
//! large net with few passes costs little. A large net with hundreds of
//! passes that no order removes gives two moves for each pass in each round.
//! The search has a limit on its work, so such a net can keep a pass that a
//! longer search removes.

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
    /// Every item that is beside a group or beside a group around it goes
    /// to one side of that group, in every rank of that group. `Side` moves
    /// one link, and that can push another link through the group. This
    /// move takes every link at once.
    Clear { group: usize, above: bool },
    /// The entry that ends before a position of a rank and the entry that
    /// starts there exchange their places. An entry is an item, or every
    /// item of one group.
    Transpose { rank: usize, at: usize },
}

/// A step of a link inside the ranks of a group that holds neither end of
/// the link.
struct Check {
    /// The index of the link and the group in `Search::pairs`.
    pair: usize,
    from: usize,
    to: usize,
    /// Whether the step starts on one side of the group and ends on the
    /// other.
    turned: bool,
}

/// The most work that one search does. The unit is one item or one step of
/// a rank that the search counts again, and one group or one item that a
/// move copies before it runs. A dense net of 100 nodes in 10 groups with
/// 390 links needs many times the limit to end. The limit is a count and
/// not a time, so the same input gives the same order on every machine.
const LIMIT: usize = 1_000_000;

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
    /// The steps that leave each rank and can pass through a group.
    checks: Vec<Vec<Check>>,
    /// The position of each item in its rank.
    at: Vec<f64>,
    /// The first position of each group in each rank.
    top: Vec<f64>,
    /// How many steps of each pair are turned.
    turns: Vec<usize>,
    /// How many pairs of steps cross after each rank.
    crossings: Vec<usize>,
    /// The passes, then the crossings.
    cost: (usize, usize),
    /// The work so far, in the unit of `LIMIT`.
    work: usize,
}

/// Changes the order of the ranks until no move removes a pass or a
/// crossing, or until the work reaches `LIMIT`.
///
/// `keys` are the numbers that gave the groups of `net.layers` their order.
/// A net with no pass keeps its order.
pub(super) fn keep_out(net: &mut Net, groups: &[Group], keys: &[f64]) {
    let Some(last) = net.layers.len().checked_sub(1) else {
        return;
    };
    let mut pairs = Vec::new();
    let mut checks: Vec<Vec<Check>> = net.layers.iter().map(|_| Vec::new()).collect();
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
                for step in path.windows(2) {
                    let rank = net.items[step[0]].rank;
                    if low <= rank && rank < high {
                        checks[rank].push(Check {
                            pair: pairs.len(),
                            from: step[0],
                            to: step[1],
                            turned: false,
                        });
                    }
                }
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
        groups,
        chains: super::chains(groups),
        turns: vec![0; pairs.len()],
        pairs,
        gaps,
        places,
        checks,
        at: vec![0.0; net.items.len()],
        top: vec![f64::INFINITY; groups.len() * net.layers.len()],
        crossings: vec![0; net.layers.len()],
        cost: (0, 0),
        work: 0,
        net,
    };
    search.recount(0, last);
    if search.cost.0 == 0 {
        return;
    }
    let mut tried = 0;
    loop {
        let before = search.cost;
        for change in search.moves() {
            // Each move is kept or undone, so the order is whole here.
            if search.work > LIMIT {
                return;
            }
            let cost = search.cost;
            let (low, high) = search.ranks(change);
            let layers = search.net.layers[low..=high].to_vec();
            let places = search.places.clone();
            // A move that changes no rank still costs these copies, so each
            // move counts before it runs. A net with hundreds of groups in
            // a line gives one `Shift` for each pair, and most change nothing.
            search.work += places.len() + layers.iter().map(Vec::len).sum::<usize>();
            tried += 1;
            debug_assert!(
                tried * places.len() <= LIMIT + places.len(),
                "the limit does not count a move that the search tried"
            );
            search.apply(change);
            // Most moves change few of the ranks that they can change.
            let changed = |rank: &usize| search.net.layers[*rank] != layers[*rank - low];
            let ends = (low..=high).find(changed).zip((low..=high).rfind(changed));
            let Some((first, last)) = ends else {
                search.places = places;
                continue;
            };
            search.recount(first, last);
            debug_assert_eq!(
                search.cost,
                search.count(),
                "the cost of the ranks of a move is not the whole cost"
            );
            if search.cost >= cost {
                search.net.layers[first..=last].clone_from_slice(&layers[first - low..=last - low]);
                search.places = places;
                search.recount(first, last);
            }
        }
        if search.cost == before {
            return;
        }
    }
}

impl Search<'_> {
    /// Counts again what an order of the ranks from `low` to `high` changes:
    /// the positions in those ranks, and the passes and the crossings of the
    /// steps that start or end there. So a move costs what it touches, and
    /// not the whole net.
    fn recount(&mut self, low: usize, high: usize) {
        let ranks = self.net.layers.len();
        for rank in low..=high {
            for group in 0..self.groups.len() {
                self.top[group * ranks + rank] = f64::INFINITY;
            }
            for (index, &i) in self.net.layers[rank].iter().enumerate() {
                self.at[i] = real(index);
                for &g in &self.net.items[i].chain {
                    let slot = &mut self.top[g * ranks + rank];
                    *slot = slot.min(real(index));
                }
            }
        }
        for gap in low.saturating_sub(1)..=high {
            self.work += self.gaps[gap].len() + self.net.layers[gap].len();
            let steps = self.gaps[gap].iter();
            let now = crossed(
                &steps
                    .map(|&(a, b)| (self.at[a], self.at[b]))
                    .collect::<Vec<_>>(),
            );
            self.cost.1 = self.cost.1 + now - self.crossings[gap];
            self.crossings[gap] = now;
            for check in &mut self.checks[gap] {
                let group = self.pairs[check.pair].1;
                let above = |i: usize, rank: usize| self.at[i] < self.top[group * ranks + rank];
                let turned = above(check.from, gap) != above(check.to, gap + 1);
                if turned == check.turned {
                    continue;
                }
                check.turned = turned;
                let turns = &mut self.turns[check.pair];
                if turned {
                    *turns += 1;
                    self.cost.0 += usize::from(*turns == 1);
                } else {
                    *turns -= 1;
                    self.cost.0 -= usize::from(*turns == 0);
                }
            }
        }
    }

    /// The passes, then the crossings, counted from nothing. It checks
    /// `recount` in a debug build.
    fn count(&self) -> (usize, usize) {
        let net = &*self.net;
        let at = positions(&net.layers, net.items.len());
        let ranks = net.layers.len();
        let mut top = vec![f64::INFINITY; self.groups.len() * ranks];
        for (i, item) in net.items.iter().enumerate() {
            for &g in &item.chain {
                let slot = &mut top[g * ranks + item.rank];
                *slot = slot.min(at[i]);
            }
        }
        let above = |i: usize, group: usize| at[i] < top[group * ranks + net.items[i].rank];
        let passes = |&&(link, group): &&(usize, usize)| {
            let (low, high) = net.spans[group].unwrap_or((0, 0));
            net.paths[link].windows(2).any(|step| {
                let inside = low <= net.items[step[0]].rank && net.items[step[1]].rank <= high;
                inside && above(step[0], group) != above(step[1], group)
            })
        };
        let crossings = self
            .gaps
            .iter()
            .map(|gap| crossed(&gap.iter().map(|&(a, b)| (at[a], at[b])).collect::<Vec<_>>()));
        (self.pairs.iter().filter(passes).count(), crossings.sum())
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
        let passes = self.pairs.iter().zip(&self.turns);
        let passes: Vec<(usize, usize)> = passes
            .filter(|&(_, &turns)| turns > 0)
            .map(|(&pair, _)| pair)
            .collect();
        let mut through = vec![false; self.groups.len()];
        for &(_, group) in &passes {
            through[group] = true;
        }
        // Another place for a group removes a pass through the group, or
        // through a group under the same parent. So the other groups keep
        // their places, and a net with few passes has few moves.
        let mut moves = Vec::new();
        let mut tried = vec![false; self.groups.len()];
        for group in (0..self.groups.len()).filter(|&g| through[g]) {
            if tried[group] {
                continue;
            }
            let family = self.family(group);
            for &member in &family {
                tried[member] = true;
                let places = (0..family.len()).filter(|&to| family[to] != member);
                moves.extend(places.map(|to| Move::Shift { group: member, to }));
            }
        }
        for &(link, group) in &passes {
            moves.extend([true, false].map(|above| Move::Side { link, group, above }));
        }
        // A pass through a group can need a clear side at a group inside
        // it: the link that passes has an end beside the inner group.
        let around = |&g: &usize| self.chains[g].iter().any(|&outer| through[outer]);
        for group in (0..self.groups.len()).filter(around) {
            moves.extend([true, false].map(|above| Move::Clear { group, above }));
        }
        for (rank, layer) in self.net.layers.iter().enumerate() {
            moves.extend((1..layer.len()).map(|at| Move::Transpose { rank, at }));
        }
        moves
    }

    /// The first and the last rank that a move changes.
    fn ranks(&self, change: Move) -> (usize, usize) {
        let net = &*self.net;
        match change {
            // A rank that the group has no item in keeps its order.
            Move::Shift { group, .. } => net.spans[group].unwrap_or((0, 0)),
            // The outermost group has every rank of the groups inside it.
            Move::Clear { group, .. } => net.spans[self.chains[group][0]].unwrap_or((0, 0)),
            Move::Side { link, .. } => {
                let path = &net.paths[link];
                (
                    net.items[path[0]].rank,
                    net.items[path[path.len() - 1]].rank,
                )
            }
            Move::Transpose { rank, .. } => (rank, rank),
        }
    }

    fn apply(&mut self, change: Move) {
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
                let (low, high) = self.net.spans[group].unwrap_or((0, 0));
                for layer in &mut self.net.layers[low..=high] {
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
            Move::Clear { group, above } => {
                let chain = &self.chains[group];
                // An item beside a group around this one has links to the
                // items beside this one. So each group of the chain gets a
                // clear side in each of its own ranks, or those links pass
                // through this group or through a group around it.
                for depth in 1..=chain.len() {
                    let Some((low, high)) = self.net.spans[chain[depth - 1]] else {
                        continue;
                    };
                    for layer in &mut self.net.layers[low..=high] {
                        clear(layer, items, &chain[..depth], above);
                    }
                }
            }
            Move::Transpose { rank, at } => transpose(&mut self.net.layers[rank], items, at),
        }
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

/// Puts every item that is beside a group, under the same parent, above the
/// group or below it. The items keep their order. `chain` is the chain of the
/// group.
fn clear(layer: &mut Vec<usize>, items: &[Item], chain: &[usize], above: bool) {
    let holds = |&j: &usize| items[j].chain.starts_with(chain);
    let Some(first) = layer.iter().position(holds) else {
        return;
    };
    let last = layer.iter().rposition(holds).unwrap_or(first);
    let outside = &chain[..chain.len() - 1];
    let goes = |at: usize, j: usize| {
        items[j].chain == outside && if above { at > last } else { at < first }
    };
    let entries = || layer.iter().copied().enumerate();
    let moved: Vec<usize> = entries()
        .filter(|&(at, j)| goes(at, j))
        .map(|e| e.1)
        .collect();
    let mut rest: Vec<usize> = entries()
        .filter(|&(at, j)| !goes(at, j))
        .map(|e| e.1)
        .collect();
    let to = if above { first } else { last + 1 - moved.len() };
    rest.splice(to..to, moved);
    *layer = rest;
}

/// Exchanges the entry that ends before `at` and the entry that starts at
/// `at`, when one of the two is an item. Two groups keep their order: every
/// rank holds it.
fn transpose(layer: &mut [usize], items: &[Item], at: usize) {
    let (upper, lower) = (&items[layer[at - 1]].chain, &items[layer[at]].chain);
    let depth = shared(upper, lower);
    if upper.len() > depth && lower.len() > depth {
        return;
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
}
