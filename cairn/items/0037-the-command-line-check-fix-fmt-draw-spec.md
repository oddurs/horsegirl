---
id: 37
uid: 82d172f1-3ef9-460f-87ba-cff7ef0838ab
title: 'The command line: check, fix, fmt, draw, spec'
type: feature
status: planned
milestone: v0.1
depends_on:
- 27
- 28
- 29
- 34
- 36
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: cli
effort: m
area: cli
track:
- agent-loop
---

## Problem

The pieces exist as a library. An agent needs commands.

## Proposal

Add `check`, `fix`, `fmt`, `draw` and `spec`. Exit 0 for clean, 1 for diagnostics at error level, 2 for a usage error. `--json` prints one object per line. `draw --to svg` writes beside the source or to `-o`.

## Acceptance criteria

- [ ] Each command has an integration test for its exit codes and its output.
- [ ] `fix` applies every safe fix and changes nothing else.
- [ ] `draw` refuses to write a picture when the file has an error, and says which.
- [ ] `--help` for every command fits on one screen.
