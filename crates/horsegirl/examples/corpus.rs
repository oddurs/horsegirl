//! Draws every file of the converted corpus and counts the files whose
//! picture has no overlap and leaves out no link. The release gate is 100 real flowcharts with no
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
use horsegirl_core::scene::Role;
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;

/// The number of corpus files that draw clean on `main`: with no overlap and
/// with every link. Every kind of overlap counts, because the gate is a
/// picture with none. Raise the number when a change draws more files clean.
/// It fell from 184 to 181 when E0420 began to refuse a thing written into
/// two groups: those files drew clean only because the model dropped the
/// second group with no word.
const BASELINE: usize = 181;

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
    /// The picture.
    Picture(Picture),
}

struct Picture {
    /// The count of each kind of overlap, in the order of [`KINDS`].
    overlaps: [usize; KINDS.len()],
    /// The number of links in the model that the picture does not draw. A
    /// picture that leaves out a link can have no overlap and still be wrong.
    left_out: usize,
}

impl Picture {
    fn is_clean(&self) -> bool {
        self.left_out == 0 && self.overlaps.iter().all(|&n| n == 0)
    }
}

fn draw(source: &str) -> Drawn {
    let errors = check(source)
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .count();
    if errors > 0 {
        return Drawn::Refused;
    }
    let model = lower(&parse(source).tree);
    let scene = lens::draw(&model, &lens::FLOW, STYLE);
    let mut counts = [0; KINDS.len()];
    for overlap in overlaps(&scene) {
        if let Some(index) = KINDS.iter().position(|&kind| kind == overlap.kind) {
            counts[index] += 1;
        }
    }
    let drawn = scene
        .shapes
        .iter()
        .filter(|shape| shape.role == Role::Link)
        .count();
    Drawn::Picture(Picture {
        overlaps: counts,
        left_out: model.links.len().saturating_sub(drawn),
    })
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
    let mut files_left_out = 0;
    for path in &paths {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        match draw(&source) {
            Drawn::Refused => refused.extend(name),
            Drawn::Picture(picture) => {
                clean += usize::from(picture.is_clean());
                files_left_out += usize::from(picture.left_out > 0);
                for (files, n) in files_with.iter_mut().zip(picture.overlaps) {
                    *files += usize::from(n > 0);
                }
            }
        }
    }

    println!(
        "corpus: {clean} of {} files draw with zero overlaps and every link (baseline {BASELINE})",
        paths.len()
    );
    for (kind, files) in KINDS.iter().zip(files_with) {
        println!("  files with {kind:?}: {files}");
    }
    println!("  files with a link left out: {files_left_out}");
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

#[cfg(test)]
mod tests {
    use super::{Drawn, draw};

    fn picture(source: &str) -> super::Picture {
        match draw(source) {
            Drawn::Picture(picture) => picture,
            Drawn::Refused => panic!("check refuses {source:?}"),
        }
    }

    #[test]
    fn a_picture_with_every_link_and_no_overlap_is_clean() {
        let picture = picture("A -> B\nB -> C\n");
        assert_eq!(picture.left_out, 0);
        assert!(picture.is_clean());
    }

    #[test]
    fn a_picture_that_leaves_out_a_link_is_not_clean() {
        // The flow lens does not yet draw a link that ends at a group.
        let picture = picture("A -> B\nA -> G\nG { C, D }\n");
        assert_eq!(picture.left_out, 1);
        assert!(!picture.is_clean());
    }
}
