---
id: 86
uid: 61999f44-7439-422b-95b9-7e4433f9336e
title: Load data files and bind thickness to a number
type: feature
status: backlog
milestone: v0.3
depends_on:
- 36
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: language
effort: m
area: interop
track:
- language
- pictures
---

## Problem

Thickness and area need numbers, and the numbers live in other files.

## Proposal

Add `load <file.csv>`, which joins rows to things or links by name and adds the columns as facts. Let a `show` line bind `thickness` to a numeric fact.

## Acceptance criteria

- [ ] `show flow { thickness: calls }` draws the reference file with link widths in proportion to the data.
- [ ] A row that names no thing is a warning with the name.
- [ ] `load` reads only paths inside the repository root.
