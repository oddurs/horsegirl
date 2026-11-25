---
id: 83
uid: d3db49f5-270c-444c-8af0-ae7f002118b5
title: seams
type: lens
status: backlog
milestone: v0.3
depends_on:
- 59
- 80
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: language
effort: s
area: lens
track:
- pictures
---

## The question this picture answers

Where owners depend on each other.

## Dials

`thing: owner` · `size: things owned` · `thickness: crossing links`

## Facts it needs

An owner on at least two things.

## Solver

Radial.

## Example

```
show seams by owner
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] `describe` reads the picture back as text.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
