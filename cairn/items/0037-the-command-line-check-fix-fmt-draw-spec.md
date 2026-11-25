---
id: 37
uid: 82d172f1-3ef9-460f-87ba-cff7ef0838ab
title: 'The command line: check, fix, fmt, draw, spec'
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 27
- 28
- 29
- 34
- 36
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: cli
effort: m
area: cli
track:
- agent-loop
---

## Problem

The pieces exist as a library. An agent needs commands.

## Proposal

Add `check`, `fix`, `fmt`, `draw` and `spec`. Exit 0 for clean, 1 for diagnostics at error level, 2 for a usage error. `--json` prints one object per line. `draw --to svg` writes beside the source or to `-o`.

## Acceptance criteria

- [x] Each command has an integration test for its exit codes and its output.
- [x] `fix` applies every safe fix and changes nothing else.
- [x] `draw` refuses to write a picture when the file has an error, and says which.
- [x] `--help` for every command fits on one screen.

## 2026-11-25

Criterion 1: crates/horsegirl/tests/cli.rs runs each of spec, check, fix, fmt and draw as a process and asserts its exit code (0, 1 and 2) and its stdout and stderr. cargo test -p horsegirl --test cli printed 31 passed, 0 failed. scripts/task check exited 0.

## 2026-11-25

Criterion 2: fix::applies_every_safe_fix_to_standard_input and fix::writes_a_file_in_place compare the whole output with the expected bytes for a file with three safe fixes (E0412, E0004, E0300) and a comment; fix::changes_nothing_that_no_safe_fix_names gives a file with odd spacing, a guess and a warning back byte for byte. In the core, apply_safe_makes_every_safe_fix_and_changes_nothing_else (4 safe fixes in one run) and apply_safe_leaves_a_suggested_fix_and_an_overlapping_fix passed: cargo test -p horsegirl-core --test diagnostic apply_safe printed 3 passed.

## 2026-11-25

Criterion 3: draw::a_file_with_an_error_gives_no_picture_and_names_each_error passed: exit 1, no shop.svg written, stderr holds error[E0412], error[E0004], error[E0300] and 'has 3 errors, so no picture was drawn'; with --json the errors are JSON lines. A lens this version does not draw (show matrix) also exits 1 with no picture.

## 2026-11-25

Criterion 4: every_help_fits_on_one_screen runs --help for the binary and for spec, check, fix, fmt and draw: each exits 0, has at most 24 lines and at most 80 columns. By hand: 16, 4, 8, 8, 6 and 10 lines; widest 76 columns. The test passed in the 31.
