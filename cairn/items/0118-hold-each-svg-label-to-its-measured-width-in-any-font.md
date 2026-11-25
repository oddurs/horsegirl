---
id: 118
uid: 7ed8e545-58a3-49b6-8767-902975f17b48
title: Hold each SVG label to its measured width in any font
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 34
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
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

- [x] Every SVG `text` element that draws a label carries `textLength` equal to its measured width and `lengthAdjust="spacingAndGlyphs"`, held by the SVG snapshot.
- [x] In headless Chromium, Firefox and WebKit, the reference flow drawn in DejaVu Sans has each label's bounding box within 1 pixel of its measured width.
- [x] A screenshot of the reference flow in DejaVu Sans shows no label outside its box.

## Proposed priority: p2 -> p0 (Oddur Sigurdsson, 2026-11-25)

Most SVG viewers lack the measured face unless it is Arial-compatible; without textLength a wider fallback such as DejaVu Sans overflows every box by up to 29 percent (SPIKE-19).

## Accepted priority: p0 (Oddur Sigurdsson, 2026-11-25)

Proposed by Oddur Sigurdsson on 2026-11-25.

## 2026-11-25

Criterion 1: svg::draw writes textLength=<label rect width> lengthAdjust="spacingAndGlyphs" on each label text; the label rect width is text::measure of its text. tests/svg.rs each_label_is_held_to_its_measured_width reads each <text> line of the checkout SVG and compares textLength to measure(text) within 0.005 (two decimals); before the change it FAILED, after it cargo test -p horsegirl-core --test svg printed 12 passed. tests/checkout.svg was blessed and holds the 17 labels byte for byte. scripts/task check exited 0.

## 2026-11-25

Criterion 2: playwright-core 1.65.1 (Chromium 1255, Firefox 1572, WebKit 2388 WPE) drew tests/checkout.svg inline in a host page with :root{--hg-font:"DejaVu Sans"}; computed font-family was DejaVu Sans in each. For all 17 labels, getBBox().width minus textLength was at most 0.27 px in Chromium (POST /checkout 87.42 vs 87.15), 0.00 px in Firefox and 0.03 px in WebKit. With textLength removed, the same labels were 7.7 to 15.7 percent wider than measured, so the face was in use. WebKit ran with its missing system libraries extracted from Debian packages into a private directory; nothing was installed on the host.

## 2026-11-25

Criterion 3: screenshots of the reference flow in DejaVu Sans in Chromium, Firefox and WebKit show every thing and group label inside its box; the bounding-box check agreed (each label bbox inside the rect that holds its anchor). Link labels have no box of their own. In the stand-in placement, POST /checkout (87 px) sits on an 80 px gap and touches the edges of Web app and API gateway in every face, as it does in Liberation Sans; that is the stand-in layout, which 0145 replaces.
