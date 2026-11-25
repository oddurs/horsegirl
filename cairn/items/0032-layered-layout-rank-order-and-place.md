---
id: 32
uid: 72b5f672-35bd-446b-ad4a-01cea1ea5842
title: 'Layered layout: rank, order and place'
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 18
- 25
- 30
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: high
surface: none
effort: l
area: layout
track:
- pictures
---

## Problem

A flow needs things placed in layers by dependency, with few crossings.

## Proposal

Build the first three phases of the layered solver the spike chose: rank, order, place. Groups stay together. The phases take an optional seed of earlier positions and ignore it for now.

## Acceptance criteria

- [x] The reference flow places 9 things in ranks that follow the links.
- [x] No two boxes overlap on any corpus diagram.
- [x] The same input gives the same positions on every run.

## 2026-11-25

From the review of SPIKE-18: a seed of one optional centre for each thing cannot hold a picture still. The prototype in spikes/layered-layout fails the no-edit run of 'cargo run --release -- seed': 33 of 189 things move when a diagram is seeded with its own picture, each by a multiple of 12 px, the gap beside a bend. The bends have no seed. Give the seed type room for the order of each layer, bends included, or for the position of each bend. Rule for the first version: a layout seeded with its own picture moves nothing. SPIKE-75 decides what the seed carries.

## 2026-11-25

From the second review of SPIKE-18: the criterion 'No two boxes overlap' passes when two boxes touch, so read it with a gap. No two boxes of one rank are closer than the node gap (24 px in the prototype) on any corpus diagram. The prototype in spikes/layered-layout breaks this rule on 4 pairs: 'cargo run --release -- draw' prints touching 1 for 07-compiler (Checks and Lens share the edge y -92) and touching 3 for 10-platform, and 0 for dagre, graphlay and elk. Cause: 'place' in src/own.rs gives a pair of neighbours a gap of 0 when one of the two is a group marker (Kind::Border), and the order phase can put the marker between two things of one rank. Do one of two things: the order phase never puts a marker between two things, or the place phase adds the node gap between neighbouring things across a marker. Smallest input: 'G {' / 'a, c, d, e' / '}' / 'b -> e' / 'd -> e' / 'b -> a' draws e at y -128 to -92 and a at y -92 to -56. Test it with a count of pairs closer than the node gap, not with an overlap count.

## 2026-11-25

Criterion 1: ran 'scripts/task check' at commit 8b7b62f. The test the_reference_flow_places_nine_things_in_ranks_that_follow_the_links passed natively and on wasm32-wasip1. It lays out examples/checkout.horse: 9 nodes, 9 links, every link goes to a higher rank and its two boxes are at least the rank gap apart on x. The ranks are Web app 0, Mobile app 0, API gateway 1, Orders 2, Payments 3, Postgres 3, Notify 4, Events 5, Email provider 6.

## 2026-11-25

Criterion 2, read with a gap as the review of SPIKE-18 asked, and with 'corpus' read as the ten diagrams in spikes/layered-layout/corpus plus examples/checkout.horse. The test no_two_boxes_are_closer_than_the_node_gap_on_any_corpus_diagram passed natively and on wasm32-wasip1: over 198 things it found no pair of boxes closer than 24 px, no node outside its group, no node closer than 24 px to a group it is not in, and no two groups closer than 24 px unless one holds the other. The smallest input of the review ('G { a, c, d, e }', 'b -> e', 'd -> e', 'b -> a') passes in two_things_of_a_group_in_one_rank_keep_the_node_gap. With the fault of the prototype put back for one run (a marker in every rank, gap 0 beside a marker) the same tests failed with 'nodes 0 and 3 are 8 apart' and 'nodes 6 and 7 are 0 apart'. The test no_generated_graph_has_a_fault holds the same rules on 300 generated graphs with loops and nested groups. Not done: the public flowcharts in corpus/flowcharts are Mermaid, and no converter exists yet (items 0040 and 0064), so the layout has not drawn them.

## 2026-11-25

Criterion 3: the test the_same_input_gives_the_same_positions_on_every_run passed natively on Linux x86-64 and on wasm32-wasip1 in 'scripts/task check'. It lays out each of the eleven files twice in one process and compares the two results, then compares every position with the committed file crates/horsegirl-core/tests/layout.positions (279 lines). Both targets match the one file, and three separate runs of 'cargo test -p horsegirl-core --test layout' printed 'test result: ok. 9 passed'. The macOS job of CI runs the same test on the pull request.

## 2026-11-25

What the next items need to know. The public entry is horsegirl_core::layout::layout(&Graph, Option<&Layout>) -> Layout. Layout.bends gives, for each link, the point where it passes each rank between its ends: item 0033 starts from those points. The seed is the earlier Layout, which holds the bends, and the layout does not read it yet: SPIKE-75 decides. The function that turns a Model into a Graph is in crates/horsegirl-core/tests/layout.rs (fn flow), because the box sizes are a choice of the flow lens: LENS-36 owns it. A link that starts or ends at a group is left out there (item 0123). Filed 0141: a group box is not yet wide enough for its name.

## Result

horsegirl_core::layout::layout places a flow: loops turn the link that closes them, ranks come from network simplex, each rank is ordered by mean-of-neighbours sweeps with one order of groups in every rank, and positions come from difference constraints. On the ten spike diagrams and the reference file no two boxes are closer than 24 px. The layout does not read the seed yet, and it routes no link.
