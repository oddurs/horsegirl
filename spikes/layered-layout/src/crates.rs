//! Candidates that already exist as crates: a dagre port, graphlay, and a port of ELK.

use dagre::graph::{Graph as DagreGraph, GraphOptions};
use dagre::{EdgeLabel, LayoutOptions as DagreOptions, NodeLabel, RankDir};
use graphlay::{Edge, GraphBuilder, LayoutOptions, Node, NodeId, Point, SeedMode, Size};
use serde_json::{Value, json};

use crate::model::{Graph, Picture, Rect, Seed};

const NODE_GAP: f64 = 24.0;
const RANK_GAP: f64 = 64.0;
const PAD: f64 = 14.0;
const TITLE: f64 = 30.0;
const PHASES: [&str; 4] = [
    "cycleBreaking",
    "layering",
    "crossingMinimization",
    "nodePlacement",
];

fn centred(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect {
        x: x - w / 2.0,
        y: y - h / 2.0,
        w,
        h,
    }
}

/// dagre has no input for earlier positions.
pub fn dagre(graph: &Graph) -> Picture {
    let options = GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    };
    let mut g: DagreGraph<NodeLabel, EdgeLabel> = DagreGraph::with_options(options);
    for (i, group) in graph.groups.iter().enumerate() {
        g.set_node(format!("g{i}"), Some(NodeLabel::default()));
        if let Some(parent) = group.parent {
            g.set_parent(&format!("g{i}"), Some(&format!("g{parent}")));
        }
    }
    for (i, node) in graph.nodes.iter().enumerate() {
        let label = NodeLabel {
            width: node.w,
            height: node.h,
            ..NodeLabel::default()
        };
        g.set_node(format!("n{i}"), Some(label));
        if let Some(group) = node.group {
            g.set_parent(&format!("n{i}"), Some(&format!("g{group}")));
        }
    }
    for (e, &(a, b)) in graph.edges.iter().enumerate() {
        let name = format!("e{e}");
        g.set_edge(
            format!("n{a}"),
            format!("n{b}"),
            Some(EdgeLabel::default()),
            Some(&name),
        );
    }
    let options = DagreOptions {
        rankdir: RankDir::LR,
        nodesep: NODE_GAP,
        ranksep: RANK_GAP,
        ..DagreOptions::default()
    };
    dagre::layout(&mut g, Some(options));

    let rect = |id: String| {
        g.node(&id).map_or_else(Rect::default, |label| {
            centred(
                label.x.unwrap_or(0.0),
                label.y.unwrap_or(0.0),
                label.width,
                label.height,
            )
        })
    };
    let edges = (0..graph.edges.len())
        .map(|e| {
            let (a, b) = graph.edges[e];
            let label = g.edge(&format!("n{a}"), &format!("n{b}"), Some(&format!("e{e}")));
            label.map_or_else(Vec::new, |label| {
                label.points.iter().map(|p| (p.x, p.y)).collect()
            })
        })
        .collect();
    Picture {
        nodes: (0..graph.nodes.len())
            .map(|i| rect(format!("n{i}")))
            .collect(),
        groups: (0..graph.groups.len())
            .map(|i| rect(format!("g{i}")))
            .collect(),
        edges,
    }
}

/// graphlay reads an earlier centre from `Node::position`. Its documentation says that it
/// ignores the positions in a graph that has groups.
pub fn graphlay(graph: &Graph, seed: Option<&Seed>) -> Result<Picture, String> {
    let mut builder = GraphBuilder::new();
    let groups: Vec<NodeId> = graph
        .groups
        .iter()
        .map(|_| {
            builder.add_node(Node {
                size: Size::new(1.0, 1.0),
                ..Node::default()
            })
        })
        .collect();
    for (i, group) in graph.groups.iter().enumerate() {
        if let (Some(parent), Some(node)) = (group.parent, builder.node_mut(groups[i])) {
            node.parent = Some(groups[parent]);
        }
    }
    let nodes: Vec<NodeId> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            builder.add_node(Node {
                size: Size::new(node.w, node.h),
                parent: node.group.map(|group| groups[group]),
                position: seed.and_then(|seed| seed[i]).map(|(x, y)| Point { x, y }),
                key: Some(node.name.clone()),
                ..Node::default()
            })
        })
        .collect();
    for &(a, b) in &graph.edges {
        builder.add_edge(Edge::new(nodes[a], nodes[b]));
    }
    let mut options = LayoutOptions::layered_default();
    options.spacing.layer_gap = RANK_GAP;
    options.spacing.node_gap = NODE_GAP;
    options.spacing.compound_padding = PAD;
    options.spacing.compound_header = TITLE - PAD;
    if seed.is_some() {
        options.seed_mode = SeedMode::Structure;
    }
    let drawing = graphlay::layout(&builder.build(), &options).map_err(|e| e.to_string())?;
    let rect = |id: &NodeId| {
        let placed = &drawing.nodes[id.index()];
        centred(
            placed.center.x,
            placed.center.y,
            placed.size.width,
            placed.size.height,
        )
    };
    Ok(Picture {
        nodes: nodes.iter().map(rect).collect(),
        groups: groups.iter().map(rect).collect(),
        edges: drawing
            .edges
            .iter()
            .map(|route| route.points.iter().map(|p| (p.x, p.y)).collect())
            .collect(),
    })
}

