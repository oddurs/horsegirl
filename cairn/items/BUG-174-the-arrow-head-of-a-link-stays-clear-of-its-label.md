---
id: 174
uid: 26472292-ae7b-4a3b-abfa-7c1cf22dbb7a
title: The arrow head of a link stays clear of its label
type: bug
status: backlog
milestone: v0.1
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: layout
effort: s
related:
- 152
- 124
---

## Problem

The layout leaves `LABEL_PAD` (6 px, `crates/horsegirl-core/src/layout/route.rs`) between a link label and each end of its step. The SVG arrow head is 8 px long and ends at the box. So where the label fills its step, the arrow head covers the last 2 px of the text, or more when the label is off the centre of the step. `crates/horsegirl-core/tests/golden/labels-and-names.svg` shows it: the head of `Web app -> API gateway` covers the `k` of `POST /checkout, then 200 -> ok`. Item 0152 makes the line stop 3 px before the label, so the head is now the only mark on the text.

## Proposal

Make the room between a label and the end of a link that holds the arrow head at least the head length plus the 3 px cut of the SVG backend. Keep the head length in one constant that the layout and the backend share.

## Acceptance criteria

- [ ] On every golden picture and on `examples/checkout.horse`, no arrow head overlaps the box of a link label.
- [ ] A test in `crates/horsegirl-core/tests/layout.rs` or `tests/svg.rs` holds the case.
- [ ] A link with no label keeps the gap it has today.
