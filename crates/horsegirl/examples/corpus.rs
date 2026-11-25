//! Draws every file of the converted corpus and counts the files whose
//! picture has no overlap. The release gate is 100 real flowcharts with no
//! overlap, so CI runs this through `scripts/task test`:
//!
//! ```sh
//! cargo run --locked -p horsegirl --example corpus
//! ```
//!
//! It fails when the count is below [`BASELINE`]. `corpus/convert.py` writes
//! the files from the Mermaid flowcharts in `corpus/flowcharts`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use horsegirl_core::check::check;
use horsegirl_core::diagnostic::Severity;
use horsegirl_core::lens;
use horsegirl_core::model::lower;
use horsegirl_core::overlap::{Kind, overlaps};
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;

/// The number of corpus files that draw with no overlap on `main`. Every
/// kind of overlap counts, because the gate is a picture with none. Raise
/// the number when a change draws more files clean.
const BASELINE: usize = 198;

/// The style of `horsegirl draw`, so that the count is of the pictures that
/// the command line draws.
const STYLE: Style = Style { size: 14.0 };

const KINDS: [Kind; 9] = [
    Kind::Boxes,
    Kind::LinkOnThing,
    Kind::LinkThroughGroup,
    Kind::LinkOnGroup,
    Kind::Links,
    Kind::LabelOnThing,
    Kind::LabelOnGroup,
    Kind::LabelOffLink,
    Kind::Labels,
];

/// What one file gives.
enum Drawn {
    /// The file has an error, so `draw` gives no picture.
    Refused,
    /// The picture, with the count of each kind of overlap in the order of
    /// [`KINDS`].
    Picture([usize; KINDS.len()]),
}

fn draw(source: &str) -> Drawn {
    let errors = check(source)
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .count();
    if errors > 0 {
        return Drawn::Refused;
    }
    let scene = lens::draw(&lower(&parse(source).tree), &lens::FLOW, STYLE);
    let mut counts = [0; KINDS.len()];
    for overlap in overlaps(&scene) {
        if let Some(index) = KINDS.iter().position(|&kind| kind == overlap.kind) {
            counts[index] += 1;
        }
    }
    Drawn::Picture(counts)
}

/// The `.horse` files in `dir`, in the same order on every machine.
fn files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "horse") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn main() -> Result<ExitCode, String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/horse");
    let paths = files(&dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    if paths.is_empty() {
        return Err(format!("no .horse files in {}", dir.display()));
    }

    let mut clean = 0;
    let mut refused = Vec::new();
    let mut files_with = [0; KINDS.len()];
    for path in &paths {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        match draw(&source) {
            Drawn::Refused => refused.extend(name),
            Drawn::Picture(counts) => {
                clean += usize::from(counts.iter().all(|&n| n == 0));
                for (files, n) in files_with.iter_mut().zip(counts) {
                    *files += usize::from(n > 0);
                }
            }
        }
    }

    println!(
        "corpus: {clean} of {} files draw with zero overlaps (baseline {BASELINE})",
        paths.len()
    );
    for (kind, files) in KINDS.iter().zip(files_with) {
        println!("  files with {kind:?}: {files}");
    }
    println!(
        "  files that check refuses to draw: {} {}",
        refused.len(),
        refused.join(" ")
    );
    if clean < BASELINE {
        eprintln!(
            "corpus: {clean} files draw clean, fewer than the baseline of {BASELINE}. A change made the pictures worse; find the files that lost their clean picture and fix the layout."
        );
        return Ok(ExitCode::FAILURE);
    }
    if clean > BASELINE {
        println!(
            "corpus: raise BASELINE in crates/horsegirl/examples/corpus.rs to {clean}, so that the gain is kept."
        );
    }
    Ok(ExitCode::SUCCESS)
}
