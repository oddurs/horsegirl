---
id: 169
uid: c8aa8f39-aa81-4917-8ddb-93b164e601ba
title: Two links from one thing to itself draw two loops and two labels apart
type: bug
status: backlog
milestone: v0.1
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
effort: s
related:
- 124
---

## Problem

`turn_back` in `crates/horsegirl-core/src/layout.rs` draws every link from a node to itself as the same loop at the right side of its box. Two such links on one node lie on each other, and their two labels have one centre. Over 3000 graphs from the generator of `no_generated_graph_has_a_fault`, with item 0124 in place, 19 label overlaps are of this kind. The test `no_generated_graph_has_a_fault` lets them pass through `two_loops_of_one_node`.

Smallest input: `a -> a: one` and `a -> a: two`.

## Proposal

Give each loop of a node its own height or its own distance from the box, and widen the room that `loops` keeps at each side so the labels stand side by side or one above the other.

## Acceptance criteria

- [ ] `a -> a: one` and `a -> a: two` draw two loops that do not lie on each other, and two labels that do not cover each other.
- [ ] `no_generated_graph_has_a_fault` no longer needs `two_loops_of_one_node`, and the helper is gone.
- [ ] A node with one loop keeps the picture it has today: `tests/layout.positions` does not change.
