---
id: 150
uid: 9395b669-8c09-4249-ab28-b92a0c70c8a0
title: The route phase orders the upright lines of a corridor in less than cubic time
type: chore
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: s
related:
- 33
---

## Problem

`tracks` in `crates/horsegirl-core/src/layout/route.rs` builds the key of every waiting trunk again for each trunk that it places. The cost is cubic in the trunks of one corridor. Two ranks of `n` boxes with two links from each box take 0.22 s at `n = 400`, 1.6 s at `n = 800` and 12.9 s at `n = 1600`. The layout before item 0033 took 0.35 s at `n = 1600`. At 200 boxes in one rank the cost is 32 ms. Measured in the review of item 0033.

## Acceptance criteria

- [ ] `tracks` gives the same tracks as today on the eleven diagrams of `crates/horsegirl-core/tests/layout.rs`: `layout.positions` does not change.
- [ ] Two ranks of 1600 boxes with two links from each box take less than 1 s in a release build.
