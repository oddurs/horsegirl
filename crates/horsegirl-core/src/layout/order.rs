//! The order phase: sort each rank so that few links cross.
//!
//! Each rank is sorted by the mean position of the neighbours of each item
//! in the rank beside it, forward and back, and the order with the fewest
//! crossings is kept.
//!
//! Two rules keep a group in one piece. In each rank, the items of a group
//! are next to each other. The groups under one parent have one order, and
//! every rank holds it. So one group is above another in every rank that
//! they share, and their boxes need no repair.
//!
//! A second part then keeps each link out of the groups that hold neither of
//! its ends: see [`super::through`].

use std::collections::BTreeMap;

use super::through::keep_out;
use super::{Group, Item, Net, real};

const ROUNDS: usize = 24;

pub(super) fn order(net: &mut Net, groups: &[Group]) {
    let count = net.items.len();
    let mut before = vec![Vec::new(); count];
    let mut after = vec![Vec::new(); count];
    for &(a, b) in &net.steps {
        after[a].push(b);
        before[b].push(a);
    }

    // The first order: a walk along the links, so that a chain starts out
    // straight.
    let mut seen = vec![false; count];
    let mut stack: Vec<usize> = (0..count).rev().collect();
    for layer in &mut net.layers {
        layer.clear();
    }
    while let Some(v) = stack.pop() {
        if !seen[v] {
            seen[v] = true;
            net.layers[net.items[v].rank].push(v);
            stack.extend(after[v].iter().rev());
        }
    }

    let mut keys = group_keys(net, groups.len());
    for layer in &mut net.layers {
        let value: Vec<f64> = (0..layer.len()).map(real).collect();
        arrange(layer, &value, &net.items, groups, &keys);
    }
    let mut best = net.layers.clone();
    let mut best_keys = keys.clone();
    let mut fewest = crossings(&net.layers, &net.steps, count);
    let last = net.layers.len().saturating_sub(1);
    for _ in 0..ROUNDS {
        if fewest == 0 {
            break;
        }
        // One set of keys serves a whole round, and a round sorts every
        // rank, so every rank holds one order of the groups when it ends.
        keys = group_keys(net, groups.len());
        let forward = (1..=last).map(|rank| (rank, &before));
        let backward = (0..last).rev().map(|rank| (rank, &after));
        for (rank, neighbours) in forward.chain(backward) {
            let at = positions(&net.layers, count);
            let value: Vec<f64> = net.layers[rank]
                .iter()
                .map(|&i| {
                    let near = &neighbours[i];
                    if near.is_empty() {
                        at[i]
                    } else {
                        near.iter().map(|&j| at[j]).sum::<f64>() / real(near.len())
                    }
                })
                .collect();
            arrange(&mut net.layers[rank], &value, &net.items, groups, &keys);
        }
        let now = crossings(&net.layers, &net.steps, count);
        if now < fewest {
            fewest = now;
            best.clone_from(&net.layers);
            best_keys.clone_from(&keys);
        }
    }
    net.layers = best;
    keep_out(net, groups, &best_keys);
}

/// The index of each item in its rank.
pub(super) fn positions(layers: &[Vec<usize>], count: usize) -> Vec<f64> {
    let mut at = vec![0.0; count];
    for layer in layers {
        for (index, &i) in layer.iter().enumerate() {
            at[i] = real(index);
        }
    }
    at
}

/// One number for each group: how far down its ranks its items are, as a
/// mean. The groups under one parent keep the order of these numbers.
fn group_keys(net: &Net, groups: usize) -> Vec<f64> {
    let mut sum = vec![(0.0, 0.0); groups];
    for layer in &net.layers {
        for (index, &i) in layer.iter().enumerate() {
            for &g in &net.items[i].chain {
                sum[g].0 += real(index) / real(layer.len());
                sum[g].1 += 1.0;
            }
        }
    }
    sum.iter()
        .map(|&(total, items)| if items > 0.0 { total / items } else { 0.0 })
        .collect()
}

