---
id: 170
uid: c4a03ce8-da70-45d9-8f1b-af77cd1be70b
title: No link line passes through the label of another link
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
effort: m
related:
- 124
- 152
---

## Problem

Item 0124 keeps two labels of one corridor apart, but a label can still lie on a line of another link. The upright line of another trunk can pass through a label, and a label on a level step, or a label that moved onto a level line, can sit where the upright lines of its corridor are. The overlap detector in `crates/horsegirl-core/src/overlap.rs` has no kind for a link across a label, so no test sees it.

A second, smaller case: `off_borders` in `crates/horsegirl-core/src/layout.rs` moves a label off a group border along its upright line. When every height off the borders covers another label, the label still moves, and it covers that label. Over 3000 graphs from the generator of `no_generated_graph_has_a_fault`, 8 pairs of labels remain of this kind (graphs 428, 503, 585, 807, 1707, 1950, 2108 and 2387). The first 300 graphs, which the test runs, have none.

## Proposal

Add a detector kind for a link that crosses the box of a label of another link. In the route phase, treat the upright lines of a corridor as obstacles when `place_label` in `layout/route.rs` chooses a place. Let `off_borders` widen its search to the level lines of the step, as `place_label` does.

## Acceptance criteria

- [x] The overlap detector reports a link that crosses the label of another link, with a test.
- [x] On the eleven test files with a label on every link, no link crosses the label of another link.
- [x] Over 3000 generated graphs, no two labels cover each other but for two loops of one node (item 0169).

## 2026-11-26

Criterion 1: at a6f7faf, scripts/task check passed a_link_across_the_label_of_another_link_is_found in tests/overlap.rs natively and on wasm. A link across the label box gives (LinkOnLabel, 2, 1); a link that shares the upright line of the label gives nothing; a link that turns off inside the label gives LinkOnLabel; a line along the edge gives nothing. A part of a link that lies on the link of the label is no collision, because two links from one thing share the line that leaves it; the SVG backend now stops that shared line at the label too.

## 2026-11-26

Criterion 2: at a6f7faf, each_link_label_sits_on_its_path_and_overlaps_no_box passed natively and on wasm with only LinkThroughGroup allowed, so no LinkOnLabel on the eleven files (255 labels). With a probe test that counts LinkOnLabel on the same eleven labelled files: 226 before the layout change, 0 after. no_link_crosses_the_label_of_another_link_in_the_reference_flow and the SVG test no_link_line_is_drawn_across_the_text_of_a_link_label failed with the src change stashed (create and charge crossed) and pass with it. The corpus gate: 224 of 239 clean, 0 files with LinkOnLabel.

## 2026-11-26

Criterion 3: a probe test (removed before commit) ran 3000 graphs of the generator of no_generated_graph_has_a_fault (seed 7) at the final code: 0 pairs of labels that cover each other after the two_loops_of_one_node filter (1 before: graph 2573, BUG-172), 0 collisions of any other kind but LinkThroughGroup and LinkOnLabel, and 1 LinkOnLabel (graph 17, filed as BUG-176; 10804 before). The seed-3 generator of no_generated_graph_with_links_to_groups_has_a_fault, run to 3000 rounds, gave no collision but LinkThroughGroup (graph 1087 of BUG-172 included). The fix that removed the 8 pairs of the item body is in the route phase: place_label now reads the group borders and name bands, so off_borders rarely has to move a label.

## 2026-11-26

Review round 1, criterion 1 unticked: the detector and the SVG backend used two rules for a line that a link shares with the link of a label. The detector tested the label box, and the backend tested the padded box. When the shared line ended inside the pad, the backend drew the rest through the text and the detector reported nothing (n2 -> n3 through 'send it', and xbrnkcj -> ftrgjbz through 'ok then'). Now overlap::meet decides for both, over the label box. With the fix, a_shared_line_that_ends_near_a_label_is_not_drawn_across_it passes; with the svg.rs of f7af7f2 it fails with [(["n2", "n3"], "send it")] and, alone, with [(["xbrnkcj", "ftrgjbz"], "ok then")]. A probe over the 5000 generated graphs of the review gave 0 drawn crossings of a label (2 before) and 0 LinkOnLabel; scripts/task check passed.

## 2026-11-26

Review round 1, speed: place_label drops the lines that cannot reach its path, drops repeated spots, sorts the spots by every cost but the crossing count, stops at the first clear spot that no line crosses, and tests a spot only against the labels and lines in a window around it. The SVG backend looks up the link of a label only for a link that enters the label box. a_wide_labelled_fan_places_its_labels_in_little_work counts the tests for a fan of 200 out and 200 in: 838 million before, 75 013 after, bound 120 000. Release horsegirl draw, 400-link fan in and out: 10.1 s before, 0.32 s after; 500 things with 800 labelled links: 7.4 s before, 0.35 s after. Criterion 3 rerun with the generator of no_generated_graph_has_a_fault at 3000 rounds: only graph 17 LinkOnLabel (BUG-176), no covered labels.
