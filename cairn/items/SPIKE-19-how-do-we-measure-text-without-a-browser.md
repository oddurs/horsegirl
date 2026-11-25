---
id: 19
uid: 81738782-b85a-4ca7-9d07-69dcd747f0dc
title: How do we measure text without a browser?
type: spike
status: planned
milestone: v0.1
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

## Question

How does the core measure text with no browser, so that the same input gives the same SVG on every machine and in WebAssembly?

## Why it matters

Node sizes come from label widths. A wrong width makes every label overflow or float.

## Time box

Two days.

## Answer

Not answered yet.

## Acceptance criteria

- [ ] The Answer section states a conclusion and the evidence for it.
- [ ] Follow-up items exist for the work the answer implies.
- [ ] The answer compares bundled metrics tables with a shaping crate, by binary size and by error against a browser's measurement on 100 labels.
