---
id: 45
uid: a26b6dfb-d23b-4b6f-a248-b72c4692c524
title: Design tokens and base components from the concept page
type: feature
status: planned
milestone: v0.1
depends_on:
- 44
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: m
area: site
track:
- site
---

## Problem

The concept page has a visual system: a neutral page, one brand colour, a type scale, and diagram tokens. The site needs it as code.

## Proposal

Port the tokens to the site as CSS variables with light and dark values, and build the base components: page shell, sticky header, section, code block, figure, table, segmented control.

## Acceptance criteria

- [ ] The tokens match the concept page: ground, ink, muted, line, accent, three radii, one spacing scale.
- [ ] Each component renders in light and dark mode with visible keyboard focus.
- [ ] The site passes an automated accessibility check with no serious finding.
