//! The input every candidate reads and the picture every candidate returns.

/// A thing with the size of its box.
pub struct Node {
    pub name: String,
    pub w: f64,
    pub h: f64,
    pub group: Option<usize>,
}

pub struct Group {
    pub name: String,
    pub parent: Option<usize>,
}

#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub groups: Vec<Group>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn centre(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// Positioned boxes and paths, indexed like the graph that produced them.
pub struct Picture {
    pub nodes: Vec<Rect>,
    pub groups: Vec<Rect>,
    pub edges: Vec<Vec<(f64, f64)>>,
}

/// The centre of each node in an earlier picture. `None` marks a node that is new.
pub type Seed = [Option<(f64, f64)>];

impl Graph {
    /// Returns the node with this name, and makes it on first mention.
    pub fn thing(&mut self, name: &str) -> usize {
        if let Some(i) = self.nodes.iter().position(|n| n.name == name) {
            return i;
        }
        // A stand-in for real text metrics, which SPIKE-19 decides.
        let w = (name.chars().count() as f64).mul_add(7.2, 28.0);
        self.nodes.push(Node {
            name: name.to_string(),
            w,
            h: 36.0,
            group: None,
        });
        self.nodes.len() - 1
    }

    pub fn link(&mut self, from: usize, to: usize) {
        if !self.edges.contains(&(from, to)) {
            self.edges.push((from, to));
        }
    }

    /// The groups around a group, outermost first, ending with the group itself.
    pub fn chain(&self, group: Option<usize>) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut at = group;
        while let Some(g) = at {
            chain.push(g);
            at = self.groups[g].parent;
        }
        chain.reverse();
        chain
    }
}

fn names(list: &str) -> impl Iterator<Item = &str> {
    list.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// Reads the subset of horsegirl that a flow needs: links, and groups in braces.
pub fn parse(source: &str) -> Result<Graph, String> {
    let mut graph = Graph::default();
    let mut open: Vec<usize> = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "}" {
            open.pop()
                .ok_or_else(|| format!("line {}: no group is open", index + 1))?;
        } else if let Some(name) = line.strip_suffix('{') {
            let parent = open.last().copied();
            graph.groups.push(Group {
                name: name.trim().to_string(),
                parent,
            });
            open.push(graph.groups.len() - 1);
        } else if let Some((from, to)) = line.split_once("->") {
            let to = to.split_once(':').map_or(to, |(to, _label)| to);
            for a in names(from) {
                for b in names(to) {
                    let (a, b) = (graph.thing(a), graph.thing(b));
                    graph.link(a, b);
                }
            }
        } else {
            let group = *open
                .last()
                .ok_or_else(|| format!("line {}: expected a link or a group", index + 1))?;
            for name in names(line) {
                let node = graph.thing(name);
                graph.nodes[node].group = Some(group);
            }
        }
    }
    if open.is_empty() {
        Ok(graph)
    } else {
        Err("a group is not closed".to_string())
    }
}
