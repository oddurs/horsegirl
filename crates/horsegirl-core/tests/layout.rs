//! The layered layout on the reference file and on the ten flow diagrams of
//! SPIKE-18, which are in `spikes/layered-layout/corpus`.
//!
//! `layout.positions` holds every position, path and label that the layout
//! gives for those eleven files. CI runs this test on Linux, on macOS and in WebAssembly, so
//! the file also proves that each target gives the same numbers. To accept a
//! change, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test layout
//! ```
//!
//! Read the diff before you commit it.

use std::fmt::Write as _;

use horsegirl_core::layout::{Graph, Group, LABEL_CLEAR, Layout, NODE_GAP, Node, RANK_GAP, layout};
use horsegirl_core::model::{Link, Model, Thing, lower};
use horsegirl_core::overlap::{Kind, overlaps};
use horsegirl_core::scene::{Placement, Point, Rect, Scene, build};
use horsegirl_core::syntax::parse;
use horsegirl_core::text::{Style, measure};

const STYLE: Style = Style { size: 14.0 };

const POSITIONS: &str = include_str!("layout.positions");

macro_rules! spike {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                "../../../spikes/layered-layout/corpus/",
                $name,
                ".horse"
            )),
        )
    };
}

/// The reference file, then the ten diagrams of the spike.
const DIAGRAMS: [(&str, &str); 11] = [
    ("checkout", include_str!("../../../examples/checkout.horse")),
    spike!("01-checkout"),
    spike!("02-ci-pipeline"),
    spike!("03-order-states"),
    spike!("04-data-pipeline"),
    spike!("05-sign-in"),
    spike!("06-cluster-request"),
    spike!("07-compiler"),
    spike!("08-module-deps"),
    spike!("09-incident"),
    spike!("10-platform"),
];

/// A file as the layout reads it, with the name of each node and group.
struct Flow {
    nodes: Vec<String>,
    groups: Vec<String>,
    graph: Graph,
    model: Model,
    /// The thing of each node, of each group and the model link of each link
    /// of the graph.
    node_ids: Vec<usize>,
    group_ids: Vec<usize>,
    link_ids: Vec<usize>,
}

/// Reads a file into the graph of its flow.
fn flow(source: &str) -> Flow {
    flow_of(lower(&parse(source).tree))
}

/// The graph of the flow of a model.
///
/// A thing that holds something is a group, and every other thing is a node
/// as wide as its name. A link that starts or ends at a group is left out:
/// item 0123 draws it.
fn flow_of(model: Model) -> Flow {
    let group_ids = model.groups();
    let group_of = |thing: usize| group_ids.iter().position(|&g| g == thing);
    let node_ids: Vec<usize> = (0..model.things.len())
        .filter(|&thing| group_of(thing).is_none())
        .collect();
    let node_of = |thing: usize| node_ids.iter().position(|&n| n == thing);
    let parent = |thing: usize| model.things[thing].parent.and_then(group_of);
    let links: Vec<(usize, (usize, usize))> = model
        .links
        .iter()
        .enumerate()
        .filter_map(|(id, link)| Some(id).zip(node_of(link.from).zip(node_of(link.to))))
        .collect();
    let label = |&(id, _): &(usize, _)| {
        let labels = &model.links[id].labels;
        (!labels.is_empty()).then(|| measure(&labels.join(", "), STYLE))
    };
    let graph = Graph {
        nodes: node_ids
            .iter()
            .map(|&thing| Node {
                width: measure(&model.things[thing].name, STYLE).width + 28.0,
                height: 36.0,
                group: parent(thing),
            })
            .collect(),
        groups: group_ids
            .iter()
            .map(|&thing| Group {
                parent: parent(thing),
            })
            .collect(),
        labels: links.iter().map(label).collect(),
        links: links.iter().map(|&(_, ends)| ends).collect(),
    };
    let name = |&thing: &usize| model.things[thing].name.clone();
    Flow {
        nodes: node_ids.iter().map(name).collect(),
        groups: group_ids.iter().map(name).collect(),
        graph,
        link_ids: links.iter().map(|&(id, _)| id).collect(),
        node_ids,
        group_ids,
        model,
    }
}

