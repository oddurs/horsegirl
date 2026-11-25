---
id: 129
uid: 2c984d5e-83f4-4dee-846f-166a65c60e6b
title: The site highlighter colours a link's target as a fact key
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: site
---

## Problem

site/lib/highlight.ts colours any name before a colon as a fact key. In a labelled link such as `Web app -> API gateway: POST /checkout`, the target `API gateway` gets the key colour, the same as `owner` in `Payments { owner: Commerce }`. The landing page and the docs show the reference file with this fault.

## Acceptance criteria

- [ ] In `A -> B: label`, `B` is not marked `hg-key`, and a test in site/tests holds it.
- [ ] In `A { owner: X }`, `owner` is still marked `hg-key`.
