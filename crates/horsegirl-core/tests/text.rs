//! Text measurement against the browser, and against itself on every target.
//!
//! The labels and the browser widths are the evidence of SPIKE-19:
//! `spikes/0019-text-metrics/browser.tsv` holds the width of each label at
//! 12 px in headless Chromium, in Liberation Sans 2.1.5 (the `arial-metrics`
//! column). CI runs these tests on Linux, on macOS and in WebAssembly.

use std::fmt::Write as _;

use horsegirl_core::text::{Style, measure};

const BROWSER: &str = include_str!("../../../spikes/0019-text-metrics/browser.tsv");
const WIDTHS: &str = include_str!("text-widths.tsv");
const STYLE: Style = Style { size: 12.0 };

/// Each label and its browser width in the face the table is made from.
fn browser() -> Vec<(&'static str, f64)> {
    let mut lines = BROWSER.lines();
    let header: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let column = header.iter().position(|h| *h == "arial-metrics").unwrap();
    lines
        .map(|line| {
            let cells: Vec<&str> = line.split('\t').collect();
            (cells[0], cells[column].parse().unwrap())
        })
        .collect()
}

#[test]
fn every_reference_label_is_within_three_percent_of_the_browser() {
    let labels = browser();
    assert_eq!(labels.len(), 108);
    let mut worst = (0.0_f64, "");
    for &(label, expected) in &labels {
        let error = (measure(label, STYLE).width - expected).abs() / expected;
        if error > worst.0 {
            worst = (error, label);
        }
    }
    assert!(
        worst.0 < 0.03,
        "{:?} is {:.3}% from the browser",
        worst.1,
        worst.0 * 100.0
    );
}

// The widths are written with the shortest text that reads back to the same
// f64, so an equal file means equal bits. A target that rounds differently
// fails here.
#[test]
fn every_target_gives_the_same_bits() {
    let mut actual = String::from("label\twidth\theight\n");
    for (label, _) in browser() {
        let size = measure(label, STYLE);
        writeln!(actual, "{label}\t{:?}\t{:?}", size.width, size.height).unwrap();
    }
    assert!(
        actual == WIDTHS,
        "the widths moved; they are now:\n{actual}"
    );
}
