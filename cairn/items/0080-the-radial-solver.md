---
id: 80
uid: 32804470-ba0d-4e03-91ef-27bbd87a0ca3
title: The radial solver
type: feature
status: backlog
milestone: v0.3
depends_on:
- 31
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: layout
track:
- pictures
---

## Problem

Rings and seams place things around a centre. Neither existing solver does that.

## Proposal

Build the radial solver: a field gives the ring, and angle is chosen to keep linked things close.

## Acceptance criteria

- [ ] The reference file draws around Payments with no overlapping labels.
