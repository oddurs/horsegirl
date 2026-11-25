---
id: 62
uid: 91ccbf98-b99a-41d6-8608-38f1a294311f
title: 'add: append a fact without rewriting the file'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 29
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: cli
effort: s
area: cli
track:
- agent-loop
---

## Problem

An agent that rewrites a whole file to add one fact can damage the rest.

## Proposal

`horsegirl add <file> '<fact>'` appends the fact in canonical form, skips a fact that is already there, and reports the lines added.

## Acceptance criteria

- [ ] Adding an existing link changes nothing and says so.
- [ ] `A -> B -> C` is added as two lines.
