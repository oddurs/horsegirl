---
id: 34
uid: 6f396b4b-c038-4575-a22d-299fd2580048
title: Draw the scene as SVG with CSS variables
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 30
- 31
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: svg
effort: m
area: render
track:
- pictures
---

## Problem

The scene has no visible form yet.

## Proposal

Write the SVG backend. Colours and fonts are CSS variables with fallbacks. Each shape has a class for its role and for its facts, such as `owner-commerce`. The output has a `title` and a `desc`.

## Acceptance criteria

- [x] The reference flow renders to an SVG that a snapshot test holds byte for byte.
- [x] The SVG is valid XML and uses no script and no external resource.
- [x] Setting `--hg-accent` in a host page changes the accent with no rebuild.

## 2026-11-25

SPIKE-19 answer: the SVG font stack is Arial, "Liberation Sans", Helvetica, sans-serif, because the core measures in Arial metrics. 0118 adds textLength to each label so other faces keep the measured width.

## 2026-11-25

0039 added the golden harness in crates/horsegirl-core/tests/golden.rs. Add the SVG as one more entry in OUTPUTS ('svg'), written for each case that draws, so a change to a picture shows as a diff beside its tree and diagnostics.

## 2026-11-25

Criterion 1: scripts/task check passed. tests/svg.rs the_reference_flow_draws_to_its_svg holds tests/checkout.svg byte for byte; it passed natively (nextest 94 passed) and on wasm32-wasip1 (svg 8 passed), so both targets give the same bytes. The placement is the stand-in in tests/support/mod.rs because the layered layout is not in main; 0145 moves the snapshot to the real layout and into the golden harness.

## 2026-11-25

Criterion 2: the_svg_is_well_formed_xml (a balanced-tag and entity check, itself tested to reject 7 malformed inputs) and the_svg_has_no_script_and_loads_nothing (no script, on* attribute, href, @import, image or foreignObject; one http, the namespace; every url() is url(#...)) passed natively and in wasm. In headless Chromium, DOMParser with image/svg+xml on tests/checkout.svg gave 0 parsererror elements and root svg.

## 2026-11-25

Criterion 3: headless Chromium, host page with checkout.svg inline: getComputedStyle(path.head).fill was rgb(212, 36, 111) (the fallback #D4246F); after document.documentElement.style.setProperty('--hg-accent','rgb(0, 128, 0)') it was rgb(0, 128, 0). A screenshot of a host page with :root{--hg-accent:#0a8a3a} shows green arrow heads; a screenshot of the bare SVG shows pink ones. The host page's own .label{color:red} rule did not reach the SVG labels (fill rgb(36, 22, 39)).

## 2026-11-25

Golden harness: no golden case draws until the layered layout (0032) is in the core, so the SVG is not in OUTPUTS yet. Filed 0145 for that.

## 2026-11-25

Review round 1: escape let U+FFFE and U+FFFF through, so criterion 2 was false for a name such as "a￿b". Unticked 2 and reopened. escape now writes U+FFFD for both; the test checker well_formed now rejects any character outside the XML Char production. With the fix removed, a_character_xml_cannot_hold_is_drawn_as_a_replacement FAILED; with the fix, cargo test -p horsegirl-core --test svg printed 9 passed, and scripts/task check exited 0.
