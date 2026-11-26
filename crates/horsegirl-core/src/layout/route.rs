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
//!
//! A label is centred on its path. Two labels of one corridor must not cover
//! each other, and a line of another step must not cross a label. A label
//! first takes the middle of its upright line, or the middle of the corridor
//! when its step is level. When a label or a line is in the way there, the
//! label moves along its upright line, or onto one of the two level lines of
//! its step, to the nearest place that covers no label and that no line
//! crosses. The place also keeps out of the name band and off the borders of
//! the groups that span the corridor. A place on a level line can be past
//! the end of the corridor: the corridor then grows to hold the label.

use std::collections::{BTreeMap, BTreeSet};

use super::{BEND_GAP, Kind, LABEL_CLEAR, Net, RANK_GAP, real};
use crate::scene::{HEAD_LENGTH, LABEL_CUT, Point};
use crate::text::Size;

/// Two level lines closer than this read as one line.
const CLOSE: f64 = 4.0;
/// The room between a label and each end of its corridor: the arrow head
/// that ends there, and the cut of the line before the label.
pub(super) const LABEL_PAD: f64 = HEAD_LENGTH + LABEL_CUT;
/// The least room between two labels of one corridor.
const LABEL_SPACE: f64 = 4.0;
/// The least room between a label and a line of another link. The SVG
/// backend stops the line of the label's own link this far from it.
const LINE_SPACE: f64 = 3.0;

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
    /// The centre of the label of a step that holds one. Its `x` is a
    /// distance from the middle of the corridor, like a track.
    pub labels: Vec<Option<Point>>,
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

/// The heights that a label keeps out of, besides lines and other labels.
/// Each is a top and a bottom.
#[derive(Clone, Debug, Default)]
pub(super) struct Bands {
    /// Each border of a group that spans the corridor. The top border is the
    /// band under it that holds the name of the group.
    pub borders: Vec<(f64, f64)>,
    /// Each group that spans the corridor and holds neither end of the link.
    /// A label there reads as a label of that group.
    pub others: Vec<(f64, f64)>,
}

/// Routes every step. `labels` gives, for each step, the size of the label
/// that sits on it, and `bands` the heights that the label keeps out of.
pub(super) fn route(
    net: &Net,
    centres: &[f64],
    labels: &[Option<Size>],
    bands: &[Bands],
) -> Routes {
    let count = net.steps.len();
    let mut routes = Routes {
        gaps: vec![RANK_GAP; net.layers.len()],
        from: vec![0.0; count],
        into: net.steps.iter().map(|&(_, to)| centres[to]).collect(),
        track: vec![None; count],
        labels: vec![None; count],
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
        let widest = steps
            .iter()
            .filter_map(|&step| labels[step])
            .map(|size| size.width)
            .fold(0.0, f64::max);
        let label_room = if widest > 0.0 {
            widest + 2.0 * LABEL_PAD + real(used.saturating_sub(1)) * BEND_GAP
        } else {
            0.0
        };
        let mut gap = RANK_GAP.max(real(used + 1) * BEND_GAP).max(label_room);
        let mut placed: Vec<(Point, Size)> = Vec::new();
        let at = |step: usize| (routes.from[step], routes.into[step], routes.track[step]);
        for &step in steps {
            let Some(size) = labels[step] else { continue };
            let others: Vec<Line> = steps
                .iter()
                .filter(|&&other| other != step)
                .flat_map(|&other| lines(at(other)))
                .collect();
            let centre = place_label(at(step), size, (&placed, &others, &bands[step]), gap);
            gap = gap.max(2.0 * (centre.x.abs() + size.width / 2.0 + LABEL_PAD));
            placed.push((centre, size));
            routes.labels[step] = Some(centre);
        }
        routes.gaps[rank] = gap;
    }
    routes
}

/// A straight part of a path in one corridor. A level part is at the height
/// `at`, from `low` to `high` across the corridor. An upright part is at the
/// distance `at` from the middle of the corridor, from `low` to `high` down
/// the page. A level part that reaches an end of the corridor goes on to an
/// item, so its end there is infinite.
#[derive(Clone, Copy, Debug)]
struct Line {
    level: bool,
    at: f64,
    low: f64,
    high: f64,
}

