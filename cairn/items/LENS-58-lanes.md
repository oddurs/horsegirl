---
id: 58
uid: 6cea6647-3a30-439a-b72d-913888f47abe
title: lanes
type: lens
status: backlog
milestone: v0.2
depends_on:
- 55
- 56
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: lens
track:
- pictures
---

## The question this picture answers

Where the handoffs between owners are.

## Dials

`x: step` · `y: owner`

## Facts it needs

A story, and an owner on every thing in it. Warning W0221 lists the things with no owner.

## Solver

Grid.

## Example

```
show lanes of checkout
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] `describe` reads the picture back as text.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
