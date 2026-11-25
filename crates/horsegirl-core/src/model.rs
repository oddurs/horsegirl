//! The model: the tree lowered into things, links, groups and facts.
//!
//! The checks and the lenses read the model and never the tree. A name means
//! the same thing wherever it is written, so `"Web app"` and `Web app` are one
//! thing, and a thing exists from its first mention.
//!
//! The model holds lists in source order and no map, so its debug output is
//! the same on every run and every machine.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::syntax::{Element, Node, NodeKind, Token, TokenKind};

/// The index of a thing in [`Model::things`].
pub type ThingId = usize;

/// Everything that a file states, without its syntax.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    /// Every thing, in the order of its first mention. A group is a thing too,
    /// so a link can end at one.
    pub things: Vec<Thing>,
    /// Every link, in the order of its first statement. A link stated twice
    /// is one link.
    pub links: Vec<Link>,
    /// Every `show` line, in order.
    pub shows: Vec<Show>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thing {
    pub name: String,
    /// The bytes of each name in the source that means this thing, in order.
    /// The first is the mention that made the thing.
    pub mentions: Vec<Range<usize>>,
    /// The group that holds this thing.
    pub parent: Option<ThingId>,
    /// The facts from the braces after the name, in order, each once.
    pub facts: Vec<Fact>,
}

/// `key: value` inside braces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub from: ThingId,
    pub to: ThingId,
    /// The labels of every statement of the link, in order, each once.
    pub labels: Vec<String>,
    /// The bytes of the first line that states the link.
    pub span: Range<usize>,
}

/// `show lens`, or `show lens of name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Show {
    pub lens: String,
    /// `of`, `from` or `by`, and the name after it.
    pub arg: Option<(String, String)>,
}

impl Model {
    /// The thing with this name.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<ThingId> {
        self.things.iter().position(|thing| thing.name == name)
    }

    /// The things that `group` holds, in order of first mention.
    #[must_use]
    pub fn parts(&self, group: ThingId) -> Vec<ThingId> {
        (0..self.things.len())
            .filter(|&id| self.things[id].parent == Some(group))
            .collect()
    }

    /// The things that hold at least one thing, in order of first mention.
    #[must_use]
    pub fn groups(&self) -> Vec<ThingId> {
        (0..self.things.len())
            .filter(|&id| !self.parts(id).is_empty())
            .collect()
    }

    /// Each distinct `owner` value, in the order of the things that state it.
    #[must_use]
    pub fn owners(&self) -> Vec<&str> {
        let mut owners: Vec<&str> = Vec::new();
        let values = self.things.iter().flat_map(|thing| &thing.facts);
        for fact in values.filter(|fact| fact.key == "owner") {
            if !owners.contains(&fact.value.as_str()) {
                owners.push(&fact.value);
            }
        }
        owners
    }
}

/// Lowers a parsed file into the model.
///
/// An `Error` node states nothing, so its names make no thing: the parser has
/// reported it already. A `-x` rule mentions its things but is not a link.
/// A story's links are plain links; the order of a story is not kept yet.
#[must_use]
pub fn lower(tree: &Node<'_>) -> Model {
    let mut lowering = Lowering::default();
    lowering.lines(&tree.children, Scope::Top);
    lowering.model
}

/// Where a line is.
#[derive(Clone, Copy)]
enum Scope<'a> {
    Top,
    /// In the braces after these things.
    Inside(&'a [ThingId]),
    Story,
}

#[derive(Default)]
struct Lowering {
    model: Model,
    // Lookups only. Output order comes from the vectors in `model`.
    names: BTreeMap<String, ThingId>,
    links: BTreeMap<(ThingId, ThingId), usize>,
}