impl Line {
    /// The line comes nearer than `reach` to a label at `centre`. `reach`
    /// is half the width and half the height of the label, each with the
    /// room it keeps from a line.
    fn crosses(&self, centre: Point, reach: (f64, f64)) -> bool {
        #[cfg(test)]
        tests::count_test();
        let point = Line {
            level: true,
            at: centre.y,
            low: centre.x,
            high: centre.x,
        };
        self.near(&point, reach)
    }

    /// The line comes nearer than `reach` to a label centred anywhere on
    /// `path`.
    fn near(&self, path: &Line, reach: (f64, f64)) -> bool {
        let ((left, right), (top, bottom)) = self.extent();
        let ((path_left, path_right), (path_top, path_bottom)) = path.extent();
        left < path_right + reach.0
            && path_left - reach.0 < right
            && top < path_bottom + reach.1
            && path_top - reach.1 < bottom
    }

    /// The range of the line across the corridor and down the page.
    fn extent(&self) -> ((f64, f64), (f64, f64)) {
        if self.level {
            ((self.low, self.high), (self.at, self.at))
        } else {
            ((self.at, self.at), (self.low, self.high))
        }
    }
}

/// The parts of the path of a step in its corridor. `step` is the height
/// where the step leaves, the height where it arrives and its track.
fn lines((from, into, track): (f64, f64, Option<f64>)) -> Vec<Line> {
    let level = |at: f64, low: f64, high: f64| Line {
        level: true,
        at,
        low,
        high,
    };
    match track {
        Some(x) => vec![
            level(from, f64::NEG_INFINITY, x),
            Line {
                level: false,
                at: x,
                low: from.min(into),
                high: from.max(into),
            },
            level(into, x, f64::INFINITY),
        ],
        None => vec![level(from, f64::NEG_INFINITY, f64::INFINITY)],
    }
}

/// The parts of `others` that do not lie on `own`. Two links from one item
/// share the line that leaves it, and a label on that line is on its own
/// link, so the shared part is no obstacle to it.
fn apart(others: &[Line], own: &[Line]) -> Vec<Line> {
    let mut pieces: Vec<Line> = others.to_vec();
    for mine in own {
        pieces = pieces
            .into_iter()
            .flat_map(|line| {
                let on = line.level == mine.level && line.at - mine.at == 0.0;
                if !on || mine.high <= line.low || line.high <= mine.low {
                    return vec![line];
                }
                let before = Line {
                    high: mine.low,
                    ..line
                };
                let after = Line {
                    low: mine.high,
                    ..line
                };
                [before, after]
                    .into_iter()
                    .filter(|piece| piece.low < piece.high)
                    .collect()
            })
            .collect();
    }
    pieces
}

