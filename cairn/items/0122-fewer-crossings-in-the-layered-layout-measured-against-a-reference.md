---
id: 122
uid: 191d43ed-072e-4ab6-a652-4f7251ea4f64
title: Fewer crossings in the layered layout, measured against a reference
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
---

## Problem

The spike prototype orders layers with barycenter sweeps only. On the ten spike diagrams it leaves 80 crossings. The ELK port leaves 36 on the same diagrams. The gap has two causes. The first is a dense graph without groups: `08-module-deps` has 22 crossings against 3, and dagre leaves 14 on it. That diagram alone is 22 of the 80. The second is nested groups: `06-cluster-request` has 9 against 1.

## Proposal

Improve the order phase of the layered solver. Add a transpose step after each sweep. The transpose step is the part for `08-module-deps`, which has no groups: dagre sweeps and transposes, and leaves 14 crossings there. Count the crossings of a group as one unit when the sweep orders the groups. Keep the rule from SPIKE-18 that groups under one parent hold one order in every layer. Record the crossing count of each spike diagram as a committed number, so that a change to the order phase shows in review.

The reference counts come from `spikes/layered-layout`. Run `cargo run --release -- draw` there. The ELK port stays in the spike. It does not become a dependency of the crate.

## Acceptance criteria

- [x] The ten diagrams in `spikes/layered-layout/corpus` draw with 54 crossings or fewer in total, which is 1.5 times the reference.
- [x] `08-module-deps` draws with 14 crossings or fewer, which is the count that dagre leaves.
- [x] No diagram of the ten has more crossings than the spike prototype left on it.
- [x] A test fails when the total rises above the committed number.
- [x] The same input gives the same order on every run.

## 2026-11-25

From item 0146: crates/horsegirl-core/src/layout/through.rs has a transpose move (fn transpose: two neighbouring entries of a rank exchange their places, and an entry is an item or every item of a group) and order.rs has fn crossed, the crossings of one corridor. The search of item 0146 runs only when the order of the sweeps has a pass through a group, so it lowers the crossings of 07-compiler and 10-platform only. The test the_crossings_of_each_corpus_diagram_do_not_rise in tests/layout.rs holds the crossing count of each of the eleven pictures: 0, 0, 0, 3, 0, 0, 4, 1, 23, 2, 18. It counts the points where two drawn paths cross.

## 2026-11-25

From pull request 48 (item 0146): the pass search of layout/through.rs counts passes first and crossings second, with no weight. One pass less can cost many crossings. The second review found, on 3000 generated graphs (seed 1), 326 graphs with more crossings than before the search: graph 2184 lost 1 pass and went from 79 to 114 crossings. The move Clear of fix round 1 adds to that: the total on seed 1 is 19183 crossings, against 19125 before any search. The eleven test diagrams do not show it. A transpose step for every picture should decide what a pass is worth in crossings.

## 2026-11-25

Criteria 1, 2, 3: ran cargo test -p horsegirl-core --test layout. the_crossings_of_each_corpus_diagram_do_not_rise passes with [0, 0, 0, 1, 0, 0, 4, 1, 11, 2, 18] for checkout and the ten spike diagrams: 37 in total on the ten, against 51 before and a limit of 54. 08-module-deps has 11, against 23 before and a limit of 14. The prototype left 1, 0, 1, 0, 0, 9, 1, 22, 2, 44, and no diagram is above its number. the_spike_diagrams_cross_no_more_than_the_reference_allows holds those three limits and passes. The numbers count the points where two drawn paths cross, as the spike does.

## 2026-11-25

Criterion 4: with the call to untangle in order.rs turned off, the_crossings_of_each_corpus_diagram_do_not_rise failed with left [0, 0, 0, 1, 0, 0, 4, 1, 23, 2, 18], and the_spike_diagrams_cross_no_more_than_the_reference_allows failed with '08-module-deps: 23 crossings, the prototype 22'. The first test also asserts the total of the ten diagrams, 37. Both pass with the call back.

## 2026-11-25

Criterion 5: scripts/task test:core exited 0. the_same_input_gives_the_same_positions_on_every_run passed natively and in WebAssembly against the one file tests/layout.positions, and it lays out each of the eleven diagrams twice. no_generated_graph_has_a_fault lays out 300 generated graphs twice and compares the two pictures. The new steps iterate vectors only and read no clock.

## 2026-11-25

What moved the numbers. The exchange of neighbours after each round of sweeps took 08-module-deps from 23 to 12. The other two crossings were in the route phase: tracks in route.rs chose the next upright line by a sum against every waiting line, and a tie put two lines the wrong way round. An exchange of neighbouring lines there took 03-order-states from 3 to 1 and 08-module-deps to 11. On 2000 generated graphs (the generator of no_generated_graph_has_a_fault, 1000 each for seeds 1 and 7, no labels) the crossings fell from 6916 to 6491 and from 6056 to 5816, and the links through a group fell from 182 to 179 and from 125 to 117. Not done: two groups under one parent never exchange their places by crossings, and a pass has no weight in crossings. Both are in the new follow-up item.

## Result

The order phase exchanges two neighbouring entries of a rank after each round of sweeps where that leaves fewer crossings, and the route phase does the same for two neighbouring upright lines of a corridor. The ten spike diagrams draw with 37 crossings, against 51 before, 80 for the prototype and 36 for the ELK port. 08-module-deps draws with 11. Two tests in tests/layout.rs hold the number of each diagram and the limits of the reference. Two groups under one parent do not exchange by crossings, and a pass has no weight in crossings: item 0166.
