---
id: 41
uid: e4a34f6a-eec8-4ef3-bc40-dacb183cd647
title: 'The first benchmark: can a model write it from the card?'
type: feature
status: planned
milestone: v0.1
depends_on:
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: high
surface: none
effort: l
area: bench
track:
- agent-loop
- language
---

## Problem

The claim is that a model writes this language correctly from one page. Nothing measures it.

## Proposal

Build the first benchmark: 100 tasks, each a description of a system and a checker. A runner gives a model the spec card and the task, then scores validity on the first try and after one `fix`. The runner takes any model behind a small adapter.

## Acceptance criteria

- [ ] The runner scores a recorded transcript with no network access, so CI can run it.
- [ ] A report lists the score for each task and the diagnostics that models hit most often.
- [ ] The tasks and the checkers are in the repository.