impl Lowering {
    fn lines(&mut self, children: &[Element<'_>], scope: Scope<'_>) {
        for node in nodes(children) {
            match node.kind {
                NodeKind::Link | NodeKind::Rule => self.link(node, scope),
                NodeKind::Things => self.things(node, scope),
                NodeKind::Facts => {
                    // Facts in a story have no thing to belong to.
                    if let Scope::Inside(owners) = scope {
                        for fact in nodes(&node.children) {
                            self.fact(fact, owners);
                        }
                    }
                }
                NodeKind::Story => {
                    if let Some(block) = child(node, NodeKind::Block) {
                        self.lines(&block.children, Scope::Story);
                    }
                }
                NodeKind::Show => self.show(node),
                _ => {}
            }
        }
    }

    fn link(&mut self, node: &Node<'_>, scope: Scope<'_>) {
        let mut ends = nodes(&node.children).filter(|n| n.kind == NodeKind::Names);
        let (Some(from), Some(to)) = (ends.next(), ends.next()) else {
            return;
        };
        let from = self.names(from, scope);
        let to = self.names(to, scope);
        if node.kind == NodeKind::Rule {
            return;
        }
        let label = child(node, NodeKind::Label).map(text);
        // span walks the whole line, so it runs once and not once per link.
        let span = span(node);
        for &a in &from {
            for &b in &to {
                self.add_link(a, b, label.as_deref(), span.clone());
            }
        }
    }

    fn add_link(&mut self, from: ThingId, to: ThingId, label: Option<&str>, span: Range<usize>) {
        let links = &mut self.model.links;
        let index = *self.links.entry((from, to)).or_insert_with(|| {
            links.push(Link {
                from,
                to,
                labels: Vec::new(),
                span,
            });
            links.len() - 1
        });
        let labels = &mut links[index].labels;
        if let Some(label) = label
            && !labels.iter().any(|known| known == label)
        {
            labels.push(label.to_owned());
        }
    }

    fn things(&mut self, node: &Node<'_>, scope: Scope<'_>) {
        let Some(names) = child(node, NodeKind::Names) else {
            return;
        };
        let ids = self.names(names, scope);
        if let Some(block) = child(node, NodeKind::Block) {
            self.lines(&block.children, Scope::Inside(&ids));
        }
    }

    fn fact(&mut self, node: &Node<'_>, owners: &[ThingId]) {
        let (Some(key), Some(value)) = (child(node, NodeKind::Key), child(node, NodeKind::Value))
        else {
            return;
        };
        let fact = Fact {
            key: text(key),
            value: text(value),
        };
        for &owner in owners {
            let facts = &mut self.model.things[owner].facts;
            if !facts.contains(&fact) {
                facts.push(fact.clone());
            }
        }
    }

    fn show(&mut self, node: &Node<'_>) {
        let Some(lens) = child(node, NodeKind::Lens) else {
            return;
        };
        let keyword = node
            .children
            .iter()
            .rev()
            .find_map(|element| match element {
                Element::Token(token) if token.kind == TokenKind::Keyword => Some(token.text),
                _ => None,
            });
        let arg = child(node, NodeKind::Arg)
            .zip(keyword)
            .map(|(arg, keyword)| (keyword.to_owned(), text(arg)));
        self.model.shows.push(Show {
            lens: text(lens),
            arg,
        });
    }

    /// The things of a `Names` node, each mentioned once more.
    fn names(&mut self, node: &Node<'_>, scope: Scope<'_>) -> Vec<ThingId> {
        nodes(&node.children)
            .filter(|n| n.kind == NodeKind::Name)
            .map(|name| self.mention(name, scope))
            .collect()
    }

    fn mention(&mut self, node: &Node<'_>, scope: Scope<'_>) -> ThingId {
        let name = text(node);
        let things = &mut self.model.things;
        let id = *self.names.entry(name).or_insert_with_key(|name| {
            things.push(Thing {
                name: name.clone(),
                mentions: Vec::new(),
                parent: None,
                facts: Vec::new(),
            });
            things.len() - 1
        });
        things[id].mentions.push(span(node));
        // A thing has one parent: the first group that holds it. The braces
        // after several names place a part in the first of them.
        if let Scope::Inside(&[group, ..]) = scope
            && self.model.things[id].parent.is_none()
            && !self.holds(id, group)
        {
            self.model.things[id].parent = Some(group);
        }
        id
    }

    /// Whether `group` is `id` or inside it, so that making `group` the parent
    /// of `id` would close a loop.
    fn holds(&self, id: ThingId, group: ThingId) -> bool {
        let mut at = Some(group);
        while let Some(thing) = at {
            if thing == id {
                return true;
            }
            at = self.model.things[thing].parent;
        }
        false
    }
}

fn nodes<'a, 's>(children: &'a [Element<'s>]) -> impl Iterator<Item = &'a Node<'s>> {
    children.iter().filter_map(|element| match element {
        Element::Node(node) => Some(node),
        Element::Token(_) => None,
    })
}

fn child<'a, 's>(node: &'a Node<'s>, kind: NodeKind) -> Option<&'a Node<'s>> {
    nodes(&node.children).find(|n| n.kind == kind)
}

fn tokens<'a, 's>(node: &'a Node<'s>, into: &mut Vec<&'a Token<'s>>) {
    for element in &node.children {
        match element {
            Element::Node(inner) => tokens(inner, into),
            Element::Token(token) => into.push(token),
        }
    }
}

/// The bytes of the source that a node fills.
fn span(node: &Node<'_>) -> Range<usize> {
    let mut all = Vec::new();
    tokens(node, &mut all);
    match (all.first(), all.last()) {
        (Some(first), Some(last)) => first.start..last.span().end,
        _ => 0..0,
    }
}

/// The text of a name, a value, a label or a lens. Quotes around the whole
/// text are not a part of it, and a run of spaces is one space, so each way
/// of writing a name means the same thing.
fn text(node: &Node<'_>) -> String {
    let mut all = Vec::new();
    tokens(node, &mut all);
    if let [token] = all.as_slice()
        && token.kind == TokenKind::Quoted
    {
        return token.text[1..token.text.len() - 1].to_owned();
    }
    all.iter()
        .map(|token| match token.kind {
            TokenKind::Space => " ",
            _ => token.text,
        })
        .collect()
}
