---
id: 146
uid: 77a30d06-ab27-4d9d-862a-111eafd3fcae
title: No link passes through a group that holds neither of its ends
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 33
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
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

- [x] `07-compiler` draws with no link through a group that holds neither of its ends.
- [x] `10-platform` draws with 8 or fewer such links, and each one that remains is named in the test with the reason it cannot be removed.
- [x] The numbers in `no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram` fall, and no other collision appears on the eleven diagrams.
- [x] The crossing count of each of the ten spike diagrams does not rise.

## 2026-11-25

From the review of item 0033. An experiment in layout/order.rs, not committed: when a sweep sorts a rank, each item takes the side of each group under its own parent that its neighbour in the rank beside it has, and the order phase keeps the order with the fewest passes first and the fewest crossings second. Result: 07-compiler falls from 1 to 0, and 10-platform stays at 17. All 17 are links between two groups with a third group between them, so only the order of the groups under one parent removes them. For the six top groups of 10-platform an order with no pass exists: Stores, Async, Services, External, with Edge on the Stores side of Services. Inside Services, 2 passes remain in the better order (Shopping on the Stores side): Orders -> Order database and Inventory -> Inventory database go through Shopping, because Shopping and Fulfilment both link to Stores between ranks that both groups have. Those 2 need a path that goes round, or other ranks. The test of item 0033 is now named no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram. The generator of no_generated_graph_has_a_fault gives one or more passes on 982 of 3000 graphs (seed 1, second review of pull request 39).

## Proposed priority: p2 -> p0 (Oddur Sigurdsson, 2026-11-25)

Criterion 1 of item 0033 (p0) says no path crosses a box on any corpus diagram. Two reviews read a group as a box, and 18 corpus links pass through a group. If the owner narrows that criterion to the box of a thing, item 0146 holds the rest of a p0 promise, and v0.1 would ship pictures with links through groups that they have no end in.

## 2026-11-25

About the note before this one: the two numbers of the experiment (0 on 07-compiler, 17 on 10-platform) were measured. The order of the top groups with no pass and the 2 passes inside Services were worked out by hand from the ranks, and no run has shown them.

## 2026-11-25

From review round 4 of item 0033: a label follows its link into the group. With a label on every link, 4 labels on 10-platform are wholly inside a group that holds neither end of their link. The overlap detector does not report a label there. The test each_link_label_sits_on_its_path_and_overlaps_no_box holds the count for each file in 'inside', and the count must fall with the links. The review counted, on 3000 generated graphs (seed 7), 82 labels across the border of such a group before the label move.

## Accepted priority: p0 (Oddur Sigurdsson, 2026-11-25)

Proposed by Oddur Sigurdsson on 2026-11-25.

## 2026-11-25

What was built, at commit e627417. crates/horsegirl-core/src/layout/through.rs is a second part of the order phase. It runs after the sweeps, and only when the order of the sweeps has a pass. It tries three moves and keeps one when the passes fall, or when the passes stay and the crossings fall: a group takes another place among the groups under its parent (Shift), every item of a link goes to one side of a group (Side), and two neighbours of a rank exchange their places (Transpose). The first experiment had Side and an exchange of two groups only, and left 9 passes on 10-platform. The order that the note of the review worked out by hand needs External to go from one end of the top groups to the other, which no exchange of two groups reaches without a worse order between. Shift reaches it. Without Transpose, 07-compiler had 3 crossings, 1 more than before.

## 2026-11-25

Decision for the pass that no order removes: the pass stays, the detector still reports it, and the test names it. A path that goes round needs a box that a link leaves through its left side, or other ranks, and both are a change to the route model or the rank phase: item 0154. On 10-platform 2 passes stay, and 2 is the fewest for its ranks. Shopping and Fulfilment are both in Services and both have ranks 3 to 5. Stores is on one side of both. With Shopping nearer to Stores, Orders -> Order database (ranks 3 to 4) and Inventory -> Inventory database (4 to 5) pass through Shopping. With Fulfilment nearer, 5 links of Shopping pass through Fulfilment.

## 2026-11-25

Criteria 1, 2 and 3. Ran 'scripts/task check' at the working tree of commit e627417: exit 0. Ran 'scripts/task test:core' with its own log: exit 0, and tests/layout.rs printed '19 passed; 0 failed' natively and on wasm32-wasip1. The test no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram now allows no link through a group on ten of the eleven diagrams, 07-compiler among them, and holds the two links of 10-platform by name: 'link Orders -> Order database passes through group Shopping' and 'link Inventory -> Inventory database passes through group Shopping'. Its comment gives the reason and it asserts the ranks that the reason rests on. Before: 1 and 17. The same test allows no other kind of collision on the eleven diagrams. each_link_label_sits_on_its_path_and_overlaps_no_box now asserts 0 labels inside a group that holds neither end, on each diagram: before, 4 on 10-platform. With the call of keep_out switched off for one run, four tests failed: the corpus test with 'link Fixes -> Source passes through group Core', the label test with 'left: 4 right: 0', the crossings test, and no_group_stands_between_two_groups_that_a_link_joins with 'link a -> c passes through group Middle'. a_link_beside_a_group_stays_on_one_side_of_it failed the same way with 'link n3 -> n0 passes through group g0'. tests/layout.positions changed for 07-compiler (22 of 39 lines) and 10-platform (88 of 96 lines), and the other nine diagrams have the same lines as before. Looked at the two new pictures as SVG in Chromium: on 07-compiler Fixes -> Source goes over Core, and on 10-platform the two named links are the only lines through a group that they have no end in.

