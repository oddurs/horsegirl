//! Candidate: our own Sugiyama pipeline.
//!
//! Five phases: orient, rank, order, place, route. The order phase and the place phase take
//! the earlier positions as a seed. The seed enters each phase in one place, which the
//! comments mark with "Seed:".

use std::collections::{BTreeMap, VecDeque};

use crate::model::{Graph, Picture, Rect, Seed};

const NODE_GAP: f64 = 24.0;
const BEND_GAP: f64 = 12.0;
const RANK_GAP: f64 = 64.0;
const PAD: f64 = 14.0;
const TITLE: f64 = 30.0;
const ORDER_ROUNDS: usize = 24;
const PLACE_ROUNDS: usize = 80;

#[derive(PartialEq)]
enum Kind {
    Thing,
    /// One point of a link that spans more than one rank.
    Bend,
    /// Marks that a group is present in a rank where it may have no thing.
    Border,
}

struct Item {
    kind: Kind,
    w: f64,
    h: f64,
    rank: usize,
    /// The groups around the item, outermost first.
    chain: Vec<usize>,
    seed: Option<f64>,
}

struct Net {
    items: Vec<Item>,
    layers: Vec<Vec<usize>>,
    /// One step of a link between neighbouring ranks, lower rank first.
    steps: Vec<(usize, usize)>,
    /// The items of each link, lower rank first. Empty for a link from a thing to itself.
    paths: Vec<Vec<usize>>,
    reversed: Vec<bool>,
    /// The first and last rank of each group that holds a thing.
    span: Vec<Option<(usize, usize)>>,
}

pub fn layout(graph: &Graph, seed: Option<&Seed>) -> Picture {
    let reversed = orient(graph);
    let links: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .zip(&reversed)
        .map(|(&(a, b), &back)| if back { (b, a) } else { (a, b) })
        .collect();
    let ranks = rank(graph.nodes.len(), &links);
    let mut net = build(graph, &links, &ranks, reversed, seed);
    order(&mut net, graph, seed);
    let (y, top, bottom) = place(&net, graph);
    route(&net, graph, &y, &top, &bottom)
}

/// Marks the links to reverse so that no cycle remains: the links that close a loop in a
/// depth-first walk. The walk starts at the things that nothing links to.
fn orient(graph: &Graph) -> Vec<bool> {
    fn visit(v: usize, graph: &Graph, out: &[Vec<usize>], state: &mut [u8], back: &mut [bool]) {
        state[v] = 1;
        for &e in &out[v] {
            let w = graph.edges[e].1;
            match state[w] {
                0 => visit(w, graph, out, state, back),
                1 => back[e] = true,
                _ => {}
            }
        }
        state[v] = 2;
    }
    let n = graph.nodes.len();
    let mut out = vec![Vec::new(); n];
    let mut has_in = vec![false; n];
    for (e, &(a, b)) in graph.edges.iter().enumerate() {
        if a != b {
            out[a].push(e);
            has_in[b] = true;
        }
    }
    let mut state = vec![0; n];
    let mut back = vec![false; graph.edges.len()];
    for root in (0..n).filter(|&v| !has_in[v]).chain(0..n) {
        if state[root] == 0 {
            visit(root, graph, &out, &mut state, &mut back);
        }
    }
    back
}

/// Longest-path ranks, then each thing moves toward the side where it has more links.
/// Network simplex is the full method. This is enough to judge the pictures.
fn rank(n: usize, links: &[(usize, usize)]) -> Vec<usize> {
    let mut preds = vec![Vec::new(); n];
    let mut succs = vec![Vec::new(); n];
    for &(a, b) in links.iter().filter(|(a, b)| a != b) {
        succs[a].push(b);
        preds[b].push(a);
    }
    let mut rank = vec![0; n];
    let mut waiting: Vec<usize> = preds.iter().map(Vec::len).collect();
    let mut ready: VecDeque<usize> = (0..n).filter(|&v| waiting[v] == 0).collect();
    while let Some(v) = ready.pop_front() {
        for &w in &succs[v] {
            rank[w] = rank[w].max(rank[v] + 1);
            waiting[w] -= 1;
            if waiting[w] == 0 {
                ready.push_back(w);
            }
        }
    }
    for _ in 0..n {
        let mut changed = false;
        for v in 0..n {
            let low = preds[v].iter().map(|&u| rank[u] + 1).max();
            let high = succs[v].iter().map(|&w| rank[w] - 1).min();
            let target = match (low, high) {
                (None, Some(high)) => high,
                (Some(low), None) => low,
                (Some(_), Some(high)) if preds[v].len() < succs[v].len() => high,
                (Some(low), Some(_)) if preds[v].len() > succs[v].len() => low,
                _ => rank[v],
            };
            changed |= target != rank[v];
            rank[v] = target;
        }
        if !changed {
            break;
        }
    }
    rank
}

