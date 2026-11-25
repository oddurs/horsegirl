---
id: 102
uid: 656fb004-6d13-41ee-ba22-6090f4b77d9c
title: Lenses written by users
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: language
effort: l
area: lens
track:
- language
- pictures
---

## Problem

Only maintainers can add a kind of picture.

## Proposal

Let a file declare `lens <name> { x: ..., y: ... }` over the four solvers, and let a project share lenses as files.

## Acceptance criteria

- [ ] A lens written in a file draws with no change to the core.
