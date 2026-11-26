//! The lenses: which field of the facts each channel of a picture shows.
//!
//! A lens is data. It names a question, binds a field to each channel, states
//! the facts it needs and picks a solver. [`bind`] reads those dials and turns
//! a model into what the solver places, so the solver knows no lens: the
//! layered layout reads boxes, groups and links, and nothing tells it that the
//! picture is a flow.

use std::fmt;

use crate::layout::{self, End, Graph, Group, Layout, Node};
use crate::model::{Model, ThingId};
use crate::scene::{self, Placement, Scene};
use crate::text::{self, Style};

/// A property of a picture that a field can set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// The position from left to right.
    X,
    /// The box that holds a thing.
    Inside,
    /// The palette slot of a thing.
    Colour,
}

/// What a channel shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// How many links lead to a thing on the longest path to it.
    Depth,
    /// The thing that holds a thing, as braces state it.
    Group,
    /// The value of the fact with this key, such as `owner`.
    Fact(&'static str),
}

/// One binding of a field to a channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dial {
    pub channel: Channel,
    pub field: Field,
}

/// The program that places the shapes of a picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solver {
    /// Ranks from left to right, as `horsegirl_core::layout` places them.
    Layered,
}

/// A fact that a lens cannot draw without.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    /// At least one link.
    Links,
}

/// A lens: the question it answers, its dials, the facts it needs and its
/// solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lens {
    /// The word after `show`.
    pub name: &'static str,
    pub question: &'static str,
    pub dials: &'static [Dial],
    pub needs: &'static [Need],
    pub solver: Solver,
}

/// The flow lens: what depends on what.
pub const FLOW: Lens = Lens {
    name: "flow",
    question: "What depends on what.",
    dials: &[
        Dial {
            channel: Channel::X,
            field: Field::Depth,
        },
        Dial {
            channel: Channel::Inside,
            field: Field::Group,
        },
        Dial {
            channel: Channel::Colour,
            field: Field::Fact("owner"),
        },
    ],
    needs: &[Need::Links],
    solver: Solver::Layered,
};

/// Every lens that this version draws.
pub const DRAWN: &[Lens] = &[FLOW];

/// The lens that `show <name>` picks, when this version draws it.
#[must_use]
pub fn find(name: &str) -> Option<&'static Lens> {
    DRAWN.iter().find(|lens| lens.name == name)
}

/// The width of the room between the name of a thing and each side of its box.
pub const NAME_PAD: f64 = 14.0;
/// The height of the box of a thing.
pub const BOX_HEIGHT: f64 = 36.0;

impl Lens {
    /// The field that this lens binds to `channel`.
    #[must_use]
    pub fn field(&self, channel: Channel) -> Option<Field> {
        let dial = self.dials.iter().find(|dial| dial.channel == channel);
        dial.map(|dial| dial.field)
    }

    /// The first dial that the solver of this lens cannot show.
    #[must_use]
    pub fn unshown(&self) -> Option<Dial> {
        // The layered solver ranks along x by the links, so x can show depth
        // alone, and it draws no other channel.
        let shown = |dial: &&Dial| match self.solver {
            Solver::Layered => matches!(
                (dial.channel, dial.field),
                (Channel::X, Field::Depth)
                    | (Channel::Inside, Field::Group)
                    | (Channel::Colour, Field::Fact(_))
            ),
        };
        self.dials.iter().find(|dial| !shown(dial)).copied()
    }

    /// The needs of this lens that `model` does not meet, in the order of
    /// [`Lens::needs`].
    #[must_use]
    pub fn missing(&self, model: &Model) -> Vec<Need> {
        let met = |need: &Need| match need {
            Need::Links => !model.links.is_empty(),
        };
        self.needs
            .iter()
            .filter(|need| !met(need))
            .copied()
            .collect()
    }
}

/// A model as the solver reads it, with the thing or the link of the model
/// that each entry of the graph stands for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bound {
    pub graph: Graph,
    /// The thing of each node of the graph.
    pub nodes: Vec<ThingId>,
    /// The thing of each group of the graph.
    pub groups: Vec<ThingId>,
    /// The index in the model of each link of the graph.
    pub links: Vec<usize>,
}