fn build(
    graph: &Graph,
    links: &[(usize, usize)],
    ranks: &[usize],
    reversed: Vec<bool>,
    seed: Option<&Seed>,
) -> Net {
    let mut items: Vec<Item> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| Item {
            kind: Kind::Thing,
            w: node.w,
            h: node.h,
            rank: ranks[i],
            chain: graph.chain(node.group),
            seed: seed.and_then(|seed| seed[i]).map(|(_, y)| y),
        })
        .collect();
    let mut span: Vec<Option<(usize, usize)>> = vec![None; graph.groups.len()];
    for item in &items {
        for &g in &item.chain {
            let (low, high) = span[g].unwrap_or((item.rank, item.rank));
            span[g] = Some((low.min(item.rank), high.max(item.rank)));
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
        let shared = items[a]
            .chain
            .iter()
            .zip(&items[b].chain)
            .take_while(|(x, y)| x == y)
            .count();
        let chain = items[a].chain[..shared].to_vec();
        let mut path = vec![a];
        for rank in ranks[a] + 1..ranks[b] {
            items.push(Item {
                kind: Kind::Bend,
                w: 0.0,
                h: 0.0,
                rank,
                chain: chain.clone(),
                seed: None,
            });
            path.push(items.len() - 1);
        }
        path.push(b);
        steps.extend(path.windows(2).map(|pair| (pair[0], pair[1])));
        paths.push(path);
    }
    for (g, span) in span.iter().enumerate() {
        let Some((low, high)) = *span else { continue };
        for rank in low..=high {
            items.push(Item {
                kind: Kind::Border,
                w: 0.0,
                h: 0.0,
                rank,
                chain: graph.chain(Some(g)),
                seed: None,
            });
        }
    }
    let rank_count = items.iter().map(|item| item.rank + 1).max().unwrap_or(0);
    let mut layers = vec![Vec::new(); rank_count];
    for (i, item) in items.iter().enumerate() {
        layers[item.rank].push(i);
    }
    Net {
        items,
        layers,
        steps,
        paths,
        reversed,
        span,
    }
}

/// Orders each layer by the mean position of its neighbours, sweeping forward and back, and
/// keeps the order with the fewest crossings.
fn order(net: &mut Net, graph: &Graph, seed: Option<&Seed>) {
    let n = net.items.len();
    let mut before = vec![Vec::new(); n];
    let mut after = vec![Vec::new(); n];
    for &(a, b) in &net.steps {
        after[a].push(b);
        before[b].push(a);
    }

    // The first order: a depth-first walk, so that a chain starts out straight.
    let mut seen = vec![false; n];
    let mut walk = Vec::new();
    let mut stack: Vec<usize> = (0..n).rev().collect();
    while let Some(v) = stack.pop() {
        if !seen[v] {
            seen[v] = true;
            walk.push(v);
            stack.extend(after[v].iter().rev());
        }
    }
    for layer in &mut net.layers {
        layer.clear();
    }
    for v in walk {
        net.layers[net.items[v].rank].push(v);
    }
    // Seed: a thing with an earlier position starts where it was.
    if seed.is_some() {
        for layer in &mut net.layers {
            let items = &net.items;
            layer.sort_by(|&a, &b| match (items[a].seed, items[b].seed) {
                (Some(x), Some(y)) => x.total_cmp(&y),
                (x, y) => y.is_some().cmp(&x.is_some()),
            });
        }
    }

    let mut key = group_keys(net, graph, seed);
    for layer in &mut net.layers {
        let value: Vec<f64> = (0..layer.len()).map(|at| at as f64).collect();
        arrange(layer, &value, &net.items, graph, &key);
    }
    let mut best = net.layers.clone();
    let mut fewest = crossings(&net.layers, &net.steps, n);
    for _ in 0..ORDER_ROUNDS {
        if seed.is_none() {
            key = group_keys(net, graph, None);
        }
        let last = net.layers.len().saturating_sub(1);
        let forward = (1..=last).map(|rank| (rank, &before));
        let backward = (0..last).rev().map(|rank| (rank, &after));
        for (rank, neighbours) in forward.chain(backward) {
            let at = positions(&net.layers, n);
            let value: Vec<f64> = net.layers[rank]
                .iter()
                .map(|&i| {
                    let near = &neighbours[i];
                    if near.is_empty() {
                        at[i]
                    } else {
                        near.iter().map(|&j| at[j]).sum::<f64>() / near.len() as f64
                    }
                })
                .collect();
            arrange(&mut net.layers[rank], &value, &net.items, graph, &key);
        }
        let count = crossings(&net.layers, &net.steps, n);
        if count < fewest {
            fewest = count;
            best = net.layers.clone();
        }
    }
    net.layers = best;
}