/// The scene of a picture: what a backend draws and the overlap detector
/// reads.
fn scene(flow: &Flow, picture: &Layout) -> Scene {
    let mut placement = Placement {
        things: vec![None; flow.model.things.len()],
        links: vec![Vec::new(); flow.model.links.len()],
        labels: vec![None; flow.model.links.len()],
    };
    for (&thing, node) in flow.node_ids.iter().zip(&picture.nodes) {
        placement.things[thing] = Some(*node);
    }
    for (&thing, group) in flow.group_ids.iter().zip(&picture.groups) {
        placement.things[thing] = *group;
    }
    for (link, &id) in flow.link_ids.iter().enumerate() {
        placement.links[id].clone_from(&picture.paths[link]);
        placement.labels[id] = picture.labels[link];
    }
    build(&flow.model, &placement, STYLE)
}

/// Every collision that the overlap detector finds in a picture, in words,
/// but for the kinds in `allowed`.
fn collisions(flow: &Flow, picture: &Layout, allowed: &[Kind]) -> Vec<String> {
    let scene = scene(flow, picture);
    let found = overlaps(&scene);
    let counted = found
        .iter()
        .filter(|overlap| !allowed.contains(&overlap.kind));
    counted.map(|overlap| overlap.describe(&scene)).collect()
}

/// How many links pass through a group that holds neither of their ends.
fn through_groups(flow: &Flow, picture: &Layout) -> usize {
    let found = overlaps(&scene(flow, picture));
    let kinds = found.iter().map(|overlap| overlap.kind);
    kinds.filter(|&kind| kind == Kind::LinkThroughGroup).count()
}

/// The file with a label on every link.
fn labelled(source: &str) -> Flow {
    let mut model = lower(&parse(source).tree);
    for link in &mut model.links {
        if link.labels.is_empty() {
            link.labels
                .push(format!("to {}", model.things[link.to].name));
        }
    }
    flow_of(model)
}

/// The room between two boxes on the axis where they are further apart. It
/// is negative when the boxes overlap.
fn room(a: &Rect, b: &Rect) -> f64 {
    let dx = f64::max(b.x - (a.x + a.width), a.x - (b.x + b.width));
    let dy = f64::max(b.y - (a.y + a.height), a.y - (b.y + b.height));
    dx.max(dy)
}

