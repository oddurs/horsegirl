//! Generates the bundled metrics table from the reference font.
//!
//! The table holds the advance of every mapped character and the adjustment
//! for each pair of covered characters that the shaper moves. Both come from
//! the shaper itself, so the table repeats what the shaper does for one and
//! two characters, and differs only where a longer context matters.

use std::fmt::Write as _;
use std::path::PathBuf;

use harfrust::{Buffer, Direction, Font, ShapeOptions, ShaperFont, shape};

/// Pairs are generated only inside these blocks, to bound the table: Basic
/// Latin, Latin-1 and Latin Extended-A, Greek, Cyrillic, and the dashes and
/// quotes of General Punctuation.
fn kerned(c: u32) -> bool {
    (0x20..=0x7e).contains(&c)
        || (0xa0..=0x17f).contains(&c)
        || (0x370..=0x4ff).contains(&c)
        || (0x2010..=0x201f).contains(&c)
}

/// Gives each index the class of every index with an equal key. Class 0 is
/// the key of all zeros, with no representative: a character outside the
/// kerned blocks also gets class 0, so class 0 must mean "no adjustment".
fn classes(n: usize, key: impl Fn(usize) -> Vec<i32>) -> (Vec<u8>, Vec<Option<usize>>) {
    let mut keys: Vec<Vec<i32>> = vec![vec![0; n]];
    let mut firsts: Vec<Option<usize>> = vec![None];
    let mut class = Vec::with_capacity(n);
    for i in 0..n {
        let k = key(i);
        let c = keys.iter().position(|f| *f == k).unwrap_or_else(|| {
            keys.push(k);
            firsts.push(Some(i));
            firsts.len() - 1
        });
        class.push(u8::try_from(c).expect("fewer than 256 classes"));
    }
    (class, firsts)
}

fn advance(font: &ShaperFont, buffer: &mut Buffer, text: &str) -> i32 {
    buffer.clear();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.guess_segment_properties();
    shape(font, buffer, ShapeOptions::new()).expect("shape");
    buffer.glyph_positions().iter().map(|p| p.x_advance).sum()
}

fn main() {
    // FONT picks another face to measure, such as a fallback the browser drew.
    println!("cargo::rerun-if-env-changed=FONT");
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = std::env::var("FONT").map_or_else(
        |_| manifest.join("font/InstrumentSans.ttf"),
        |f| manifest.join(f),
    );
    println!("cargo::rerun-if-changed={}", path.display());
    println!("cargo::rustc-env=SPIKE_FONT={}", path.display());
    let bytes = std::fs::read(&path).expect("read the font");
    let font = Font::new(bytes, 0).expect("font");
    let shaper = ShaperFont::new(&font);
    let mut buffer = Buffer::new();

    let mut chars: Vec<char> = font
        .charmap()
        .unicodes()
        .filter_map(|(c, _)| char::from_u32(c))
        .filter(|c| !c.is_control())
        .collect();
    chars.sort_unstable();
    chars.dedup();

    let mut advances = Vec::new();
    for &c in &chars {
        let a = advance(&shaper, &mut buffer, c.encode_utf8(&mut [0; 4]));
        advances.push((c, u16::try_from(a).expect("advance fits u16")));
    }

    // The shaper's adjustment for every ordered pair of covered characters.
    let covered: Vec<(char, i32)> = advances
        .iter()
        .filter(|(c, _)| kerned(*c as u32))
        .map(|&(c, a)| (c, i32::from(a)))
        .collect();
    let n = covered.len();
    let mut adjust = vec![0i32; n * n];
    for (i, &(l, la)) in covered.iter().enumerate() {
        for (j, &(r, ra)) in covered.iter().enumerate() {
            let text: String = [l, r].iter().collect();
            adjust[i * n + j] = advance(&shaper, &mut buffer, &text) - la - ra;
        }
    }

    // Characters whose rows are equal share a left class, and characters whose
    // columns are equal share a right class, as kerning classes do in a font.
    // The value of a pair then depends only on its two classes.
    let row = |i: usize| adjust[i * n..(i + 1) * n].to_vec();
    let col = |j: usize| (0..n).map(|i| adjust[i * n + j]).collect::<Vec<_>>();
    let (left, lefts) = classes(n, row);
    let (right, rights) = classes(n, col);
    let mut matrix = vec![0i16; lefts.len() * rights.len()];
    for (a, i) in lefts.iter().enumerate() {
        for (b, j) in rights.iter().enumerate() {
            if let (Some(i), Some(j)) = (i, j) {
                matrix[a * rights.len() + b] = i16::try_from(adjust[i * n + j]).expect("fits i16");
            }
        }
    }
    let class_of = |c: char, classes: &[u8]| {
        covered
            .iter()
            .position(|&(k, _)| k == c)
            .map_or(0, |i| classes[i])
    };

    let mut out = String::new();
    writeln!(
        out,
        "pub const UNITS_PER_EM: u16 = {};",
        font.units_per_em()
    )
    .unwrap();
    writeln!(out, "pub const RIGHT_CLASSES: usize = {};", rights.len()).unwrap();
    writeln!(
        out,
        "pub static ADVANCES: [(char, u16, u8, u8); {}] = [",
        advances.len()
    )
    .unwrap();
    for &(c, a) in &advances {
        let (l, r) = (class_of(c, &left), class_of(c, &right));
        writeln!(out, "({c:?}, {a}, {l}, {r}),").unwrap();
    }
    writeln!(out, "];").unwrap();
    writeln!(
        out,
        "pub static PAIRS: [i16; {}] = {matrix:?};",
        matrix.len()
    )
    .unwrap();

    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("table.rs");
    std::fs::write(path, out).unwrap();
}