fn positions(layers: &[Vec<usize>], n: usize) -> Vec<f64> {
    let mut at = vec![0.0; n];
    for layer in layers {
        for (index, &i) in layer.iter().enumerate() {
            at[i] = index as f64;
        }
    }
    at
}

/// One number for each group. Groups with one parent keep the order of these numbers in
/// every layer, so that their boxes never need to cross.
fn group_keys(net: &Net, graph: &Graph, seed: Option<&Seed>) -> Vec<f64> {
    let mut sum = vec![(0.0, 0.0); graph.groups.len()];
    if seed.is_some() {
        // Seed: a group goes where its things were.
        for item in &net.items {
            if let Some(y) = item.seed {
                for &g in &item.chain {
                    sum[g].0 += y;
                    sum[g].1 += 1.0;
                }
            }
        }
    } else {
        for layer in &net.layers {
            for (index, &i) in layer.iter().enumerate() {
                for &g in &net.items[i].chain {
                    sum[g].0 += index as f64 / layer.len() as f64;
                    sum[g].1 += 1.0;
                }
            }
        }
    }
    sum.iter()
        .map(|&(total, count)| if count > 0.0 { total / count } else { 0.0 })
        .collect()
}

/// Sorts one layer by `value`, which gives a number for each position in `layer`.
///
/// A group stays in one piece: its members sort as one entry with the mean of their values.
/// Some entries are pinned: every group, and every thing with a seed. The pinned entries
/// under one parent exchange their values so that they keep the order of their keys.
fn arrange(layer: &mut [usize], value: &[f64], items: &[Item], graph: &Graph, key: &[f64]) {
    let groups = graph.groups.len();
    // Entry numbers: a group is its own index, and an item is `groups` plus its index.
    let mut sort_value: BTreeMap<usize, f64> = BTreeMap::new();
    let mut members: BTreeMap<usize, f64> = BTreeMap::new();
    let mut tie: BTreeMap<usize, usize> = BTreeMap::new();
    let mut pinned: BTreeMap<Option<usize>, Vec<(f64, usize)>> = BTreeMap::new();
    for (index, &i) in layer.iter().enumerate() {
        sort_value.insert(groups + i, value[index]);
        tie.insert(groups + i, layer.len() + index);
        for &g in &items[i].chain {
            *sort_value.entry(g).or_default() += value[index];
            *members.entry(g).or_default() += 1.0;
        }
        // Seed: a thing with an earlier position is pinned by its earlier position.
        if let Some(y) = items[i].seed {
            pinned
                .entry(items[i].chain.last().copied())
                .or_default()
                .push((y, groups + i));
        }
    }
    for (&g, &count) in &members {
        *sort_value.entry(g).or_default() /= count;
        pinned
            .entry(graph.groups[g].parent)
            .or_default()
            .push((key[g], g));
    }
    for entries in pinned.values_mut() {
        entries.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut values: Vec<f64> = entries.iter().map(|(_, entry)| sort_value[entry]).collect();
        values.sort_by(f64::total_cmp);
        for (index, (&(_, entry), value)) in entries.iter().zip(values).enumerate() {
            sort_value.insert(entry, value);
            tie.insert(entry, index);
        }
    }
    let path = |i: usize| -> Vec<(f64, usize)> {
        let entries = items[i].chain.iter().copied().chain([groups + i]);
        entries
            .map(|entry| (sort_value[&entry], tie[&entry]))
            .collect()
    };
    let mut keyed: Vec<(Vec<(f64, usize)>, usize)> = layer.iter().map(|&i| (path(i), i)).collect();
    keyed.sort_by(|(a, _), (b, _)| {
        let differ = a.iter().zip(b).find(|(x, y)| x != y);
        differ.map_or(a.len().cmp(&b.len()), |(x, y)| {
            x.0.total_cmp(&y.0).then(x.1.cmp(&y.1))
        })
    });
    for (slot, (_, i)) in layer.iter_mut().zip(keyed) {
        *slot = i;
    }
}

