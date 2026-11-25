//! A fuzz run of the parser: no input makes it panic or lose a byte.
//!
//! The inputs come from a fixed seed, so a failure repeats. The test runs a
//! short round on every target. For a long run, set the number of rounds:
//!
//! ```sh
//! HORSEGIRL_FUZZ_ROUNDS=40000000 cargo test --release -p horsegirl-core --test fuzz
//! ```

use horsegirl_core::syntax::{Element, Node, NodeKind, Token, parse};

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");

/// A file that uses each form the parser reads, right and wrong.
const FORMS: &str = "\
# a comment
\"Sensors: wind\", Web app -> Hub: \"GET /users/{id}\" ?  @ src/a.ts:17
A -x B: never, ever -> not: once
G { A, B { owner: X, note: \"install, list\" } ? }
story checkout {
  A -> B: pay
}
show sequence of checkout
show rings from \"API gateway\"
owner: X
A -> -> B
A -> \"B
Renderer PixiJS / @pixi/react -> B: mail bob@example.com
A {x} -> B
";

/// The pieces that random input is made from. The marks of the language come
/// more often than they do in real text, so the parser meets odd orders.
const PIECES: [&str; 40] = [
    "->",
    "-x",
    "-",
    ">",
    ",",
    ":",
    "{",
    "}",
    "@",
    "?",
    "\"",
    "#",
    " ",
    " ",
    " ",
    "  ",
    "\t",
    "\n",
    "\n",
    "\n",
    "\r\n",
    "\r",
    "show",
    "story",
    "of",
    "from",
    "by",
    "flow",
    "A",
    "B",
    "Web app",
    "owner",
    "-xy",
    "a?",
    "é",
    "名",
    "\u{feff}",
    "\u{2028}",
    "\"a, b\"",
    "src/a.rs:3",
];

/// Xorshift: small, and the same on every target.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, limit: usize) -> usize {
        usize::try_from(self.next() % limit as u64).unwrap()
    }

    fn piece(&mut self) -> &'static str {
        PIECES[self.below(PIECES.len())]
    }
}

/// Random pieces in a row.
fn soup(random: &mut Random) -> String {
    let length = random.below(60);
    (0..length).map(|_| random.piece()).collect()
}

/// A good file with a few random edits: a piece added, a part removed, or a
/// part repeated.
fn mutant(random: &mut Random, seed: &str) -> String {
    let mut chars: Vec<char> = seed.chars().collect();
    for _ in 0..=random.below(6) {
        let at = random.below(chars.len() + 1);
        let end = (at + random.below(12)).min(chars.len());
        match random.below(3) {
            0 => drop(chars.splice(at..at, random.piece().chars())),
            1 => drop(chars.drain(at..end)),
            _ => {
                let part = chars[at..end].to_vec();
                drop(chars.splice(at..at, part));
            }
        }
    }
    chars.into_iter().collect()
}

/// Checks what every tree promises, and returns how deep it nests.
fn check<'s>(node: &Node<'s>, tokens: &mut Vec<Token<'s>>) -> usize {
    let mut depth = 0;
    for child in &node.children {
        match child {
            Element::Token(token) => tokens.push(*token),
            Element::Node(inner) => {
                // The model skips an `Error` node, so it must hold no half-read fact.
                assert!(
                    node.kind != NodeKind::Error || inner.kind == NodeKind::Block,
                    "an Error node holds a {:?}",
                    inner.kind
                );
                depth = depth.max(check(inner, tokens));
            }
        }
    }
    depth + 1
}

fn exercise(source: &str) {
    let result = parse(source);
    assert!(
        result.tree.to_string() == source,
        "lost a byte of {source:?}"
    );

    let mut tokens = Vec::new();
    let depth = check(&result.tree, &mut tokens);
    // Four nodes for each level of braces, and 64 levels at most.
    assert!(depth < 300, "the tree of {source:?} is {depth} deep");
    let mut end = 0;
    for token in &tokens {
        assert!(
            token.start == end && !token.text.is_empty(),
            "{token:?} in {source:?}"
        );
        end = token.span().end;
    }

    let mut last = 0;
    for diagnostic in &result.diagnostics {
        let spans = [
            Some(&diagnostic.span),
            diagnostic.fix.as_ref().map(|fix| &fix.span),
        ];
        for span in spans.into_iter().flatten() {
            // `get` is `None` for a span that is reversed, past the end, or inside a character.
            assert!(
                source.get(span.clone()).is_some(),
                "{diagnostic:?} in {source:?}"
            );
        }
        assert!(
            diagnostic.span.start >= last,
            "{diagnostic:?} is out of order in {source:?}"
        );
        last = diagnostic.span.start;
    }

    assert!(parse(source) == result, "two runs differ on {source:?}");
}

#[test]
fn no_input_makes_the_parser_panic_or_lose_a_byte() {
    let rounds =
        std::env::var("HORSEGIRL_FUZZ_ROUNDS").map_or(3_000, |rounds| rounds.parse().unwrap());
    let mut random = Random(0x9E37_79B9_7F4A_7C15);
    for _ in 0..rounds {
        exercise(&soup(&mut random));
        exercise(&mutant(&mut random, CHECKOUT));
        exercise(&mutant(&mut random, FORMS));
    }
}
