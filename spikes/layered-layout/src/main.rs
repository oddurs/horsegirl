//! Compares layered layout designs for SPIKE-18. Run it from this directory:
//!
//! ```text
//! cargo run --release -- draw   # writes pictures/<candidate>/<diagram>.svg and prints the scores
//! cargo run --release -- seed   # adds one thing to each diagram and counts the things that move,
//!                               # then seeds each diagram with its own picture and no edit
//! ```

mod crates;
mod measure;
mod model;
mod own;
mod svg;

use std::fs;

use model::{Graph, Picture, Seed};

fn corpus() -> Result<Vec<(String, Graph)>, String> {
    let mut paths: Vec<_> = fs::read_dir("corpus")
        .map_err(|e| format!("corpus: {e}"))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    paths.sort();
    let mut graphs = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let graph = model::parse(&source).map_err(|e| format!("{}: {e}", path.display()))?;
        graphs.push((name, graph));
    }
    Ok(graphs)
}

fn draw() -> Result<(), String> {
    let candidates = ["own", "dagre", "graphlay", "elk"];
    println!(
        "| diagram | things | links | candidate | size | overlaps | touching | group faults | \
         links through a box | crossings | bends | link length |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    let mut totals = [[0usize; 6]; 4];
    for (name, graph) in corpus()? {
        for (index, candidate) in candidates.iter().enumerate() {
            let picture = match *candidate {
                "own" => own::layout(&graph, None),
                "dagre" => crates::dagre(&graph),
                "graphlay" => crates::graphlay(&graph, None)?,
                _ => crates::elk(&graph, None)?,
            };
            let directory = format!("pictures/{candidate}");
            fs::create_dir_all(&directory).map_err(|e| format!("{directory}: {e}"))?;
            let path = format!("{directory}/{name}.svg");
            fs::write(&path, svg::draw(&graph, &picture)).map_err(|e| format!("{path}: {e}"))?;
            let s = measure::score(&graph, &picture);
            println!(
                "| {name} | {} | {} | {candidate} | {:.0} x {:.0} | {} | {} | {} | {} | {} | {} | {:.0} |",
                graph.nodes.len(),
                graph.edges.len(),
                s.width,
                s.height,
                s.overlaps,
                s.touching,
                s.group_faults,
                s.through_box + s.unrouted,
                s.crossings,
                s.bends,
                s.length,
            );
            let row = [
                s.overlaps,
                s.touching,
                s.group_faults,
                s.through_box + s.unrouted,
                s.crossings,
                s.bends,
            ];
            for (total, value) in totals[index].iter_mut().zip(row) {
                *total += value;
            }
        }
    }
    println!(
        "\n| candidate | overlaps | touching | group faults | links through a box | crossings | \
         bends |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for (candidate, t) in candidates.iter().zip(totals) {
        println!(
            "| {candidate} | {} | {} | {} | {} | {} | {} |",
            t[0], t[1], t[2], t[3], t[4], t[5]
        );
    }
    Ok(())
}

/// How far each thing that is in both pictures moved, along each axis. A shift of the whole
/// picture is not a move, so the median shift is taken out first.
fn shifts(before: &Picture, after: &Picture) -> Vec<(f64, f64)> {
    let shifts: Vec<(f64, f64)> = before
        .nodes
        .iter()
        .zip(&after.nodes)
        .map(|(a, b)| (b.centre().0 - a.centre().0, b.centre().1 - a.centre().1))
        .collect();
    let median = |axis: fn(&(f64, f64)) -> f64| {
        let mut values: Vec<f64> = shifts.iter().map(axis).collect();
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    let (dx, dy) = (median(|s| s.0), median(|s| s.1));
    shifts.iter().map(|s| (s.0 - dx, s.1 - dy)).collect()
}

/// The things that are in both pictures and that moved by more than one pixel.
fn moved(before: &Picture, after: &Picture) -> usize {
    shifts(before, after)
        .iter()
        .filter(|s| s.0.abs() > 1.0 || s.1.abs() > 1.0)
        .count()
}

/// The longest move of one thing, in pixels.
fn farthest(before: &Picture, after: &Picture) -> f64 {
    shifts(before, after)
        .iter()
        .map(|s| s.0.hypot(s.1))
        .fold(0.0, f64::max)
}

/// The pairs of things that are in both pictures and that changed sides: one was above the
/// other and is now below it, or one was left of the other and is now right of it.
fn swapped(before: &Picture, after: &Picture) -> usize {
    let side = |a: f64, b: f64| {
        if (a - b).abs() <= 1.0 {
            0
        } else if a < b {
            -1
        } else {
            1
        }
    };
    let count = before.nodes.len().min(after.nodes.len());
    let mut swaps = 0;
    for i in 0..count {
        for j in i + 1..count {
            let (a, b) = (before.nodes[i].centre(), before.nodes[j].centre());
            let (c, d) = (after.nodes[i].centre(), after.nodes[j].centre());
            if side(a.0, b.0) * side(c.0, d.0) < 0 || side(a.1, b.1) * side(c.1, d.1) < 0 {
                swaps += 1;
            }
        }
    }
    swaps
}

fn seed() -> Result<(), String> {
    println!(
        "Each row adds one thing, `New step`, with one link from the middle thing of the file.\n\
         A cell is: things that moved, pairs of things that changed sides, and crossings in \
         the new picture.\n"
    );
    println!(
        "| diagram | things | own, no seed | own, seed | graphlay, no seed | graphlay, seed | \
         elk, no seed | elk, seed | dagre |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    let mut totals = [[0usize; 3]; 7];
    let mut unedited_rows = Vec::new();
    let mut unedited_totals = [[0usize; 3]; 3];
    for (name, mut graph) in corpus()? {
        let things = graph.nodes.len();
        let before = [
            own::layout(&graph, None),
            crates::graphlay(&graph, None)?,
            crates::elk(&graph, None)?,
            crates::dagre(&graph),
        ];

        // No edit: each candidate gets its own picture back as the seed. A layout that
        // holds a picture still moves nothing here.
        let own_centres: Vec<_> = before[0].nodes.iter().map(|r| Some(r.centre())).collect();
        let graphlay_centres: Vec<_> = before[1].nodes.iter().map(|r| Some(r.centre())).collect();
        let unedited = [
            own::layout(&graph, Some(&own_centres as &Seed)),
            crates::graphlay(&graph, Some(&graphlay_centres as &Seed))?,
            crates::elk(&graph, Some(&before[2]))?,
        ];
        let mut cells = Vec::new();
        for (index, picture) in unedited.iter().enumerate() {
            let earlier = &before[index];
            let cell = [
                moved(earlier, picture),
                measure::score(&graph, earlier).crossings,
                measure::score(&graph, picture).crossings,
            ];
            for (total, value) in unedited_totals[index].iter_mut().zip(cell) {
                *total += value;
            }
            cells.push(format!(
                "{}, {:.0}, {} to {}",
                cell[0],
                farthest(earlier, picture),
                cell[1],
                cell[2]
            ));
        }
        unedited_rows.push(format!("| {name} | {things} | {} |", cells.join(" | ")));
        let from = things / 2;
        let group = graph.nodes[from].group;
        let added = graph.thing("New step");
        graph.nodes[added].group = group;
        graph.link(from, added);

        let centres = |picture: &Picture| -> Vec<Option<(f64, f64)>> {
            let old = picture.nodes.iter().map(|r| Some(r.centre()));
            old.chain([None]).collect()
        };
        let own_seed: Vec<Option<(f64, f64)>> = centres(&before[0]);
        let graphlay_seed = centres(&before[1]);
        // A new thing needs some position for ELK: it starts to the right of the thing it
        // links from.
        let mut elk_seed = Picture {
            nodes: before[2].nodes.clone(),
            groups: before[2].groups.clone(),
            edges: Vec::new(),
        };
        let source = before[2].nodes[from];
        elk_seed.nodes.push(model::Rect {
            x: source.x + source.w + 64.0,
            ..source
        });

        let after: [Picture; 7] = [
            own::layout(&graph, None),
            own::layout(&graph, Some(&own_seed as &Seed)),
            crates::graphlay(&graph, None)?,
            crates::graphlay(&graph, Some(&graphlay_seed as &Seed))?,
            crates::elk(&graph, None)?,
            crates::elk(&graph, Some(&elk_seed))?,
            crates::dagre(&graph),
        ];
        let earlier = [0, 0, 1, 1, 2, 2, 3];
        let mut cells = Vec::new();
        for (index, picture) in after.iter().enumerate() {
            let earlier = &before[earlier[index]];
            let cell = [
                moved(earlier, picture),
                swapped(earlier, picture),
                measure::score(&graph, picture).crossings,
            ];
            for (total, value) in totals[index].iter_mut().zip(cell) {
                *total += value;
            }
            cells.push(format!("{}, {}, {}", cell[0], cell[1], cell[2]));
        }
        println!("| {name} | {things} | {} |", cells.join(" | "));
    }
    let cells: Vec<String> = totals
        .iter()
        .map(|t| format!("{}, {}, {}", t[0], t[1], t[2]))
        .collect();
    println!("| total | 189 | {} |", cells.join(" | "));

    println!(
        "\nNo edit: each row seeds a diagram with its own picture. dagre has no input for \
         positions.\nA cell is: things that moved, the longest move in pixels, and crossings \
         before and after.\n"
    );
    println!("| diagram | things | own | graphlay | elk |");
    println!("| --- | --- | --- | --- | --- |");
    for row in unedited_rows {
        println!("{row}");
    }
    let cells: Vec<String> = unedited_totals
        .iter()
        .map(|t| format!("{}, {} to {}", t[0], t[1], t[2]))
        .collect();
    println!("| total | 189 | {} |", cells.join(" | "));
    Ok(())
}

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("draw") => draw(),
        Some("seed") => seed(),
        _ => Err("usage: layered-layout-spike draw|seed".to_string()),
    };
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
