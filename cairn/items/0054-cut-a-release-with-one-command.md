---
id: 54
uid: 64cfa2c0-354e-4ac9-bdd9-0419aa2f6c5c
title: Cut a release with one command
type: chore
status: planned
milestone: v0.1
depends_on:
- 42
created: 2026-11-25
updated: 2026-11-25
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
