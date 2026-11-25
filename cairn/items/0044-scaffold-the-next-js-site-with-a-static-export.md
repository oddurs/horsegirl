---
id: 44
uid: 2a374060-be4a-4d0f-aaa5-c091e9690e67
title: Scaffold the Next.js site with a static export
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: site
track:
- site
---

## Problem

The project has no home. The concept page is a single private file.

## Proposal

Scaffold a Next.js site in `site/`: App Router, TypeScript in strict mode, static export, a base path of `/horsegirl`, a pinned Node version, and a dev server on a fixed port. Add `site` verbs to `scripts/task`: format, lint, type check, build.

## Acceptance criteria

- [ ] `scripts/task check` builds the static site into `site/out`.
- [ ] The exported site works when served from a sub-path.
- [ ] No runtime dependency is added without a line in the pull request that says what it replaces.
