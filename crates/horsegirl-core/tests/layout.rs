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

use horsegirl_core::layout::{
    GROUP_PAD, GROUP_TITLE, Graph, Group, LABEL_CLEAR, Layout, NODE_GAP, Node, RANK_GAP, layout,
};
use horsegirl_core::lens;
use horsegirl_core::model::{Link, Model, Thing, lower};
use horsegirl_core::overlap::{Kind, overlaps};
use horsegirl_core::scene::{Point, Rect, Scene, build};
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

/// The graph of the flow of a model, as the flow lens binds it.
fn flow_of(model: Model) -> Flow {
    let bound = lens::bind(&model, &lens::FLOW, STYLE);
    let name = |&thing: &usize| model.things[thing].name.clone();
    Flow {
        nodes: bound.nodes.iter().map(name).collect(),
        groups: bound.groups.iter().map(name).collect(),
        graph: bound.graph,
        link_ids: bound.links,
        node_ids: bound.nodes,
        group_ids: bound.groups,
        model,
    }
}

/// The scene of a picture: what a backend draws and the overlap detector
/// reads.
fn scene(flow: &Flow, picture: &Layout) -> Scene {
    let bound = lens::Bound {
        graph: flow.graph.clone(),
        nodes: flow.node_ids.clone(),
        groups: flow.group_ids.clone(),
        links: flow.link_ids.clone(),
    };
    build(
        &flow.model,
        &lens::place(&flow.model, &bound, picture),
        STYLE,
    )
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

/// Each link that passes through a group that holds neither of its ends, in
/// words.
fn through_groups(flow: &Flow, picture: &Layout) -> Vec<String> {
    let scene = scene(flow, picture);
    let found = overlaps(&scene);
    let through = found
        .iter()
        .filter(|overlap| overlap.kind == Kind::LinkThroughGroup);
    through.map(|overlap| overlap.describe(&scene)).collect()
}

/// How many labels share room with a group that holds neither end of their
/// link. Such a label lies across no border, so it is wholly inside the
/// group, and the overlap detector does not report it.
fn labels_in_other_groups(graph: &Graph, picture: &Layout) -> usize {
    let inside = |link: usize, centre: &Point| {
        let size = graph.labels[link].unwrap();
        let label = Rect {
            x: centre.x - size.width / 2.0,
            y: centre.y - size.height / 2.0,
            width: size.width,
            height: size.height,
        };
        let (from, to) = graph.links[link];
        let mut own = around(graph, graph.nodes[from].group);
        own.extend(around(graph, graph.nodes[to].group));
        let groups = picture.groups.iter().enumerate();
        let mut other = groups.filter_map(|(g, group)| group.filter(|_| !own.contains(&g)));
        other.any(|group| room(&group, &label) < 0.0)
    };
    let labels = picture.labels.iter().enumerate();
    let labels = labels.filter_map(|(link, centre)| Some(link).zip(centre.as_ref()));
    labels
        .filter(|&(link, centre)| inside(link, centre))
        .count()
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
fn a_group_box_is_as_wide_as_its_name_plus_the_pad_on_each_side() {
    let name = "A group name of thirty letters";
    assert_eq!(name.len(), 30);
    let width = measure(name, STYLE).width;
    let graph = Graph {
        nodes: vec![Node {
            width: 50.0,
            height: 36.0,
            group: Some(0),
        }],
        groups: vec![Group {
            parent: None,
            width,
        }],
        ..Graph::default()
    };
    let picture = layout(&graph, None);
    let group = picture.groups[0].unwrap();
    assert!(
        group.width >= width + 2.0 * GROUP_PAD,
        "the box is {} wide, the name {width}",
        group.width
    );
    assert!(holds(&group, &picture.nodes[0]));
}

#[test]
fn a_group_whose_name_fits_keeps_its_position() {
    let graph = |width: f64| Graph {
        nodes: vec![
            Node {
                width: 120.0,
                height: 36.0,
                group: Some(0),
            },
            Node {
                width: 120.0,
                height: 36.0,
                group: None,
            },
        ],
        groups: vec![Group {
            parent: None,
            width,
        }],
        links: vec![(0, 1)],
        ..Graph::default()
    };
    let without = layout(&graph(0.0), None);
    let with = layout(&graph(60.0), None);
    assert_eq!(without, with);
}

#[test]
fn a_wide_name_of_an_inner_group_widens_the_group_around_it() {
    let graph = Graph {
        nodes: vec![Node {
            width: 50.0,
            height: 36.0,
            group: Some(1),
        }],
        groups: vec![
            Group {
                parent: None,
                width: 10.0,
            },
            Group {
                parent: Some(0),
                width: 300.0,
            },
        ],
        ..Graph::default()
    };
    let picture = layout(&graph, None);
    let (outer, inner) = (picture.groups[0].unwrap(), picture.groups[1].unwrap());
    assert!(inner.width >= 300.0 + 2.0 * GROUP_PAD);
    assert!(holds(&outer, &inner));
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
// No path crosses the box of a thing. Two paths pass through a group that
// holds neither of their ends, both on `10-platform`, and no order of the
// ranks has fewer.
//
// `Shopping` and `Fulfilment` are both inside `Services`, and both have
// ranks 3, 4 and 5. `Stores` is outside `Services`, so it is on one side of
// both. Each of the two has a link to `Stores` between two of those ranks.
// The one that is further from `Stores` then reaches it through the other. A
// path turns between two ranks only, so it cannot go round. With `Shopping`
// nearer, the two links of `Fulfilment` below pass. With `Fulfilment`
// nearer, five links of `Shopping` would pass: `Cart -> Cache`, both links
// from `Catalog` and one from `Pricing` to `Stores`, and
// `Search -> Search index`.
#[test]
fn no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram() {
    let mut links = 0;
    for (name, source) in DIAGRAMS {
        let flow = flow(source);
        links += flow.graph.links.len();
        let picture = layout(&flow.graph, None);
        let collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup]);
        assert!(collisions.is_empty(), "{name}: {collisions:#?}");
        let through = through_groups(&flow, &picture);
        if name == "10-platform" {
            assert_eq!(
                through,
                [
                    "link `Orders -> Order database` passes through group `Shopping`",
                    "link `Inventory -> Inventory database` passes through group `Shopping`",
                ]
            );
            let rank = |thing: &str| rank_of(&flow, &picture, thing);
            assert_eq!((rank("Orders"), rank("Order database")), (3, 4));
            assert_eq!((rank("Inventory"), rank("Inventory database")), (4, 5));
            assert_eq!((rank("Cart"), rank("Cache")), (4, 5));
        } else {
            assert!(through.is_empty(), "{name}: {through:#?}");
        }
    }
    assert_eq!(links, 255);
}

/// How many points two links cross at, on each of the eleven pictures.
fn corpus_crossings() -> Vec<usize> {
    DIAGRAMS
        .iter()
        .map(|(_, source)| crossings(&layout(&flow(source).graph, None)))
        .collect()
}

// The committed numbers. The order phase removes a pass through a group
// before it removes a crossing, and that must not cost a crossing here:
// before it removed a pass, `07-compiler` had 2 crossings and `10-platform`
// had 49. Before the order phase exchanged neighbours, `03-order-states` had
// 3 crossings, `08-module-deps` had 23 and `09-incident` had 2. A number
// that falls is a better picture: write the new number here.
#[test]
fn the_crossings_of_each_corpus_diagram_do_not_rise() {
    let crossed = corpus_crossings();
    assert_eq!(crossed, [0, 0, 0, 1, 0, 0, 4, 1, 11, 0, 18]);
    assert_eq!(crossed[1..].iter().sum::<usize>(), 35);
}

// The reference is the scores table of `spikes/layered-layout/README.md`.
// On the ten diagrams of the spike, its prototype left the crossings of
// `PROTOTYPE`, the ELK port left 36 in total, and dagre left 14 on
// `08-module-deps`. These limits do not move when a committed number falls.
#[test]
fn the_spike_diagrams_cross_no_more_than_the_reference_allows() {
    const PROTOTYPE: [usize; 10] = [1, 0, 1, 0, 0, 9, 1, 22, 2, 44];
    let crossed = &corpus_crossings()[1..];
    for ((name, _), (now, limit)) in DIAGRAMS[1..].iter().zip(crossed.iter().zip(PROTOTYPE)) {
        assert!(
            *now <= limit,
            "{name}: {now} crossings, the prototype {limit}"
        );
    }
    assert!(crossed[7] <= 14, "08-module-deps: {}", crossed[7]);
    let total: usize = crossed.iter().sum();
    assert!(total <= 36 * 3 / 2, "{total} crossings on the ten diagrams");
}

// The ten diagrams of the spike have no label, so each link gets one here.
// A box is the box of a thing or of a group: no label lies on the box of a
// thing, across the border of a group or on another label, and no label is
// inside a group that holds neither end of its link. The two links of `10-platform` that
// pass through `Shopping` have their labels outside it.
#[test]
fn each_link_label_sits_on_its_path_and_overlaps_no_box() {
    let mut labels = 0;
    for (name, source) in DIAGRAMS {
        let flow = labelled(source);
        let picture = layout(&flow.graph, None);
        labels += picture.labels.iter().flatten().count();
        let faults = faults(&flow.graph, &picture);
        assert!(faults.is_empty(), "{name}: {faults:#?}");
        let collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup]);
        assert!(collisions.is_empty(), "{name}: {collisions:#?}");
        assert_eq!(labels_in_other_groups(&flow.graph, &picture), 0, "{name}");
    }
    assert_eq!(labels, 255);
}

