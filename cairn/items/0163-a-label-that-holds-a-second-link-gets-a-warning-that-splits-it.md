---
id: 163
uid: bb464413-cece-49df-a742-4568c0e1ac4f
title: A label that holds a second link gets a warning that splits it
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 131
---

### Problem

An agent that knows D2 writes `A -> B: pay -> C: ship`. The parser reads one link from `A` to `B` with the label `pay -> C: ship`, because a label holds arrows and colons as they are. The file passes `check` and draws the wrong picture. Item 131 gives a fix to `A -> B -> C`, and this form is not covered.

### Proposal

When a label holds an arrow, then a name, then a colon, as in `pay -> C: ship`, give a warning with its own code. Carry a suggested fix that writes `A -> B: pay` and `B -> C: ship` on two lines with the same indent. A label with an arrow and no colon after it, as in `calls -> returns`, stays as it is, because it is a common label.

### Acceptance criteria

- [x] `A -> B: pay -> C: ship` gives one warning with its own code, and its suggested fix gives the lines `A -> B: pay` and `B -> C: ship`.
- [x] `A -> B: calls -> returns` gives no diagnostic.
- [x] A label in double quotes that holds `-> C: x` gives no diagnostic.

## 2026-11-26

cargo test -p horsegirl-core --test check label: 2 passed. 'A -> B: pay -> C: ship' gives one W0109 with a suggested fix that gives 'A -> B: pay' and 'B -> C: ship', and the result checks clean. 'A -> B: calls -> returns', 'A -> B: "pay -> C: ship"', 'A -> B: "x" -> C' and 'A -> B: pay: ship' give no W0109. scripts/task check exit 0.