fn crossings(layers: &[Vec<usize>], steps: &[(usize, usize)], n: usize) -> usize {
    let at = positions(layers, n);
    let mut by_rank: Vec<Vec<(f64, f64)>> = vec![Vec::new(); layers.len()];
    let mut rank_of = vec![0; n];
    for (rank, layer) in layers.iter().enumerate() {
        for &i in layer {
            rank_of[i] = rank;
        }
    }
    for &(a, b) in steps {
        by_rank[rank_of[a]].push((at[a], at[b]));
    }
    let mut count = 0;
    for gap in &by_rank {
        for (index, p) in gap.iter().enumerate() {
            for q in &gap[index + 1..] {
                if (p.0 - q.0) * (p.1 - q.1) < 0.0 {
                    count += 1;
                }
            }
        }
    }
    count
}

/// A difference constraint: `position[b] - position[a] >= gap`.
struct Limits {
    low: Vec<Vec<(usize, f64)>>,
    high: Vec<Vec<(usize, f64)>>,
}

impl Limits {
    fn add(&mut self, a: usize, b: usize, gap: f64) {
        self.low[b].push((a, gap));
        self.high[a].push((b, gap));
    }

    fn low(&self, v: usize, at: &[f64]) -> Option<f64> {
        self.low[v]
            .iter()
            .map(|&(a, gap)| at[a] + gap)
            .reduce(f64::max)
    }

    fn high(&self, v: usize, at: &[f64]) -> Option<f64> {
        self.high[v]
            .iter()
            .map(|&(b, gap)| at[b] - gap)
            .reduce(f64::min)
    }
}