## 2026-11-25

Criterion 4. The crossing count is the number of points where a level part of one path crosses an upright part of another path, in the picture that the crate draws (fn crossings in tests/layout.rs). Before the change a probe printed, for the reference file and the ten spike diagrams: 0, 0, 0, 3, 0, 0, 4, 2, 23, 2, 49. After: 0, 0, 0, 3, 0, 0, 4, 1, 23, 2, 18. No number rose: 07-compiler fell from 2 to 1 and 10-platform from 49 to 18. The new test the_crossings_of_each_corpus_diagram_do_not_rise holds the eleven numbers, and it failed with 'left: [0, 0, 0, 3, 0, 0, 4, 2, 23, 2, 49]' with keep_out switched off. Not measured: the count of the tool in spikes/layered-layout, which draws with the prototype of the spike and not with the crate.

## 2026-11-25

Generated graphs, measured with a probe that is not committed: the generator of no_generated_graph_has_a_fault with 3000 graphs. Seed 1: graphs with a pass 982 before and 358 after, passes 4395 before and 860 after, crossings 19125 before and 18533 after. Seed 7: 990 and 347 graphs, 4399 and 867 passes, 18859 and 18166 crossings. On all 6000 graphs the pass count of the search was equal to the count of kind LinkThroughGroup of the overlap detector, and faults and collisions found nothing but LinkThroughGroup and Labels. Not known: how many of the passes that remain another order removes. The search tries one move at a time. Cost: 10-platform takes 1.2 ms in a release build, 0.5 ms before. A random graph of up to 96 nodes and 23 groups took 0.31 s at the worst, 13 ms before: item 0155 has the table and the work.

## Result

A second part of the order phase (layout/through.rs) searches for an order of the ranks with no link through a group that holds neither of its ends. It moves a group among the groups under its parent, puts a link on one side of a group, and exchanges two neighbours of a rank. On the eleven test diagrams the passes fall from 18 to 2, and no crossing count rises. The 2 that stay are on 10-platform, no order of its ranks has fewer, and the corpus test names both: item 0154 has the path that goes round. The search is slow on a large graph with many passes: item 0155.

## 2026-11-25

Fix round 1 of pull request 48. Finding 1 (a pass stays on a graph of 6 nodes and one group): true. New move Clear in layout/through.rs: every item beside a group, under the same parent, goes above the group or below it in every rank of the group. New tests every_link_beside_a_group_takes_one_side_of_it (the graph of the review) and no_link_passes_through_the_only_group_of_a_generated_graph (500 graphs with one group). With the layout code of commit 9ecebec both failed: 'link n4 -> n3 passes through group g0' and 'link n12 -> n10 passes through group g0'. A probe that is not committed ran 20000 graphs with one group, three times (seeds 3 and 5 with 2 to 31 nodes, seed 9 with 2 to 7 nodes): 0 keep a pass. On the 3000 graphs of the generator the passes fall again: seed 1 from 860 to 640 (graphs with a pass 358 to 261), seed 7 from 867 to 576 (347 to 240). The crossings rise: seed 1 from 18533 to 19183, seed 7 from 18166 to 19072. tests/layout.positions did not change, so the eleven pictures and the four criteria are as before.

## 2026-11-25

Fix round 1, finding 2 (cost on a structured graph): true. Three changes, and none changes a result: the pass and crossing numbers of the 6000 generated graphs were equal before and after each one. (1) Shift is tried only for a group that a link passes through and for the groups under the same parent. (2) The search keeps the positions, the crossings of each corridor and the state of each step that can pass, and after a move it counts again only the ranks that the move changed. A debug build compares that with a count from nothing after every move. (3) fn crossed in layout/order.rs counts with a merge sort and not with a look at every pair. Measured in a release build, one layout call, the least of five runs, k copies of 10-platform with three links from each copy to the next, 1, 2, 3, 4 and 8 copies. Commit 9ecebec: 1.2 ms, 6.9 ms, 28 ms, 63 ms, 0.51 s. Now: 1.3 ms, 3.6 ms, 7.8 ms, 13 ms, 49 ms. With the search switched off: 0.4, 1.2, 2.2, 3.3, 9.8 ms. Not solved: a random graph of up to 192 nodes and 47 groups takes 3.0 s at the worst (9.1 s before, 62 ms with no search). Item 0155 has both tables and the work that is left. 'scripts/task check' exited 0, and tests/layout.rs printed '21 passed; 0 failed' on wasm32-wasip1.

## 2026-11-25

Correction to the note before this one: change (1), the limit on Shift, came in the same step as Clear, so no run shows that (1) alone changes no result. Changes (2) and (3) and the count of the changed ranks were each measured apart: 640 and 576 passes, 19183 and 19072 crossings, before and after.

## 2026-11-25

Review round 2: Move::Clear now clears each group of the chain of the group, so a group inside a group gets a clear side at every level. Before the change, every_link_beside_a_group_inside_a_group_takes_one_side_of_it printed 'link n0 -> n5 passes through group Inner' and the generated test of 1000 graphs with one group inside another failed at graph 55. After the change, cargo test -p horsegirl-core --test layout printed '23 passed; 0 failed', tests/layout.positions did not change, and scripts/task check exited 0.