/// Sorts one rank by `value`, which gives a number for each position.
///
/// A group sorts as one entry with the mean of the values of its items. The
/// groups under one parent then exchange their values, so that they stand in
/// the order of their keys and a node between two groups keeps its place.
pub(super) fn arrange(
    layer: &mut [usize],
    value: &[f64],
    items: &[Item],
    groups: &[Group],
    keys: &[f64],
) {
    // An entry is a group, by its index, or an item, by its index after the
    // groups. The second number of an entry settles equal values.
    let first_item = groups.len();
    let mut entries: BTreeMap<usize, (f64, usize)> = BTreeMap::new();
    let mut members: BTreeMap<usize, f64> = BTreeMap::new();
    for (index, &i) in layer.iter().enumerate() {
        entries.insert(first_item + i, (value[index], layer.len() + index));
        for &g in &items[i].chain {
            entries.entry(g).or_default().0 += value[index];
            *members.entry(g).or_default() += 1.0;
        }
    }
    let mut siblings: BTreeMap<Option<usize>, Vec<usize>> = BTreeMap::new();
    for (&g, &items) in &members {
        entries.entry(g).or_default().0 /= items;
        siblings.entry(groups[g].parent).or_default().push(g);
    }
    for family in siblings.values_mut() {
        family.sort_by(|&a, &b| keys[a].total_cmp(&keys[b]).then(a.cmp(&b)));
        let mut values: Vec<f64> = family.iter().map(|g| entries[g].0).collect();
        values.sort_by(f64::total_cmp);
        for (index, (&g, value)) in family.iter().zip(values).enumerate() {
            entries.insert(g, (value, index));
        }
    }

    // An item sorts by its groups from the outside in, then by itself.
    let path = |i: usize| -> Vec<(f64, usize)> {
        let outside_in = items[i].chain.iter().copied().chain([first_item + i]);
        outside_in.map(|entry| entries[&entry]).collect()
    };
    let mut keyed: Vec<(Vec<(f64, usize)>, usize)> = layer.iter().map(|&i| (path(i), i)).collect();
    keyed.sort_by(|(a, _), (b, _)| {
        let step = |(x, y): (&(f64, usize), &(f64, usize))| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1));
        a.iter()
            .zip(b)
            .map(step)
            .find(|order| order.is_ne())
            .unwrap_or(a.len().cmp(&b.len()))
    });
    for (slot, (_, i)) in layer.iter_mut().zip(keyed) {
        *slot = i;
    }
}

/// How many pairs of steps cross between two neighbouring ranks.
fn crossings(layers: &[Vec<usize>], steps: &[(usize, usize)], count: usize) -> usize {
    let at = positions(layers, count);
    let mut rank_of = vec![0; count];
    for (rank, layer) in layers.iter().enumerate() {
        for &i in layer {
            rank_of[i] = rank;
        }
    }
    let mut by_rank: Vec<Vec<(f64, f64)>> = vec![Vec::new(); layers.len()];
    for &(a, b) in steps {
        by_rank[rank_of[a]].push((at[a], at[b]));
    }
    by_rank.iter().map(|gap| crossed(gap)).sum()
}

/// How many pairs cross among the steps between two ranks. A step is the
/// position of its item in each of the two ranks. Two steps that share an
/// item do not cross.
pub(super) fn crossed(gap: &[(f64, f64)]) -> usize {
    let mut steps = gap.to_vec();
    steps.sort_by(|p, q| p.0.total_cmp(&q.0).then(p.1.total_cmp(&q.1)));
    let mut ends: Vec<f64> = steps.iter().map(|step| step.1).collect();
    inversions(&mut ends)
}

/// Sorts the values and returns how many pairs were in the wrong order. A
/// merge sort counts them in fewer steps than a look at every pair.
fn inversions(values: &mut [f64]) -> usize {
    if values.len() < 2 {
        return 0;
    }
    let (left, right) = values.split_at_mut(values.len() / 2);
    let mut count = inversions(left) + inversions(right);
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if right[j] < left[i] {
            // Every value that is left in the first half is larger.
            count += left.len() - i;
            merged.push(right[j]);
            j += 1;
        } else {
            merged.push(left[i]);
            i += 1;
        }
    }
    merged.extend_from_slice(&left[i..]);
    merged.extend_from_slice(&right[j..]);
    values.copy_from_slice(&merged);
    count
}