/// The centre of a label that covers none of the labels `placed` in its
/// corridor and that no line of another step crosses. `step` is the height
/// where the step leaves, the height where it arrives and its track,
/// `others` holds the lines of the other steps of the corridor, and `bands`
/// the heights of the borders and of the groups that the label keeps out of.
/// The x of each centre is a distance from the middle of the corridor, and
/// the corridor is `gap` wide.
///
/// The first choice is the middle of the upright line, or of the corridor
/// for a level step. Then each place where the label stands beside a placed
/// label, beside a line or beside a band: on the upright line above or
/// under it, or on a level line to its left or right. A place across no
/// border comes first, then a place outside every other group, then a place
/// that no line crosses, then a place that keeps the corridor as wide as it
/// is, and then the nearer place. A place on a level line right of every
/// placed label always covers none, so a place is always found. Some steps
/// have no place that no line crosses: a line of another step can run close
/// beside the whole path. The label then takes the place that the fewest
/// lines cross.
fn place_label(
    step: (f64, f64, Option<f64>),
    size: Size,
    (placed, others, bands): (&[(Point, Size)], &[Line], &Bands),
    gap: f64,
) -> Point {
    let (from, into, track) = step;
    let first = Point {
        x: track.unwrap_or(0.0),
        y: f64::midpoint(from, into),
    };
    // Every place that the label can take is on its own path, so a line that
    // stays out of reach of the path can never cross the label. Most lines of
    // a busy corridor are of that kind, and each one would add places to try
    // and work to each place.
    let own = lines(step);
    let mut path = own.clone();
    path.push(Line {
        level: true,
        at: first.y,
        low: first.x,
        high: first.x,
    });
    let reach = Obstacles::reach(size);
    let others: Vec<Line> = apart(others, &own)
        .into_iter()
        .filter(|line| path.iter().any(|part| line.near(part, reach)))
        .collect();
    let obstacles = Obstacles::new(size, placed, &others);
    let half = size.height / 2.0;
    let within = |bands: &[(f64, f64)], at: &Point| {
        let mut bands = bands.iter();
        bands.any(|&(top, bottom)| top - half < at.y && at.y < bottom + half)
    };
    let in_bands = |at: &Point| (within(&bands.borders, at), within(&bands.others, at));
    if obstacles.clear(&first)
        && in_bands(&first) == (false, false)
        && obstacles.crossed(&first, 1) == 0
    {
        return first;
    }
    let mut spots = vec![first];
    // The upright line, from the height where the step leaves to the height
    // where it arrives.
    if let Some(x) = track {
        let (top, bottom) = (from.min(into), from.max(into));
        spots.extend(
            [top, bottom]
                .into_iter()
                .chain(heights(size, placed, &others, bands))
                .filter(|y| top <= *y && *y <= bottom)
                .map(|y| Point { x, y }),
        );
    }
    // The level line at the height where the step leaves runs from the left
    // end of the corridor to the track, and the line at the height where it
    // arrives from the track to the right end. A level step has one line
    // across the whole corridor.
    let (left, right) = match track {
        Some(x) => ((from, f64::NEG_INFINITY, x), (into, x, f64::INFINITY)),
        None => (
            (from, f64::NEG_INFINITY, f64::INFINITY),
            (from, f64::NEG_INFINITY, f64::INFINITY),
        ),
    };
    let mut xs = places_across(size, placed, &others);
    xs.push(first.x);
    for (y, low, high) in [left, right] {
        for &x in xs.iter().filter(|x| x.is_finite()) {
            spots.push(Point {
                x: x.clamp(low, high),
                y,
            });
        }
    }
    // The ends of many lines clamp to one corner of the path. A spot after
    // the first at the same place has the same cost, so it can never win.
    let mut seen = BTreeSet::new();
    spots.retain(|at| seen.insert((at.x.to_bits(), at.y.to_bits())));
    let ranked = spots
        .into_iter()
        .map(|at| {
            let grows = 2.0 * (at.x.abs() + size.width / 2.0 + LABEL_PAD) - gap;
            let far = (at.x - first.x).abs() + (at.y - first.y).abs();
            (at, (in_bands(&at), grows.max(0.0), far))
        })
        .collect();
    choose(ranked, &obstacles)
}

/// The bands that a spot is in, how much it widens the corridor and how far
/// it is from the first choice.
type Rank = ((bool, bool), f64, f64);

/// The best of `spots`: the clear spot of the least [`Rank`] whose bands no
/// line crosses, or else the one of those bands that the fewest lines cross.
///
/// Each spot is sorted by its rank first, which leaves out the count of the
/// lines that cross it: that count is the dear one to find. The first clear
/// spot that no line crosses is then the best of its bands, and only when
/// every spot of those bands is crossed must the lines of each be counted.
fn choose(mut spots: Vec<(Point, Rank)>, obstacles: &Obstacles) -> Point {
    spots.sort_by(
        |(_, (bands_a, grow_a, far_a)), (_, (bands_b, grow_b, far_b))| {
            bands_a
                .cmp(bands_b)
                .then(grow_a.total_cmp(grow_b))
                .then(far_a.total_cmp(far_b))
        },
    );
    let mut open = spots.into_iter().filter(|(at, _)| obstacles.clear(at));
    let (best, (bands_of_best, _, _)) = open
        .next()
        .expect("a place right of every label covers none");
    let mut crossed = Vec::new();
    let same_bands = open.take_while(|(_, (bands, _, _))| *bands == bands_of_best);
    for at in std::iter::once(best).chain(same_bands.map(|(at, _)| at)) {
        if obstacles.crossed(&at, 1) == 0 {
            return at;
        }
        crossed.push(at);
    }
    let mut best = crossed[0];
    let mut fewest = obstacles.crossed(&best, usize::MAX);
    for &at in &crossed[1..] {
        let count = obstacles.crossed(&at, fewest);
        if count < fewest {
            (best, fewest) = (at, count);
        }
    }
    best
}

