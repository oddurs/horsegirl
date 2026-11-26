---
id: 176
uid: f1c47645-c1d0-49fb-b8a9-666c8bd839b6
title: A label has a place that no line crosses when two lines run beside its path
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
related:
- 170
effort: m
---

## Problem

Item 0170 makes `place_label` in `crates/horsegirl-core/src/layout/route.rs` treat the lines of the other steps of a corridor as obstacles. Over 3000 graphs of the generator of `no_generated_graph_has_a_fault` (seed 7), one label still has a line across it: graph 17 gives "link `n6 -> n12` crosses label `n4 -> n12`". The label is on a level line at y 66. The line of `n6 -> n12` runs at y 59, 7 px above it, through the whole corridor. The upright line of the label is 12 px long and sits under the band of the name of `g0`, and its left level line is in that band. So every place crosses a line or lies across a border. The two bends in the next rank are closer than the height of a label.

`no_generated_graph_has_a_fault` allows at most one such crossing for this case.

## Proposal

Keep two level lines that carry a label at least the height of the label apart: let the place phase give a bend whose step holds a label room above and under it, or let the route phase move an arrival that runs beside a labelled line.

## Acceptance criteria

- [ ] Graph 17 of seed 7 draws with no `LinkOnLabel` collision.
- [ ] `no_generated_graph_has_a_fault` allows no `LinkOnLabel` collision.
- [ ] Over 3000 graphs of seed 7 and of seed 3, no link crosses the label of another link.
