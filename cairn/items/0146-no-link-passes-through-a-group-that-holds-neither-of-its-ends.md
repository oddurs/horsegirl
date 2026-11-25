---
id: 146
uid: 77a30d06-ab27-4d9d-862a-111eafd3fcae
title: No link passes through a group that holds neither of its ends
type: feature
status: backlog
milestone: v0.1
depends_on:
- 33
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
related:
- 122
---

## Problem

The route phase draws every upright line in the corridor between two ranks. The order phase decides on which side of a group a link passes each rank. It does not count a group as something a link can cross. So it can put a link above a group in one rank and below the group in the next rank, and the upright line then passes through the group box. The link crosses no box of a thing, but the picture shows a line through a group that the link has nothing to do with.

The overlap detector reports this as `Kind::LinkThroughGroup`. On the eleven diagrams of `crates/horsegirl-core/tests/layout.rs` it reports 18: 1 on `07-compiler` (`Fixes -> Source` through `Core`) and 17 on `10-platform`. The test `no_path_crosses_a_box_on_any_corpus_diagram` holds these two numbers.

One attempt did not help: counting the top and the bottom of each group as two steps in `crossings` of `layout/order.rs` left both numbers as they are. The sweeps never propose the better order, so a count alone does not find it.

## Proposal

Make the order phase keep a link on one side of a group. Two parts:

- The bends of one link that lie under the same parent keep one side of each group under that parent in every rank, as the groups under one parent keep one order today.
- When a sweep orders a rank, an item that links into a group stands next to that group, and a transpose step (item 0122) counts a pass through a group as two crossings.

Some passes cannot be removed: a link between two groups with a third group between them in every rank must pass through the third group or go round the whole picture. Decide for that case whether the path goes round, or whether the pass stays and the detector does not count it.

## Acceptance criteria

- [ ] `07-compiler` draws with no link through a group that holds neither of its ends.
- [ ] `10-platform` draws with 8 or fewer such links, and each one that remains is named in the test with the reason it cannot be removed.
- [ ] The numbers in `no_path_crosses_a_box_on_any_corpus_diagram` fall, and no other collision appears on the eleven diagrams.
- [ ] The crossing count of each of the ten spike diagrams does not rise.
