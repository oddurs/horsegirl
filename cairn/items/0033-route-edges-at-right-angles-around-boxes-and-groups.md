---
id: 33
uid: 7978c04f-2397-4fe3-9dd3-28fda10516a3
title: Route edges at right angles around boxes and groups
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] No path crosses a box on any corpus diagram.
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
