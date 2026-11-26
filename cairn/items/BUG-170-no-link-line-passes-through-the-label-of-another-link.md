---
id: 170
uid: c4a03ce8-da70-45d9-8f1b-af77cd1be70b
title: No link line passes through the label of another link
type: bug
status: backlog
milestone: v0.1
created: 2026-11-26
updated: 2026-11-26
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

- [ ] The overlap detector reports a link that crosses the label of another link, with a test.
- [ ] On the eleven test files with a label on every link, no link crosses the label of another link.
- [ ] Over 3000 generated graphs, no two labels cover each other but for two loops of one node (item 0169).
