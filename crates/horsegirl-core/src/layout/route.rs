//! The route phase: where each link turns.
//!
//! A link leaves the right side of a box and enters the left side of the
//! next. Between two ranks is a corridor that holds no box. A step that
//! changes height turns twice in the corridor, so every upright line is in a
//! corridor and no path crosses the box of a node. A corridor can be inside
//! a group, so an upright line crosses a group that its link has no end in
//! when the order phase put the link on two sides of the group.
//!
//! The steps that leave one item share one upright line, their trunk. Two
//! trunks whose heights overlap get different tracks, so no two upright lines
//! lie on each other. A level line must not lie on another either: when a
//! trunk leaves its item at the height where a second trunk arrives, the
//! first trunk gets the track nearer to its item, and it has turned off the
//! line before the second trunk joins.

use std::collections::BTreeMap;

use super::{BEND_GAP, Kind, Net, RANK_GAP, real};

/// Two level lines closer than this read as one line.
const CLOSE: f64 = 4.0;
/// The room between a label and each end of its corridor.
pub(super) const LABEL_PAD: f64 = 6.0;

/// Where the steps of [`Net::steps`] turn. Each list has one entry for each
/// step.
pub(super) struct Routes {
    /// The width of the corridor after each rank.
    pub gaps: Vec<f64>,
    /// The height where the step leaves its first item.
    pub from: Vec<f64>,
    /// The height where the step reaches its second item.
    pub into: Vec<f64>,
    /// The upright line of a step that changes height, as a distance from the
    /// middle of the corridor.
    pub track: Vec<Option<f64>>,
}

/// The upright line that the steps from one item share.
struct Trunk {
    /// The height where the trunk leaves its item.
    y: f64,
    /// The steps that change height.
    steps: Vec<usize>,
    top: f64,
    bottom: f64,
}

/// Routes every step. `labels` gives, for each step, the width of the label
/// that sits on it, or 0.
pub(super) fn route(net: &Net, centres: &[f64], labels: &[f64]) -> Routes {
    let count = net.steps.len();
    let mut routes = Routes {
        gaps: vec![RANK_GAP; net.layers.len()],
        from: vec![0.0; count],
        into: net.steps.iter().map(|&(_, to)| centres[to]).collect(),
        track: vec![None; count],
    };
    let mut by_rank = vec![Vec::new(); net.layers.len()];
    for (step, &(from, _)) in net.steps.iter().enumerate() {
        by_rank[net.items[from].rank].push(step);
    }
    // The height where a link leaves each item. A bend passes its link on at
    // the height where the link arrived.
    let mut leaves = centres.to_vec();
    let mut free = vec![false; count];
    for (rank, steps) in by_rank.iter().enumerate() {
        for &step in steps {
            routes.from[step] = leaves[net.steps[step].0];
        }
        untie(net, steps, &routes.from, &mut routes.into, &mut free);
        for &step in steps {
            let to = net.steps[step].1;
            if net.items[to].kind == Kind::Bend {
                leaves[to] = routes.into[step];
            }
        }
        let trunks = trunks(net, steps, &routes.from, &routes.into);
        let tracks = tracks(&trunks, &routes.into, &free);
        let used = tracks.iter().map(|track| track + 1).max().unwrap_or(0);
        for (trunk, &track) in trunks.iter().zip(&tracks) {
            let offset = (real(track) - real(used - 1) / 2.0) * BEND_GAP;
            for &step in &trunk.steps {
                routes.track[step] = Some(offset);
            }
        }
        // A label is centred on a track or on the middle of the corridor, so
        // the corridor holds the label beside the outermost track.
        let widest = steps.iter().map(|&step| labels[step]).fold(0.0, f64::max);
        let label_room = if widest > 0.0 {
            widest + 2.0 * LABEL_PAD + real(used.saturating_sub(1)) * BEND_GAP
        } else {
            0.0
        };
        routes.gaps[rank] = RANK_GAP.max(real(used + 1) * BEND_GAP).max(label_room);
    }
    routes
}

/// The trunks of one corridor, in the order of their items.
fn trunks(net: &Net, steps: &[usize], from: &[f64], into: &[f64]) -> Vec<Trunk> {
    let mut by_item: BTreeMap<usize, Trunk> = BTreeMap::new();
    for &step in steps {
        let (y, end) = (from[step], into[step]);
        if y - end == 0.0 {
            continue;
        }
        let trunk = by_item.entry(net.steps[step].0).or_insert(Trunk {
            y,
            steps: Vec::new(),
            top: y,
            bottom: y,
        });
        trunk.steps.push(step);
        trunk.top = trunk.top.min(end);
        trunk.bottom = trunk.bottom.max(end);
    }
    by_item.into_values().collect()
}

/// The step that makes `first` stand before `second`: a step of `second`
/// that arrives at the height where `first` leaves its item. If `second`
/// stood first, that step would lie on the line of `first`, and the picture
/// would show a link from the item of `first` to the end of the step.
fn before(first: &Trunk, second: &Trunk, into: &[f64], free: &[bool]) -> Option<usize> {
    let on_line = |&step: &usize| !free[step] && (into[step] - first.y).abs() < CLOSE;
    second.steps.iter().copied().find(on_line)
}

