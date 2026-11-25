---
id: 14
uid: 1b515c0c-83d7-404f-909e-032b8b7d1e15
title: The core is Rust, compiled to native and to WebAssembly
type: decision
status: done
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: none
effort: s
area: cli
track:
- release
---

## Context

The tool must run as a command, as an MCP server and in a browser, and give the same bytes in each.

## Options and tradeoffs

Rust: one static binary and a WebAssembly build from the same code; fewer contributors and slower layout experiments. TypeScript: more contributors; needs a runtime for the command line. Go: a large WebAssembly output.

## Decision

The core is Rust. Lenses and themes are written in horsegirl, so most contributions do not need Rust. CI enforces a size budget on the WebAssembly build.

## Revisit when

The WebAssembly build cannot meet its size budget, or layout work stalls on compile times.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
