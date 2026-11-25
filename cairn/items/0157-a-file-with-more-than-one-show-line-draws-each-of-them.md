---
id: 157
uid: 6affc4b2-1721-479c-bb64-cd63aa333456
title: A file with more than one show line draws each of them
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: cli
area: cli
effort: s
related:
- 37
---

## Problem

`horsegirl draw` draws the lens of the first `show` line and ignores the others. A file that states `show flow` and `show lanes of S` gets one picture, and nothing says that the second line was left out.

## Proposal

Draw one picture for each `show` line, named after the file and the lens, or give a warning that names each `show` line that was not drawn. Decide which when a second lens draws.

## Acceptance criteria

- [ ] A file with two `show` lines of lenses that this version draws gives two pictures, or a warning that names the second line.
- [ ] An integration test holds the file names or the warning.
