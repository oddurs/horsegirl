---
id: 77
uid: d45db747-e735-49be-bff8-2bb424b5dcf4
title: Evidence on facts, and verify to find stale ones
type: feature
status: backlog
milestone: v0.3
depends_on:
- 25
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: check
track:
- agent-loop
- language
---

## Problem

A diagram written from code goes stale in silence.

## Proposal

Parse `@ path:line` at the end of a fact as its evidence. `horsegirl verify` reads each cited file and reports a fact whose line no longer mentions either end of the link, or whose file is gone.

## Acceptance criteria

- [ ] A moved file gives a diagnostic with the old path.
- [ ] `verify` reads only paths inside the repository root.
- [ ] Facts with evidence draw solid, and guesses draw dashed, in every lens.
