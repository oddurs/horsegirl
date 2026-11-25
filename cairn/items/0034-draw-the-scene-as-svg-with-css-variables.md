---
id: 34
uid: 6f396b4b-c038-4575-a22d-299fd2580048
title: Draw the scene as SVG with CSS variables
type: feature
status: planned
milestone: v0.1
depends_on:
- 30
- 31
created: 2026-11-25
updated: 2026-11-25
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

- [ ] The reference flow renders to an SVG that a snapshot test holds byte for byte.
- [ ] The SVG is valid XML and uses no script and no external resource.
- [ ] Setting `--hg-accent` in a host page changes the accent with no rebuild.

## 2026-11-25

SPIKE-19 answer: the SVG font stack is Arial, "Liberation Sans", Helvetica, sans-serif, because the core measures in Arial metrics. 0118 adds textLength to each label so other faces keep the measured width.

## 2026-11-25

0039 added the golden harness in crates/horsegirl-core/tests/golden.rs. Add the SVG as one more entry in OUTPUTS ('svg'), written for each case that draws, so a change to a picture shows as a diff beside its tree and diagnostics.
