---
id: 97
uid: d4a0de01-f903-4544-a00b-771a613f6fa3
title: Fuzz the parser and the solvers
type: chore
status: backlog
milestone: v1.0
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: check
track:
- language
- pictures
---

## Purpose

horsegirl reads files that other people wrote. A panic or a hang is a vulnerability.

## Approach

Fuzz the parser, the model and each solver in CI, with a time limit on layout. Fix what the fuzzers find.

## Acceptance criteria

- [ ] A nightly fuzz run of one hour for each target finds no panic and no hang for seven nights.
- [ ] Layout of any input ends or fails with a diagnostic within the time limit.
