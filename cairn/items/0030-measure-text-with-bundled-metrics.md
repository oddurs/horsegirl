---
id: 30
uid: e66b6f6e-e7e6-4f25-9bdc-89a216afec01
title: Measure text with bundled metrics
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
depends_on:
- 19
- 22
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: svg
effort: m
area: render
track:
- pictures
---

## Problem

A node must be as wide as its label, and the answer must be the same on every machine.

## Proposal

Implement the text measurement that the spike chose, behind one function: text and style in, width and height out.

## Acceptance criteria

- [x] Widths for 100 reference labels are within 3 percent of the browser measurement recorded in the spike.
- [ ] The function gives the same result on Linux, macOS and in WebAssembly, held by CI.

## 2026-11-25

SPIKE-19 answer: a table of advances plus kerning-class pair matrices, in integer font units, generated offline from Liberation Sans 2.1.5 (Arial metrics) and checked in as Rust source; the core reads no font. Characters outside the table measure as one em. The 100 reference labels are spikes/0019-text-metrics/labels.txt and the browser widths are the arial-metrics column of spikes/0019-text-metrics/browser.tsv. The spike prototype was within 0.051 percent on every label.

## 2026-11-25

SPIKE-19 review: pairs now cover Basic Latin, Latin-1, Latin Extended-A, Greek, Cyrillic and the dashes and quotes; class 0 of each matrix is reserved for no adjustment, because every character outside those blocks takes class 0 (spikes/0019-text-metrics/src/lib.rs test). The Liberation Sans table is 101 by 112 classes, 43.2 kB wasm32 (9.2 kB gzip). On 108 labels (100 Latin, 8 with arrows, Greek and Cyrillic) the table is within 1.149 percent of the arial-metrics column. Limits: the evidence covers Latin, Greek and Cyrillic left to right only. Hebrew gets advances only and was not measured. The browser does not kern across a change of script and the table does, which makes 'Тест → ΔΓ' 1.149 percent narrow; consider dropping a pair whose characters are in different scripts.

## 2026-11-25

Criterion 1: scripts/task check ran crates/horsegirl-core/tests/text.rs natively (nextest: 12 tests run, 12 passed) and on wasm32-wasip1 under Node 24.22.3 (2 passed). every_reference_label_is_within_three_percent_of_the_browser holds all 108 labels of spikes/0019-text-metrics/browser.tsv (arial-metrics column, 12 px) under 3 percent. Measured against that column: first 100 labels mean 0.014% max 0.051% ('a – b'); all 108 mean 0.024% max 1.149% ('Тест → ΔΓ'), the same as the spike prototype. tools/text-metrics/generate.sh run twice on LiberationSans-Regular.ttf (sha256 bade59d8...) gave the same metrics.rs: 2327 characters, 101 by 112 classes.

## 2026-11-25

Criterion 2: every_target_gives_the_same_bits compares the f64 width and height of 108 labels with crates/horsegirl-core/tests/text-widths.tsv. Watched pass on x86_64 Linux and on wasm32-wasip1 under Node. With one width in the file changed to 47.8126, the wasm32-wasip1 run failed with exit status 1, so the runner reports a failure. macOS runs in the new CI job core-macos (scripts/task test:core); this criterion is ticked only after that job passes on the pull request.
