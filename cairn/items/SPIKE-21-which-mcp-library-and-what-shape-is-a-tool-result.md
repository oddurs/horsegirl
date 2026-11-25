---
id: 21
uid: 82c3e088-3d2e-4c5c-a0e1-9498797c3b52
title: Which MCP library, and what shape is a tool result?
type: spike
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: medium
surface: mcp
effort: s
area: mcp
track:
- agent-loop
---

## Question

Which Rust library and transport does the MCP server use, and what does a tool result look like for a diagnostic with a fix?

## Why it matters

The MCP server is the first interface an agent meets. Its tool shapes become a contract.

## Time box

Two days.

## Answer

### Conclusion

- **Library:** `rmcp` 3.11.0, the official Rust SDK
  (`modelcontextprotocol/rust-sdk`, Apache-2.0, MSRV 1.88). Pin the exact
  version. Turn off the default features and use `server`, `macros` and
  `transport-io` only.
- **Transport:** stdio only. The client starts `horsegirl mcp` as a child
  process. horsegirl does not listen on a port, so there is no HTTP transport.
- **Protocol eras:** the server serves both. A legacy client sends
  `initialize` (`2025-11-25` and earlier). A modern client sends
  `server/discover` and puts `_meta` on each request (`2026-07-28`). rmcp does
  both with no extra code.
- **Tool input:** `check` takes `{ "source": "<the whole file>" }`. It takes
  text, not a path, so a call has no file system state.
- **Tool result:** `check` returns its result as `structuredContent`, and the
  tool declares the matching `outputSchema`. The result also has one text block
  that holds the same JSON, as the specification recommends. rmcp's `Json<T>`
  return type adds both.
- **`isError`:** `false` when the check ran, also when it found errors. A
  diagnostic is the result, not a failure. `isError` is `true` only when the
  tool could not run, for example when `source` is missing. An unknown tool
  name is a JSON-RPC error `-32602`. rmcp gives all three.

### The shape of a `check` result

Keys are sorted by name on the wire: rmcp converts the result to a
`serde_json::Value`, and its map sorts keys. The CLI `--json` line (0024)
must use the same types and the same conversion, so that both surfaces give
the same bytes.

```json
{
  "diagnostics": [
    {
      "code": "E0001",
      "fix": {
        "confidence": "safe",
        "description": "Replace `-->` with `->`.",
        "edits": [
          {
            "span": { "end": { "column": 9, "line": 1 }, "start": { "column": 6, "line": 1 } },
            "text": "->"
          }
        ]
      },
      "message": "`-->` is not an arrow. Write `->` for a link.",
      "severity": "error",
      "span": { "end": { "column": 9, "line": 1 }, "start": { "column": 6, "line": 1 } }
    }
  ],
  "errors": 1,
  "notes": 0,
  "warnings": 0
}
```

This is the result for the source `Café --> Payments`. The code `E0001` is a
stand-in; 0024 allocates the real codes.

A **diagnostic** has five keys. Each key is always present.

| Key | Type | Meaning |
| --- | --- | --- |
| `code` | string | `E`, `W` or `N` and four digits. Stable across releases. |
| `severity` | `"error"`, `"warning"` or `"note"` | Agrees with the first letter of `code`. |
| `message` | string | Names the cause and says what to do next. |
| `span` | span | Where the cause is. |
| `fix` | fix or `null` | The replacement. `null` when there is none. |

A **span** is `{ "start": position, "end": position }`. `end` is exclusive.
A **position** is `{ "line": n, "column": n }`. Both start at 1. A column
counts Unicode scalar values, not bytes and not UTF-16 units: an agent counts
characters, and a name can hold any letter. A language server (0103) converts
to UTF-16 at its edge.

A **fix** has three keys.

| Key | Type | Meaning |
| --- | --- | --- |
| `confidence` | `"safe"` or `"suggested"` | `safe`: `fix` applies it with no question. `suggested`: an agent decides. |
| `description` | string | What the fix does, in one sentence. |
| `edits` | array of edit | One or more edits. |

An **edit** is `{ "span": span, "text": string }`. The text replaces the
span, and an empty text deletes it. Every span refers to the original source.
The edits of one fix do not overlap and are sorted by start position.

The result object has `diagnostics`, sorted by start position and then by
code, and the counts `errors`, `warnings` and `notes`, so an agent can stop
without reading the array.

### Evidence

The prototype is in `spikes/mcp/`. It is a separate Cargo project with an
empty `[workspace]`, so it is not part of the crate or of `scripts/task`. Its
`check` is a stand-in: it reports each `-->` and offers `->` as a safe fix,
because the parser does not exist yet.

