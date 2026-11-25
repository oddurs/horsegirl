---
id: 56
uid: 9bb84588-529d-44c7-a674-0e027f86ec5e
title: The grid solver
type: feature
status: backlog
milestone: v0.2
depends_on:
- 31
- 33
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

Sequence and lanes place things on two named axes, not by dependency. The layered solver cannot do that.

## Proposal

Build the grid solver: each axis is bound to a field, cells hold things, and links route between cells at right angles.

## Acceptance criteria

- [ ] The solver places the checkout story on step by owner with no overlap.
- [ ] Positions are deterministic.
