//! Writes the bundled metrics table of the horsegirl core to standard output.
//!
//! Usage: text-metrics <font file>
//!
//! The table holds the advance of every mapped character and the adjustment
//! for each pair of covered characters that the shaper moves. Both come from
//! the shaper itself, so the table repeats what the shaper does for one and
//! two characters, and differs only where a longer context matters (SPIKE-19).

use std::fmt::Write as _;
use std::process::ExitCode;

use harfrust::{Buffer, Direction, Font, ShapeOptions, ShaperFont, shape};

/// Pairs are generated only inside these blocks, to bound the table: Basic
/// Latin, Latin-1 and Latin Extended-A, Greek, Cyrillic, and the dashes and
/// quotes of General Punctuation.
fn kerned(c: char) -> bool {
    let c = u32::from(c);
    (0x20..=0x7e).contains(&c)
        || (0xa0..=0x17f).contains(&c)
        || (0x370..=0x4ff).contains(&c)
        || (0x2010..=0x201f).contains(&c)
}

/// Gives each index the class of every index with an equal key. Class 0 is
/// the key of all zeros, with no representative: a character outside the
/// kerned blocks also gets class 0, so class 0 must mean "no adjustment".
fn classes(
    n: usize,
    key: impl Fn(usize) -> Vec<i32>,
) -> Result<(Vec<u8>, Vec<Option<usize>>), String> {
    let mut keys: Vec<Vec<i32>> = vec![vec![0; n]];
    let mut firsts: Vec<Option<usize>> = vec![None];
    let mut class = Vec::with_capacity(n);
    for i in 0..n {
        let k = key(i);
        let c = if let Some(c) = keys.iter().position(|f| *f == k) {
            c
        } else {
            keys.push(k);
            firsts.push(Some(i));
            firsts.len() - 1
        };
        class.push(u8::try_from(c).map_err(|_| "the font needs more than 256 pair classes")?);
    }
    Ok((class, firsts))
}

fn advance(font: &ShaperFont, buffer: &mut Buffer, text: &str) -> Result<i32, String> {
    buffer.clear();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.guess_segment_properties();
    shape(font, buffer, ShapeOptions::new()).map_err(|e| format!("shape {text:?}: {e:?}"))?;
    Ok(buffer.glyph_positions().iter().map(|p| p.x_advance).sum())
}

/// Every character as an escape, so the source holds no invisible or
/// right-to-left character.
fn literal(c: char) -> String {
    if (c.is_ascii_graphic() && c != '\'' && c != '\\') || c == ' ' {
        format!("'{c}'")
    } else {
        format!("'\\u{{{:x}}}'", u32::from(c))
    }
}

fn generate(bytes: Vec<u8>) -> Result<String, String> {
    let font = Font::new(bytes, 0).ok_or("the file is not a font")?;
    let shaper = ShaperFont::new(&font);
    let mut buffer = Buffer::new();

    let metrics = font.metrics();
    let line = metrics.hhea_line.ok_or("the font has no hhea table")?;
    let ascender = i16::try_from(line.ascender.to_i32()).map_err(|e| e.to_string())?;
    let descender = i16::try_from(line.descender.to_i32()).map_err(|e| e.to_string())?;

    let mut chars: Vec<char> = font
        .charmap()
        .unicodes()
        .filter_map(|(c, _)| char::from_u32(c))
        .filter(|c| !c.is_control())
        .collect();
    chars.sort_unstable();
    chars.dedup();

    let mut advances = Vec::with_capacity(chars.len());
    for &c in &chars {
        let a = advance(&shaper, &mut buffer, c.encode_utf8(&mut [0; 4]))?;
        let a = u16::try_from(a).map_err(|_| format!("the advance of {c:?} is {a}"))?;
        advances.push((c, a));
    }

    // The shaper's adjustment for every ordered pair of covered characters.
    let covered: Vec<(char, i32)> = advances
        .iter()
        .filter(|(c, _)| kerned(*c))
        .map(|&(c, a)| (c, i32::from(a)))
        .collect();
    let n = covered.len();
    let mut adjust = vec![0i32; n * n];
    for (i, &(l, la)) in covered.iter().enumerate() {
        for (j, &(r, ra)) in covered.iter().enumerate() {
            let text: String = [l, r].iter().collect();
            adjust[i * n + j] = advance(&shaper, &mut buffer, &text)? - la - ra;
        }
    }

    // Characters whose rows are equal share a left class, and characters whose
    // columns are equal share a right class, as kerning classes do in a font.
    // The value of a pair then depends only on its two classes.
    let row = |i: usize| adjust[i * n..(i + 1) * n].to_vec();
    let col = |j: usize| (0..n).map(|i| adjust[i * n + j]).collect::<Vec<_>>();
    let (left, lefts) = classes(n, row)?;
    let (right, rights) = classes(n, col)?;
    let mut matrix = vec![0i16; lefts.len() * rights.len()];
    for (a, i) in lefts.iter().enumerate() {
        for (b, j) in rights.iter().enumerate() {
            if let (Some(i), Some(j)) = (i, j) {
                let v = adjust[i * n + j];
                matrix[a * rights.len() + b] = i16::try_from(v)
                    .map_err(|_| format!("the pair adjustment {v} is too large"))?;
            }
        }
    }
    let class_of = |c: char, classes: &[u8]| {
        covered
            .iter()
            .position(|&(k, _)| k == c)
            .map_or(0, |i| classes[i])
    };

    let upm = metrics.units_per_em;
    let mut out = format!(
        "// @generated by tools/text-metrics/generate.sh. Do not edit.
//
// Liberation Sans 2.1.5 Regular, which has the advances of Arial.
// The font is under the SIL Open Font License 1.1.
// The table holds measurements of it, not its outlines.

pub(super) const UNITS_PER_EM: u16 = {upm};
pub(super) const ASCENDER: i16 = {ascender};
pub(super) const DESCENDER: i16 = {descender};
pub(super) const RIGHT_CLASSES: usize = {right_classes};

/// Each character, its advance, its left class and its right class, sorted
/// by character.
pub(super) static ADVANCES: [(char, u16, u8, u8); {count}] = [
",
        right_classes = rights.len(),
        count = advances.len(),
    );
    for &(c, a) in &advances {
        let (l, r) = (class_of(c, &left), class_of(c, &right));
        writeln!(out, "({}, {a}, {l}, {r}),", literal(c)).map_err(|e| e.to_string())?;
    }
    write!(
        out,
        "];

/// The adjustment for each left class, row by row, against each right class.
pub(super) static PAIRS: [i16; {}] = {matrix:?};
",
        matrix.len()
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: text-metrics <font file>");
        return ExitCode::from(2);
    };
    let result = std::fs::read(&path)
        .map_err(|e| format!("read {path}: {e}"))
        .and_then(generate);
    match result {
        Ok(table) => {
            print!("{table}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("text-metrics: {e}");
            ExitCode::FAILURE
        }
    }
}
