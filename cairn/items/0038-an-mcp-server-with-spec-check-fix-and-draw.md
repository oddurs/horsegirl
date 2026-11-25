---
id: 38
uid: 32f2182f-6523-4e8f-983e-17a620495615
title: An MCP server with spec, check, fix and draw
type: feature
status: planned
milestone: v0.1
depends_on:
- 21
- 37
created: 2026-11-25
updated: 2026-11-25
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

- [ ] A scripted MCP client calls each tool and receives the documented shape.
- [ ] `check` returns every diagnostic with its fix in one call.

## 2026-11-25

SPIKE-21 answered: build on rmcp 3.11.0 (exact pin, default features off, features server, macros, transport-io), stdio only, dual-era. check returns Json<T>: structuredContent plus one text block with the same JSON; isError is false when diagnostics are found. Use #[tool_handler(name = "horsegirl")] so serverInfo is not 'rmcp', and replace schemars' 'format: uint32' with schema_with. Prototype: spikes/mcp/.
