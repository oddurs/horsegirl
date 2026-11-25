---
id: 10
uid: 818f1be6-eab8-493e-a23b-751c47a65ded
key: v1.0
title: Gallop
type: milestone
status: backlog
depends_on:
- 9
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
due: 2027-09-30
---

## Ships

The language, the output and the tool contracts are frozen, documented and tested, so that breaking one is a bug.

## Acceptance criteria

- [ ] The language specification is a versioned document, and a conformance suite tests it.
- [ ] A second reader, independent of the Rust core, passes the conformance suite.
- [ ] Install is one command on Linux, macOS and Windows, and CI verifies each path.
- [ ] A stranger completes the tutorial with no help.

## Explicitly not in this milestone

- Any new lens, command or piece of syntax. New surface goes to `later`.