/// Places every item along the layer. Returns the centre of each item and the top and bottom
/// of each group.
///
/// The order and the gaps are difference constraints, so no box can overlap another. The
/// first solution packs everything upward. Each round then moves every item toward the
/// place it wants, as far as the constraints allow.
fn place(net: &Net, graph: &Graph) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = net.items.len();
    let groups = graph.groups.len();
    let top = |g: usize| n + 2 * g;
    let bottom = |g: usize| n + 2 * g + 1;
    let vars = n + 2 * groups;
    let mut limits = Limits {
        low: vec![Vec::new(); vars],
        high: vec![Vec::new(); vars],
    };

    for (i, item) in net.items.iter().enumerate() {
        if let Some(&g) = item.chain.last() {
            limits.add(top(g), i, TITLE + item.h / 2.0);
            limits.add(i, bottom(g), PAD + item.h / 2.0);
        }
    }
    for (g, group) in graph.groups.iter().enumerate() {
        if let Some(parent) = group.parent {
            limits.add(top(parent), top(g), TITLE);
            limits.add(bottom(g), bottom(parent), PAD);
        }
    }
    for layer in &net.layers {
        for pair in layer.windows(2) {
            let (a, b) = (&net.items[pair[0]], &net.items[pair[1]]);
            // The entries to keep apart are the first groups that only one of the two is in.
            let shared = a
                .chain
                .iter()
                .zip(&b.chain)
                .take_while(|(x, y)| x == y)
                .count();
            let (upper, upper_half) = match a.chain.get(shared) {
                Some(&g) => (bottom(g), 0.0),
                None => (pair[0], a.h / 2.0),
            };
            let (lower, lower_half) = match b.chain.get(shared) {
                Some(&g) => (top(g), 0.0),
                None => (pair[1], b.h / 2.0),
            };
            let both_items = shared == a.chain.len() && shared == b.chain.len();
            let gap = if both_items && (a.kind == Kind::Border || b.kind == Kind::Border) {
                0.0
            } else if both_items && (a.kind == Kind::Bend || b.kind == Kind::Bend) {
                BEND_GAP
            } else {
                NODE_GAP
            };
            limits.add(upper, lower, gap + upper_half + lower_half);
        }
    }

    // The first solution: every variable as high on the page as the constraints allow.
    // Seed: a thing starts at its earlier position, and moves down only when it is pushed.
    let mut at = vec![f64::NEG_INFINITY; vars];
    for (i, item) in net.items.iter().enumerate() {
        at[i] = item.seed.unwrap_or(f64::NEG_INFINITY);
    }
    let mut waiting: Vec<usize> = limits.low.iter().map(Vec::len).collect();
    let mut ready: Vec<usize> = (0..vars).filter(|&v| waiting[v] == 0).collect();
    let mut solved = Vec::new();
    while let Some(v) = ready.pop() {
        solved.push(v);
        for &(w, gap) in &limits.high[v] {
            at[w] = f64::max(at[w], at[v] + gap);
            waiting[w] -= 1;
            if waiting[w] == 0 {
                ready.push(w);
            }
        }
    }
    assert_eq!(
        solved.len(),
        vars,
        "the order of the groups differs between two layers"
    );
    // A variable that nothing pushed rests against what is below it.
    for &v in solved.iter().rev() {
        if at[v] == f64::NEG_INFINITY {
            at[v] = limits.high(v, &at).unwrap_or(0.0);
        }
    }

    // A thing wants the middle of the things it links to, so that a link can run straight.
    // A bend wants the height of the thing its link starts at, so that the link turns once.
    let mut linked: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut anchor: Vec<usize> = (0..n).collect();
    for path in &net.paths {
        if let (Some(&first), Some(&last)) = (path.first(), path.last()) {
            linked[first].push(last);
            linked[last].push(first);
            for &bend in &path[1..path.len() - 1] {
                anchor[bend] = first;
            }
        }
    }
    let mut by_depth: Vec<usize> = (0..groups).collect();
    by_depth.sort_by_key(|&g| graph.chain(Some(g)).len());

    for round in 0..PLACE_ROUNDS {
        // Open each group as far as its surroundings allow, so that its members can move.
        // The side that opens first alternates, so that two groups share the room between them.
        for side in [round % 2, 1 - round % 2] {
            for &g in &by_depth {
                if side == 0 {
                    at[top(g)] = limits.low(top(g), &at).unwrap_or(at[top(g)] - 500.0);
                } else {
                    at[bottom(g)] = limits.high(bottom(g), &at).unwrap_or(at[bottom(g)] + 500.0);
                }
            }
        }
        let mut sequence: Vec<usize> = net.layers.iter().flatten().copied().collect();
        if round % 2 == 1 {
            sequence.reverse();
        }
        for i in sequence {
            let item = &net.items[i];
            let wanted = if let Some(y) = item.seed {
                // Seed: a thing with an earlier position wants that position back.
                y
            } else if item.kind == Kind::Border {
                let g = item.chain[item.chain.len() - 1];
                let inside = |other: &&Item| other.kind == Kind::Thing && other.chain.contains(&g);
                let ys: Vec<f64> = (0..n)
                    .filter(|&j| inside(&&net.items[j]))
                    .map(|j| at[j])
                    .collect();
                ys.iter().sum::<f64>() / ys.len() as f64
            } else if item.kind == Kind::Bend {
                at[anchor[i]]
            } else if linked[i].is_empty() {
                at[i]
            } else {
                let mut ys: Vec<f64> = linked[i].iter().map(|&j| at[j]).collect();
                ys.sort_by(f64::total_cmp);
                (ys[(ys.len() - 1) / 2] + ys[ys.len() / 2]) / 2.0
            };
            let low = limits.low(i, &at).unwrap_or(f64::NEG_INFINITY);
            let high = limits.high(i, &at).unwrap_or(f64::INFINITY);
            at[i] = wanted.max(low).min(high);
        }
        // Close each group around its members again, innermost first.
        for &g in by_depth.iter().rev() {
            at[top(g)] = limits.high(top(g), &at).unwrap_or(at[top(g)]);
            at[bottom(g)] = limits.low(bottom(g), &at).unwrap_or(at[bottom(g)]);
        }
    }
    let tops = (0..groups).map(|g| at[top(g)]).collect();
    let bottoms = (0..groups).map(|g| at[bottom(g)]).collect();
    at.truncate(n);
    (at, tops, bottoms)
}

