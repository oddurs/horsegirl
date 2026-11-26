---
id: 179
uid: 573fb86c-b9f6-4030-8736-25dda052c06b
title: A drawing keeps a margin round every box
type: bug
status: backlog
milestone: v0.1
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
effort: s
area: render
---

## Problem

The SVG of `examples/checkout.horse` has the view box `-1 -1 1082.67 172`. The box of `Web app` starts at the top left corner, so the border of the box touches the edge of the picture. A host that shows the SVG with no padding cuts the border, and the picture looks cramped beside text.

## Proposal

Add one margin on each side of the view box in the SVG backend. Take its size from the theme tokens, so a host can change it.

## Acceptance criteria

- [ ] On every golden case that draws, each box, line, arrow head and label is 16 px or more inside the view box. A test holds this.
- [ ] The margin is a theme token, listed in `docs/theme.md`.
- [ ] `CHANGELOG.md` states the change to the SVG.
