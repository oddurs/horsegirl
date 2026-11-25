//! The plain theme: its contrast, and its listing in the reference.

mod support;

use horsegirl_core::svg::draw;
use horsegirl_core::theme::{TOKENS, Token};
use support::scene;

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");
const REFERENCE: &str = include_str!("../../../docs/theme.md");

/// The relative luminance of a `#RRGGBB` colour, as WCAG 2 defines it.
fn luminance(hex: &str) -> f64 {
    let channel = |at: usize| {
        let v = f64::from(u8::from_str_radix(&hex[at..at + 2], 16).unwrap()) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
}

fn contrast(a: &str, b: &str) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn token(name: &str) -> Token {
    *TOKENS.iter().find(|t| t.name == name).unwrap()
}

#[test]
fn the_contrast_check_knows_black_on_white() {
    assert!((contrast("#000000", "#FFFFFF") - 21.0).abs() < 1e-9);
    assert!((contrast("#777777", "#FFFFFF") - 4.48).abs() < 0.01);
}

#[test]
fn the_text_of_the_reference_flow_has_4_5_to_1_in_both_modes() {
    let svg = draw(&scene(CHECKOUT), "checkout");
    // Each fill that a label of the picture sits on.
    let mut fills = vec!["paper", "c0"];
    for slot in 1..=3 {
        if svg.contains(&format!("slot-{slot}")) {
            fills.push(["c1", "c2", "c3"][slot - 1]);
        }
    }
    assert_eq!(fills.len(), 5, "the flow has three owners to colour");
    for fill in fills {
        let (ink, fill) = (token("ink"), token(fill));
        assert!(
            contrast(ink.light, fill.light) >= 4.5,
            "light {}",
            fill.name
        );
        assert!(contrast(ink.dark, fill.dark) >= 4.5, "dark {}", fill.name);
    }
    let (on, accent) = (token("accent-ink"), token("accent"));
    assert!(contrast(on.light, accent.light) >= 4.5);
    assert!(contrast(on.dark, accent.dark) >= 4.5);
}

#[test]
fn the_style_paints_the_text_with_ink_and_each_owner_with_its_slot() {
    let svg = draw(&scene(CHECKOUT), "checkout");
    assert!(svg.contains(
        ".horsegirl .label{fill:var(--hg-ink,light-dark(#1F2328,#E6E8EB));white-space:pre}"
    ));
    for slot in 1..=3 {
        assert!(svg.contains(&format!(".thing.slot-{slot}{{fill:var(--hg-c{slot},")));
    }
    // Five owners wrap round three slots.
    assert!(svg.contains("class=\"thing slot-1 owner-apps\""));
    assert!(svg.contains("class=\"thing slot-3 owner-commerce\""));
    assert!(svg.contains("class=\"thing slot-1 owner-growth\""));
}

#[test]
fn twelve_tokens_are_listed_in_the_reference_with_both_values() {
    assert_eq!(TOKENS.len(), 12);
    for t in TOKENS {
        let row = format!(
            "| `--hg-{}` | `{}` | `{}` | {} |",
            t.name, t.light, t.dark, t.role
        );
        assert!(REFERENCE.contains(&row), "docs/theme.md lacks: {row}");
    }
}
