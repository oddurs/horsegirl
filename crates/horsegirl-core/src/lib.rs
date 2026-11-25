//! horsegirl: an agent-first diagram language.
//!
//! The crate is at the start of its roadmap. Today it carries the one-page
//! language card that an agent reads before it writes a file, the parser that
//! reads a file into a lossless tree, the model that the tree lowers into, and
//! the text measurement that sizes each node to its label.

pub mod model;
pub mod syntax;
pub mod text;

/// The whole language on one page.
///
/// An agent has never seen horsegirl, so the specification must fit in a
/// prompt. This text is that specification, and `horsegirl spec` prints it.
pub const SPEC: &str = include_str!("spec.txt");

/// The number of rules in the language card.
///
/// The count is part of the promise ("five rules"), so a test holds it.
#[must_use]
pub fn rule_count() -> usize {
    SPEC.lines()
        .filter(|line| line.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_card_has_five_rules() {
        assert_eq!(rule_count(), 5);
    }

    #[test]
    fn the_card_fits_in_a_prompt() {
        // One page: a model reads it once. Growth past this needs a decision.
        assert!(SPEC.split_whitespace().count() < 200);
    }
}
