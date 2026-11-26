---
id: 152
uid: 4625474d-379e-4a64-a585-c267c37d3352
title: A link label stays legible where it sits on its own line
type: bug
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-26
created: 2026-11-25
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: render
effort: s
related:
- 124
---

## Problem

The layout puts the centre of a link label on the path of its link (item 0033). The SVG backend draws the label text with no background. So on a level step the line runs through the middle of the word, and the word reads as struck through. `examples/checkout.horse` drawn by the flow lens shows it on `create`, `charge`, `paid`, `publish` and `send`. The overlap detector does not count a label on its own link, so no test sees it.

## Proposal

Draw a link label on a background of the page colour, with a small pad, so its own line stops at the label. Or move the label off a level line by half its height plus a gap. Choose the one that keeps the label on its path for `describe`.

## Acceptance criteria

- [ ] On `examples/checkout.horse`, no link line is drawn across the text of its own label in the SVG.
- [ ] The label keeps its colour token in light and dark mode.
- [ ] A test holds the case.
