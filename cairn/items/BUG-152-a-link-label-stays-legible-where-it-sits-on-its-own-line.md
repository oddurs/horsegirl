---
id: 152
uid: 4625474d-379e-4a64-a585-c267c37d3352
title: A link label stays legible where it sits on its own line
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: svg
area: render
effort: s
related:
- 124
---

## Problem

The layout puts the centre of a link label on the path of its link (item 0033). The SVG backend draws the label text with no background. So on a level step the line runs through the middle of the word, and the word reads as struck through. `examples/checkout.horse` drawn by the flow lens shows it on `create`, `charge`, `paid`, `publish` and `send`. The overlap detector does not count a label on its own link, so no test sees it.

## Proposal

Draw a link label on a background of the page colour, with a small pad, so its own line stops at the label. Or move the label off a level line by half its height plus a gap. Choose the one that keeps the label on its path for `describe`.

## Acceptance criteria

- [x] On `examples/checkout.horse`, no link line is drawn across the text of its own label in the SVG.
- [x] The label keeps its colour token in light and dark mode.
- [x] A test holds the case.

## 2026-11-26

Chose to cut the line, not to draw a background. A background must match the page or the group under the label (charge and paid sit on the group fill), and a theme can restyle both. Moving the label off the line would take it off its path, which describe and the overlap detector rely on. The SVG backend now draws a link as a <path> and leaves out each part of the line within 3 px of a label of the same two things. The scene and the layout do not change.

## 2026-11-26

Criterion 1: webshot of crates/horsegirl-core/tests/checkout.svg before and after (Chromium). Before: the line runs through create, charge, paid, publish, send, save and POST /checkout. After: each of the 7 link lines stops at its own label. Not fixed here: create is still crossed by the line of API gateway -> Payments, and charge by the line of Orders -> Postgres, which share a level line and turn up or down inside the label. Those are lines of another link: BUG-170. Filed BUG-174: on labels-and-names.svg the 8 px arrow head covers the end of a label that fills its step, because the layout leaves only 6 px.

## 2026-11-26

Criterion 2: the .label rule is unchanged (fill var(--hg-ink, light-dark(#1F2328,#E6E8EB))), and the test every_colour_and_the_font_is_a_css_variable_with_a_fallback passed. webshot of the SVG inline with --hg-scheme:dark and with --hg-scheme:light: every link label draws in the ink colour of its mode, on the page and on the group.

## 2026-11-26

Criterion 3: no_link_line_is_drawn_across_the_text_of_its_own_label in crates/horsegirl-core/tests/svg.rs reads the drawn paths of examples/checkout.horse and checks no segment enters the box of any of the 7 link labels, and that each path keeps its first and last point. With the cut turned off for one run it failed with 'link ["Web app", "API gateway"] crosses its label "POST /checkout"'. scripts/task check at commit ab649e3: exit 0, 342 tests passed natively, and the test passed again on wasm32-wasip1.
