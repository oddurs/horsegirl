---
id: 51
uid: 3b84421c-a112-4fea-b7e6-0605d38f82c7
title: One command takes an item and one command finishes it
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: workflow
track:
- workflow
- agent-loop
---

## Problem

An agent still has to choose a branch name, read the item, and remember the closing steps.

## Proposal

Add `scripts/agent take [<id>]`: with no id it takes the first item in the `agent` view. It derives the branch name from the item's type and title, starts the worktree, claims the item, and prints the item as a prompt. Add `scripts/agent finish`: it runs the checks, refuses if a criterion is unticked, closes the item, commits, and opens the pull request with auto-merge.

## Acceptance criteria

- [x] `scripts/agent take` on a clean checkout ends with a worktree, a claimed item and the prompt on standard output.
- [x] `scripts/agent finish` on an item with an unticked criterion stops and names the criterion.
- [x] The pull request body is built from the item: problem, approach, and a `Refs:` line.
- [x] The test script covers both commands against a scratch repository.