1. A real client answered. `npx @modelcontextprotocol/inspector@2.16.0 --cli
   ./target/debug/horsegirl-mcp-spike --method tools/call --tool-name check
   --tool-arg 'source=Café --> Payments'` exited 0 and printed the result
   above as `structuredContent`, one text block with the same JSON, and
   `"isError": false`. `spikes/mcp/check.golden.json` is that
   `structuredContent`, and `cargo test` in `spikes/mcp` compares the
   prototype with it.
2. The wire, captured with `tee`, shows the reference client speaks the
   legacy era: `initialize` with `"protocolVersion":"2025-11-25"`, then
   `notifications/initialized`, `tools/list` and `tools/call`.
3. The modern era works too. A hand-written `server/discover` with `_meta`
   returned `"supportedVersions":["2024-11-05","2025-03-26","2025-06-18","2025-11-25","2026-07-28"]`,
   and a modern `tools/call` returned the same `structuredContent` with
   `"resultType":"complete"`.
4. Errors: `"arguments":{}` gave `"isError":true` with the text
   ``failed to deserialize parameters: missing field `source` ``. The tool
   name `draw` gave `{"code":-32602,"message":"tool not found"}`.
5. Cost: 71 crates in the normal dependency tree, a 4.5 MB release binary,
   and a 26 s release build with no earlier release artefacts on this
   machine.

### What the prototype found that 0038 must do

- schemars writes `"format": "uint32"` for a `u32`. That is not a JSON Schema
  format, and the client's validator printed 14 warnings. A `schema_with`
  function that states `"minimum": 1` (positions) or `"minimum": 0` (counts)
  removed every warning.
- `#[tool_router(server_handler)]` names the server `rmcp`. Use
  `#[tool_handler(name = "horsegirl", version = ...)]` so that `serverInfo`
  names horsegirl.
- After the workspace split (0022), the diagnostic types live in the core
  crate with `Serialize`. The `JsonSchema` derive sits behind a core feature
  that the binary turns on; schemars does no I/O. rmcp and tokio go in the
  binary crate only.

### Alternatives rejected

- **`rust-mcp-sdk`:** version 2.x serves only `2026-07-28`, and 1.x serves
  only `2025-11-25`. The reference client speaks `2025-11-25` today, so 2.x
  does not talk to it, and 1.x stops working when clients move on.
- **A hand-written JSON-RPC loop on `serde_json`:** it saves about 65 crates.
  But it must implement both eras, `server/discover`, `_meta`,
  `UnsupportedProtocolVersionError` and `resultType`, and follow each new
  revision of the specification. The specification changed its lifecycle in
  `2026-07-28`. rmcp carries that cost for us.
- **Streamable HTTP:** it needs a listening server, and horsegirl does not
  run one.

### Where this answer is weak

- The dependency tree is large for a stdio loop: tokio, futures, tracing,
  chrono and uuid come with rmcp. They are in the binary only, never in the
  core or the WebAssembly build.
- An `isError: true` result has no `structuredContent`, although the tool
  declares an `outputSchema`. The reference client printed it, reported
  `tool_is_error` and did not flag the missing `structuredContent`. A stricter
  client may.
- Only one real client was tried. The modern era was tested with hand-written
  messages, not with a modern client.

### Follow-up items

- 0024 fixes this shape in the core types, documents it and holds it with a
  snapshot test.
- 0038 builds the server on rmcp with `spec`, `check`, `fix` and `draw`.
- 0039 holds the diagnostics JSON in golden files.
- 0117 holds the tool schemas with a golden test and tests both eras.

## Acceptance criteria

- [x] The Answer section states a conclusion and the evidence for it.
- [x] Follow-up items exist for the work the answer implies.
- [x] A prototype server answers one tool call from a real MCP client.
- [x] The answer fixes the JSON shape of a diagnostic and of a fix.

## 2026-11-25

Criterion 3: npx @modelcontextprotocol/inspector@2.16.0 --cli ./target/debug/horsegirl-mcp-spike --method tools/call --tool-name check --tool-arg 'source=Café --> Payments' exited 0 and printed structuredContent with one E0001 diagnostic (span line 1 columns 6-9, safe fix '->'), a text block with the same JSON, isError false. Wire capture showed initialize protocolVersion 2025-11-25. A hand-written server/discover returned supportedVersions up to 2026-07-28. cargo test in spikes/mcp: 1 passed against check.golden.json.

## 2026-11-25

Criteria 1 and 4: the Answer section states rmcp 3.11.0 over stdio, dual-era, and the JSON shape of a check result, a diagnostic, a span, a position, a fix and an edit, with the evidence. Criterion 2: follow-ups are 0024 (core types and snapshot), 0038 (server), 0039 (golden files), and new 0117 (tool schema golden test in both eras); notes on 0024 and 0038 point at the answer. scripts/task check passed.
