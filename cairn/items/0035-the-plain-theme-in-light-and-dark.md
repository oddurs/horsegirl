---
id: 35
uid: 7a142f13-59f7-4f47-9886-72be24bd6404
title: The plain theme, in light and dark
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 34
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] The reference flow passes a contrast check of 4.5 to 1 for text in both modes.
- [x] The tokens are listed in the reference documentation.

## 2026-11-25

SPIKE-19 answer: the plain theme's label font is Arial, "Liberation Sans", Helvetica, sans-serif, the face the core measures in. A theme can name another face; 0118 keeps positions unchanged.

## 2026-11-25

Ran cargo test -p horsegirl-core --test theme: 4 passed, including the_text_of_the_reference_flow_has_4_5_to_1_in_both_modes (ink on paper, c0, c1 to c3, and accent-ink on accent, light and dark) and twelve_tokens_are_listed_in_the_reference_with_both_values (docs/theme.md). Looked at the SVG in light and dark in a browser with webshot. scripts/task check passed.
