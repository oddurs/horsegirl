---
id: 33
uid: 7978c04f-2397-4fe3-9dd3-28fda10516a3
title: Route edges at right angles around boxes and groups
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: high
surface: none
effort: l
area: layout
track:
- pictures
---

## Problem

Edges drawn as straight lines cross boxes and each other. Right-angle routing is what makes a diagram look arranged.

## Proposal

Route each link as an orthogonal path between ports on its two boxes. Paths avoid boxes and leave a group through its border once.

## Acceptance criteria

- [ ] No path crosses a box on any corpus diagram.
- [x] Each link label sits on its path and overlaps no box.
- [x] The overlap detector that the corpus test uses is part of this item.

## 2026-11-25

From item 0032: Layout.bends holds, for each link, one point for each rank between its two ends, from its start to its end. A link that the layout turned round to break a loop has its points in the order of the link as the file states it. The points keep 12 px from a neighbour in the same rank and are outside every group that does not hold both ends. The place phase gives each position along a rank as a sum of whole pixels and gaps, so two boxes on one line have equal centres.

## 2026-11-25

Criterion 1, with 'box' read as the box of a thing, as the proposal reads it ('Paths avoid boxes and leave a group through its border once'), and 'corpus' read as examples/checkout.horse plus the ten diagrams in spikes/layered-layout/corpus, as item 0032 read it. Ran 'scripts/task check' at commit b5e8d47: exit 0, and the test no_path_crosses_a_box_on_any_corpus_diagram passed natively and on wasm32-wasip1. It draws 255 links and the overlap detector finds no link across the box of a thing, no link that crosses the border of its own group more than once, no two links on each other and no two boxes that overlap. With every upright line moved 120 px for one run the same test failed with 'link Web app -> API gateway crosses thing Mobile app'. no_generated_graph_has_a_fault holds the same rules on 300 generated graphs. Not zero: 18 links pass through a group that holds neither of their ends, 1 on 07-compiler and 17 on 10-platform. The order phase puts such a link on two sides of the group in two ranks, and the route phase cannot go round. The test holds the two numbers and item 0146 has the work. If 'box' must include such a group, this criterion is not met.

## 2026-11-25

Criterion 2: the test each_link_label_sits_on_its_path_and_overlaps_no_box passed in the same run, natively and on wasm32-wasip1. It gives every link of the eleven files a label, 255 labels, and the detector finds no label on the box of a thing and no label whose centre is off its link. The reference file as it is has 7 labels and no collision of any kind. With the label room removed for one run the test failed with 'label Mobile app -> API gateway covers thing Mobile app'. Not covered: two labels can cover each other (29 pairs on the eleven files with a label on every link, 13 of them on 08-module-deps), and an upright line of another link can pass through a label. Item 0124 gives each label its own room.

## 2026-11-25

Criterion 3: horsegirl_core::overlap::overlaps reads a Scene and returns each collision with its kind and its two shapes. crates/horsegirl-core/tests/overlap.rs has 13 tests, one or more for each kind, and they passed natively and on wasm32-wasip1. The corpus tests in tests/layout.rs build the scene of each picture with scene::build and run the detector on it.

## 2026-11-25

What the next items need to know. Layout.paths has the points of each link and Layout.labels the centre of each label; Graph.labels takes the size of each label. fn scene in crates/horsegirl-core/tests/layout.rs turns a Layout into a Placement and a Scene: LENS-36 owns that step. A bend is no longer outside every group that does not hold both ends: it lies inside a group of one end while that group has the rank of the bend. A path can pass a bend up to 6 px from the point in Layout.bends, when two upright lines each arrive where the other leaves. The gap between two ranks is no longer fixed: it grows with the upright lines and with the widest label between the two ranks. A node with a link to itself makes its rank wider by the loop room at both sides. Looked at the eleven pictures as SVG in Chromium.

## Result

The layout routes each link at right angles and turns only between two ranks, so no path crosses the box of a thing on the eleven test diagrams, and each label is on its path and off every box. horsegirl_core::overlap::overlaps reads a scene and reports each collision. 18 links still pass through a group that holds neither of their ends: item 0146.

## 2026-11-25

Review round 1: criterion 1 is unticked and the item is open again. Both reviews read 'box' as the box of a thing or of a group, as the title and the code do. With that meaning the criterion is not true: 18 links pass through a group that holds neither of their ends, 1 on 07-compiler and 17 on 10-platform. Zero is not possible on 10-platform with the route phase as it is, for a reason that no order removes. Shopping has ranks 3 to 5 and Fulfilment has ranks 3 to 6, both inside Services. Orders (rank 3, Fulfilment) links to Order database (rank 4, Stores), and Cart (rank 4, Shopping) links to Cache (rank 5, Stores). Stores is on one side of Services. Each of the two groups must then be on that side of the other, and a path that only goes to the right cannot go round. The criterion becomes true in one of two ways. The owner narrows it to 'the box of a thing', and item 0146 holds the group case. Or item 0146 lands first with a path that goes round a group, or with other ranks. The comments on Layout::paths, on the layout module and on the route module now say 'no box of a node' and name the group case. The test is now no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram.

## 2026-11-25

Review round 2: both reviews found a link label across the border of a group, and no detector kind saw it. The earlier note is wrong where it says the reference file has 'no collision of any kind': the label 'save' of Orders -> Postgres had y 107.2 to 122.8 and the bottom border of Services is at y 110. Criterion 2 was unticked. Added Kind::LabelOnGroup to the detector: a label shares room with a group and the group does not hold it. With the kind and no other change, three tests failed: 'checkout: label Orders -> Postgres lies across the border of group Services' in both corpus tests and 'graph 5: label n10 -> n9 lies across the border of group g1' in no_generated_graph_has_a_fault. A probe gave 1 label on the reference file as it is and 9 on the eleven files with a label on every link (checkout 1, 01 1, 02 1, 06 1, 07 1, 10 4), the counts of the reviews. The layout now moves such a label along its upright line to LABEL_CLEAR (3 px) from the border, at the height nearest to the middle of the line (fn off_borders in layout.rs). After the change the probe gave 0, and the tests allow no collision of this kind. Ran 'scripts/task check': exit 0, with each_link_label_sits_on_its_path_and_overlaps_no_box, no_generated_graph_has_a_fault, a_label_on_a_line_that_leaves_a_group_is_off_the_border_of_the_group and a_label_across_the_border_of_a_group_is_found passed natively and on wasm32-wasip1. For one run, 20 000 generated graphs with node heights from 36 px to 112.5 px gave no collision but LinkThroughGroup and Labels. layout.positions changed on one line: the label of Orders -> Postgres went from y 115 to y 120.8203125. Criterion 2 is ticked again with 'box' read as the box of a thing or of a group: no label is on the box of a thing or across the border of a group. Not covered: a label can be inside a group that holds neither end of its link, 6 labels on 10-platform by the count of the review, because the link passes through that group (item 0146). A label stays across a border when its upright line has no clear height, and no test graph has that case. Did not look at the new picture in a browser. Filed from the minor findings: BUG-147 (two links between the same two things in opposite directions), BUG-148 (an arrival that cannot move, a node lower than 24 px), 0149 (a thing in the wrong group is not seen by the detector), 0150 (tracks is cubic).
