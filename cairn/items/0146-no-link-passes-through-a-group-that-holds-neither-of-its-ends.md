---
id: 146
uid: 77a30d06-ab27-4d9d-862a-111eafd3fcae
title: No link passes through a group that holds neither of its ends
type: feature
status: backlog
milestone: v0.1
depends_on:
- 33
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
related:
- 122
---

## Problem

The route phase draws every upright line in the corridor between two ranks. The order phase decides on which side of a group a link passes each rank. It does not count a group as something a link can cross. So it can put a link above a group in one rank and below the group in the next rank, and the upright line then passes through the group box. The link crosses no box of a thing, but the picture shows a line through a group that the link has nothing to do with.

The overlap detector reports this as `Kind::LinkThroughGroup`. On the eleven diagrams of `crates/horsegirl-core/tests/layout.rs` it reports 18: 1 on `07-compiler` (`Fixes -> Source` through `Core`) and 17 on `10-platform`. The test `no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram` holds these two numbers.

One attempt did not help: counting the top and the bottom of each group as two steps in `crossings` of `layout/order.rs` left both numbers as they are. The sweeps never propose the better order, so a count alone does not find it.

## Proposal

Make the order phase keep a link on one side of a group. Two parts:

- The bends of one link that lie under the same parent keep one side of each group under that parent in every rank, as the groups under one parent keep one order today.
- When a sweep orders a rank, an item that links into a group stands next to that group, and a transpose step (item 0122) counts a pass through a group as two crossings.

Some passes cannot be removed: a link between two groups with a third group between them in every rank must pass through the third group or go round the whole picture. Decide for that case whether the path goes round, or whether the pass stays and the detector does not count it.

## Acceptance criteria

- [ ] `07-compiler` draws with no link through a group that holds neither of its ends.
- [ ] `10-platform` draws with 8 or fewer such links, and each one that remains is named in the test with the reason it cannot be removed.
- [ ] The numbers in `no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram` fall, and no other collision appears on the eleven diagrams.
- [ ] The crossing count of each of the ten spike diagrams does not rise.

## 2026-11-25

From the review of item 0033. An experiment in layout/order.rs, not committed: when a sweep sorts a rank, each item takes the side of each group under its own parent that its neighbour in the rank beside it has, and the order phase keeps the order with the fewest passes first and the fewest crossings second. Result: 07-compiler falls from 1 to 0, and 10-platform stays at 17. All 17 are links between two groups with a third group between them, so only the order of the groups under one parent removes them. For the six top groups of 10-platform an order with no pass exists: Stores, Async, Services, External, with Edge on the Stores side of Services. Inside Services, 2 passes remain in the better order (Shopping on the Stores side): Orders -> Order database and Inventory -> Inventory database go through Shopping, because Shopping and Fulfilment both link to Stores between ranks that both groups have. Those 2 need a path that goes round, or other ranks. The test of item 0033 is now named no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram. The generator of no_generated_graph_has_a_fault gives one or more passes on 982 of 3000 graphs (seed 1, second review of pull request 39).

## Proposed priority: p2 -> p0 (Oddur Sigurdsson, 2026-11-25)

Criterion 1 of item 0033 (p0) says no path crosses a box on any corpus diagram. Two reviews read a group as a box, and 18 corpus links pass through a group. If the owner narrows that criterion to the box of a thing, item 0146 holds the rest of a p0 promise, and v0.1 would ship pictures with links through groups that they have no end in.

## 2026-11-25

About the note before this one: the two numbers of the experiment (0 on 07-compiler, 17 on 10-platform) were measured. The order of the top groups with no pass and the 2 passes inside Services were worked out by hand from the ranks, and no run has shown them.
