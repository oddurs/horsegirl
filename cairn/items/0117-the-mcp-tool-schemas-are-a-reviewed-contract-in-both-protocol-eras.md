---
id: 117
uid: a9ddcef3-2b4a-434d-8279-be65c664be32
title: The MCP tool schemas are a reviewed contract in both protocol eras
type: feature
status: backlog
milestone: v0.1
depends_on:
- 38
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: mcp
area: mcp
effort: s
related:
- 21
---

## Problem

The `inputSchema` and `outputSchema` of each MCP tool are a public contract. rmcp derives them from Rust types, so a renamed field changes the contract with no visible diff. The server must also serve two protocol eras: legacy clients send `initialize` (`2025-11-25`), and modern clients send `server/discover` and `_meta` on each request (`2026-07-28`). rmcp handles both today, but no test holds that.

## Proposal

Add a golden test that starts the server, sends `tools/list`, and compares the result with a committed JSON file. Run the same `tools/call check` once as a legacy client and once as a modern client. SPIKE-21 has the shapes and the wire messages.

## Acceptance criteria

- [ ] A golden test holds the `tools/list` result, and a changed schema shows as a diff in the pull request.
- [ ] A test calls `check` after a legacy `initialize` and gets the documented `structuredContent`.
- [ ] A test calls `check` with modern `_meta` after `server/discover` and gets the same `structuredContent`.
- [ ] No schema in the tool list has a `format` that is not a JSON Schema 2020-12 format.
