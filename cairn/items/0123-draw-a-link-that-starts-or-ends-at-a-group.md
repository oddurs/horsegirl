---
id: 123
uid: 2a2bf815-3818-459e-82ea-da40b5cfc781
title: Draw a link that starts or ends at a group
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 32
- 33
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: diagnostics
area: layout
---

## Problem

A group is a thing, so a file can link to it: `API gateway -> Services`. The layered solver from SPIKE-18 places only links between things that hold nothing. The spike left this case out, and no candidate was measured on it.

## Proposal

Decide how the flow lens draws a link whose end is a group, and build it in the layered solver. The link ends at the border of the group box. The rank phase treats the group as one unit for that link, so that the group is not ranked before a thing that links to it.

## Acceptance criteria

- [x] `A -> G` with `G { B, C }` draws a path that ends at the border of the box of `G` and enters no box inside it.
- [x] `G -> A` draws a path that starts at the border of the box of `G`.
- [x] A link from a thing inside `G` to `G` itself gives a diagnostic or a picture, and the choice is written in the lens reference.
- [x] No two boxes overlap on a diagram that has such links.

## 2026-11-25

The corpus gate (crates/horsegirl/examples/corpus.rs, item 0040) counts a file as clean only when its picture draws every link. On 2026-11-25 the flow lens leaves out 113 links in 25 corpus files, because lens::bind drops a link whose end is a group. When this item draws those links, the count of clean files can rise or fall: a link that is now drawn can cross a box or a label. Run cargo run -p horsegirl --example corpus and set BASELINE to the new count.

## 2026-11-26

Criterion 1: cargo test -p horsegirl-core --test layout a_link_to_a_group_ends_at_the_border_of_the_group_box passed. For 'A -> G' with 'G { B, C }' the last point of the path has x equal to the left edge of the box of G and a y inside it, no point of the path is inside the box of G, A ranks before B and C, and the overlap detector reports nothing. tests/lens.rs a_link_to_or_from_a_group_meets_its_border_and_draws_with_no_overlap passed: in the scene, the link A -> G ends at x = the left edge of the group shape.

## 2026-11-26

Criterion 2: a_link_from_a_group_starts_at_the_border_of_the_group_box passed: for 'G -> A' the first point of the path has x equal to the right edge of the box of G, and A ranks after B and C. The lens test also holds 'G -> D' to the right edge of the group shape.

## 2026-11-26

Criterion 3: the choice is a diagnostic. W0110 (warning) reports a link between a group and itself or a thing inside it at any depth, and lens::bind leaves the link out. tests/check.rs a_link_between_a_group_and_what_it_holds_is_a_warning passed (three W0110 for 'Orders -> Services', 'Services -> Payments' with Payments in an inner group, 'Services -> Services'; none for 'Web -> Services' or 'Inner -> Orders'). docs/lenses.md has the section 'A link to a group' that states the choice; docs/reference.md and docs/diagnostics.md list W0110.

## 2026-11-26

Criterion 4: scripts/task check exited 0 (341 tests passed natively, the wasm run passed). no_generated_graph_with_links_to_groups_has_a_fault passed: 300 generated graphs with group ends, nested groups and labels give no fault and no collision but LinkThroughGroup. A local run of 30000 graphs gave no Boxes overlap; its only other collisions were label-on-label, which the node-only generator also gives (seed 7, graph 2573). cargo run -p horsegirl --example corpus printed '224 of 239 files draw with zero overlaps and every link', 'files with Boxes: 0'; main printed 206. BASELINE is now 224. The two files with a link left out, 0035 and 0220, hold W0110 links. Weak spot: a group with things on both sides of the other end (a loop through the group) gets its port in a middle rank, so the link ends inside the group box, not at its border.

## Result

The flow lens draws a link that starts or ends at a group. layout::Graph names each end as End::Node or End::Group; the rank phase reads a group end as every node it holds, and the link runs to a port in the group, so its path meets the border of the group box and enters no box inside. A link between a group and itself or what it holds gives W0110 and is left out. The corpus gate rose to 224 clean files. Follow-ups: BUG-171 (W0107 inside a linked group), BUG-172 (label on label in generated graphs), 0173 (a group split by other links).
