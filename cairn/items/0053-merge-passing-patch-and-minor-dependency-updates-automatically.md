---
id: 53
uid: 4126ecaa-194b-42d4-9939-69d3aba7bf92
title: Merge passing patch and minor dependency updates automatically
type: chore
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
effort: s
area: workflow
track:
- workflow
---

## Purpose

Dependency pull requests wait for a click that adds nothing when the checks pass.

## Approach

Set Dependabot pull requests for patch and minor updates to merge themselves when the `required` check passes. Major updates still wait for a person.

## Acceptance criteria

- [ ] A patch update merges with no human action.
- [ ] A major update does not.

## 2026-11-25

Added .github/workflows/dependabot-automerge.yml and scripts/dependabot-update-type. scripts/task check passes, including tests/dependabot.sh (8 cases: patch, minor, group, major, mixed group, unknown kind, no metadata, empty). I watched no real Dependabot pull request, so criteria 1 and 2 stay unticked.

## 2026-11-25

Round 1: added scripts/dependabot-automerge (disables auto-merge on manual) and tests/automerge.sh; scripts/task check passed.
