---
id: 31
uid: 94fbc561-d5e3-40b7-9d64-3c3720b105e2
title: 'The scene: positioned shapes that every backend draws'
type: feature
status: planned
milestone: v0.1
depends_on:
- 25
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: render
track:
- pictures
---

## Problem

Each backend must draw the same picture. That needs one description of positioned shapes that every backend reads and no backend changes.

## Proposal

Define the scene: boxes, lines with bend points, labels, groups, each with a position, a size, a role (thing, link, group, label) and the facts that style reads, such as owner.

## Acceptance criteria

- [ ] The scene serialises to stable JSON, held by a snapshot test.
- [ ] A scene carries enough to draw the reference flow with no access to the model.
