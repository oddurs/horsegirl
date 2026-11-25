---
id: 60
uid: bb1c812b-3795-4fe5-92d1-09e16024bd03
title: 'suggest: which lenses these facts support'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 59
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: cli
effort: s
area: describe
track:
- agent-loop
---

## Problem

An agent does not know which pictures its facts can support.

## Proposal

`horsegirl suggest` lists each lens the facts support, the question it answers, and for the others the facts that are missing.

## Acceptance criteria

- [ ] A file with no story lists sequence and lanes as unavailable and says to add a story.
