---
id: 72
uid: 058f364e-5029-460a-9420-ca3259c6425b
title: A picture budget that folds groups
type: feature
status: backlog
milestone: v0.2
depends_on:
- 36
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: language
effort: m
area: lens
track:
- pictures
---

## Problem

Agents write too much, and a picture with sixty boxes helps nobody.

## Proposal

Give each lens a budget of things. Over the budget, fold groups into single boxes, largest first, and emit note N0300 that lists what was folded and how to draw one group alone. `show flow at depth 1` folds on request.

## Acceptance criteria

- [ ] A 61-thing file with four groups draws under the budget and names the four groups.
- [ ] The budget is a project setting.
