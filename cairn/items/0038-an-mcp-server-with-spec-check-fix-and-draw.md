---
id: 38
uid: 32f2182f-6523-4e8f-983e-17a620495615
title: An MCP server with spec, check, fix and draw
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 21
- 37
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: mcp
effort: m
area: mcp
track:
- agent-loop
---

## Problem

An agent in an MCP client should not shell out and parse terminal text.

## Proposal

Serve `spec`, `check`, `fix` and `draw` as MCP tools over stdio, using the library and shapes from the spike. `horsegirl mcp --config` prints the client configuration.

## Acceptance criteria

- [x] A scripted MCP client calls each tool and receives the documented shape.
- [x] `check` returns every diagnostic with its fix in one call.

## 2026-11-25

SPIKE-21 answered: build on rmcp 3.11.0 (exact pin, default features off, features server, macros, transport-io), stdio only, dual-era. check returns Json<T>: structuredContent plus one text block with the same JSON; isError is false when diagnostics are found. Use #[tool_handler(name = "horsegirl")] so serverInfo is not 'rmcp', and replace schemars' 'format: uint32' with schema_with. Prototype: spikes/mcp/.

## 2026-11-25

Criterion 1: crates/horsegirl/tests/mcp.rs is a scripted client. It starts horsegirl mcp, sends a legacy initialize, then tools/list and tools/call for spec, check, fix and draw, and holds each result to docs/mcp.md and to the command line (check to check --json, fix to fix, draw svg to draw on stdin). cargo test -p horsegirl --test mcp: 10 passed, 30 runs in a row with no failure. The reference client agreed: npx @modelcontextprotocol/inspector@2.16.0 --cli target/debug/horsegirl mcp called each tool with isError false (spec 746 characters, fix applied 2 to 'Web app --> API@v2', draw an svg, check structuredContent with N0101 and its safe fix), and tools/list --strict reported no schema warning once svg became anyOf string or null. A hand-written server/discover with _meta answered supportedVersions up to 2026-07-28, and a modern tools/call check returned the same structuredContent. scripts/task check exited 0.

## 2026-11-25

Criterion 2: test check_returns_every_diagnostic_with_its_fix_in_one_call sends one tools/call check for a file with 4 errors, 2 warnings and 1 note, 4 safe fixes and 3 diagnostics with fix null, and the structuredContent equals every check --json line plus the counts. Unit test mcp::tests::a_diagnostic_is_the_same_bytes_as_its_json_line holds each MCP diagnostic to the bytes of Diagnostic::to_json. Both passed.
