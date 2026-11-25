---
id: 122
uid: 191d43ed-072e-4ab6-a652-4f7251ea4f64
title: Fewer crossings in the layered layout, measured against a reference
type: feature
status: backlog
milestone: v0.1
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
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

- [ ] The ten diagrams in `spikes/layered-layout/corpus` draw with 54 crossings or fewer in total, which is 1.5 times the reference.
- [ ] `08-module-deps` draws with 14 crossings or fewer, which is the count that dagre leaves.
- [ ] No diagram of the ten has more crossings than the spike prototype left on it.
- [ ] A test fails when the total rises above the committed number.
- [ ] The same input gives the same order on every run.

## 2026-11-25

From item 0146: crates/horsegirl-core/src/layout/through.rs has a transpose move (fn transpose: two neighbouring entries of a rank exchange their places, and an entry is an item or every item of a group) and order.rs has fn crossed, the crossings of one corridor. The search of item 0146 runs only when the order of the sweeps has a pass through a group, so it lowers the crossings of 07-compiler and 10-platform only. The test the_crossings_of_each_corpus_diagram_do_not_rise in tests/layout.rs holds the crossing count of each of the eleven pictures: 0, 0, 0, 3, 0, 0, 4, 1, 23, 2, 18. It counts the points where two drawn paths cross.
