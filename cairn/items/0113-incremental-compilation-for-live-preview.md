---
id: 113
uid: fd36256c-fe53-42cb-8afd-947229e7f818
title: Incremental compilation for live preview
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p3
risk: medium
surface: none
effort: m
area: layout
track:
- pictures
---

## Problem

A large file redraws from nothing on every keystroke.

## Proposal

Make parsing, checking and layout incremental, so the time to redraw follows the size of the edit.

## Acceptance criteria

- [ ] A one-line edit in a 500-thing file redraws in under 20 milliseconds.
