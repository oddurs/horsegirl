---
id: 69
uid: 758a1fbd-b61a-4a2c-afa6-5f962bcde4e5
title: MCP tools for describe, suggest, query and add
type: feature
status: backlog
milestone: v0.2
depends_on:
- 38
- 59
- 61
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: mcp
effort: s
area: mcp
track:
- agent-loop
---

## Problem

The MCP server has four tools. The agent loop needs the read-back and the append.

## Proposal

Add `describe`, `suggest`, `query` and `add` as MCP tools.

## Acceptance criteria

- [ ] A scripted client completes write, check, fix, describe and draw with no shell.
