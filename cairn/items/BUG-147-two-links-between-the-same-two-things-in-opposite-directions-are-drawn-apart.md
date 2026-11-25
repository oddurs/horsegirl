---
id: 147
uid: 61cd95bd-0d87-417c-9478-789f15a1a3d3
title: Two links between the same two things in opposite directions are drawn apart
type: bug
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: m
related:
- 33
---

## Problem

`a -> b` with `b -> a: back` draws both links on one line, from the right side of `a` to the left side of `b`, with the label `back` in the middle. The reader cannot tell which link has the label. `Kind::Links` in `crates/horsegirl-core/src/overlap.rs` skips two links that share an end, so the detector does not report the pair. Found in the review of item 0033.

## Acceptance criteria

- [ ] The overlap detector reports two links between the same two things that lie on each other, with a test in `crates/horsegirl-core/tests/overlap.rs`.
- [ ] The layout draws `a -> b` and `b -> a` on two lines, and each label is on its own link.
- [ ] The corpus tests in `crates/horsegirl-core/tests/layout.rs` pass with the new rule.
