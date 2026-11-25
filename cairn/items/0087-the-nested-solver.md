---
id: 87
uid: 42f0d563-2d79-4b1f-b7cf-9a77caa7a778
title: The nested solver
type: feature
status: backlog
milestone: v0.3
depends_on:
- 31
- 86
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: medium
surface: none
effort: m
area: layout
track:
- pictures
---

## Problem

A treemap places things inside their group with an area from a number. That is a fourth kind of layout.

## Proposal

Build the nested solver: each group is a rectangle divided among its parts by a numeric fact.

## Acceptance criteria

- [ ] Areas are within 2 percent of the proportions in the data.
