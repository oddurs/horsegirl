---
id: 117
uid: a9ddcef3-2b4a-434d-8279-be65c664be32
title: The MCP tool schemas are a reviewed contract in both protocol eras
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 38
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] A golden test holds the `tools/list` result, and a changed schema shows as a diff in the pull request.
- [x] A test calls `check` after a legacy `initialize` and gets the documented `structuredContent`.
- [x] A test calls `check` with modern `_meta` after `server/discover` and gets the same `structuredContent`.
- [x] No schema in the tool list has a `format` that is not a JSON Schema 2020-12 format.

## 2026-11-25

Criterion 1: test the_tool_list_is_the_committed_contract_in_both_eras compares tools/list, legacy and modern, with crates/horsegirl/tests/golden/tools.json (585 lines). Before the file existed it failed and printed the list. After HORSEGIRL_BLESS=1 it passed. Changing the doc comment 'How many safe fixes were made.' in src/mcp.rs made it fail with 'the tool schemas moved; read the diff', so a schema change must show as a diff of tools.json; the edit was reverted.

## 2026-11-25

Criteria 2 and 3: check_after_a_legacy_initialize_returns_the_documented_result (initialize 2025-11-25, no resultType) and check_with_modern_meta_after_discover_returns_the_same_result (server/discover, then tools/call with io.modelcontextprotocol/protocolVersion 2026-07-28 and clientCapabilities in _meta, resultType complete) both equal the same literal structuredContent for 'Café --> Payments': one N0101 note with a safe fix, columns 6-9. a_modern_request_without_its_meta_is_refused gets -32602, so the modern path is really taken. Criterion 4: no_schema_has_a_format_that_json_schema_does_not_define walks every format keyword and allows only the 19 JSON Schema 2020-12 formats; the_format_walk_finds_a_format_that_json_schema_does_not_define shows it catches uint32. cargo test -p horsegirl --test mcp: 16 passed. scripts/task check exited 0.

## Result

tools/list is held to crates/horsegirl/tests/golden/tools.json in both eras; check returns the documented structuredContent after initialize and after server/discover with _meta; only JSON Schema 2020-12 formats are allowed.
