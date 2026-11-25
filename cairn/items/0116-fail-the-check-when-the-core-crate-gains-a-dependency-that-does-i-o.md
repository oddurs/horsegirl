---
id: 116
uid: e3c21ab5-79a6-4854-a8b4-394f052cbb20
title: Fail the check when the core crate gains a dependency that does I/O
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: none
area: workflow
---

## Purpose

The core crate must stay pure so it compiles to WebAssembly. Today it has no dependency, and nothing stops a later change from adding one that reads files, the network or the clock.

## Approach

Add a check to `scripts/task lint` that reads `cargo tree -p horsegirl-core` and fails on a dependency outside an allow list.

## Acceptance criteria

- [x] `scripts/task lint` fails when a dependency outside the allow list is added to `horsegirl-core`.
- [x] `scripts/task lint` passes on the current core crate.

## 2026-11-25

Added a fastrand dependency to horsegirl-core by hand: scripts/check-core-deps exited 1 and named fastrand. Reverted it. On the real crate it exits 0. tests/core-deps.sh passes, and scripts/task check passed in full.
