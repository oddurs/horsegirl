---
id: 79
uid: 3ed8c4dd-c7fe-4840-86d9-c5f8ebc1c5e6
title: Comment the picture diff on a pull request
type: feature
status: backlog
milestone: v0.3
depends_on:
- 71
- 78
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: s
area: workflow
track:
- workflow
---

## Problem

The picture diff exists as a command. It must appear where review happens.

## Proposal

The GitHub Action comments on a pull request with the diff picture for each changed file, and updates the same comment on a new push.

## Acceptance criteria

- [ ] A pull request that adds one link gets one comment with one picture.
