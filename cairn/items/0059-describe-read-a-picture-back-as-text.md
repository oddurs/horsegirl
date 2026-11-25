---
id: 59
uid: 07887224-1539-4558-80e7-1610c6be58aa
title: 'describe: read a picture back as text'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 31
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: cli
effort: m
area: describe
track:
- agent-loop
---

## Problem

An agent cannot see the picture it drew. It needs the picture as text, and a report on whether the picture is good.

## Proposal

`horsegirl describe` prints what the chosen lens shows: counts, the deepest path, the busiest thing, handoffs, then the drawn size and the number of overlaps and crossings. `--json` gives the same as data. Every SVG takes its `title` and `desc` from this text.

## Acceptance criteria

- [ ] The read-back of the reference file matches a snapshot for each lens.
- [ ] The overlap and crossing counts are measured from the scene, not estimated.
- [ ] A picture with a seeded overlap reports it.
