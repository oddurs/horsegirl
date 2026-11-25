---
id: 33
uid: 7978c04f-2397-4fe3-9dd3-28fda10516a3
title: Route edges at right angles around boxes and groups
type: feature
status: planned
milestone: v0.1
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
- [ ] Each link label sits on its path and overlaps no box.
- [ ] The overlap detector that the corpus test uses is part of this item.

## 2026-11-25

From item 0032: Layout.bends holds, for each link, one point for each rank between its two ends, from its start to its end. A link that the layout turned round to break a loop has its points in the order of the link as the file states it. The points keep 12 px from a neighbour in the same rank and are outside every group that does not hold both ends. The place phase gives each position along a rank as a sum of whole pixels and gaps, so two boxes on one line have equal centres.
