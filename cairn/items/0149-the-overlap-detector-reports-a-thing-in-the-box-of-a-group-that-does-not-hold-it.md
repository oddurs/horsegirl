---
id: 149
uid: fee2812e-f09d-4081-a49b-ae4ced537c14
title: The overlap detector reports a thing in the box of a group that does not hold it
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: check
effort: m
related:
- 33
---

## Problem

`Kind::Boxes` in `crates/horsegirl-core/src/overlap.rs` uses geometry only: a group box that holds the box of a thing is never a collision. A scene with a thing inside the box of a group that is not its parent is wrong, and `overlaps` returns no collision. A link to that thing is also read as a link inside the group. `Shape` has no parent, so the detector cannot know. `faults` in `crates/horsegirl-core/tests/layout.rs` holds the rule for the layered layout only. Found in the review of item 0033. Item 0040 counts overlaps on the corpus and can need this case.

## Acceptance criteria

- [ ] The detector knows the group of each thing, from the scene or from an argument.
- [ ] A thing inside the box of a group that does not hold it is a collision, with a test in `crates/horsegirl-core/tests/overlap.rs`.
- [ ] The module comment of `overlap.rs` states what the detector cannot see.
