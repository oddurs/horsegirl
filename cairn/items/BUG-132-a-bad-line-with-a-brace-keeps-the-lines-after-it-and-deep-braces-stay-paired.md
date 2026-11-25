---
id: 132
uid: 4184876c-d493-409d-8fdd-addaab8ce969
title: A bad line with a brace keeps the lines after it, and deep braces stay paired
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: m
related:
- 23
---

## Problem

The reviews of item 23 found two faults in how the parser recovers at a brace.

1. `A -> B: uses {braces\nC -> D\nshow flow\n` gives one `Error` node that holds a `Block` with `C -> D` and `show flow` inside. The recovery in `statement` opens a block for every `{`, also for one that does not close on its line. The model skips an `Error` node, so the two good lines are lost.
2. 70 nested groups, correctly closed, give six E0007 and six E0003. Past 64 levels the `}` of a group that was too deep closes an outer group, so the parts that follow sit in the wrong group.

## Proposal

In the recovery of `statement`, open a block only when its `}` is on the same line or the `{` ends its line. The parser already knows the `}` of each `{` on a line (`closes` in `parser.rs`). At the depth limit, skip the too-deep block as a balanced unit.

## Acceptance criteria

- [ ] `A -> B: uses {braces\nC -> D\nshow flow\n` gives a tree in which `C -> D` is a `Link` and `show flow` is a `Show` at the top level.
- [ ] 70 nested groups that close correctly give E0007 and no E0003, and a thing after the last `}` is at the top level.
- [ ] The fuzz test passes with both inputs in its seed file.

## 2026-11-25

From the review of item 23: `story s {} x` gives E0001 for `x`, and the line is an Error node. The parser still gives W0010 for the story on that line. The model skips an Error node, so the warning names a story that does not exist. Give W0010 only when the story line parses, and add this input to the tests.