/// The ELK port reads an earlier position from the `x` and `y` of a node when the four
/// phases use their `INTERACTIVE` strategy.
pub fn elk(graph: &Graph, seed: Option<&Picture>) -> Result<Picture, String> {
    fn child(graph: &Graph, group: Option<usize>, seed: Option<&Picture>) -> Vec<Value> {
        // A position in ELK is relative to the box around it.
        let origin = seed
            .zip(group)
            .map_or((0.0, 0.0), |(seed, g)| (seed.groups[g].x, seed.groups[g].y));
        let mut children = Vec::new();
        for (g, inner) in graph.groups.iter().enumerate() {
            if inner.parent == group {
                let mut value = json!({
                    "id": format!("g{g}"),
                    "layoutOptions": { "elk.padding": format!(
                        "[top={TITLE},left={PAD},bottom={PAD},right={PAD}]"
                    ) },
                    "children": child(graph, Some(g), seed),
                });
                // ELK refuses a graph whose groups do not use the strategy of the root.
                if seed.is_some() {
                    for phase in PHASES {
                        value["layoutOptions"][format!("elk.layered.{phase}.strategy")] =
                            json!("INTERACTIVE");
                    }
                }
                if let Some(r) = seed.map(|seed| seed.groups[g]).filter(|r| r.w > 0.0) {
                    value["x"] = json!(r.x - origin.0);
                    value["y"] = json!(r.y - origin.1);
                }
                children.push(value);
            }
        }
        for (i, node) in graph.nodes.iter().enumerate() {
            if node.group == group {
                let mut value = json!({ "id": format!("n{i}"), "width": node.w, "height": node.h });
                if let Some(r) = seed.and_then(|seed| seed.nodes.get(i)) {
                    value["x"] = json!(r.x - origin.0);
                    value["y"] = json!(r.y - origin.1);
                }
                children.push(value);
            }
        }
        children
    }
    let mut options = json!({
        "elk.algorithm": "layered",
        "elk.direction": "RIGHT",
        "elk.edgeRouting": "ORTHOGONAL",
        "elk.hierarchyHandling": "INCLUDE_CHILDREN",
        "elk.spacing.nodeNode": NODE_GAP,
        "elk.layered.spacing.nodeNodeBetweenLayers": RANK_GAP,
    });
    if seed.is_some() {
        for phase in PHASES {
            options[format!("elk.layered.{phase}.strategy")] = json!("INTERACTIVE");
        }
    }
    let edges: Vec<Value> = graph
        .edges
        .iter()
        .enumerate()
        .map(|(e, (a, b))| {
            json!({ "id": format!("e{e}"), "sources": [format!("n{a}")], "targets": [format!("n{b}")] })
        })
        .collect();
    let input = json!({
        "id": "root",
        "layoutOptions": options,
        "children": child(graph, None, seed),
        "edges": edges,
    });
    let output = elk_layout::layout_value(input, "{}").map_err(|e| e.to_string())?;

    fn collect(
        value: &Value,
        origin: (f64, f64),
        found: &mut Vec<(String, Rect)>,
        edges: &mut Vec<Value>,
    ) {
        let number = |key: &str| value[key].as_f64().unwrap_or(0.0);
        let id = value["id"].as_str().unwrap_or("").to_string();
        let at = if id == "root" {
            origin
        } else {
            (origin.0 + number("x"), origin.1 + number("y"))
        };
        found.push((
            id,
            Rect {
                x: at.0,
                y: at.1,
                w: number("width"),
                h: number("height"),
            },
        ));
        edges.extend(value["edges"].as_array().into_iter().flatten().cloned());
        for inner in value["children"].as_array().into_iter().flatten() {
            collect(inner, at, found, edges);
        }
    }
    let mut found = Vec::new();
    let mut routed = Vec::new();
    collect(&output, (0.0, 0.0), &mut found, &mut routed);
    let rect = |id: String| {
        found
            .iter()
            .find(|(name, _)| *name == id)
            .map_or_else(Rect::default, |(_, r)| *r)
    };
    let edges = (0..graph.edges.len())
        .map(|e| {
            let Some(edge) = routed.iter().find(|edge| edge["id"] == format!("e{e}")) else {
                return Vec::new();
            };
            // The points of a link are relative to the box that ELK names as its container.
            let origin = rect(edge["container"].as_str().unwrap_or("root").to_string());
            let point = |p: &Value| {
                (
                    origin.x + p["x"].as_f64().unwrap_or(0.0),
                    origin.y + p["y"].as_f64().unwrap_or(0.0),
                )
            };
            let mut points = Vec::new();
            for section in edge["sections"].as_array().into_iter().flatten() {
                points.push(point(&section["startPoint"]));
                points.extend(
                    section["bendPoints"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(point),
                );
                points.push(point(&section["endPoint"]));
            }
            points
        })
        .collect();
    Ok(Picture {
        nodes: (0..graph.nodes.len())
            .map(|i| rect(format!("n{i}")))
            .collect(),
        groups: (0..graph.groups.len())
            .map(|i| rect(format!("g{i}")))
            .collect(),
        edges,
    })
}
