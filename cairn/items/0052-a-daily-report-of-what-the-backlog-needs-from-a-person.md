---
id: 52
uid: 788e9701-2ed3-46e5-912c-af7eb50f5376
title: A daily report of what the backlog needs from a person
type: feature
status: planned
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
effort: s
area: workflow
track:
- workflow
---

## Problem

A claim that has gone cold, a proposal that waits, and an item whose criteria are all ticked but which is still open are invisible unless somebody looks.

## Proposal

Add a scheduled workflow that runs `harrow --plain --lens needs` and `cairn check --strict`, and opens or updates one issue titled 'Backlog needs a person' with the result. It closes the issue when the list is empty.

## Acceptance criteria

- [ ] A proposal on an item appears in the issue within a day.
- [ ] An empty list closes the issue.
- [ ] The workflow writes to issues only, and has no other permission.