/// Reads the dials of `lens` and turns `model` into the graph of its solver.
///
/// A thing that holds something is a group when the lens binds `inside` to
/// `group`; every other thing is a box as wide as its name. A link that starts
/// or ends at a group meets the border of the group box. A link between a
/// group and itself or a thing inside it is left out, and the check reports
/// it as `W0110`.
///
/// # Panics
///
/// When `lens` binds a channel to a field that its solver cannot show. The
/// lenses are constants, and a test binds each one.
#[must_use]
pub fn bind(model: &Model, lens: &Lens, style: Style) -> Bound {
    if let Some(dial) = lens.unshown() {
        panic!("the {} solver cannot show {dial}", lens.solver);
    }
    let groups = if lens.field(Channel::Inside) == Some(Field::Group) {
        model.groups()
    } else {
        Vec::new()
    };
    let group_of = |thing: ThingId| groups.iter().position(|&g| g == thing);
    let nodes: Vec<ThingId> = (0..model.things.len())
        .filter(|&thing| group_of(thing).is_none())
        .collect();
    let node_of = |thing: ThingId| nodes.iter().position(|&n| n == thing);
    let parent = |thing: ThingId| model.things[thing].parent.and_then(group_of);
    let end = |thing: ThingId| match group_of(thing) {
        Some(group) => End::Group(group),
        None => End::Node(node_of(thing).expect("a thing that is no group is a node")),
    };
    let links: Vec<(usize, (End, End))> = model
        .links
        .iter()
        .enumerate()
        .filter(|(_, link)| {
            let holds = |group: ThingId, thing: ThingId| {
                group_of(group).is_some() && model.within(thing, group)
            };
            !holds(link.from, link.to) && !holds(link.to, link.from)
        })
        .map(|(id, link)| (id, (end(link.from), end(link.to))))
        .collect();
    let label = |&(id, _): &(usize, _)| {
        let labels = &model.links[id].labels;
        (!labels.is_empty()).then(|| text::measure(&labels.join(", "), style))
    };
    let graph = Graph {
        nodes: nodes
            .iter()
            .map(|&thing| Node {
                width: text::measure(&model.things[thing].name, style).width + 2.0 * NAME_PAD,
                height: BOX_HEIGHT,
                group: parent(thing),
            })
            .collect(),
        groups: groups
            .iter()
            .map(|&thing| Group {
                parent: parent(thing),
                width: text::measure(&model.things[thing].name, style).width,
            })
            .collect(),
        labels: links.iter().map(label).collect(),
        links: links.iter().map(|&(_, ends)| ends).collect(),
    };
    Bound {
        graph,
        nodes,
        groups,
        links: links.iter().map(|&(id, _)| id).collect(),
    }
}

/// Where a picture of the solver puts each thing and each link of the model.
#[must_use]
pub fn place(model: &Model, bound: &Bound, picture: &Layout) -> Placement {
    let mut placement = Placement {
        things: vec![None; model.things.len()],
        links: vec![Vec::new(); model.links.len()],
        labels: vec![None; model.links.len()],
    };
    for (&thing, node) in bound.nodes.iter().zip(&picture.nodes) {
        placement.things[thing] = Some(*node);
    }
    for (&thing, group) in bound.groups.iter().zip(&picture.groups) {
        placement.things[thing] = *group;
    }
    for (link, &id) in bound.links.iter().enumerate() {
        placement.links[id].clone_from(&picture.paths[link]);
        placement.labels[id] = picture.labels[link];
    }
    placement
}

/// The picture of `model` through `lens`: bind, solve, place, and mark the
/// fact that picks the colour of each thing.
#[must_use]
pub fn draw(model: &Model, lens: &Lens, style: Style) -> Scene {
    let bound = bind(model, lens, style);
    let picture = match lens.solver {
        Solver::Layered => layout::layout(&bound.graph, None),
    };
    let mut scene = scene::build(model, &place(model, &bound, &picture), style);
    scene.colour = match lens.field(Channel::Colour) {
        Some(Field::Fact(key)) => Some(key.to_owned()),
        _ => None,
    };
    scene
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::X => "x",
            Self::Inside => "inside",
            Self::Colour => "colour",
        })
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Depth => "depth",
            Self::Group => "group",
            Self::Fact(key) => key,
        })
    }
}

impl fmt::Display for Dial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.channel, self.field)
    }
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Layered => "layered",
        })
    }
}

impl fmt::Display for Need {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Links => "links",
        })
    }
}

/// The lenses as a Markdown table, as `docs/lenses.md` holds it.
#[must_use]
pub fn reference_markdown() -> String {
    let header = "| Lens | Question | Dials | Needs | Solver |\n| --- | --- | --- | --- | --- |\n";
    let rows = DRAWN.iter().map(|lens| {
        let dials: Vec<String> = lens.dials.iter().map(|d| format!("`{d}`")).collect();
        let needs: Vec<String> = lens.needs.iter().map(ToString::to_string).collect();
        format!(
            "| `{}` | {} | {} | {} | {} |\n",
            lens.name,
            lens.question,
            dials.join(", "),
            needs.join(", "),
            lens.solver
        )
    });
    std::iter::once(header.to_owned()).chain(rows).collect()
}
