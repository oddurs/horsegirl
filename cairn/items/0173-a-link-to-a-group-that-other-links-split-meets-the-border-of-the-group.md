---
id: 173
uid: a93d9d76-fe0b-42c1-801b-edfb76164d16
title: A link to a group that other links split meets the border of the group
type: feature
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
---

## Problem

Item 0123 draws a link to a group to the border of the group box. When other links put things of the group on both sides of the other end, as in `Orders -> Audit`, `Audit -> Payments` and `Audit -> Services` with `Services { Orders, Payments }`, no rank of the group faces the other end. The link then ends at a port in a middle rank, inside the group box. The picture shows an arrow that ends in the empty room of the group.

## Acceptance criteria

- [ ] A decision states how such a link reads: an end on the top or bottom border of the group, a warning, or the present port.
- [ ] The flow lens draws the example above to the chosen form, with no collision from the overlap detector.