// `n3 -> n0` is beside `g0`, which has the ranks of both ends. The sweeps put
// `n3` above `g0` and `n0` below it, and the path went through the box of
// `g0`. A file gives its things in another order, which the sweeps sort
// well, so this graph is not from a file.
#[test]
fn a_link_beside_a_group_stays_on_one_side_of_it() {
    let node = |group: Option<usize>| Node {
        width: 50.0,
        height: 36.0,
        group,
    };
    let graph = Graph {
        nodes: vec![node(None), node(Some(0)), node(Some(0)), node(None)],
        groups: vec![Group {
            parent: None,
            width: 0.0,
        }],
        links: vec![(3, 0), (0, 1)],
        labels: Vec::new(),
    };
    let picture = layout(&graph, None);
    assert_eq!(picture.ranks, [1, 2, 0, 0]);
    let flow = named(graph, vec![None; 2]);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// From the review of the pass search. `n4 -> n3` went through `g0`. To put
// that one link on one side of `g0` pushes `n1 -> n3` or `n4 -> n0` through
// it, so the search kept the pass. Every link beside the group must change
// its side at once.
#[test]
fn every_link_beside_a_group_takes_one_side_of_it() {
    let node = |group: Option<usize>| Node {
        width: 50.0,
        height: 36.0,
        group,
    };
    let groups = [None, None, Some(0), None, None, Some(0)];
    let graph = Graph {
        nodes: groups.map(node).to_vec(),
        groups: vec![Group {
            parent: None,
            width: 0.0,
        }],
        links: vec![(1, 3), (4, 3), (4, 5), (4, 0)],
        labels: Vec::new(),
    };
    let picture = layout(&graph, None);
    let flow = named(graph, vec![None; 4]);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// With one group an order with no pass exists: every item outside the group
// is above it in every rank. So no generated graph with one group has a link
// through the group.
#[test]
fn no_link_passes_through_the_only_group_of_a_generated_graph() {
    let mut next = sequence(3);
    for round in 0..500 {
        let count = 2 + next(30);
        let nodes: Vec<Node> = (0..count)
            .map(|_| Node {
                width: 50.0,
                height: 36.0,
                group: (next(2) == 0).then_some(0),
            })
            .collect();
        let links: Vec<(usize, usize)> = (0..next(2 * count))
            .map(|_| (next(count), next(count)))
            .collect();
        let graph = Graph {
            nodes,
            groups: vec![Group {
                parent: None,
                width: 0.0,
            }],
            labels: Vec::new(),
            links,
        };
        let picture = layout(&graph, None);
        let texts = vec![None; graph.links.len()];
        let through = through_groups(&named(graph.clone(), texts), &picture);
        assert!(
            through.is_empty(),
            "graph {round}: {through:#?}\n{graph:#?}"
        );
    }
}

// From the second review of the pass search. `n0 -> n5` went through `Inner`.
// To put the items of `Outer` on one side of `Inner` pushes `n5 -> n6`
// through it, and to put `n0 -> n5` on one side pushes `n0 -> n2` through
// it. The items beside `Outer` must change their side in the same move.
#[test]
fn every_link_beside_a_group_inside_a_group_takes_one_side_of_it() {
    let source = "n0 -> n5\nn2 -> n4\nn5 -> n6\nn2 -> n1\nn0 -> n2\n\
                  Outer {\n  n1, n5\n  Inner {\n    n3, n4\n  }\n}\n";
    let flow = flow(source);
    let picture = layout(&flow.graph, None);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// With a chain of groups, each inside the one before, an order with no pass
// exists: in every rank, every item that is beside a group of the chain is
// above that group.
fn no_link_passes_through_a_chain_of_groups(seed: u32, depth: usize, most: usize, rounds: usize) {
    let mut next = sequence(seed);
    for round in 0..rounds {
        let count = 2 + next(most);
        let nodes: Vec<Node> = (0..count)
            .map(|_| Node {
                width: 50.0,
                height: 36.0,
                group: next(depth + 1).checked_sub(1),
            })
            .collect();
        let links: Vec<(usize, usize)> = (0..next(2 * count))
            .map(|_| (next(count), next(count)))
            .collect();
        let graph = Graph {
            nodes,
            groups: (0..depth)
                .map(|g| Group {
                    parent: g.checked_sub(1),
                    width: 0.0,
                })
                .collect(),
            labels: Vec::new(),
            links,
        };
        let picture = layout(&graph, None);
        let texts = vec![None; graph.links.len()];
        let through = through_groups(&named(graph.clone(), texts), &picture);
        assert!(
            through.is_empty(),
            "graph {round}: {through:#?}\n{graph:#?}"
        );
    }
}

#[test]
fn no_link_passes_through_a_group_inside_a_group_of_a_generated_graph() {
    no_link_passes_through_a_chain_of_groups(25, 2, 12, 1000);
}

// From the third review of the pass search. Graph 70 of this sequence kept
// `n14 -> n16` through the outer group. The outer group has ranks that the
// inner group does not have, and the items beside the outer group kept their
// side in those ranks.
#[test]
fn no_link_passes_through_a_group_inside_a_group_of_a_large_generated_graph() {
    no_link_passes_through_a_chain_of_groups(6, 2, 25, 400);
}

#[test]
fn no_link_passes_through_a_chain_of_three_or_four_groups_of_a_generated_graph() {
    no_link_passes_through_a_chain_of_groups(11, 3, 25, 400);
    no_link_passes_through_a_chain_of_groups(12, 4, 25, 400);
}

// `Middle` has the ranks of both ends of `a -> c`. The sweeps put `Middle`
// between `From` and `To`, and the path went through the box of `Middle`.
// Each thing is in a group, so only another order of the groups keeps the
// path out.
#[test]
fn no_group_stands_between_two_groups_that_a_link_joins() {
    let source = "a -> d\na -> c\nTo {\n  c\n}\nFrom {\n  a\n}\nMiddle {\n  b, d\n}\n";
    let flow = flow(source);
    let picture = layout(&flow.graph, None);
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// The smallest input from the review of this phase. The upright line of
// `a -> c` leaves the group through its top, and the middle of the line is
// 3 px from that border. The name of the group is drawn under that border,
// so the label must be clear of the band that holds the name too: a long
// name reaches under the label.
#[test]
fn a_label_on_a_line_that_leaves_a_group_is_off_the_border_of_the_group() {
    for name in ["G", "Services and stores"] {
        let flow = flow(&format!(
            "{name} {{\n  a, b\n}}\na -> c: charge\nb -> d\nc -> b\n"
        ));
        let picture = layout(&flow.graph, None);
        let collisions = collisions(&flow, &picture, &[]);
        assert!(collisions.is_empty(), "{name}: {collisions:#?}");
        let (label, group) = (picture.labels[0].unwrap(), picture.groups[0].unwrap());
        let half = flow.graph.labels[0].unwrap().height / 2.0;
        let above = group.y - (label.y + half);
        let under = label.y - half - (group.y + GROUP_TITLE);
        assert!(
            (above - LABEL_CLEAR).abs() < 1e-6 || (under - LABEL_CLEAR).abs() < 1e-6,
            "{name}: {above} above the border, {under} under the name"
        );
        let path = &picture.paths[0];
        assert_eq!((path[1].x, path[2].x), (label.x, label.x));
        let (top, bottom) = (path[1].y.min(path[2].y), path[1].y.max(path[2].y));
        assert!(top < label.y && label.y < bottom, "{name}: {path:?}");
    }
}

// Each of the two links from `a` ends at the height where a link from `b`
// starts, and each of the two from `b` ends where a link from `a` starts. No
// order of the two upright lines keeps the level lines apart, so one link
// arrives beside the middle of its box.
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

// `a -> c` passes the rank of `b`, and its label is on its first step, in
// the corridor of `a -> b`. Both labels were in the middle of the trunk that
// leaves `a`, and they covered each other.
#[test]
fn two_labels_of_one_corridor_do_not_cover_each_other() {
    let flow = flow("a -> b: to b\nb -> c\na -> c: to c\n");
    let picture = layout(&flow.graph, None);
    assert_eq!(picture.labels.iter().flatten().count(), 2);
    let faults = faults(&flow.graph, &picture);
    assert!(faults.is_empty(), "{faults:#?}");
    let collisions = collisions(&flow, &picture, &[]);
    assert!(collisions.is_empty(), "{collisions:#?}");
}

// The room for labels must not change a picture that has no label.
#[test]
fn a_link_without_a_label_keeps_the_rank_gap() {
    for source in ["a -> b\nb -> c\na -> c\n", "a -> b: to b\nb -> c\n"] {
        let flow = flow(source);
        let picture = layout(&flow.graph, None);
        let [_, b, c] = [picture.nodes[0], picture.nodes[1], picture.nodes[2]];
        assert_eq!(c.x - (b.x + b.width), RANK_GAP, "{source:?}");
    }
    let flow = flow("a -> b\nb -> c\na -> c\n");
    let picture = layout(&flow.graph, None);
    let [a, b] = [picture.nodes[0], picture.nodes[1]];
    assert_eq!(b.x - (a.x + a.width), RANK_GAP);
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

/// A fixed sequence of numbers for a generated graph. Each call gives a
/// number below its argument.
fn sequence(seed: u32) -> impl FnMut(usize) -> usize {
    let mut state = seed;
    move |below| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 16) as usize % below
    }
}

// From the third review of the pass search. A net of 100 nodes in 10 groups
// with 390 links has hundreds of passes that no order removes. The search
// took 40 times as long as every other phase together. The search now stops
// at a limit on its work. The limit is a count, so the picture is the same
// on every run. A test in `src/layout.rs` shows that the search stops.
#[test]
fn a_dense_graph_has_no_fault_when_the_pass_search_stops() {
    let mut next = sequence(9);
    let nodes: Vec<Node> = (0..100)
        .map(|i| Node {
            width: 50.0,
            height: 36.0,
            group: Some(i % 10),
        })
        .collect();
    let graph = Graph {
        nodes,
        groups: vec![
            Group {
                parent: None,
                width: 0.0
            };
            10
        ],
        labels: Vec::new(),
        links: (0..390).map(|_| (next(100), next(100))).collect(),
    };
    let picture = layout(&graph, None);
    let faults = faults(&graph, &picture);
    assert!(faults.is_empty(), "{faults:#?}");
}

// From the fourth review of the pass search. A line of 400 nodes, each two
// in their own group, then 40 nodes in 8 groups with 60 links. One round
// tries another place for each of 208 groups, and most of those moves change
// no rank. The limit did not count such a move, so the search did 5 times
// the work of every other phase. A debug build now stops the test when the
// search tries a move that the limit does not count.
#[test]
fn a_move_that_changes_no_rank_counts_toward_the_limit() {
    let mut next = sequence(1);
    let line = 400;
    let nodes: Vec<Node> = (0..line + 40)
        .map(|i| Node {
            width: 50.0,
            height: 36.0,
            group: Some(if i < line { i / 2 } else { line / 2 + i % 8 }),
        })
        .collect();
    let mut links: Vec<(usize, usize)> = (0..line).map(|i| (i, i + 1)).collect();
    links.extend((0..60).map(|_| (line + next(40), line + next(40))));
    let graph = Graph {
        nodes,
        groups: vec![
            Group {
                parent: None,
                width: 0.0
            };
            line / 2 + 8
        ],
        labels: Vec::new(),
        links,
    };
    let picture = layout(&graph, None);
    let faults = faults(&graph, &picture);
    assert!(faults.is_empty(), "{faults:#?}");
}

// links from a node to itself. The diagrams above cannot reach every order
// of groups, and a wrong order would stop the place phase.
#[test]
fn no_generated_graph_has_a_fault() {
    let mut next = sequence(7);
    for round in 0..300 {
        let groups: Vec<Group> = (0..next(6))
            .map(|g| Group {
                parent: (g > 0 && next(2) == 0).then(|| next(g)),
                width: 40.0 * f64::from(u8::try_from(next(8)).unwrap()),
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
        // of. Two links from a node to itself draw one loop with one place
        // for both labels: item 0169 has that.
        let flow = named(graph, texts);
        let mut collisions = collisions(&flow, &picture, &[Kind::LinkThroughGroup]);
        collisions.retain(|found| !two_loops_of_one_node(found));
        assert!(collisions.is_empty(), "graph {round}: {collisions:#?}");
    }
}

// From the second review of the exchange of neighbours. The order phase
// exchanged neighbours after each round, and the next round swept from the
// exchanged ranks. Graph 2290 of this generator then drew 58 crossings, and
// 28 before the exchange. The exchange now runs once, on the best order.
#[test]
fn the_exchange_of_neighbours_does_not_add_crossings_to_a_generated_graph() {
    let mut next = sequence(1);
    let mut graph = None;
    for _ in 0..=2290 {
        let groups: Vec<Group> = (0..next(6))
            .map(|g| Group {
                parent: (g > 0 && next(2) == 0).then(|| next(g)),
                width: 0.0,
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
        graph = Some(Graph {
            nodes,
            groups,
            links,
            labels: Vec::new(),
        });
    }
    let graph = graph.unwrap();
    let size = (graph.nodes.len(), graph.links.len(), graph.groups.len());
    assert_eq!(size, (23, 37, 0));
    let crossed = crossings(&layout(&graph, None));
    assert!(crossed <= 28, "{crossed} crossings");
}

/// A collision of the labels of two links from one node to itself.
fn two_loops_of_one_node(found: &str) -> bool {
    let Some((first, second)) = found.split_once(" covers ") else {
        return false;
    };
    match (link_names(first), link_names(second)) {
        (Some((a, b)), Some((c, d))) => a == b && b == c && c == d,
        _ => false,
    }
}

/// The names of the two ends of a label as the overlap detector names it.
fn link_names(label: &str) -> Option<(&str, &str)> {
    let link = label.strip_prefix("label `")?.strip_suffix('`')?;
    link.split_once(" -> ")
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
        groups: vec![
            Group {
                parent: Some(1),
                width: 0.0,
            },
            Group {
                parent: None,
                width: 0.0,
            },
        ],
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

/// How many points two links cross at: a level part of one path and an
/// upright part of another, each strictly inside the other.
fn crossings(picture: &Layout) -> usize {
    let parts = |path: &[Point]| -> Vec<(Point, Point)> {
        path.windows(2).map(|pair| (pair[0], pair[1])).collect()
    };
    let within = |x: f64, a: f64, b: f64| a.min(b) < x && x < a.max(b);
    let cross = |level: &(Point, Point), upright: &(Point, Point)| {
        level.0.y - level.1.y == 0.0
            && upright.0.x - upright.1.x == 0.0
            && within(upright.0.x, level.0.x, level.1.x)
            && within(level.0.y, upright.0.y, upright.1.y)
    };
    let paths: Vec<Vec<(Point, Point)>> = picture.paths.iter().map(|path| parts(path)).collect();
    let mut count = 0;
    for (index, a) in paths.iter().enumerate() {
        for b in &paths[index + 1..] {
            for p in a {
                for q in b {
                    count += usize::from(cross(p, q) || cross(q, p));
                }
            }
        }
    }
    count
}
