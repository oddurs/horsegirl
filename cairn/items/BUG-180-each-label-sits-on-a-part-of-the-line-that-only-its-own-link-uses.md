---
id: 180
uid: 0e73288b-980b-48c4-bb2d-f6e598492eb5
title: Each label sits on a part of the line that only its own link uses
type: bug
status: backlog
milestone: v0.2
created: 2026-11-26
updated: 2026-11-26
priority: p1
risk: low
surface: svg
effort: m
area: layout
related:
- 170
---

## Problem

In `examples/checkout.horse`, `API gateway -> Payments` and `Orders -> Payments` join into one line before Payments. The label `charge` sits on the part that both links use. A reader cannot tell which link `charge` names, and cannot follow either link to its source.

## Proposal

When two links share a line, put each label on a part that only its own link uses, before the two lines join. When no such part is long enough for the label, keep the two lines apart up to the last bend.

## Acceptance criteria

- [ ] The overlap detector reports a label on a part of a line that two or more links use.
- [ ] `examples/checkout.horse` and the eleven layout test files give no such report. A test holds this.
- [ ] The generated graphs of `no_generated_graph_has_a_fault` give no such report, or the test holds the count and names the cause.
