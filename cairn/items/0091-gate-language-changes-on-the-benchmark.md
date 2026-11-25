---
id: 91
uid: 4fd7596f-8c47-4639-8817-e92f6c71c6ac
title: Gate language changes on the benchmark
type: feature
status: backlog
milestone: v0.3
depends_on:
- 41
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: bench
track:
- language
- workflow
---

## Problem

A change to the syntax can make models worse at writing it, and nobody would notice.

## Proposal

Run the benchmark in CI on every pull request whose item has `surface: language`, against recorded model outputs and a nightly live run. A drop in the score fails the check.

## Acceptance criteria

- [ ] A pull request that removes a forgiving-reader rule fails the gate.
- [ ] The gate's threshold and the last three scores are in the report.
