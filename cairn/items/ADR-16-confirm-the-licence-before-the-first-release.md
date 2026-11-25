---
id: 16
uid: 73e6ff44-96e9-43b1-aec0-7599a929c4a7
title: Confirm the licence before the first release
type: decision
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

MIT or Apache-2.0, at the user's option.

The adoption plan depends on other projects embedding the core: editors, documentation platforms, and in the end a code host. Some of those need the patent grant that Apache-2.0 carries and MIT does not. MIT stays as the second option because it is compatible with GPL-2.0 projects, which Apache-2.0 alone is not. Offering both is the convention for Rust libraries, so nobody who embeds the core has to ask.

The change is made now, while the owner holds all of the copyright. After outside contributions it would need every contributor to agree. A contribution is licensed the same way, with no additional terms, and CONTRIBUTING.md says so.

The specification text and the conformance suite follow the same licence. The files under `corpus/` keep their authors' licences.

## Revisit when

A host or a company asks for a patent grant.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
- [x] `Cargo.toml`, the licence files and the README agree.
