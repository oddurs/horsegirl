---
id: 16
uid: 73e6ff44-96e9-43b1-aec0-7599a929c4a7
title: Confirm the licence before the first release
type: decision
status: planned
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: s
area: release
track:
- release
---

## Context

The repository was created under the MIT licence. Rust projects often use MIT and Apache-2.0 together, because Apache-2.0 carries a patent grant that some hosts require.

## Options and tradeoffs

MIT only: one short file, the same as cairn and harrow. MIT and Apache-2.0: the Rust convention, and a patent grant, at the cost of a second licence file and a contributor notice.

## Decision

Not decided. The owner decides before the first release, because changing a licence after outside contributions needs every contributor to agree.

## Revisit when

A host or a company asks for a patent grant.

## Acceptance criteria

- [ ] The choice, its evidence and its consequences are recorded.
- [ ] `Cargo.toml`, the licence files and the README agree.
