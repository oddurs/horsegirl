//! The checker of a task: the facts that a correct answer states.
//!
//! A checker is written in horsegirl. Each thing, part, fact, link, rule and
//! `show` line in it is one expectation, and an answer meets it when its own
//! model states the same. An answer may state more than the checker.
//!
//! Names, labels and values match without regard to case, because the card
//! does not say that case matters and a model may capitalise a name. The order
//! of a story is not in the model yet, so a story is checked by its links.

use horsegirl_core::model::{Model, lower};
use horsegirl_core::syntax::parse;

/// One fact that a correct answer states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expectation {
    Thing(String),
    /// A part and the group that holds it.
    Part {
        part: String,
        group: String,
    },
    Fact {
        thing: String,
        key: String,
        value: String,
    },
    Link {
        from: String,
        to: String,
        labels: Vec<String>,
        guess: bool,
    },
    Rule {
        from: String,
        to: String,
        reason: Option<String>,
    },
    Show {
        lens: String,
        arg: Option<String>,
    },
}

impl Expectation {
    /// The expectation as a line of horsegirl, for the report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Expectation::Thing(name) => format!("thing {name}"),
            Expectation::Part { part, group } => format!("{group} {{ {part} }}"),
            Expectation::Fact { thing, key, value } => format!("{thing} {{ {key}: {value} }}"),
            Expectation::Link {
                from,
                to,
                labels,
                guess,
            } => {
                let label = if labels.is_empty() {
                    String::new()
                } else {
                    format!(": {}", labels.join(" / "))
                };
                let guess = if *guess { " ?" } else { "" };
                format!("{from} -> {to}{label}{guess}")
            }
            Expectation::Rule { from, to, reason } => match reason {
                Some(reason) => format!("{from} -x {to}: {reason}"),
                None => format!("{from} -x {to}"),
            },
            Expectation::Show { lens, arg } => match arg {
                Some(arg) => format!("show {lens} {arg}"),
                None => format!("show {lens}"),
            },
        }
    }
}

/// Every expectation of a checker, in the order of its model.
#[must_use]
pub fn expectations(checker: &str) -> Vec<Expectation> {
    let model = lower(&parse(checker).tree);
    let name = |id: usize| model.things[id].name.clone();
    let mut all = Vec::new();
    for thing in &model.things {
        all.push(Expectation::Thing(thing.name.clone()));
        if let Some(group) = thing.parent {
            all.push(Expectation::Part {
                part: thing.name.clone(),
                group: name(group),
            });
        }
        for fact in &thing.facts {
            all.push(Expectation::Fact {
                thing: thing.name.clone(),
                key: fact.key.clone(),
                value: fact.value.clone(),
            });
        }
    }
    for link in &model.links {
        all.push(Expectation::Link {
            from: name(link.from),
            to: name(link.to),
            labels: link.labels.clone(),
            guess: link.guess,
        });
    }
    for rule in &model.rules {
        all.push(Expectation::Rule {
            from: name(rule.from),
            to: name(rule.to),
            reason: rule.reason.clone(),
        });
    }
    for show in &model.shows {
        all.push(Expectation::Show {
            lens: show.lens.clone(),
            arg: show
                .arg
                .as_ref()
                .map(|(keyword, arg)| format!("{keyword} {arg}")),
        });
    }
    all
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The thing in `model` with this name, in any case.
fn find(model: &Model, name: &str) -> Option<usize> {
    model
        .things
        .iter()
        .position(|thing| same(&thing.name, name))
}

/// Whether `answer` states `expectation`.
fn meets(answer: &Model, expectation: &Expectation) -> bool {
    match expectation {
        Expectation::Thing(name) => find(answer, name).is_some(),
        Expectation::Part { part, group } => find(answer, part)
            .and_then(|id| answer.things[id].parent)
            .is_some_and(|parent| same(&answer.things[parent].name, group)),
        Expectation::Fact { thing, key, value } => find(answer, thing).is_some_and(|id| {
            answer.things[id]
                .facts
                .iter()
                .any(|fact| same(&fact.key, key) && same(&fact.value, value))
        }),
        Expectation::Link {
            from,
            to,
            labels,
            guess,
        } => {
            let (Some(from), Some(to)) = (find(answer, from), find(answer, to)) else {
                return false;
            };
            answer.links.iter().any(|link| {
                link.from == from
                    && link.to == to
                    && link.guess == *guess
                    && labels
                        .iter()
                        .all(|label| link.labels.iter().any(|known| same(known, label)))
            })
        }
        Expectation::Rule { from, to, reason } => {
            let (Some(from), Some(to)) = (find(answer, from), find(answer, to)) else {
                return false;
            };
            answer.rules.iter().any(|rule| {
                rule.from == from
                    && rule.to == to
                    && reason.as_ref().is_none_or(|reason| {
                        rule.reason
                            .as_ref()
                            .is_some_and(|known| same(known, reason))
                    })
            })
        }
        Expectation::Show { lens, arg } => answer.shows.iter().any(|show| {
            let shown = show
                .arg
                .as_ref()
                .map(|(keyword, arg)| format!("{keyword} {arg}"));
            same(&show.lens, lens)
                && match (arg, shown) {
                    (None, None) => true,
                    (Some(arg), Some(shown)) => same(arg, &shown),
                    _ => false,
                }
        }),
    }
}

/// Each expectation that the file `answer` does not meet, as a line of
/// horsegirl, in the order of the checker.
#[must_use]
pub fn missing(expectations: &[Expectation], answer: &str) -> Vec<String> {
    let model = lower(&parse(answer).tree);
    expectations
        .iter()
        .filter(|expectation| !meets(&model, expectation))
        .map(Expectation::describe)
        .collect()
}
