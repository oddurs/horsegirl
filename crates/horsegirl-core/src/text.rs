//! Text measurement with a bundled metrics table.
//!
//! A node is as wide as its label, so the width must be the same on every
//! machine and in WebAssembly. The core reads no font. It sums integer font
//! units from a table generated from Liberation Sans 2.1.5, which has the
//! advances of Arial (SPIKE-19), and converts the sum to pixels in one step.

mod metrics;

use metrics::{ADVANCES, ASCENDER, DESCENDER, PAIRS, RIGHT_CLASSES, UNITS_PER_EM};

/// How a label is drawn.
///
/// The table holds one face at one weight, so the size is the only choice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    /// The font size in pixels: the length of one em.
    pub size: f64,
}

/// The box that a line of text fills, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    /// The ascender to the descender of the face. It does not depend on the text.
    pub height: f64,
}

/// Measures one line of text.
///
/// The width is the sum of the advance of each character and the pair
/// adjustment of each two adjacent characters. A character that the table
/// does not hold measures as one em, so an unknown label grows rather than
/// overflows its box.
#[must_use]
pub fn measure(text: &str, style: Style) -> Size {
    let mut units = 0.0;
    let mut previous: Option<Entry> = None;
    for c in text.chars() {
        let entry = entry(c);
        units += entry.map_or(f64::from(UNITS_PER_EM), |e| f64::from(e.1));
        if let (Some(l), Some(r)) = (previous, entry) {
            units += f64::from(pair(l, r));
        }
        previous = entry;
    }
    let scale = style.size / f64::from(UNITS_PER_EM);
    Size {
        width: units * scale,
        height: f64::from(i32::from(ASCENDER) - i32::from(DESCENDER)) * scale,
    }
}

/// A character, its advance, its left pair class and its right pair class.
type Entry = (char, u16, u8, u8);

fn entry(c: char) -> Option<Entry> {
    ADVANCES
        .binary_search_by_key(&c, |e| e.0)
        .ok()
        .map(|i| ADVANCES[i])
}

fn pair(left: Entry, right: Entry) -> i16 {
    PAIRS[usize::from(left.2) * RIGHT_CLASSES + usize::from(right.3)]
}

#[cfg(test)]
mod tests {
    use super::{ADVANCES, Style, UNITS_PER_EM, entry, measure, pair};

    fn width(text: &str) -> f64 {
        measure(text, Style { size: 12.0 }).width
    }

    #[test]
    fn the_table_is_sorted_so_a_binary_search_finds_each_character() {
        assert!(ADVANCES.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn a_character_outside_the_table_measures_one_em() {
        let size = Style { size: 12.0 };
        assert_eq!(measure("\u{4e00}", size).width, 12.0);
        assert_eq!(measure("\u{4e00}\u{4e01}", size).width, 24.0);
        assert!(entry('\u{4e00}').is_none());
    }

    #[test]
    fn empty_text_has_no_width_and_one_line_of_height() {
        let size = measure("", Style { size: 12.0 });
        assert_eq!(size.width, 0.0);
        assert!((size.height - 12.0 * 2288.0 / 2048.0).abs() < 1e-12);
    }

    #[test]
    fn width_scales_with_the_font_size() {
        let small = measure("Payments", Style { size: 12.0 }).width;
        let large = measure("Payments", Style { size: 24.0 }).width;
        assert_eq!(large, 2.0 * small);
    }

    #[test]
    fn a_kerned_pair_is_narrower_than_its_advances() {
        let advances = ['A', 'V']
            .iter()
            .map(|&c| f64::from(entry(c).unwrap().1))
            .sum::<f64>();
        assert!(width("AV") < advances * 12.0 / f64::from(UNITS_PER_EM));
    }

    // An arrow and the Hebrew letter alef are in the table and outside the
    // kerned blocks. Their class is 0, which must not take the adjustments of
    // another class.
    #[test]
    fn a_character_outside_the_kerned_blocks_has_no_pair_adjustment() {
        for c in ['→', '\u{5d0}'] {
            let e = entry(c).unwrap();
            for k in [' ', 'A', 'T', 'V', 'Y', '1', '“'] {
                let k = entry(k).unwrap();
                assert_eq!(pair(e, k), 0, "pair {c:?} {:?}", k.0);
                assert_eq!(pair(k, e), 0, "pair {:?} {c:?}", k.0);
            }
        }
    }
}
