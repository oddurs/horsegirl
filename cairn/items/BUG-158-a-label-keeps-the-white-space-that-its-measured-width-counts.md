---
id: 158
uid: aece6a49-edf2-45e2-be01-d2d400f6f4e9
title: A label keeps the white space that its measured width counts
type: bug
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: svg
area: render
---

## Problem

`text::measure` counts every space in a label. SVG collapses leading,
trailing and repeated white space in a `text` element. The label box and
`textLength` (item 0118) hold the counted width, so the drawn text is
stretched to a width that it does not have.

Reproduction: draw `"  Wide name  " -> E`. The label writes
`textLength="74.03"`. The natural width of the collapsed text `Wide name` is
about 61 px, so the renderer stretches the label by 21 percent. Firefox draws
it 4 px over its box.

A possible fix is `white-space: pre` on `.horsegirl .label`, so the renderer
draws the spaces that the measure counts. The other direction is to collapse
the white space before the measure.

## Acceptance criteria

- [ ] The drawn width of `"  Wide name  "` and of `"Wide  name"` equals the measured width of the same text within 1 px in Chromium, Firefox and WebKit, with `textLength` removed for the measurement.
- [ ] A test in `crates/horsegirl-core/tests/svg.rs` holds the rule for a label with leading, trailing and repeated spaces.
- [ ] `docs/theme.md` states how a label treats white space.
- [ ] `CHANGELOG.md` has an entry, because the SVG output changes.
