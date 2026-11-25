//! The plain theme: twelve colour tokens, each with a light and a dark value.
//!
//! The values are the `--d-*` variables of the plain theme in
//! `docs/concept/index.html`. The SVG backend writes each token as the CSS
//! variable `--hg-<name>` with the light and dark value as its fallback, and
//! `docs/theme.md` lists them. A test holds the three in step.

/// A colour token of the plain theme.
#[derive(Clone, Copy, Debug)]
pub struct Token {
    /// The name without the `--hg-` prefix.
    pub name: &'static str,
    /// The value in light mode, as a hex colour.
    pub light: &'static str,
    /// The value in dark mode, as a hex colour.
    pub dark: &'static str,
    /// What the token paints.
    pub role: &'static str,
}

/// The font stack of every label. The core measures text in this face, so a
/// theme that names another face keeps every position.
pub const FONT: &str = "Arial,\"Liberation Sans\",Helvetica,sans-serif";

/// The twelve tokens, in the order the documentation lists them.
pub const TOKENS: [Token; 12] = [
    Token {
        name: "ink",
        light: "#1F2328",
        dark: "#E6E8EB",
        role: "Text and arrow strokes",
    },
    Token {
        name: "paper",
        light: "#FFFFFF",
        dark: "#15171B",
        role: "The fill of a thing",
    },
    Token {
        name: "line",
        light: "#D8DCE2",
        dark: "#343941",
        role: "The stroke of a group",
    },
    Token {
        name: "muted",
        light: "#667080",
        dark: "#9AA4B2",
        role: "The stroke of a thing",
    },
    Token {
        name: "accent",
        light: "#2F5FD0",
        dark: "#7AA2FF",
        role: "Arrow heads and the first owner stroke",
    },
    Token {
        name: "accent-ink",
        light: "#FFFFFF",
        dark: "#0B1020",
        role: "Text on an accent fill",
    },
    Token {
        name: "c0",
        light: "#F3F4F6",
        dark: "#23272E",
        role: "The fill of a group",
    },
    Token {
        name: "c1",
        light: "#E1EAFB",
        dark: "#1D2B47",
        role: "The fill of the first owner",
    },
    Token {
        name: "c2",
        light: "#E0F2E8",
        dark: "#17322A",
        role: "The fill of the second owner",
    },
    Token {
        name: "c3",
        light: "#FAEFD6",
        dark: "#3A2F17",
        role: "The fill of the third owner",
    },
    Token {
        name: "s2",
        light: "#1F7A55",
        dark: "#5FCB9B",
        role: "The stroke of the second owner",
    },
    Token {
        name: "s3",
        light: "#A8621B",
        dark: "#E3A75C",
        role: "The stroke of the third owner",
    },
];

/// The value of token `name` as CSS: the page's own `--hg-<name>`, or the
/// light and dark value when the page sets none.
///
/// # Panics
///
/// Panics when `name` is not a token. The callers in this crate pass literals.
#[must_use]
pub fn var(name: &str) -> String {
    let t = TOKENS
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("{name} is not a token of the plain theme"));
    format!("var(--hg-{name},light-dark({},{}))", t.light, t.dark)
}
