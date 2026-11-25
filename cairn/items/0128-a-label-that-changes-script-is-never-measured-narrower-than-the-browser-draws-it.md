---
id: 128
uid: dc2807b1-e309-4eb7-90f6-60578b9a50c3
title: A label that changes script is never measured narrower than the browser draws it
type: feature
status: backlog
milestone: later
depends_on:
- 30
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: svg
area: render
effort: s
---

## Problem

The browser does not apply a kerning pair across a change of script, and the metrics table of the core does. `Тест → ΔΓ` measures 1.149 percent narrower than Chromium draws it in Liberation Sans, because the table kerns the space before `Δ` by -113 units (SPIKE-19). A label that is measured narrow overflows its box. A label that is measured wide only leaves space.

## Proposal

In `horsegirl_core::text::measure`, or in the generator in `tools/text-metrics`, drop the pair adjustment where the two characters resolve to different scripts. A character of the common script, such as a space or a digit, takes the script of the run it is in, as the browser does.

## Acceptance criteria

- [ ] `Тест → ΔΓ` is no narrower than the `arial-metrics` width in `spikes/0019-text-metrics/browser.tsv`.
- [ ] The first 100 labels of that file stay within 0.1 percent of the browser.
- [ ] A test holds both.
