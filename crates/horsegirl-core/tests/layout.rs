//! The layered layout on the reference file and on the ten flow diagrams of
//! SPIKE-18, which are in `spikes/layered-layout/corpus`.
//!
//! `layout.positions` holds every position that the layout gives for those
//! eleven files. CI runs this test on Linux, on macOS and in WebAssembly, so
//! the file also proves that each target gives the same numbers. To accept a
//! change, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test layout
//! ```
//!
//! Read the diff before you commit it.

use std::fmt::Write as _;

use horsegirl_core::layout::{Graph, Group, Layout, NODE_GAP, Node, RANK_GAP, layout};
use horsegirl_core::model::lower;
use horsegirl_core::scene::Rect;
use horsegirl_core::syntax::parse;
use horsegirl_core::text::{Style, measure};

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
}

/// Reads a file into the graph of its flow.
///
/// A thing that holds something is a group, and every other thing is a node
/// as wide as its name. A link that starts or ends at a group is left out:
/// item 0123 draws it.
fn flow(source: &str) -> Flow {
    let model = lower(&parse(source).tree);
    let group_ids = model.groups();
    let group_of = |thing: usize| group_ids.iter().position(|&g| g == thing);
    let node_ids: Vec<usize> = (0..model.things.len())
        .filter(|&thing| group_of(thing).is_none())
        .collect();
    let node_of = |thing: usize| node_ids.iter().position(|&n| n == thing);
    let parent = |thing: usize| model.things[thing].parent.and_then(group_of);
    let graph = Graph {
        nodes: node_ids
            .iter()
            .map(|&thing| Node {
                width: measure(&model.things[thing].name, Style { size: 14.0 }).width + 28.0,
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
        links: model
            .links
            .iter()
            .filter_map(|link| node_of(link.from).zip(node_of(link.to)))
            .collect(),
    };
    let name = |&thing: &usize| model.things[thing].name.clone();
    Flow {
        nodes: node_ids.iter().map(name).collect(),
        groups: group_ids.iter().map(name).collect(),
        graph,
    }
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
    let points = picture.bends.iter().flatten().map(|p| (p.x, p.y));
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
    faults
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
        let links = (0..next(2 * count))
            .map(|_| (next(count), next(count)))
            .collect();
        let graph = Graph {
            nodes,
            groups,
            links,
        };
        let picture = layout(&graph, None);
        let faults = faults(&graph, &picture);
        assert!(faults.is_empty(), "graph {round}: {faults:#?}\n{graph:#?}");
        assert_eq!(picture, layout(&graph, None), "graph {round}");
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
    for (&(from, to), bends) in flow.graph.links.iter().zip(&picture.bends) {
        if !bends.is_empty() {
            let points: Vec<String> = bends.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
            let (from, to) = (&flow.nodes[from], &flow.nodes[to]);
            writeln!(text, "bends  {from} -> {to}  {}", points.join(" ")).unwrap();
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
    };
    let picture = layout(&graph, None);
    assert_eq!(picture.groups[0], None);
    let group = picture.groups[1].unwrap();
    assert_eq!((group.x, group.y), (0.0, 0.0));
    assert!(holds(&group, &picture.nodes[0]));
    assert_eq!(picture.bends, [[]]);
}
