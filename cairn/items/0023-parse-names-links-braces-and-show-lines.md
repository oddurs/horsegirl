---
id: 23
uid: 5c753b0b-8975-4ac6-84fc-71f6cb5876c9
title: Parse names, links, braces and show lines
type: feature
status: planned
milestone: v0.1
depends_on:
- 17
- 22
created: 2026-11-25
updated: 2026-11-25
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

- [ ] Every line of the reference file `examples/checkout.horse` parses into the expected tree, held by a snapshot test.
- [ ] Printing the tree back gives the input byte for byte, for every file in the corpus.
- [ ] A file with three bad lines reports three errors and still returns the tree for the good lines.
- [ ] A fuzz run of ten minutes finds no panic.