/// Moves arrivals until the trunks of one corridor have an order.
///
/// Two trunks can each arrive at the height where the other leaves, and then
/// no order keeps both lines apart. One step of such a loop then arrives a
/// few pixels from the middle of its box, at a height that no trunk leaves
/// from. A step that was moved, or that has no such height, is `free`: it
/// orders no trunk, so each round ends one loop and the rounds end.
fn untie(net: &Net, steps: &[usize], from: &[f64], into: &mut [f64], free: &mut [bool]) {
    loop {
        let trunks = trunks(net, steps, from, into);
        let Some(step) = tied(&trunks, into, free) else {
            return;
        };
        free[step] = true;
        let item = &net.items[net.steps[step].1];
        // An arrival stays inside its box, and a bend stays clear of the
        // bend beside it.
        let (unit, reach) = match item.kind {
            Kind::Node => (BEND_GAP / 2.0, item.height / 2.0 - CLOSE),
            Kind::Bend | Kind::Marker => (BEND_GAP / 2.0 - 1.0, BEND_GAP / 2.0 - 1.0),
        };
        // The side that the step comes from is the first choice.
        let side = if from[step] < into[step] { -1.0 } else { 1.0 };
        let clear = |y: &f64| trunks.iter().all(|trunk| (trunk.y - y).abs() >= CLOSE);
        let moved = [1.0, -1.0, 2.0, -2.0]
            .iter()
            .map(|times| times * side * unit)
            .filter(|by| by.abs() <= reach)
            .map(|by| into[step] + by)
            .find(clear);
        if let Some(y) = moved {
            into[step] = y;
        }
    }
}

/// A step on a loop of [`before`], if the trunks have one.
fn tied(trunks: &[Trunk], into: &[f64], free: &[bool]) -> Option<usize> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        New,
        Open,
        Done,
    }
    fn visit(
        first: usize,
        trunks: &[Trunk],
        into: &[f64],
        free: &[bool],
        state: &mut [State],
    ) -> Option<usize> {
        state[first] = State::Open;
        for second in (0..trunks.len()).filter(|&second| second != first) {
            let Some(step) = before(&trunks[first], &trunks[second], into, free) else {
                continue;
            };
            match state[second] {
                State::Open => return Some(step),
                State::New => {
                    if let Some(step) = visit(second, trunks, into, free, state) {
                        return Some(step);
                    }
                }
                State::Done => {}
            }
        }
        state[first] = State::Done;
        None
    }
    let mut state = vec![State::New; trunks.len()];
    (0..trunks.len()).find_map(|trunk| {
        (state[trunk] == State::New)
            .then(|| visit(trunk, trunks, into, free, &mut state))
            .flatten()
    })
}

/// The track of each trunk, counted from the rank that the trunks leave.
///
/// The trunks are put in a row one at a time. The next trunk is one that no
/// waiting trunk must stand before, and among those the one that crosses the
/// fewest lines when it stands before the rest. A trunk then takes the first
/// track after every trunk of the row that is beside it.
fn tracks(trunks: &[Trunk], into: &[f64], free: &[bool]) -> Vec<usize> {
    let within = |y: f64, trunk: &Trunk| trunk.top < y && y < trunk.bottom;
    // The crossings of two trunks when `left` has the lower track: the line
    // from the item of `right` crosses `left`, and each line from `left` to
    // an end crosses `right`.
    let crossings = |left: &Trunk, right: &Trunk| {
        let ends = left.steps.iter().filter(|&&step| within(into[step], right));
        real(usize::from(within(right.y, left)) + ends.count())
    };
    let beside = |a: &Trunk, b: &Trunk| a.top - BEND_GAP < b.bottom && b.top - BEND_GAP < a.bottom;

    let mut track = vec![0; trunks.len()];
    let mut row: Vec<usize> = Vec::new();
    let mut waiting: Vec<usize> = (0..trunks.len()).collect();
    while !waiting.is_empty() {
        let key = |&trunk: &usize| {
            let rest = || waiting.iter().filter(move |&&other| other != trunk);
            let held = rest()
                .filter(|&&other| before(&trunks[other], &trunks[trunk], into, free).is_some())
                .count();
            let cost: f64 = rest()
                .map(|&other| {
                    crossings(&trunks[trunk], &trunks[other])
                        - crossings(&trunks[other], &trunks[trunk])
                })
                .sum();
            (held, cost, trunks[trunk].y)
        };
        let keys: Vec<(usize, f64, f64)> = waiting.iter().map(key).collect();
        let next = (0..waiting.len())
            .min_by(|&a, &b| {
                let ((held_a, cost_a, y_a), (held_b, cost_b, y_b)) = (keys[a], keys[b]);
                held_a
                    .cmp(&held_b)
                    .then(cost_a.total_cmp(&cost_b))
                    .then(y_a.total_cmp(&y_b))
            })
            .expect("a trunk is waiting");
        let trunk = waiting.remove(next);
        track[trunk] = row
            .iter()
            .filter(|&&placed| beside(&trunks[placed], &trunks[trunk]))
            .map(|&placed| track[placed] + 1)
            .max()
            .unwrap_or(0);
        row.push(trunk);
    }
    track
}
