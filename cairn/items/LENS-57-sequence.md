---
id: 57
uid: a490f22b-3f49-42cd-947c-40b86f8f7d28
title: sequence
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

What happens next.

## Dials

`x: thing` · `y: step`

## Facts it needs

A story. A reply is inferred when a link returns to the thing that called.

## Solver

Grid.

## Example

```
show sequence of checkout
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] `describe` reads the picture back as text.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