fn holds(outer: &Rect, inner: &Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// The groups around a group, with the group.
fn around(graph: &Graph, group: Option<usize>) -> Vec<usize> {
    let mut chain = Vec::new();
    let mut at = group;
    while let Some(g) = at {
        chain.push(g);
        at = graph.groups[g].parent;
    }
    chain
}

/// Everything that is wrong with a picture, one line for each fault.
///
/// A sum of gaps and sizes can lose its last bit, so a gap may be short by a
/// millionth of a pixel.
fn faults(graph: &Graph, picture: &Layout) -> Vec<String> {
    let near = NODE_GAP - 1e-6;
    let mut faults = Vec::new();
    for (i, a) in picture.nodes.iter().enumerate() {
        for (j, b) in picture.nodes.iter().enumerate().skip(i + 1) {
            if room(a, b) < near {
                faults.push(format!("nodes {i} and {j} are {} apart", room(a, b)));
            }
        }
    }
    for (i, node) in graph.nodes.iter().enumerate() {
        let chain = around(graph, node.group);
        for (g, group) in picture.groups.iter().enumerate() {
            let Some(group) = group else { continue };
            let r = &picture.nodes[i];
            if chain.contains(&g) && !holds(group, r) {
                faults.push(format!("node {i} is outside its group {g}"));
            }
            if !chain.contains(&g) && room(group, r) < near {
                faults.push(format!("node {i} is {} from group {g}", room(group, r)));
            }
        }
    }
    for (g, a) in picture.groups.iter().enumerate() {
        for (h, b) in picture.groups.iter().enumerate().skip(g + 1) {
            let (Some(a), Some(b)) = (a, b) else { continue };
            let nested = around(graph, Some(g)).contains(&h) || around(graph, Some(h)).contains(&g);
            if nested {
                let (outer, inner) = if around(graph, Some(g)).contains(&h) {
                    (b, a)
                } else {
                    (a, b)
                };
                if !holds(outer, inner) {
                    faults.push(format!("groups {g} and {h} are not one inside the other"));
                }
            } else if room(a, b) < near {
                faults.push(format!("groups {g} and {h} are {} apart", room(a, b)));
            }
        }
    }
    let boxes = picture.nodes.iter().chain(picture.groups.iter().flatten());
    let corners = boxes.map(|r| (r.x, r.y));
    let points = picture.bends.iter().chain(&picture.paths).flatten();
    let points = points.map(|p| (p.x, p.y));
    let (x, y) = corners
        .chain(points)
        .fold((f64::INFINITY, f64::INFINITY), |a, b| {
            (a.0.min(b.0), a.1.min(b.1))
        });
    if !picture.nodes.is_empty() && (x, y) != (0.0, 0.0) {
        faults.push(format!("the top left corner is at {x}, {y}"));
    }
    for (&(from, to), bends) in graph.links.iter().zip(&picture.bends) {
        let (low, high) = (picture.ranks[from], picture.ranks[to]);
        if from != to && low == high {
            faults.push(format!("link {from} -> {to} stays in rank {low}"));
        }
        if bends.len() + 1 != low.abs_diff(high).max(1) {
            faults.push(format!("link {from} -> {to} has {} bends", bends.len()));
        }
    }
    for (link, &(from, to)) in graph.links.iter().enumerate() {
        let path = &picture.paths[link];
        let (Some(start), Some(end)) = (path.first(), path.last()) else {
            faults.push(format!("link {from} -> {to} has no path"));
            continue;
        };
        if path
            .windows(2)
            .any(|pair| (pair[0].x - pair[1].x) * (pair[0].y - pair[1].y) != 0.0)
        {
            faults.push(format!("link {from} -> {to} has a slanted part: {path:?}"));
        }
        if !on_border(start, &picture.nodes[from]) || !on_border(end, &picture.nodes[to]) {
            faults.push(format!(
                "link {from} -> {to} does not join its boxes: {path:?}"
            ));
        }
        let labelled = graph.labels.get(link).copied().flatten().is_some();
        if labelled != picture.labels[link].is_some() {
            faults.push(format!("link {from} -> {to} has a label: {}", !labelled));
        }
    }
    faults
}

/// A point is on the left or the right side of a box.
fn on_border(point: &Point, r: &Rect) -> bool {
    let on_side = (point.x - r.x).abs() < 1e-6 || (point.x - r.x - r.width).abs() < 1e-6;
    on_side && r.y < point.y && point.y < r.y + r.height
}

fn rank_of(flow: &Flow, picture: &Layout, name: &str) -> usize {
    picture.ranks[flow.nodes.iter().position(|n| n == name).unwrap()]
}

#[test]
fn the_reference_flow_places_nine_things_in_ranks_that_follow_the_links() {
    let flow = flow(DIAGRAMS[0].1);
    let picture = layout(&flow.graph, None);
    assert_eq!(picture.nodes.len(), 9);
    assert_eq!(flow.graph.links.len(), 9);
    for &(from, to) in &flow.graph.links {
        let (a, b) = (&flow.nodes[from], &flow.nodes[to]);
        assert!(picture.ranks[from] < picture.ranks[to], "{a} -> {b}");
        let (a_box, b_box) = (&picture.nodes[from], &picture.nodes[to]);
        assert!(a_box.x + a_box.width + RANK_GAP <= b_box.x, "{a} -> {b}");
    }
    let ranks: Vec<(&str, usize)> = flow
        .nodes
        .iter()
        .map(String::as_str)
        .zip(picture.ranks.iter().copied())
        .collect();
    assert_eq!(
        ranks,
        [
            ("Web app", 0),
            ("Mobile app", 0),
            ("API gateway", 1),
            ("Payments", 3),
            ("Postgres", 3),
            ("Orders", 2),
            ("Notify", 4),
            ("Events", 5),
            ("Email provider", 6),
        ]
    );
    assert_eq!(flow.groups, ["Services"]);
}

#[test]
fn no_two_boxes_are_closer_than_the_node_gap_on_any_corpus_diagram() {
    let mut things = 0;
    for (name, source) in DIAGRAMS {
        let flow = flow(source);
        things += flow.nodes.len();
        let faults = faults(&flow.graph, &layout(&flow.graph, None));
        assert!(faults.is_empty(), "{name}: {faults:#?}");
    }
    // The ten diagrams of the spike hold 189 things, and the reference file 9.
    assert_eq!(things, 198);
}

// The overlap detector reads each of the eleven pictures as a backend does.
// This test holds the box of each thing only. A path does cross the box of a
// group, 18 times on two diagrams: the order phase puts a link on one side
// of a group in one rank and on the other side in the next, and the path
// then passes through the group. The backlog item "No link passes through a
// group that holds neither of its ends" has that work, and the counts here
// fall with it.
#[test]
fn no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram() {
    let mut links = 0;
    let mut through = Vec::new();
    for (name, source) in DIAGRAMS {
        let flow = flow(source);
        links += flow.graph.links.len();
        let picture = layout(&flow.graph, None);
        let collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup]);
        assert!(collisions.is_empty(), "{name}: {collisions:#?}");
        through.push(through_groups(&flow, &picture));
    }
    assert_eq!(links, 255);
    assert_eq!(through, [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 17]);
}

