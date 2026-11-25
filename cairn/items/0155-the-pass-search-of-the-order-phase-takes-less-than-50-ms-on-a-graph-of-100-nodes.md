---
id: 155
uid: 58847d0c-5e48-4d0f-a4b8-6e4829e89751
title: The pass search of the order phase takes less than 50 ms on a graph of 100 nodes
type: chore
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
- 146
- 150
- 98
---

## Problem

`keep_out` in `crates/horsegirl-core/src/layout/through.rs` tries one move at a time. A move that changes a group counts every pass and every crossing of the net again, and each try copies the order of every rank. The cost of one round is the number of moves times the number of steps. On a diagram that a person reads this is small: `10-platform` (34 things, 8 groups) takes 1.2 ms, and took 0.5 ms before the search. On a large graph with many passes it is not small.

Measured in a release build with the generator of `no_generated_graph_has_a_fault` (seed 1, 200 graphs), with every limit of the generator multiplied by a scale. The time is the slowest `layout` call of the 200.

| scale | nodes, links, groups up to | before the search | with the search |
| --- | --- | --- | --- |
| 1 | 24, 48, 5 | 0.8 ms | 2.8 ms |
| 2 | 48, 96, 11 | 2.1 ms | 10 ms |
| 4 | 96, 192, 23 | 13 ms | 0.31 s |
| 8 | 192, 384, 47 | 55 ms | 9.5 s |

These graphs are random, and most of them have hundreds of passes that no order removes. A real diagram of that size has fewer. The search needed 9 rounds or fewer on every graph.

## Proposal

Count less for each move. A move of one item changes the passes of the links through that item only, and the crossings beside its ranks only. `Transpose` of two items already counts two corridors. `Side` and a `Transpose` with a group can do the same with a list of the links of each item. `Shift` changes every rank, so try a place for a group only when a link passes through it or through a group beside it. A limit on the rounds is the last resort: the search then stops with a pass that it could remove.

## Acceptance criteria

- [ ] The slowest `layout` call on the 200 graphs of scale 4 takes less than 50 ms in a release build.
- [ ] The slowest `layout` call on the 200 graphs of scale 8 takes less than 1 s in a release build.
- [ ] `tests/layout.positions` does not change.
- [ ] The passes on the 3000 graphs of seed 1 at scale 1 do not rise above 860.
