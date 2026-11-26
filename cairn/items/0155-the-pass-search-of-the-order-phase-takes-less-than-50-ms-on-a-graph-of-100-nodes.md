---
id: 155
uid: 58847d0c-5e48-4d0f-a4b8-6e4829e89751
title: The pass search of the order phase takes less than 50 ms on a graph of 100 nodes
type: chore
status: backlog
milestone: v0.2
created: 2026-11-25
updated: 2026-11-26
priority: p1
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

`keep_out` in `crates/horsegirl-core/src/layout/through.rs` tries one move at a time. After a move it counts again the steps that start or end in the ranks that the move changed. It tries another place for a group only when a link passes through that group or through a group under the same parent. So a large diagram with few passes costs little. A large graph with hundreds of passes that no order removes is still slow: each pass gives two `Side` moves in each round, and each move that fails is counted twice, once to try it and once to undo it.

Measured in a release build with the generator of `no_generated_graph_has_a_fault` (seed 1, 200 graphs), with every limit of the generator multiplied by a scale. The time is the slowest `layout` call of the 200. The machine ran other work at the same time, so read each number as a size and not as a limit.

| scale | nodes, links, groups up to | no search | first search (commit e627417) | search today |
| --- | --- | --- | --- | --- |
| 1 | 24, 48, 5 | 0.8 ms | 1.7 ms | 1.5 ms |
| 2 | 48, 96, 11 | 2.1 ms | 7.5 ms | 6.6 ms |
| 4 | 96, 192, 23 | 13 ms | 0.30 s | 0.17 s |
| 8 | 192, 384, 47 | 62 ms | 9.1 s | 3.0 s |

These graphs are random, and most of them have hundreds of passes that no order removes.

A structured diagram of that size was slow too, and the first search did not show that. The second review of pull request 48 joined copies of `10-platform` with three links from each copy to the next. The same measure here, one `layout` call, the least of five runs:

| copies | nodes | groups | no search | first search | search today |
| --- | --- | --- | --- | --- | --- |
| 1 | 34 | 8 | 0.4 ms | 1.2 ms | 1.3 ms |
| 2 | 68 | 16 | 1.2 ms | 6.9 ms | 3.6 ms |
| 4 | 136 | 32 | 3.3 ms | 63 ms | 13 ms |
| 8 | 272 | 64 | 9.8 ms | 0.51 s | 49 ms |

## Proposal

Make a move that fails cost less. `recount` runs a second time to undo a move: keep what it changed and write that back. `Side` moves one link for each pass in each round: skip a `Side` that failed in the round before when no rank of its link changed since. A limit on the rounds is the last resort: the search then stops with a pass that it could remove.

## Acceptance criteria

- [ ] The slowest `layout` call on the 200 graphs of scale 4 takes less than 50 ms in a release build.
- [ ] The slowest `layout` call on the 200 graphs of scale 8 takes less than 1 s in a release build.
- [ ] `tests/layout.positions` does not change.
- [ ] The passes on the 3000 graphs of seed 1 at scale 1 do not rise above 640.

## 2026-11-25

Pull request 48, fix round 1, did the first part of the old proposal: Shift only for a group with a pass and the groups under the same parent, and a count of the changed ranks only. The body now states the code as it is and what is left. The sentence 'A real diagram of that size has fewer' was wrong for the time: copies of 10-platform were slow too, and the body has that table.

## 2026-11-25

Pull request 48, fix round 3, added the last resort of the proposal: keep_out stops when its work reaches LIMIT = 1 000 000 items and steps counted again in through.rs. The third review found dense files that this item did not measure. Release build, horsegirl draw -o -, least of 5 runs: 100 things in 10 top-level groups with 390 random links took 3.6 s with no limit, 173 ms with the limit and 97 ms with no search; 150 things in 15 groups with 579 links took 27 s, 319 ms and 228 ms; 80 things in 20 groups with 160 links took 0.46 s, 89 ms and 15 ms. So the search now adds less than 0.1 s to a file. What is left for this item: a search that reaches the limit stops with passes that a longer search removes, so a move that fails must cost less before the limit can go. Measure the tables of the body again: the limit changes the column 'search today'.

## 2026-11-25

The fourth review of pull request 48 measured small graphs: with up to 52 nodes in 8 groups and up to 3 links for each node, 16 of 800 searches stop at LIMIT. So the limit is not a guard for rare files only. The earlier note on item 146 gives 65 721 as the largest work of the generated tests; that number is for those tests only. The same review found that a move that changes no rank added nothing to the work: a line of 1600 nodes, each two in their own group, then 40 nodes in 8 groups with 60 links, took 3.6 s in horsegirl draw against 0.62 s on main. The search now counts each move before it runs (the groups and the items of the ranks that it copies). After that change the same file takes 0.69 s and the file with 400 nodes takes 0.085 s, release build, one run each. The count of 16 of 800 is from before that change, so more searches now stop at the limit. Measure it again with the tables of the body.

## 2026-11-26

From the reviews of #67 (item 0122), release build, random links, no group unless stated. layout time on main / with untangle in each round / with untangle once on the best order: 300 nodes 900 links 1.42 s / 5.15 s / 1.63 s; 600 nodes 1800 links 18.8 s / 52.4 s / 20.6 s; 400 nodes 1200 links 20 groups 1.97 s / 5.03 s / 1.74 s; 2000 nodes 2400 links 3.73 s / 7.71 s / 4.08 s; 100 nodes 390 links 10 groups 0.24 s / 0.54 s / 0.23 s. #67 ships the third column. untangle has no limit on its work: each exchange that stays removes a crossing, so it ends.

## 2026-11-26

Triage: v0.2 at p1. The playground (0068) lays out on every key press, in WebAssembly.
