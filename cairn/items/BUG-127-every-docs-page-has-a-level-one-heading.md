---
id: 127
uid: 24f2f07f-46c3-4a86-9a70-b2b52c2e6c85
title: Every docs page has a level-one heading
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: none
area: site
---

## Problem

`docs/roadmap-intro.md` has no `#` heading. The site renders the file at
`/docs/roadmap-intro/`, and axe reports `page-has-heading-one` (moderate) on
that page in all four runs of `check-a11y`. The check passes because only
serious and critical findings fail it, so the note is easy to miss.

## Acceptance criteria

- [x] `/docs/roadmap-intro/` has exactly one `h1`, and `check-a11y` prints no `page-has-heading-one` note for any page.
- [x] A docs page without a level-one heading either gets one from its title or fails the build, with a test in `site/tests/docs.test.ts`.
- [x] `cairn render` still produces the same `ROADMAP.md` from `docs/roadmap-intro.md`.

## 2026-11-25

scripts/task check passed. check-a11y printed 0 findings for all 32 page loads, including /docs/roadmap-intro/ (grep for page-has-heading found 0 lines). New test in site/tests/docs.test.ts passed. cairn render left the roadmap-intro text of ROADMAP.md unchanged (only BUG-127 moved to doing).
