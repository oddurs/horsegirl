---
id: 23
uid: 5c753b0b-8975-4ac6-84fc-71f6cb5876c9
title: Parse names, links, braces and show lines
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 17
- 22
- 119
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: high
surface: language
effort: l
area: language
track:
- language
---

## Problem

Nothing reads a horsegirl file yet.

## Proposal

Write a lexer and a parser for rules 1 to 3 and rule 5 of the spec card: names, links with fan-out and labels, braces with parts and facts, and `show` lines. The tree is lossless, so a formatter can print it back. The parser recovers after a bad line and reports every error in one pass.

## Acceptance criteria

- [x] Every line of the reference file `examples/checkout.horse` parses into the expected tree, held by a snapshot test.
- [x] Printing the tree back gives the input byte for byte, for every file in the corpus.
- [x] A file with three bad lines reports three errors and still returns the tree for the good lines.
- [x] A fuzz run of ten minutes finds no panic.

## 2026-11-25

Criterion 1: ran scripts/task check. It printed 'PASS horsegirl-core::syntax the_reference_file_parses_into_the_expected_tree' natively, and the WebAssembly run of tests/syntax.rs printed 'test result: ok. 13 passed'. The test compares the tree of examples/checkout.horse with crates/horsegirl-core/tests/checkout.tree (159 lines) and expects the one diagnostic W0010 for the story.

## 2026-11-25

Criterion 2: scripts/task check printed 'PASS horsegirl-core::syntax every_file_in_the_corpus_prints_back_byte_for_byte'. The test reads 239 files of corpus/flowcharts, examples/checkout.horse and the 10 files of spikes/layered-layout/corpus, asserts that each prints back equal to its source, asserts the counts (239, 11), and asserts that no .horse file has an error. The test is native only: the WebAssembly run gets no directory, and checks the reference file alone.

## 2026-11-25

Criterion 3: scripts/task check printed 'PASS horsegirl-core::syntax three_bad_lines_give_three_errors_and_the_good_lines_still_parse'. Seven lines, bad on lines 2, 4 and 6: the diagnostics are exactly E0009 line 2, E0006 line 4, E0001 line 6, and the tree is Link, Error, Link, Error, Things, Error, Show.

## 2026-11-25

Criterion 4: ran HORSEGIRL_FUZZ_ROUNDS=13000000 cargo test --release -p horsegirl-core --test fuzz on the committed parser (7fb6b5c). It printed 'test result: ok. 1 passed; 0 failed; finished in 774.43s' (12 min 54 s, 39 million inputs). Each input is checked for: no panic, print-back equal to the input, tokens that tile the source, spans on character boundaries, diagnostics in order, and two equal runs. The fuzzer is in the repository, with a fixed seed, and makes inputs from random pieces and from edits of two good files. It is not guided by coverage; item 97 adds that in CI.
