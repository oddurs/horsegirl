---
id: 22
uid: 97ec1a3a-bb96-4b95-b865-c58a0b3231cd
title: Split the crate into a workspace with a pure core
type: chore
status: planned
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: none
effort: s
area: workflow
track:
- workflow
---

## Purpose

One crate cannot serve a command line, an MCP server and a WebAssembly build without dragging each one's dependencies into the others.

## Approach

Make a Cargo workspace: `crates/horsegirl-core` (no I/O, no clock), `crates/horsegirl` (the binary). Keep `scripts/task` as the only entry point.

## Acceptance criteria

- [ ] `scripts/task check` passes on the workspace.
- [ ] The core crate has no dependency that does I/O.
- [ ] `horsegirl spec` still prints the card.
