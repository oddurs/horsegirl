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
