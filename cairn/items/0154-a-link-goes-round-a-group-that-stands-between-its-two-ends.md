---
id: 154
uid: 2fb2d51b-0752-4486-ad04-eed0caa8d421
title: A link goes round a group that stands between its two ends
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: l
related:
- 146
---

## Problem

A path turns only in the corridor between two ranks. So a link between two neighbouring ranks goes straight from one height to the other, and it passes through each group that has both ranks and stands between its two ends. The order phase removes such a pass when another order of the ranks has none (item 0146). Some graphs have no such order. On `10-platform`, `Shopping` and `Fulfilment` are both inside `Services` and both have ranks 3, 4 and 5. `Stores` is on one side of both, and each of the two has a link to `Stores` between two of those ranks. `Orders -> Order database` and `Inventory -> Inventory database` then pass through `Shopping`. The test `no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram` names both links.

## Proposal

Give such a link a path that goes round the group, or give the two groups other ranks. Two ways to look at first:

- The path leaves its box, goes along the corridor before the first rank of the group or after its last rank, and comes back. The link then needs an item in each rank that it passes twice, and a box needs a path that leaves through its left side.
- The rank phase moves one of the two groups by a rank, so that the two groups do not share the two ranks of the link.

Both change what a reader sees for every link of the group, so compare the two on `10-platform` before choosing.

## Acceptance criteria

- [ ] `10-platform` draws with no link through a group that holds neither of its ends.
- [ ] The test `no_path_crosses_the_box_of_a_thing_on_any_corpus_diagram` allows no collision of kind `LinkThroughGroup` on the eleven diagrams.
- [ ] The crossing count of each of the eleven diagrams in `the_crossings_of_each_corpus_diagram_do_not_rise` does not rise.
- [ ] `tests/layout.positions` changes only for `10-platform`.