// The ten diagrams of the spike have no label, so each link gets one here.
// Two labels can still cover each other: item 0124 gives each label its own
// room. A box is the box of a thing or of a group: no label lies on the box
// of a thing or across the border of a group. A label can still be inside a
// group that holds neither end of its link, when the link passes through
// that group.
#[test]
fn each_link_label_sits_on_its_path_and_overlaps_no_box() {
    let mut labels = 0;
    for (name, source) in DIAGRAMS {
        let flow = labelled(source);
        let picture = layout(&flow.graph, None);
        labels += picture.labels.iter().flatten().count();
        let faults = faults(&flow.graph, &picture);
        assert!(faults.is_empty(), "{name}: {faults:#?}");
        let collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup, Kind::Labels]);
        assert!(collisions.is_empty(), "{name}: {collisions:#?}");
    }
    assert_eq!(labels, 255);
}

// Each of the two links from `a` ends at the height where a link from `b`
// starts, and each of the two from `b` ends where a link from `a` starts. No
// order of the two upright lines keeps the level lines apart, so one link
// arrives beside the middle of its box.
// The smallest input from the review of this phase. The upright line of
// `a -> c` leaves `G` through its top, and the middle of the line is 3 px
// from that border.
#[test]
fn a_label_on_a_line_that_leaves_a_group_is_off_the_border_of_the_group() {
    let flow = flow("G {\n  a, b\n}\na -> c: charge\nb -> d\nc -> b\n");
    let picture = layout(&flow.graph, None);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
    let (label, group) = (picture.labels[0].unwrap(), picture.groups[0].unwrap());
    let size = flow.graph.labels[0].unwrap();
    let room = label.y - size.height / 2.0 - group.y;
    assert!((room - LABEL_CLEAR).abs() < 1e-6, "{room}");
    let path = &picture.paths[0];
    assert_eq!((path[1].x, path[2].x), (label.x, label.x));
    let (top, bottom) = (path[1].y.min(path[2].y), path[1].y.max(path[2].y));
    assert!(top < label.y && label.y < bottom, "{path:?}");
}

