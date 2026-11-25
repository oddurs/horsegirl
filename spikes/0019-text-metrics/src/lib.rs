//! Two ways to measure the advance of a label, in font units.

/// The bundled table: advances and pair adjustments, no font file.
pub mod table {
    include!(concat!(env!("OUT_DIR"), "/table.rs"));

    fn entry(c: char) -> Option<&'static (char, u16, u8, u8)> {
        ADVANCES
            .binary_search_by_key(&c, |e| e.0)
            .ok()
            .map(|i| &ADVANCES[i])
    }

    /// A character the table does not hold is measured as one em, so an
    /// unknown label grows rather than overflows.
    #[must_use]
    pub fn advance(c: char) -> i32 {
        entry(c).map_or(i32::from(UNITS_PER_EM), |e| i32::from(e.1))
    }

    #[must_use]
    pub fn pair(l: char, r: char) -> i32 {
        match (entry(l), entry(r)) {
            (Some(a), Some(b)) => {
                i32::from(PAIRS[usize::from(a.2) * RIGHT_CLASSES + usize::from(b.3)])
            }
            _ => 0,
        }
    }

    /// The sum of advances, with no pair adjustments.
    #[must_use]
    pub fn width_plain(text: &str) -> i32 {
        text.chars().map(advance).sum()
    }

    /// The sum of advances and the adjustment for each adjacent pair.
    #[must_use]
    pub fn width(text: &str) -> i32 {
        let chars: Vec<char> = text.chars().collect();
        let pairs: i32 = chars.windows(2).map(|w| pair(w[0], w[1])).sum();
        width_plain(text) + pairs
    }
}

#[cfg(test)]
mod tests {
    use super::table::{pair, width, width_plain};

    // An arrow is in every reference face and outside the kerned blocks. A
    // character outside them must not take the adjustments of another class.
    #[test]
    fn a_character_outside_the_kerned_blocks_has_no_pair_adjustment() {
        for c in ['→', '\u{4e00}'] {
            for k in [' ', 'A', 'T', 'V', 'Y', '1', '“'] {
                assert_eq!(pair(c, k), 0, "pair {c:?} {k:?}");
                assert_eq!(pair(k, c), 0, "pair {k:?} {c:?}");
            }
        }
        assert_eq!(width("A→A"), width_plain("A→A"));
    }
}

/// The font file itself, shaped by a port of `HarfBuzz`.
#[cfg(feature = "shape")]
pub mod shaped {
    use harfrust::{Buffer, Direction, Font, ShapeOptions, ShaperFont, shape};

    pub static FONT: &[u8] = include_bytes!(env!("SPIKE_FONT"));

    #[must_use]
    pub fn width(font: &Font, text: &str) -> i32 {
        let shaper = ShaperFont::new(font);
        let mut buffer = Buffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.guess_segment_properties();
        shape(&shaper, &mut buffer, ShapeOptions::new()).expect("shape");
        buffer.glyph_positions().iter().map(|p| p.x_advance).sum()
    }

    #[must_use]
    pub fn font() -> Font {
        Font::new(FONT, 0).expect("the bundled font parses")
    }
}
