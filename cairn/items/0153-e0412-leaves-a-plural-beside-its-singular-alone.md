---
id: 153
uid: 7e3e9205-c151-4537-84df-8cc7026aa106
title: E0412 leaves a plural beside its singular alone
type: chore
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: check
---

## Purpose

Item 120 left plurals alone: `Agents` beside `Agent` still gives E0412 in 3 corpus flowcharts (6 labels). Decide whether a plural is a typo, then change the rule or record why not.

## Approach

Run `python3 corpus/measure.py` before and after. Related: 0120.

## Acceptance criteria

- [ ] `Agent` beside `Agents` gives no E0412, or the item records why it does.
- [ ] A test holds the case.