#[test]
fn two_pairs_of_links_that_cross_do_not_lie_on_each_other() {
    let flow = flow("a -> c, d\nb -> c, d\n");
    let picture = layout(&flow.graph, None);
    let middle = |name: &str| {
        let r = picture.nodes[flow.nodes.iter().position(|n| n == name).unwrap()];
        r.y + r.height / 2.0
    };
    assert_eq!((middle("a"), middle("b")), (middle("c"), middle("d")));
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

#[test]
fn a_link_from_a_node_to_itself_is_a_loop_at_the_side_of_its_box() {
    let flow = flow("a -> a: again and again\na -> b\nG {\n  a\n}\n");
    let picture = layout(&flow.graph, None);
    let (a, path) = (picture.nodes[0], &picture.paths[0]);
    assert_eq!(path.len(), 4);
    assert!(path.iter().all(|point| point.x >= a.x + a.width));
    let faults = faults(&flow.graph, &picture);
    assert!(faults.is_empty(), "{faults:#?}");
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// `x -> c` passes the ranks of `a` and `b`. It enters `G` at the left side
// and stays inside, so it crosses the border of `G` once and the border of
// `Inner` never.
#[test]
fn a_long_link_into_a_group_enters_the_group_at_its_side() {
    let flow = flow("x -> a, c\na -> b\nb -> c\nG {\n  c\n  Inner {\n    a, b\n  }\n}\n");
    let picture = layout(&flow.graph, None);
    let group = picture.groups[flow.groups.iter().position(|g| g == "G").unwrap()].unwrap();
    let link = flow.graph.links.iter().position(|&ends| ends == (0, 2));
    let bends = &picture.bends[link.unwrap()];
    assert_eq!(bends.len(), 2);
    for bend in bends {
        assert!(
            group.y < bend.y && bend.y < group.y + group.height,
            "{bend:?}"
        );
    }
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

#[test]
fn a_label_wider_than_the_rank_gap_widens_the_gap() {
    let text = "a label that is much wider than the gap";
    let flow = flow(&format!("a -> b: {text}\nb -> c\n"));
    let picture = layout(&flow.graph, None);
    let [a, b, c] = [picture.nodes[0], picture.nodes[1], picture.nodes[2]];
    assert!(b.x - (a.x + a.width) > measure(text, STYLE).width);
    assert_eq!(c.x - (b.x + b.width), RANK_GAP);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// The smallest input from the review of SPIKE-18. The prototype put the
// marker of `G` between `e` and `a` in one rank, and the two boxes touched.
#[test]
fn two_things_of_a_group_in_one_rank_keep_the_node_gap() {
    let flow = flow("G {\n  a, c, d, e\n}\nb -> e\nd -> e\nb -> a\n");
    let picture = layout(&flow.graph, None);
    assert_eq!(rank_of(&flow, &picture, "e"), rank_of(&flow, &picture, "a"));
    let faults = faults(&flow.graph, &picture);
    assert!(faults.is_empty(), "{faults:#?}");
}

// 300 graphs from a fixed sequence of numbers, with loops, nested groups and
// links from a node to itself. The diagrams above cannot reach every order
// of groups, and a wrong order would stop the place phase.
#[test]
fn no_generated_graph_has_a_fault() {
    let mut state: u32 = 7;
    let mut next = |below: usize| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 16) as usize % below
    };
    for round in 0..300 {
        let groups: Vec<Group> = (0..next(6))
            .map(|g| Group {
                parent: (g > 0 && next(2) == 0).then(|| next(g)),
            })
            .collect();
        let count = 1 + next(24);
        let nodes: Vec<Node> = (0..count)
            .map(|_| Node {
                width: 40.0 + 10.0 * f64::from(u8::try_from(next(12)).unwrap()),
                height: 36.0,
                group: (!groups.is_empty() && next(3) > 0).then(|| next(groups.len())),
            })
            .collect();
        let links: Vec<(usize, usize)> = (0..next(2 * count))
            .map(|_| (next(count), next(count)))
            .collect();
        let texts: Vec<Option<String>> = (0..links.len())
            .map(|_| (next(3) == 0).then(|| "word ".repeat(1 + next(4))))
            .collect();
        let graph = Graph {
            nodes,
            groups,
            links,
            labels: texts
                .iter()
                .map(|text| text.as_ref().map(|text| measure(text, STYLE)))
                .collect(),
        };
        let picture = layout(&graph, None);
        let faults = faults(&graph, &picture);
        assert!(faults.is_empty(), "graph {round}: {faults:#?}\n{graph:#?}");
        assert_eq!(picture, layout(&graph, None), "graph {round}");
        // A generated graph has groups that no order keeps every link out
        // of, and its labels are many for its size.
        let flow = named(graph, texts);
        let collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup, Kind::Labels]);
        assert!(collisions.is_empty(), "graph {round}: {collisions:#?}");
    }
}

/// A graph that no file states, with a name for each node and each group, so
/// that the scene can hold it.
fn named(graph: Graph, texts: Vec<Option<String>>) -> Flow {
    let count = graph.nodes.len();
    let thing = |name: &String, group: Option<usize>| Thing {
        name: name.clone(),
        mentions: Vec::new(),
        parent: group.map(|g| count + g),
        facts: Vec::new(),
    };
    let nodes: Vec<String> = (0..count).map(|i| format!("n{i}")).collect();
    let groups: Vec<String> = (0..graph.groups.len()).map(|g| format!("g{g}")).collect();
    let boxes = nodes
        .iter()
        .zip(&graph.nodes)
        .map(|(n, node)| thing(n, node.group));
    let around = groups
        .iter()
        .zip(&graph.groups)
        .map(|(g, group)| thing(g, group.parent));
    let links = graph
        .links
        .iter()
        .zip(texts)
        .map(|(&(from, to), text)| Link {
            from,
            to,
            labels: text.into_iter().collect(),
            span: 0..0,
            guess: false,
        });
    let model = Model {
        things: boxes.chain(around).collect(),
        links: links.collect(),
        ..Model::default()
    };
    Flow {
        node_ids: (0..count).collect(),
        group_ids: (count..count + groups.len()).collect(),
        link_ids: (0..graph.links.len()).collect(),
        nodes,
        groups,
        graph,
        model,
    }
}

