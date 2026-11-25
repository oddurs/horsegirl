---
id: 40
uid: c1dddc48-cda1-443a-abb3-953b10aba464
title: Draw the corpus in CI and count overlaps
type: chore
status: planned
milestone: v0.1
depends_on:
- 17
- 33
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: bench
track:
- pictures
---

## Purpose

The release gate is 100 real flowcharts with no overlap. That needs the corpus converted and a test that counts.

## Approach

Convert the corpus from the grammar spike to horsegirl by script, draw each file, and run the overlap detector. Report the count in CI.

## Acceptance criteria

- [ ] CI prints how many of the corpus files draw with zero overlaps.
- [ ] The job fails when the count drops below the committed baseline.
