---
id: 63
uid: 370ac248-bc2e-4803-8464-dae807e3f57a
title: A text backend for terminals
type: feature
status: backlog
milestone: v0.2
depends_on:
- 31
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: cli
effort: m
area: render
track:
- pictures
- agent-loop
---

## Problem

An agent answers in text, and a commit message or a code comment cannot hold an SVG.

## Proposal

Write a text backend that draws the scene with ASCII characters, with an option for box-drawing characters and a width limit.

## Acceptance criteria

- [ ] The reference flow fits in 100 columns and a snapshot holds it.
- [ ] A label never overflows its box.