/// Every position of one picture, one box on each line.
fn positions(name: &str, flow: &Flow, picture: &Layout) -> String {
    let mut text = format!("== {name}\n");
    let rect = |r: &Rect| format!("x {} y {} w {} h {}", r.x, r.y, r.width, r.height);
    for (i, node) in flow.nodes.iter().enumerate() {
        let rank = picture.ranks[i];
        writeln!(text, "rank {rank}  {node}  {}", rect(&picture.nodes[i])).unwrap();
    }
    for (group, r) in flow.groups.iter().zip(&picture.groups) {
        let r = r.as_ref().map_or_else(|| "none".to_owned(), rect);
        writeln!(text, "group  {group}  {r}").unwrap();
    }
    let points = |points: &[Point]| {
        let points: Vec<String> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
        points.join(" ")
    };
    for (link, &(from, to)) in flow.graph.links.iter().enumerate() {
        let (from, to) = (&flow.nodes[from], &flow.nodes[to]);
        if !picture.bends[link].is_empty() {
            let bends = points(&picture.bends[link]);
            writeln!(text, "bends  {from} -> {to}  {bends}").unwrap();
        }
        let path = points(&picture.paths[link]);
        writeln!(text, "path  {from} -> {to}  {path}").unwrap();
        if let Some(label) = picture.labels[link] {
            writeln!(text, "label  {from} -> {to}  {},{}", label.x, label.y).unwrap();
        }
    }
    text
}

#[test]
fn the_same_input_gives_the_same_positions_on_every_run() {
    let mut text = String::new();
    for (name, source) in DIAGRAMS {
        let flow = flow(source);
        let picture = layout(&flow.graph, None);
        assert_eq!(picture, layout(&flow.graph, None), "{name}");
        text.push_str(&positions(name, &flow, &picture));
    }
    #[cfg(not(target_family = "wasm"))]
    if std::env::var_os("HORSEGIRL_BLESS").is_some_and(|v| v == "1") {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/layout.positions");
        std::fs::write(path, &text).unwrap();
        return;
    }
    assert!(
        text == POSITIONS,
        "the positions differ from tests/layout.positions. If the change is intended, run:\n  \
         HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test layout\n\
         and read the diff before you commit it.\n\n{text}"
    );
}

// The layout does not read the seed yet. The rule is in place from the first
// version, so the work that reads the seed starts with a test that holds it.
#[test]
fn a_layout_seeded_with_its_own_picture_moves_nothing() {
    for (name, source) in DIAGRAMS {
        let flow = flow(source);
        let picture = layout(&flow.graph, None);
        assert_eq!(layout(&flow.graph, Some(&picture)), picture, "{name}");
    }
}

// `Reverse sync -> CRM dump` closes a loop that starts at `CRM dump`. The
// prototype turned `CRM dump -> Landing bucket` and put the source last.
#[test]
fn a_loop_turns_the_link_that_closes_it() {
    let flow = flow(DIAGRAMS[4].1);
    let picture = layout(&flow.graph, None);
    assert_eq!(rank_of(&flow, &picture, "CRM dump"), 0);
    assert_eq!(rank_of(&flow, &picture, "Landing bucket"), 1);
    assert!(rank_of(&flow, &picture, "Reverse sync") > 1);
}

#[test]
fn an_empty_graph_gives_an_empty_picture() {
    assert_eq!(layout(&Graph::default(), None), Layout::default());
}

#[test]
fn a_group_that_holds_no_node_has_no_box() {
    let node = Node {
        width: 50.0,
        height: 36.0,
        group: Some(1),
    };
    let graph = Graph {
        nodes: vec![node],
        groups: vec![Group { parent: Some(1) }, Group { parent: None }],
        links: vec![(0, 0)],
        labels: Vec::new(),
    };
    let picture = layout(&graph, None);
    assert_eq!(picture.groups[0], None);
    let group = picture.groups[1].unwrap();
    assert_eq!((group.x, group.y), (0.0, 0.0));
    assert!(holds(&group, &picture.nodes[0]));
    assert_eq!(picture.bends, [[]]);
}
