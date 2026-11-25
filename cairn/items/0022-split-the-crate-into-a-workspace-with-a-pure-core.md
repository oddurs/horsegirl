---
id: 22
uid: 97ec1a3a-bb96-4b95-b865-c58a0b3231cd
title: Split the crate into a workspace with a pure core
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] `scripts/task check` passes on the workspace.
- [x] The core crate has no dependency that does I/O.
- [x] `horsegirl spec` still prints the card.

## 2026-11-25

Ran scripts/task check on the workspace: fmt:check, clippy -D warnings, cairn check (ok: 114 items), 4 nextest tests, tests/agent.sh ok, build. Exit code 0.

## 2026-11-25

Ran cargo tree -p horsegirl-core: it prints only horsegirl-core v0.0.0. The core has no dependency at all.

## 2026-11-25

Ran cargo run -p horsegirl -- spec and diffed with crates/horsegirl-core/src/spec.txt: identical. Test spec_prints_the_card passes.

## 2026-11-25

Fix round 1: README install line now 'cargo install --path crates/horsegirl --locked'; installed binary printed 'horsegirl 0.0.0'.
