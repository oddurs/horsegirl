---
id: 29
uid: 962335df-38e4-43f7-81d8-ee0c4a99d04f
title: A formatter with one canonical form
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: language
effort: m
area: language
track:
- language
---

## Problem

A repository of files written by many agents drifts into many styles, and diffs become noise.

## Proposal

`horsegirl fmt` prints one canonical form: one fact per line, single spaces around arrows, facts in braces in a fixed order. Comments are kept.

## Acceptance criteria

- [x] `fmt` of a formatted file changes nothing.
- [x] `fmt` never changes the model: a property test compares the model before and after on the corpus.
- [x] `fmt --check` exits 1 when a file would change.

## 2026-11-25

Item 26 adds -x rules and ? guesses to the model. fmt must print both unchanged: add 'A -x B: reason' and 'A -> B ?' to its idempotence tests.

## 2026-11-25

scripts/task check passed. cargo test -p horsegirl-core --test format: 7 passed. a_formatted_file_does_not_change asserts format(examples/checkout.horse) equals the file; random files and all 11 .horse files in the repository assert format(format(x)) == format(x). HORSEGIRL_FUZZ_ROUNDS=6000000 cargo test --release -p horsegirl-core --test format printed 7 passed in 12.70 s (no counterexample after fixing an empty-label colon and a name that starts with #).

## 2026-11-25

Model property (criterion 2): tests/format.rs projects each tree to its facts (no layout, no quotes, one space between words, facts of a line sorted by key) and compares the projection before and after on the 11 .horse files and on seeded random files; comments and the E-free reparse are compared too. Weak spot: the projection is a tree projection, because no model exists yet; item 0025 should repoint it. A file with a bare name that starts with # and holds a quote is returned unchanged, because it has no canonical form that keeps its meaning.

## 2026-11-25

Criterion 3: crates/horsegirl/tests/cli.rs fmt::check_exits_1_when_the_file_would_change and check_exits_0_for_a_formatted_file passed in scripts/task check (cargo nextest, 46 tests).

## 2026-11-25

Fix round 1: a bare fact value that ends in ' ?' is now quoted, so the fact sort cannot turn it into a guess. Test: quotes_stay_only_where_a_name_needs_them and a generator piece.
