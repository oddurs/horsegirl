//! Compares each way of measuring with the browser's widths.
//!
//! Usage: compare browser.tsv <face> [--rows]
//! The file has a header row, then a label and one width in pixels per face.
//! The named face is the one this build measures; the others are fallbacks.

use text_metrics_spike::{shaped, table};

const SIZE: f64 = 12.0;

struct Stats {
    name: String,
    errors: Vec<f64>,
}

impl Stats {
    fn print(&self) {
        let n = self.errors.len();
        let abs: Vec<f64> = self.errors.iter().map(|e| e.abs()).collect();
        let mean = abs.iter().sum::<f64>() / n as f64;
        let max = abs.iter().copied().fold(0.0, f64::max);
        let over1 = abs.iter().filter(|e| **e > 0.01).count();
        let over3 = abs.iter().filter(|e| **e > 0.03).count();
        let narrower = self.errors.iter().filter(|e| **e < -0.005).count();
        println!(
            "{:<34} {:>8.3}% {:>8.3}% {:>6} {:>6} {:>9}",
            self.name,
            mean * 100.0,
            max * 100.0,
            over1,
            over3,
            narrower
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: compare browser.tsv <face> [--rows]";
    let path = args.next().expect(usage);
    let face = args.next().expect(usage);
    let rows = args.next().as_deref() == Some("--rows");
    let text = std::fs::read_to_string(path).expect("read the browser widths");
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().expect("header").split('\t').collect();

    let font = shaped::font();
    let upm = f64::from(table::UNITS_PER_EM);
    let px = |units: i32| f64::from(units) * SIZE / upm;

    let mut plain = Stats {
        name: "table, advances only".into(),
        errors: vec![],
    };
    let mut kerned = Stats {
        name: "table, advances and pairs".into(),
        errors: vec![],
    };
    let mut shaping = Stats {
        name: "shaping crate".into(),
        errors: vec![],
    };
    let reference_column = header[1..]
        .iter()
        .position(|f| *f == face)
        .expect("the face is a column of the file");
    let mut fallbacks: Vec<(usize, Stats)> = header[1..]
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != reference_column)
        .map(|(i, f)| {
            let name = format!("browser, fallback face {f}");
            (
                i,
                Stats {
                    name,
                    errors: vec![],
                },
            )
        })
        .collect();

    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let label = cells[0];
        let browser: Vec<f64> = cells[1..]
            .iter()
            .map(|c| c.parse().expect("a width"))
            .collect();
        let reference = browser[reference_column];
        let rel = |w: f64| (w - reference) / reference;
        let (p, k, s) = (
            px(table::width_plain(label)),
            px(table::width(label)),
            px(shaped::width(&font, label)),
        );
        plain.errors.push(rel(p));
        kerned.errors.push(rel(k));
        shaping.errors.push(rel(s));
        for (i, stats) in &mut fallbacks {
            stats.errors.push(rel(browser[*i]));
        }
        if rows {
            println!("{label:<45} {reference:>9.3} {p:>9.3} {k:>9.3} {s:>9.3}");
        }
    }

    println!(
        "{:<34} {:>9} {:>9} {:>6} {:>6} {:>9}",
        "method (against the browser)", "mean", "max", ">1%", ">3%", "narrower"
    );
    plain.print();
    kerned.print();
    shaping.print();
    for (_, f) in &fallbacks {
        f.print();
    }
    println!(
        "table: {} advances, {} left by {} right pair classes, {} units per em",
        table::ADVANCES.len(),
        table::PAIRS.len() / table::RIGHT_CLASSES,
        table::RIGHT_CLASSES,
        table::UNITS_PER_EM
    );
}
