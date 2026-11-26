---
id: 115
uid: d12ee02f-508d-42d0-9a4d-8fa3ea3d4e04
title: Reserve the horsegirl name on crates.io and npm
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: none
area: release
effort: s
related:
- 20
- 96
---

## Why

SPIKE-20 found the name free on crates.io and npm on 2026-11-25. Nothing holds it for the project. A squatter can take it before the first release.

## Acceptance criteria

- [x] The crate `horsegirl` is published on crates.io as a placeholder under the owner account.
- [x] The npm package `horsegirl` exists under the owner account.
- [x] A note on this item records both registry URLs.

The owner runs the publish step. An agent must not publish.

## 2026-11-26

Published placeholders, version 0.0.0, no code, README pointing at the repository. crates.io: https://crates.io/crates/horsegirl (the API lists horsegirl 0.0.0). npm: https://www.npmjs.com/package/horsegirl (the registry lists maintainer oddurs). Both checked on 2026-11-26.
