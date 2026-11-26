---
id: 172
uid: 7bf6efc4-f4d3-4cff-aa2a-94c8fb2649a1
title: Two labels of one corridor do not cover each other on any generated graph
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
---

## Problem

`no_generated_graph_has_a_fault` in `crates/horsegirl-core/tests/layout.rs` runs 300 graphs of the sequence with seed 7. With 3000 rounds, graph 2573 gives "label `n6 -> n8` covers label `n6 -> n5`". The generator with group ends (seed 3) gives the same kind of collision at graph 1087. The two labels belong to links between nodes, so the cause is in the route phase, not in links to groups.

## Acceptance criteria

- [ ] Graph 2573 of seed 7 and graph 1087 of seed 3 draw with no `Labels` collision.
- [ ] A test holds both graphs.
- [ ] The two generated tests run 3000 rounds with no `Labels` collision, or the test states the round count that the time budget allows.
