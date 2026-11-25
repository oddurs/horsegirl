---
id: 36
uid: 2bafa221-7bb2-44a5-b915-904571bee5f9
title: flow
type: lens
status: planned
milestone: v0.1
depends_on:
- 31
- 32
- 33
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

What depends on what.

## Dials

`x: depth` · `inside: group` · `colour: owner`

## Facts it needs

Links. With no links the compiler says that there is nothing to order.

## Solver

Layered.

## Example

```
Web app, Mobile app -> API gateway
API gateway -> Orders
show flow
```

## Acceptance criteria

- [ ] The lens draws the example with no overlap.
- [ ] Missing facts produce a diagnostic that names them.
- [ ] The lens is in the language reference with its dials.
- [ ] The lens is declared as data (field to channel), not as code paths in the solver.

## 2026-11-25

From the review of item 23: `show sequence of S, T` parses as two things with no message, and a `show` line whose argument needs quotes is not read as a `show` line. BUG-136 holds the inputs and the criteria. The flow lens must not rely on the parser to reject such a line until BUG-136 is done.

## 2026-11-25

From item 0032: the layered layout is horsegirl_core::layout::layout. It reads a Graph of boxes with sizes, groups and links, and knows nothing of the model. fn flow in crates/horsegirl-core/tests/layout.rs turns a Model into that Graph for the tests: a thing that holds something is a group, every other thing is a node as wide as its name at 14 px plus 28 px. The lens owns that step and the sizes.