/// The placed labels and the lines that a label of one size can meet, each
/// sorted so that a place is tested only against those near it. Each window
/// is a pixel wider than the reach, so rounding cannot leave out a label or
/// a line in reach.
struct Obstacles<'a> {
    size: Size,
    /// The placed labels by height.
    labels: Vec<&'a (Point, Size)>,
    /// How far from a place in height a placed label can cover it.
    window: f64,
    /// The level lines by height.
    level: Vec<Line>,
    /// The upright lines by distance across.
    upright: Vec<Line>,
}

impl<'a> Obstacles<'a> {
    fn new(size: Size, placed: &'a [(Point, Size)], lines: &[Line]) -> Self {
        let mut labels: Vec<&(Point, Size)> = placed.iter().collect();
        labels.sort_by(|(a, _), (b, _)| a.y.total_cmp(&b.y));
        let tallest = placed.iter().map(|(_, s)| s.height).fold(0.0, f64::max);
        let (mut level, mut upright): (Vec<Line>, Vec<Line>) =
            lines.iter().partition(|line| line.level);
        level.sort_by(|a, b| a.at.total_cmp(&b.at));
        upright.sort_by(|a, b| a.at.total_cmp(&b.at));
        Self {
            size,
            labels,
            window: f64::midpoint(size.height, tallest) + LABEL_SPACE + 1.0,
            level,
            upright,
        }
    }

    /// Half the width and half the height of a label of `size`, each with
    /// the room that it keeps from a line.
    fn reach(size: Size) -> (f64, f64) {
        (
            size.width / 2.0 + LINE_SPACE,
            size.height / 2.0 + LINE_SPACE,
        )
    }

    /// A label at `at` covers none of the placed labels.
    fn clear(&self, at: &Point) -> bool {
        let start = self
            .labels
            .partition_point(|(other, _)| other.y <= at.y - self.window);
        let near = self.labels[start..]
            .iter()
            .take_while(|(other, _)| other.y < at.y + self.window);
        near.into_iter().all(|(other, other_size)| {
            #[cfg(test)]
            tests::count_test();
            let wide = f64::midpoint(self.size.width, other_size.width) + LABEL_SPACE;
            let high = f64::midpoint(self.size.height, other_size.height) + LABEL_SPACE;
            (at.x - other.x).abs() >= wide || (at.y - other.y).abs() >= high
        })
    }

    /// The count of the lines that cross a label at `at`, up to `limit`.
    fn crossed(&self, at: &Point, limit: usize) -> usize {
        let reach = Self::reach(self.size);
        near(&self.level, at.y, reach.1)
            .chain(near(&self.upright, at.x, reach.0))
            .filter(|line| line.crosses(*at, reach))
            .take(limit)
            .count()
    }
}

/// The lines of `lines`, in the order of `at`, whose `at` is within `reach`
/// of `at`, and maybe a few more.
fn near(lines: &[Line], at: f64, reach: f64) -> impl Iterator<Item = &Line> {
    let start = lines.partition_point(|line| line.at <= at - reach - 1.0);
    lines[start..]
        .iter()
        .take_while(move |line| line.at < at + reach + 1.0)
}

/// The heights where a label of `size` stands just above or under a placed
/// label, a band or a line.
fn heights(size: Size, placed: &[(Point, Size)], others: &[Line], bands: &Bands) -> Vec<f64> {
    let half = size.height / 2.0;
    let mut ys = Vec::new();
    for (other, other_size) in placed {
        let reach = f64::midpoint(size.height, other_size.height) + LABEL_SPACE;
        ys.extend([other.y - reach, other.y + reach]);
    }
    for &(top, bottom) in bands.borders.iter().chain(&bands.others) {
        ys.extend([top - half - LABEL_CLEAR, bottom + half + LABEL_CLEAR]);
    }
    let reach = half + LINE_SPACE;
    for line in others {
        let (top, bottom) = if line.level {
            (line.at, line.at)
        } else {
            (line.low, line.high)
        };
        ys.extend([top - reach, bottom + reach]);
    }
    ys
}

