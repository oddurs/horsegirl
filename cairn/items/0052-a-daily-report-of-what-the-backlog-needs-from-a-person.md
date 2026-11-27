---
id: 52
uid: 788e9701-2ed3-46e5-912c-af7eb50f5376
title: A daily report of what the backlog needs from a person
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-27
created: 2026-11-25
updated: 2026-11-27
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
- [x] The workflow writes to issues only, and has no other permission.

## 2026-11-25

Local run of the collect step: empty backlog gave an empty body.md; after 'cairn propose 52 priority=p1' it gave a body listing '0052 proposal priority=p1' (probe reverted). The issue step (create, edit, close) and the schedule have not run: a schedule runs only from main.

## 2026-11-25

Criterion 3: needs.yml declares 'permissions: issues: write' only, so every other scope is none. Checked by reading the file, not by a run.

## 2026-11-25

Criteria 1 and 2 stay unticked: they need a real scheduled or dispatched run on main.

## 2026-11-25

A follow-up check is needed after merge: run the workflow with workflow_dispatch, then tick 1 and 2.
