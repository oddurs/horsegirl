---
id: 45
uid: a26b6dfb-d23b-4b6f-a248-b72c4692c524
title: Design tokens and base components from the concept page
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 44
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] The tokens match the concept page: ground, ink, muted, line, accent, three radii, one spacing scale.
- [x] Each component renders in light and dark mode with visible keyboard focus.
- [x] The site passes an automated accessibility check with no serious finding.

## 2026-11-25

Criterion 1: site/tests/tokens.test.ts reads the v8 block and the diagram-token block of docs/concept/index.html and compares every token with site/app/tokens.css (light, dark, --d-*). Ran pnpm test in site/: pass 3, fail 0. With --r2 changed to 11px and dark --muted to #9a9fae, the test failed with '--r2 in the light tokens' and '--muted in the dark tokens'. --bg is named --ground on the site.

## 2026-11-25

Criterion 2: site/scripts/check-a11y.mjs presses Tab through each page in light and dark at 1280px and 390px and fails on a focus stop with no outline of 2px or more. Ran scripts/task check: exit 0, 'check-a11y: 8 page loads, no serious finding, focus visible at every stop' (10 stops on /design/). With the focus rule set to outline:none, it printed 'no visible focus on a "Skip to content"' and the others, and exited 1. Screenshots of /design/ read by eye: light desktop and phone with webshot; dark desktop and phone with the browser tool, with the focus ring on the segmented control and on the table region.

## 2026-11-25

Criterion 3: the site build runs axe-core 4.14.0 on every exported page, in light and dark, at 1280px and 390px, and fails on a serious or a critical finding. scripts/task check printed 0 findings for each of the 8 page loads. With light --muted set to #c8c9cf, it printed 'serious color-contrast' and exited 1.

## 2026-11-25

Not ported: the webfonts (filed as 0126), the data-theme override and the rosette, blueprint and ink diagram themes. The site follows prefers-color-scheme only, because no control sets a theme yet.

## Result

The site has the concept page tokens in site/app/tokens.css, held to the concept page by site/tests/tokens.test.ts, and seven base components in site/components, shown on /design/. The site build fails on a serious axe finding or a focus stop with no outline, in light and dark, at 1280px and 390px.
