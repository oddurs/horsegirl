---
id: 70
uid: 5b59d2c7-6205-47bb-994d-da14eb952675
title: Benchmark the read-back, across three model families
type: feature
status: backlog
milestone: v0.2
depends_on:
- 41
- 59
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: bench
track:
- agent-loop
---

## Problem

Validity is half the claim. The other half is that the picture says what the agent meant.

## Proposal

Extend the benchmark: each task has a statement of intent, and the score includes whether the read-back matches it. Add seeded wrong diagrams and measure how often a model catches them from the read-back alone. Run three model families.

## Acceptance criteria

- [ ] The report has three scores for each model: valid first try, valid after one fix, read-back matches.
- [ ] Ten seeded wrong diagrams are in the task set.
