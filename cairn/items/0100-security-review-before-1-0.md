---
id: 100
uid: 5423988b-e9fb-4cb8-8729-3d4d8d5a0402
title: Security review before 1.0
type: chore
status: backlog
milestone: v1.0
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: release
track:
- release
---

## Purpose

Importers and `load` read paths that a file names. A release needs a review of what a hostile file can do.

## Approach

Review file access, resource limits and the supply chain: path handling, dependency audit, release attestation, and the permissions of every workflow.

## Acceptance criteria

- [ ] A hostile-file test set is in the repository and passes.
- [ ] `SECURITY.md` states the threat model.
