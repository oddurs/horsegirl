---
id: 98
uid: a61c6c49-77fd-4433-b916-593dd279e48e
title: Performance and size budgets, enforced
type: chore
status: backlog
milestone: v1.0
depends_on:
- 67
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: workflow
track:
- pictures
- release
---

## Purpose

A tool that is slow on a large file or heavy in a browser loses users quietly.

## Approach

Set and enforce budgets: time to draw a 500-thing flow, memory, and the compressed WebAssembly size. Add a benchmark job that fails on a regression over 10 percent.

## Acceptance criteria

- [ ] The budgets are written in the repository and CI enforces each one.
