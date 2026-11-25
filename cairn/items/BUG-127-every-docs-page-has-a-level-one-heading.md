---
id: 127
uid: 24f2f07f-46c3-4a86-9a70-b2b52c2e6c85
title: Every docs page has a level-one heading
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

`docs/roadmap-intro.md` has no `#` heading. The site renders the file at
`/docs/roadmap-intro/`, and axe reports `page-has-heading-one` (moderate) on
that page in all four runs of `check-a11y`. The check passes because only
serious and critical findings fail it, so the note is easy to miss.

## Acceptance criteria

- [ ] `/docs/roadmap-intro/` has exactly one `h1`, and `check-a11y` prints no `page-has-heading-one` note for any page.
- [ ] A docs page without a level-one heading either gets one from its title or fails the build, with a test in `site/tests/docs.test.ts`.
- [ ] `cairn render` still produces the same `ROADMAP.md` from `docs/roadmap-intro.md`.
