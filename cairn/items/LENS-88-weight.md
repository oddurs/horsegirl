---
id: 88
uid: c6021c26-dd3c-4ad7-a420-b8094ba33c69
title: weight
type: lens
status: backlog
milestone: v0.3
depends_on:
- 59
- 87
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: language
effort: s
area: lens
track:
- pictures
---

## The question this picture answers

What is big.

## Dials

`inside: parent` · `area: size`

## Facts it needs

Groups, and a numeric fact on each thing.

## Solver

Nested.

## Example

```
show weight { area: lines }
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] `describe` reads the picture back as text.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
