---
id: 81
uid: d8955fad-f01e-41aa-88e5-77d93d9b8e29
title: rings
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

How far a failure spreads from one thing.

## Dials

`radius: hops from focus` · `colour: owner`

## Facts it needs

A focus thing that exists. An unknown focus is an error with the nearest name.

## Solver

Radial.

## Example

```
show rings from Payments
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] `describe` reads the picture back as text.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
