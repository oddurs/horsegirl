---
id: 139
uid: 3fb9b40e-d2cd-40a5-bfbd-8c8c2217e216
title: A thing placed in two groups, or a group placed inside itself, is an error
type: bug
status: backlog
milestone: v0.1
depends_on:
- 24
- 25
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 27
---

## Problem

The model gives each thing one parent: the first group whose braces hold it. `G { A }` and then `H { A }` leaves `A` in `G` and `H` empty, with no diagnostic. `A { B }` and then `B { A }` would make a loop, so the model drops the second placement, also with no diagnostic. `A, B { C }` places `C` in `A` only. An agent that writes one of these lines learns nothing.

## Proposal

Report the placement that the model drops. Give it a stable code, the span of the name in the later braces, and a message that names the group that holds the thing already. Where removing the later name is safe, the fix removes it.

## Acceptance criteria

- [ ] `G { A }` then `H { A }` gives one error at the second `A` that names `G`.
- [ ] `A { B }` then `B { A }` gives one error at the second `A` that says the group would hold itself.
- [ ] `A, B { C }` gives one error that says a part belongs to one group.
- [ ] The reference file gives no such error.
