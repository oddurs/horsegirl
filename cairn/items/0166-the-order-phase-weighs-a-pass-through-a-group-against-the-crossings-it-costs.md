---
id: 166
uid: d795b05d-21dd-46cb-b5be-2b5afbfa7f20
title: The order phase weighs a pass through a group against the crossings it costs
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: m
related:
- 122
- 146
---

## Problem

The order phase has two parts with two goals. The sweeps and the exchange of neighbours in `crates/horsegirl-core/src/layout/order.rs` count crossings only. The pass search in `crates/horsegirl-core/src/layout/through.rs` counts passes first and crossings second, with no weight. So one pass less can cost many crossings. The second review of pull request 48 found, on 3000 generated graphs (seed 1), 326 graphs with more crossings after the search than before it: graph 2184 lost 1 pass and went from 79 to 114 crossings.

The exchange of neighbours from item 0122 also keeps the order of two groups under one parent, because every rank must hold that order. So it cannot remove a crossing that another order of two such groups removes. `06-cluster-request` draws with 4 crossings, and the ELK port of `spikes/layered-layout` leaves 1.

## Proposal

Give the two parts one cost: the crossings, plus a fixed number of crossings for each pass. Choose the number on generated graphs. Let the search try another place for a group among the groups under its parent in every picture, and not only when a link passes through a group. That move changes every rank of the group at once, so the rule of one order in every rank holds.

## Acceptance criteria

- [ ] On 3000 generated graphs (the generator of `no_generated_graph_has_a_fault`, seed 1), no graph has more than a stated number of crossings above its count before the pass search, and the item states the number and the weight of a pass.
- [ ] `06-cluster-request` draws with 3 crossings or fewer.
- [ ] No number in `the_crossings_of_each_corpus_diagram_do_not_rise` rises.
- [ ] The tests that allow no pass through a group in `crates/horsegirl-core/tests/layout.rs` pass unchanged.
