---
id: 75
uid: d92f1b3c-2eb3-480a-a94c-28c957158808
title: How does a layout stay put across edits?
type: spike
status: backlog
milestone: v0.3
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: high
surface: none
effort: l
area: layout
track:
- pictures
---

## Question

How does the layered solver use earlier positions so that a small edit moves few nodes, and where do the earlier positions come from?

## Why it matters

Stability is the property no open tool has. The design decides whether a sidecar file exists, which is a contract.

## Time box

Five days.

## Answer

Not answered yet.

## Acceptance criteria

- [ ] The Answer section states a conclusion and the evidence for it.
- [ ] Follow-up items exist for the work the answer implies.
- [ ] The answer compares at least two cost functions on 20 recorded edits, by nodes moved and by crossings added.
- [ ] A layout seeded with its own picture, with no edit, moves no thing on every corpus diagram, and the answer names what the seed carries to make that true.
- [ ] The answer says where positions live: derived from git history, or a committed sidecar.

## 2026-11-25

From the review of SPIKE-18: the prototype seeded with its own picture and no edit moves 33 of 189 things (04-data-pipeline 6, 05-sign-in 2, 07-compiler 6, 08-module-deps 4, 10-platform 15) and crossings go from 80 to 99. The seed held centres of things only, and the bends had none. Measure the no-edit run first: 'cargo run --release -- seed' in spikes/layered-layout prints it. A cost function is not comparable until this run moves nothing.
