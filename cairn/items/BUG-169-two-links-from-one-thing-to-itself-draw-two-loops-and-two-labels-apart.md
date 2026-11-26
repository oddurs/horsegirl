---
id: 169
uid: c8aa8f39-aa81-4917-8ddb-93b164e601ba
title: Two links from one thing to itself draw two loops and two labels apart
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-26
updated: 2026-11-26
closed_at: 2026-11-26
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

- [x] `a -> a: one` and `a -> a: two` draw two loops that do not lie on each other, and two labels that do not cover each other.
- [x] `no_generated_graph_has_a_fault` no longer needs `two_loops_of_one_node`, and the helper is gone.
- [x] A node with one loop keeps the picture it has today: `tests/layout.positions` does not change.

## 2026-11-26

A file cannot state two links from one thing to itself: a -> a: one and a -> a: two lower to one model link with the labels 'one, two'. The tests build the Graph directly. Fix: each loop takes a column at the right side of the box and a side of the middle, two loops to a column; from the third loop on, outer loop lines pass through the labels of the nearer column (not reported by the detector, same ends).

## 2026-11-26

Criterion 1: cargo test -p horsegirl-core --test layout loops_apart failed before the change (second loop above the middle; three loops with identical points) and printed 'test two_links_from_a_node_to_itself_draw_two_loops_apart ... ok' and 'three_links_..._three_loops_apart ... ok' after it, asserting no part lies on another and the label boxes do not share room. Criterion 2: two_loops_of_one_node and link_names removed; no_generated_graph_has_a_fault ... ok and no_generated_graph_with_links_to_groups_has_a_fault ... ok. Criterion 3: the_same_input_gives_the_same_positions_on_every_run ... ok with tests/layout.positions unchanged (git diff shows no change to it). scripts/task check exited 0.
