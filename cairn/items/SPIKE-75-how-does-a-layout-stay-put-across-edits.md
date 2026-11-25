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
- [ ] The answer says where positions live: derived from git history, or a committed sidecar.
