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
    /// Every `-x` rule, in the order of its first statement. A rule is a claim
    /// that a link must not exist, so it is never a link.
    pub rules: Vec<Rule>,
    /// Every `show` line, in order.
    pub shows: Vec<Show>,
    /// Every name in braces that the model does not place there, in source
    /// order.
    pub misplaced: Vec<Misplaced>,
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
    /// Every statement of the link ends in `?`. One firm statement makes the
    /// link firm, because the file then states it as true.
    pub guess: bool,
}

/// `from -x to: reason`: a link that must not exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub from: ThingId,
    pub to: ThingId,
    /// The words after the colon, if there are any.
    pub reason: Option<String>,
    /// The bytes of the first line that states the rule.
    pub span: Range<usize>,
    /// Every statement of the rule ends in `?`. One firm statement makes the
    /// rule firm, as it does for a link.
    pub guess: bool,
}

/// A name in braces that the model does not place in the group of the
/// braces, because a thing is in one group and no group holds itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misplaced {
    pub thing: ThingId,
    /// The group of the braces. For braces after several names, the first.
    pub group: ThingId,
    /// The bytes of the name in the braces.
    pub span: Range<usize>,
    pub why: Why,
    /// The bytes to delete to remove the name and its comma, when the name
    /// is one of several on a line that only names things. The thing then
    /// keeps its group and its other mentions, so the file means the same.
    pub removable: Option<Range<usize>>,
}

/// Why the model does not place a thing in a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// The thing is in this other group already, from earlier braces.
    Held(ThingId),
    /// The group of the braces is the thing or inside it.
    Loop,
    /// The braces follow these names. The model places the thing in the
    /// first of them when no other rule stops it, so the parent of the
    /// thing, and not this list, says where the thing is.
    Shared(Vec<ThingId>),
}

