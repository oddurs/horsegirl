---
id: 35
uid: 7a142f13-59f7-4f47-9886-72be24bd6404
title: The plain theme, in light and dark
type: feature
status: planned
milestone: v0.1
depends_on:
- 34
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: theme
effort: s
area: render
track:
- pictures
---

## Problem

The default look must sit inside any document, in light and in dark mode.

## Proposal

Define the plain theme as twelve tokens with light and dark values, and bind the palette to the `owner` fact.

## Acceptance criteria

- [ ] The reference flow passes a contrast check of 4.5 to 1 for text in both modes.
- [ ] The tokens are listed in the reference documentation.

## 2026-11-25

SPIKE-19 answer: the plain theme's label font is Arial, "Liberation Sans", Helvetica, sans-serif, the face the core measures in. A theme can name another face; 0118 keeps positions unchanged.