/// The distances from the middle of the corridor where a label of `size`
/// stands just left or right of a placed label or of a line that ends in
/// the corridor, and past all of them on each side.
fn places_across(size: Size, placed: &[(Point, Size)], others: &[Line]) -> Vec<f64> {
    let ends = others.iter().flat_map(|line| {
        if line.level {
            [line.low, line.high]
        } else {
            [line.at, line.at]
        }
    });
    let ends: Vec<f64> = ends.filter(|end| end.is_finite()).collect();
    let half = size.width / 2.0;
    let rightmost = placed
        .iter()
        .map(|(other, other_size)| other.x + other_size.width / 2.0 + LABEL_SPACE)
        .chain(ends.iter().map(|end| end + LINE_SPACE))
        .fold(f64::NEG_INFINITY, f64::max);
    let leftmost = placed
        .iter()
        .map(|(other, other_size)| other.x - other_size.width / 2.0 - LABEL_SPACE)
        .chain(ends.iter().map(|end| end - LINE_SPACE))
        .fold(f64::INFINITY, f64::min);
    let mut xs = vec![rightmost + half, leftmost - half];
    for (other, other_size) in placed {
        let reach = f64::midpoint(size.width, other_size.width) + LABEL_SPACE;
        xs.extend([other.x - reach, other.x + reach]);
    }
    let reach = half + LINE_SPACE;
    xs.extend(ends.iter().flat_map(|end| [end - reach, end + reach]));
    xs
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
/// fewest lines when it stands before the rest. Two neighbours of the row
/// then exchange their places where that removes a crossing. A trunk then
/// takes the first track after every trunk of the row that is beside it.
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
        row.push(waiting.remove(next));
    }
    // The choice above weighs a trunk against every waiting trunk at once,
    // so two neighbours of the row can stand the wrong way round. They
    // exchange their places where that removes a crossing and no step ties
    // them. Each exchange removes a crossing, so the loop ends.
    let mut improved = true;
    while improved {
        improved = false;
        for at in 1..row.len() {
            let (left, right) = (&trunks[row[at - 1]], &trunks[row[at]]);
            if before(left, right, into, free).is_none()
                && crossings(right, left) < crossings(left, right)
            {
                row.swap(at - 1, at);
                improved = true;
            }
        }
    }
    let mut track = vec![0; trunks.len()];
    for (at, &trunk) in row.iter().enumerate() {
        track[trunk] = row[..at]
            .iter()
            .filter(|&&placed| beside(&trunks[placed], &trunks[trunk]))
            .map(|&placed| track[placed] + 1)
            .max()
            .unwrap_or(0);
    }
    track
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fmt::Write as _;

    use crate::lens::{self, FLOW};
    use crate::model::lower;
    use crate::scene::Role;
    use crate::syntax::parse;
    use crate::text::Style;

    thread_local! {
        /// How often a place of a label was tested against a line or against
        /// a placed label.
        static TESTS: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn count_test() {
        TESTS.with(|tests| tests.set(tests.get() + 1));
    }

    /// The tests to place the labels of a fan of `count` labelled links out
    /// of one thing and `count` labelled links into another, drawn as
    /// `horsegirl draw` draws it.
    fn tests_for_fan(count: usize) -> usize {
        let mut source = String::new();
        for i in 0..count {
            writeln!(source, "hub -> t{i}: label {i}\nt{i} -> sink: back {i}")
                .expect("a String takes any write");
        }
        let model = lower(&parse(&source).tree);
        TESTS.with(|tests| tests.set(0));
        let scene = lens::draw(&model, &FLOW, Style { size: 14.0 });
        let labels = scene.shapes.iter();
        let labels = labels.filter(|s| s.role == Role::Label && s.of.len() == 2);
        assert_eq!(labels.count(), 2 * count);
        TESTS.with(Cell::get)
    }

    // In a fan, each label has the lines and the labels of the whole fan
    // near its path. Each place that it tried was once tested against each
    // of them, so the work grew with the cube of the fan: 838 million tests
    // for a fan of 200. The test counts that work, not a clock, so a slow
    // runner cannot fail it.
    #[test]
    fn a_wide_labelled_fan_places_its_labels_in_little_work() {
        let tests = tests_for_fan(200);
        assert!(tests < 3 * 200 * 200, "a fan of 200 took {tests} tests");
    }
}