/// Sets the position of each rank and draws each link at right angles. A link turns only in
/// the gap between two ranks, where no box is.
fn route(net: &Net, graph: &Graph, y: &[f64], top: &[f64], bottom: &[f64]) -> Picture {
    let ranks = net.layers.len();
    // A group needs room for its border: one pad for itself and one for each group inside it.
    let mut level = vec![1.0; graph.groups.len()];
    for g in (0..graph.groups.len()).rev() {
        if let Some(parent) = graph.groups[g].parent {
            level[parent] = f64::max(level[parent], level[g] + 1.0);
        }
    }
    let mut ends = vec![0.0; ranks + 1];
    let mut starts = vec![0.0; ranks + 1];
    for (g, span) in net.span.iter().enumerate() {
        if let Some((low, high)) = *span {
            starts[low] = f64::max(starts[low], level[g] * PAD);
            ends[high] = f64::max(ends[high], level[g] * PAD);
        }
    }
    let width: Vec<f64> = net
        .layers
        .iter()
        .map(|layer| layer.iter().map(|&i| net.items[i].w).fold(0.0, f64::max))
        .collect();
    let mut left = vec![0.0; ranks];
    for rank in 1..ranks {
        left[rank] = left[rank - 1] + width[rank - 1] + ends[rank - 1] + RANK_GAP + starts[rank];
    }

    // Each bent step takes a track in its gap. Steps from one item share a track.
    let mut track: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for rank in 0..ranks.saturating_sub(1) {
        let mut sources: Vec<(usize, f64)> = Vec::new();
        for &(a, b) in net
            .steps
            .iter()
            .filter(|&&(a, _)| net.items[a].rank == rank)
        {
            if (y[a] - y[b]).abs() > 0.5 {
                match sources.iter_mut().find(|(source, _)| *source == a) {
                    Some((_, down)) => *down += y[b] - y[a],
                    None => sources.push((a, y[b] - y[a])),
                }
            }
        }
        // A step that goes down and starts lower turns earlier, and so for steps that go up.
        sources.sort_by(|p, q| {
            let (p_down, q_down) = (p.1 > 0.0, q.1 > 0.0);
            let by_y = if p_down {
                y[q.0].total_cmp(&y[p.0])
            } else {
                y[p.0].total_cmp(&y[q.0])
            };
            q_down.cmp(&p_down).then(by_y)
        });
        let from = left[rank] + width[rank] + ends[rank] + 8.0;
        let to = left[rank + 1] - starts[rank + 1] - 8.0;
        for (index, &(source, _)) in sources.iter().enumerate() {
            let share = (index + 1) as f64 / (sources.len() + 1) as f64;
            track.insert((rank, source), from + (to - from) * share);
        }
    }

    let x_of =
        |i: usize| left[net.items[i].rank] + (width[net.items[i].rank] - net.items[i].w) / 2.0;
    let mut edges = Vec::new();
    for (path, &reversed) in net.paths.iter().zip(&net.reversed) {
        let mut points = Vec::new();
        if let (Some(&first), Some(&last)) = (path.first(), path.last()) {
            points.push((x_of(first) + net.items[first].w, y[first]));
            for pair in path.windows(2) {
                if let Some(&x) = track.get(&(net.items[pair[0]].rank, pair[0]))
                    && (y[pair[0]] - y[pair[1]]).abs() > 0.5
                {
                    points.push((x, y[pair[0]]));
                    points.push((x, y[pair[1]]));
                }
            }
            points.push((x_of(last), y[last]));
        }
        if reversed {
            points.reverse();
        }
        edges.push(points);
    }

    let nodes = (0..graph.nodes.len())
        .map(|i| {
            let item = &net.items[i];
            Rect {
                x: x_of(i),
                y: y[i] - item.h / 2.0,
                w: item.w,
                h: item.h,
            }
        })
        .collect();
    let groups = net
        .span
        .iter()
        .enumerate()
        .map(|(g, span)| {
            span.map_or_else(Rect::default, |(low, high)| {
                let x = left[low] - level[g] * PAD;
                let right = left[high] + width[high] + level[g] * PAD;
                Rect {
                    x,
                    y: top[g],
                    w: right - x,
                    h: bottom[g] - top[g],
                }
            })
        })
        .collect();
    Picture {
        nodes,
        groups,
        edges,
    }
}
