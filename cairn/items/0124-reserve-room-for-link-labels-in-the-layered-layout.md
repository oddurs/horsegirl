---
id: 124
uid: b752ed2a-54bf-43c6-a533-4c7b1997adb6
title: Reserve room for link labels in the layered layout
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
---

## Problem

A link label needs room between two ranks. The router can put a label on a path only if the place phase left that room. The SPIKE-18 prototype has no labels, so the rank gap is fixed and a long label would cover a box.

## Proposal

Give a labelled link one extra point in the layered solver: an item with the size of the label, in a rank between the two ends. The order phase and the place phase treat the item like a bend with a size. The layout returns the centre of each label, and the router draws the path through it.

## Acceptance criteria

- [x] A labelled link between two things in neighbouring ranks leaves a gap at least as wide as the label.
- [x] The layout returns one position for each label, and no label box overlaps a thing box on the ten spike diagrams with a label added to every link.
- [x] A link without a label keeps the gap it has today.

## 2026-11-25

From item 0033: the route phase already places each label. A label is centred on the upright line of the middle step of its link, or in the middle of the corridor when that step is level, and the corridor between the two ranks is as wide as the widest label plus 6 px at each side. So no label overlaps the box of a thing. What remains for this item: two labels can cover each other (29 pairs on the eleven test files with a label on every link; the detector kind is Labels), an upright line of another link can pass through a label, and the label of a level step sits where the upright lines of its corridor are. The test each_link_label_sits_on_its_path_and_overlaps_no_box allows the kind Labels: remove that when this item is done.

## 2026-11-25

From item 0033, review round 3: a link label can lie on the name of a group. The layout now moves a label that lies across the top border of a group, or in the 30 px band under it that holds the name (GROUP_TITLE), to 3 px above the border or 3 px under the band. Cases that remain for this item: the upright line of the label has no clear height (a short line), or the label is on a level step in the corridor and so is on no upright line. Measured on 3000 graphs from the generator of no_generated_graph_has_a_fault with each group named 'Services and stores N': 10 link labels lie on a group name in 10 graphs (49 in 44 graphs before the change). The detector reports them as kind Labels, which the corpus test and the generated-graph test allow. The same run has 92 cases of a group name across the border or the name of another group: item 0141 has that.

## 2026-11-25

From review round 4 of item 0033: a loop label has 0.2 px to the link that leaves the same box. Input 'G {\n  a\n}\na -> a: again\na -> b\n': the label box ends at y 47.82 and 'a -> b' leaves at y 48, so the line runs under the whole word, and no detector kind sees it. Also: the box of a label can start above y 0 ('a -> d: skip\na -> b\nb -> c\nc -> d\n' gives y -7.82 to 7.82), because to_origin reads no label. scene::build moves the scene to its own bounds, so the SVG is correct.

## 2026-11-26

Set surface to svg: a labelled picture can change (labels that covered each other now stand apart), so the change has a CHANGELOG entry under Changed.

## 2026-11-26

Criterion 1: 'scripts/task check' at commit 8f27c58 passed a_label_wider_than_the_rank_gap_widens_the_gap natively and on wasm32-wasip1. Input 'a -> b: a label that is much wider than the gap' / 'b -> c': the room between the boxes of a and b is larger than the measured width of the label, and b to c keeps RANK_GAP. The new test two_labels_of_one_corridor_do_not_cover_each_other ('a -> b: to b', 'b -> c', 'a -> c: to c') passed too, with no collision of any kind; with the src change stashed it failed with 'label a -> b covers label a -> c'.

## 2026-11-26

Criterion 2: each_link_label_sits_on_its_path_and_overlaps_no_box passed at 8f27c58 natively and on wasm32-wasip1. It gives every link of the ten spike diagrams and checkout.horse a label (255 labels), checks that each labelled link has exactly one centre and each unlabelled link none (faults), and allows only the kind LinkThroughGroup: no LabelOnThing, no LabelOnGroup, no LabelOffLink and, new in this item, no Labels. Before the change the same test without the Labels allowance found 24 pairs of labels that covered each other.

## 2026-11-26

Criterion 3: a_link_without_a_label_keeps_the_rank_gap passed at 8f27c58 natively and on wasm32-wasip1: 'a -> b' / 'b -> c' / 'a -> c' keeps exactly RANK_GAP between each pair of ranks, and in 'a -> b: to b' / 'b -> c' the unlabelled corridor keeps RANK_GAP. the_same_input_gives_the_same_positions_on_every_run passed with tests/layout.positions unchanged (git diff shows no change to the file), so every corpus picture, checkout with its labels included, has the positions it had.

## 2026-11-26

Measured over 3000 graphs of the generator of no_generated_graph_has_a_fault (round count raised for one local run, not committed): Labels overlaps fell from 716 to 27. 19 are two loops of one node (filed BUG-169). 8 come from the move off a group border, where every height off the borders covers a label; with that move switched off the route phase leaves 0. Filed BUG-170 for those and for link lines that pass through a label. The committed 300-graph test now allows Labels only for two loops of one node.

## Result

The route phase places each label of a corridor so that it covers no other label of that corridor: on its upright line, or on a level line of its step, and the corridor grows when no place inside it is free. A link without a label keeps the gap it had, and tests/layout.positions did not change. On the eleven test files with a label on every link no label covers another. Two loops of one node (BUG-169), a label that moves off a group border onto another label, and a link line through a label (BUG-170) remain.
