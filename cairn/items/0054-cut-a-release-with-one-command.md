---
id: 54
uid: 64cfa2c0-354e-4ac9-bdd9-0419aa2f6c5c
title: Cut a release with one command
type: chore
status: planned
milestone: v0.1
created: 2026-11-25
updated: 2026-11-27
related:
- 42
priority: p1
risk: medium
surface: none
effort: m
area: workflow
track:
- workflow
- release
---

## Purpose

A release needs a version bump, a changelog section and a tag. Each is a chance to make a mistake.

## Approach

Add `scripts/agent release <version>`: it moves the Unreleased section of the changelog under the version, bumps the crate versions, opens a pull request, and after the merge the tag is pushed by a workflow.

## Acceptance criteria

- [ ] One command and one merged pull request produce a tagged release.
- [ ] A pull request that changes a public contract (`surface` is not `none`) without a changelog entry fails CI.

## 2026-11-27

Owner decision: 0054 no longer depends on 0042. The release workflow of 0042 exists and its dry run builds all four archives. The first real tag, v0.1.0, is cut with the command this item builds, and that tag closes the last criterion of 0042 as well.
