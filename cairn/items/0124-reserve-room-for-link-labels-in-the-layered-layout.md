---
id: 124
uid: b752ed2a-54bf-43c6-a533-4c7b1997adb6
title: Reserve room for link labels in the layered layout
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

A link label needs room between two ranks. The router can put a label on a path only if the place phase left that room. The SPIKE-18 prototype has no labels, so the rank gap is fixed and a long label would cover a box.

## Proposal

Give a labelled link one extra point in the layered solver: an item with the size of the label, in a rank between the two ends. The order phase and the place phase treat the item like a bend with a size. The layout returns the centre of each label, and the router draws the path through it.

## Acceptance criteria

- [ ] A labelled link between two things in neighbouring ranks leaves a gap at least as wide as the label.
- [ ] The layout returns one position for each label, and no label box overlaps a thing box on the ten spike diagrams with a label added to every link.
- [ ] A link without a label keeps the gap it has today.

## 2026-11-25

From item 0033: the route phase already places each label. A label is centred on the upright line of the middle step of its link, or in the middle of the corridor when that step is level, and the corridor between the two ranks is as wide as the widest label plus 6 px at each side. So no label overlaps the box of a thing. What remains for this item: two labels can cover each other (29 pairs on the eleven test files with a label on every link; the detector kind is Labels), an upright line of another link can pass through a label, and the label of a level step sits where the upright lines of its corridor are. The test each_link_label_sits_on_its_path_and_overlaps_no_box allows the kind Labels: remove that when this item is done.

## 2026-11-25

From item 0033, review round 3: a link label can lie on the name of a group. The layout now moves a label that lies across the top border of a group, or in the 30 px band under it that holds the name (GROUP_TITLE), to 3 px above the border or 3 px under the band. Cases that remain for this item: the upright line of the label has no clear height (a short line), or the label is on a level step in the corridor and so is on no upright line. Measured on 3000 graphs from the generator of no_generated_graph_has_a_fault with each group named 'Services and stores N': 10 link labels lie on a group name in 10 graphs (49 in 44 graphs before the change). The detector reports them as kind Labels, which the corpus test and the generated-graph test allow. The same run has 92 cases of a group name across the border or the name of another group: item 0141 has that.
