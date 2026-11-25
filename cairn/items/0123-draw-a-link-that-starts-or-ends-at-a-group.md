---
id: 123
uid: 2a2bf815-3818-459e-82ea-da40b5cfc781
title: Draw a link that starts or ends at a group
type: feature
status: backlog
milestone: v0.1
depends_on:
- 32
- 33
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
---

## Problem

A group is a thing, so a file can link to it: `API gateway -> Services`. The layered solver from SPIKE-18 places only links between things that hold nothing. The spike left this case out, and no candidate was measured on it.

## Proposal

Decide how the flow lens draws a link whose end is a group, and build it in the layered solver. The link ends at the border of the group box. The rank phase treats the group as one unit for that link, so that the group is not ranked before a thing that links to it.

## Acceptance criteria

- [ ] `A -> G` with `G { B, C }` draws a path that ends at the border of the box of `G` and enters no box inside it.
- [ ] `G -> A` draws a path that starts at the border of the box of `G`.
- [ ] A link from a thing inside `G` to `G` itself gives a diagnostic or a picture, and the choice is written in the lens reference.
- [ ] No two boxes overlap on a diagram that has such links.
