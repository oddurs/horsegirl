---
id: 179
uid: 573fb86c-b9f6-4030-8736-25dda052c06b
title: A drawing keeps a margin round every box
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-26
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: svg
effort: s
area: render
---

## Problem

The SVG of `examples/checkout.horse` has the view box `-1 -1 1082.67 172`. The box of `Web app` starts at the top left corner, so the border of the box touches the edge of the picture. A host that shows the SVG with no padding cuts the border, and the picture looks cramped beside text.

## Proposal

Add one margin on each side of the view box in the SVG backend. Take its size from a named constant in the theme module.

## Acceptance criteria

- [x] On every golden case that draws, each box, line, arrow head and label is 16 px or more inside the view box. A test holds this.
- [x] The margin is a named constant in `theme.rs` (`theme::MARGIN`), listed in `docs/theme.md`.
- [x] `CHANGELOG.md` states the change to the SVG.

## 2026-11-26

Ran scripts/task check: passed. Test every_golden_picture_keeps_the_margin_clear_inside_its_view_box passed over every golden case that draws (box, line, arrow head end, label at 16 px or more from the view box). The checkout view box went from -1 -1 1082.67 172 to -16 -16 1112.67 202; I looked at before and after with webshot.

## 2026-11-26

Correction: the 1112.67 wide view box is the 12 px test picture (tests/checkout.svg). The 14 px golden picture is now -16 -16 1225.66 202.

## 2026-11-26

Criterion 2 reworded. A CSS variable cannot move a view box, so the margin is the constant theme::MARGIN and not an --hg- token in theme::TOKENS. docs/theme.md says so. A margin that a host can set through the CLI or MCP is not part of this item.