/// `show lens`, or `show lens of name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Show {
    pub lens: String,
    /// `of`, `from` or `by`, and the name after it.
    pub arg: Option<(String, String)>,
    /// The bytes of the lens name, so a check can point at an unknown lens.
    pub span: Range<usize>,
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

    /// `thing` is `group`, or `group` holds it at any depth.
    #[must_use]
    pub fn within(&self, thing: ThingId, group: ThingId) -> bool {
        let mut at = Some(thing);
        // The model places no group inside itself, so the walk ends. The
        // limit keeps it finite for a model built by hand.
        for _ in 0..=self.things.len() {
            match at {
                Some(id) if id == group => return true,
                Some(id) => at = self.things[id].parent,
                None => return false,
            }
        }
        false
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
/// reported it already. A `-x` rule mentions its things and is stored as a
/// rule, not as a link.
/// A story's links are plain links; the order of a story is not kept yet.
#[must_use]
pub fn lower(tree: &Node<'_>) -> Model {
    let mut lowering = Lowering::default();
    lowering.lines(&tree.children, Scope::Top);
    lowering.settle();
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
        let from = self.names(from, scope, false);
        let to = self.names(to, scope, false);
        let label = child(node, NodeKind::Label).map(text);
        // span walks the whole line, so it runs once and not once per link.
        let span = span(node);
        let guess = node.children.iter().any(
            |element| matches!(element, Element::Token(token) if token.kind == TokenKind::Guess),
        );
        if node.kind == NodeKind::Rule {
            for &a in &from {
                for &b in &to {
                    self.add_rule(a, b, label.as_deref(), span.clone(), guess);
                }
            }
            return;
        }
        for &a in &from {
            for &b in &to {
                self.add_link(a, b, label.as_deref(), span.clone(), guess);
            }
        }
    }

    fn add_rule(
        &mut self,
        from: ThingId,
        to: ThingId,
        reason: Option<&str>,
        span: Range<usize>,
        guess: bool,
    ) {
        let known =
            self.model.rules.iter_mut().find(|rule| {
                rule.from == from && rule.to == to && rule.reason.as_deref() == reason
            });
        if let Some(rule) = known {
            rule.guess &= guess;
        } else {
            self.model.rules.push(Rule {
                from,
                to,
                reason: reason.map(str::to_owned),
                span,
                guess,
            });
        }
    }

    fn add_link(
        &mut self,
        from: ThingId,
        to: ThingId,
        label: Option<&str>,
        span: Range<usize>,
        guess: bool,
    ) {
        let links = &mut self.model.links;
        let index = *self.links.entry((from, to)).or_insert_with(|| {
            links.push(Link {
                from,
                to,
                labels: Vec::new(),
                span,
                guess,
            });
            links.len() - 1
        });
        links[index].guess &= guess;
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
        // A line with a block, a citation or a stray token says more than
        // its names, so a name on it is never removed.
        let bare = node.children.iter().all(|element| match element {
            Element::Node(inner) => inner.kind == NodeKind::Names,
            Element::Token(token) => {
                matches!(
                    token.kind,
                    TokenKind::Space | TokenKind::Newline | TokenKind::Semi
                )
            }
        });
        let ids = self.names(names, scope, bare);
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
            span: span(lens),
        });
    }

    /// The things of a `Names` node, each mentioned once more. `bare` says
    /// that the line only names things, so a name on it can be removed.
    fn names(&mut self, node: &Node<'_>, scope: Scope<'_>, bare: bool) -> Vec<ThingId> {
        let names: Vec<&Node<'_>> = nodes(&node.children)
            .filter(|n| n.kind == NodeKind::Name)
            .collect();
        let spans: Vec<Range<usize>> = names.iter().map(|name| span(name)).collect();
        // A name and the comma and spaces up to the next name; the last name
        // takes the comma before it instead.
        let removable = |i: usize| match (i.checked_sub(1), spans.get(i + 1)) {
            _ if !bare || spans.len() < 2 => None,
            (_, Some(next)) => Some(spans[i].start..next.start),
            (Some(before), None) => Some(spans[before].end..spans[i].end),
            (None, None) => None,
        };
        names
            .iter()
            .enumerate()
            .map(|(i, name)| self.mention(name, scope, removable(i)))
            .collect()
    }

    fn mention(
        &mut self,
        node: &Node<'_>,
        scope: Scope<'_>,
        removable: Option<Range<usize>>,
    ) -> ThingId {
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
        let span = span(node);
        things[id].mentions.push(span.clone());
        if let Scope::Inside(groups) = scope {
            self.place(id, groups, span, removable);
        }
        id
    }

    /// Makes the first of `groups` the parent of `id`, or records why not. A
    /// thing has one parent: the first group whose braces hold it. A name in
    /// the braces of a group that holds the thing already, at any depth, says
    /// nothing new and is not an error. `settle` decides that against the
    /// final parents.
    fn place(
        &mut self,
        id: ThingId,
        groups: &[ThingId],
        span: Range<usize>,
        removable: Option<Range<usize>>,
    ) {
        let [group, ..] = *groups else {
            return;
        };
        let parent = self.model.things[id].parent;
        let can_enter = !self.within(group, id);
        let (why, removable) = if groups.len() > 1 {
            // The model still places the part in the first group when it can.
            if parent.is_none() && can_enter {
                self.model.things[id].parent = Some(group);
            }
            (Why::Shared(groups.to_vec()), None)
        } else if !can_enter {
            (Why::Loop, removable)
        } else if let Some(holder) = parent.filter(|_| !self.within(id, group)) {
            (Why::Held(holder), removable)
        } else {
            if parent.is_none() {
                self.model.things[id].parent = Some(group);
            }
            return;
        };
        self.model.misplaced.push(Misplaced {
            thing: id,
            group,
            span,
            why,
            removable,
        });
    }

    /// Drops each `Held` that the final parents make sound. A later line can
    /// place the holder inside the group of the braces, so a check at the
    /// mention depends on line order. A parent never changes once set, so a
    /// `Loop` stays true and needs no second look.
    fn settle(&mut self) {
        let misplaced = std::mem::take(&mut self.model.misplaced);
        self.model.misplaced = misplaced
            .into_iter()
            .filter(|wrong| {
                !matches!(wrong.why, Why::Held(_)) || !self.within(wrong.thing, wrong.group)
            })
            .collect();
    }

    /// Whether `inner` is `outer` or inside it, at any depth.
    fn within(&self, inner: ThingId, outer: ThingId) -> bool {
        let mut at = Some(inner);
        while let Some(thing) = at {
            if thing == outer {
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
