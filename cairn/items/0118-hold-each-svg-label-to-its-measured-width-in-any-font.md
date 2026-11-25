---
id: 118
uid: 7ed8e545-58a3-49b6-8767-902975f17b48
title: Hold each SVG label to its measured width in any font
type: feature
status: backlog
milestone: v0.1
depends_on:
- 34
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: svg
area: render
effort: s
track:
- pictures
---

## Problem

The core measures labels in Arial metrics, from a table made from Liberation Sans 2.1.5 (SPIKE-19). The SVG font stack is `Arial, "Liberation Sans", Helvetica, sans-serif` (0034). A viewer without any face in that stack draws the label in a fallback face. In Chromium, on the 100 Latin reference labels, DejaVu Sans is up to 28.6 percent wider than Liberation Sans, and every label is wider. Instrument Sans, the face of the concept page, is up to 14.3 percent away. A wider face overflows its box. A theme that changes the font (0066) must not change a position, so the measured width must hold in every face.

## Proposal

Put `textLength` on each SVG `text` element, set to the measured width, with `lengthAdjust="spacingAndGlyphs"`. The font stack names Arial first. With Liberation Sans, the measured width is within 1.2 percent of the browser on the 108 reference labels, so the adjustment is not visible. Arial itself was not measured. With a fallback face, the label is squeezed or spread to fit its box. SPIKE-19 checked this in Chromium only.

## Acceptance criteria

- [ ] Every SVG `text` element that draws a label carries `textLength` equal to its measured width and `lengthAdjust="spacingAndGlyphs"`, held by the SVG snapshot.
- [ ] In headless Chromium, Firefox and WebKit, the reference flow drawn in DejaVu Sans has each label's bounding box within 1 pixel of its measured width.
- [ ] A screenshot of the reference flow in DejaVu Sans shows no label outside its box.

## Proposed priority: p2 -> p0 (Oddur Sigurdsson, 2026-11-25)

Most SVG viewers lack the measured face unless it is Arial-compatible; without textLength a wider fallback such as DejaVu Sans overflows every box by up to 29 percent (SPIKE-19).
