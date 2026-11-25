---
id: 18
uid: 7e4d4ca9-0d39-4196-828f-bad060fa9dbb
title: Which layered layout design do we build on?
type: spike
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
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

## Question

Which layered layout design gives acceptable flow pictures soonest: an existing crate, a port of the ELK layered phases, or our own Sugiyama pipeline?

## Why it matters

The flow lens, the corpus test and later stability work all depend on this choice. The stability rule needs the ordering and placement phases to accept seed positions.

## Time box

Five days.

## Answer

### Conclusion

Build our own Sugiyama pipeline in the core crate, with five phases: orient, rank, order, place, route. Do not depend on a layout crate, and do not port ELK. Use the published designs of dagre and ELK as the reference for each phase, and copy no code from them.

Our own pipeline is the only candidate that keeps the arrangement of a diagram with groups when it gets earlier positions. elk also takes earlier positions, and does not keep the arrangement: 232 pairs of things change sides with elk and 18 with own. The prototype does not yet hold a picture still: a seed of centres alone is not enough, and the seed must also carry the order of each layer. The prototype also draws four pairs of boxes with no gap between them, and item 0032 must not copy that fault. Our own pipeline is also the smallest candidate: its native code is 3.5 times smaller than dagre, 6.5 times smaller than graphlay and 39 times smaller than elk. The port of ELK draws the best pictures today. If the stability rule did not exist, the port of ELK would be the answer.

### The candidates

| name | what it is | licence |
| --- | --- | --- |
| own | a prototype of our own pipeline, [`src/own.rs`](../../spikes/layered-layout/src/own.rs), 736 lines, no dependency | MIT |
| dagre | the crate `dagre` 0.1.2, a port of dagre.js | Apache-2.0 |
| graphlay | the crate `graphlay` 0.4.0 | MIT or Apache-2.0 |
| elk | the crate `elk-layout` 0.12.2, a port of the ELK layered phases from Java | EPL-2.0 or GPL-3.0-or-later |

`dagre` and `graphlay` stand for "an existing crate". `elk-layout` stands for "a port of the ELK layered phases": the port exists, so the spike measured it and did not write one. The prototype, the adapters and the commands are in [`spikes/layered-layout`](../../spikes/layered-layout/README.md).

### Evidence: the pictures

All four candidates drew the same ten diagrams with the same box sizes and the same gaps. One renderer drew every picture and one module scored every picture. The totals over the ten diagrams, from `cargo run --release -- draw`:

| candidate | overlaps | touching | group faults | links through a box | crossings |
| --- | --- | --- | --- | --- | --- |
| own | 0 | 4 | 0 | 0 | 80 |
| dagre | 0 | 0 | 0 | 8 | 74 |
| graphlay | 0 | 0 | 0 | 0 | 159 |
| elk | 0 | 0 | 0 | 0 | 36 |

"Overlaps" counts pairs of boxes that overlap by more than half a pixel. "Touching" counts pairs of boxes that do not overlap and that have less than one pixel between them.

- No candidate overlaps two boxes or breaks a group.
- own draws four pairs of boxes with a gap of 0 px, where the node gap is 24 px. In `07-compiler`, `Checks` is at y -128 to -92 and `Lens` is at y -92 to -56. In `10-platform` the pairs are `Search index` and `Data lake`, `Analytics sink` and `Notifier`, `Cache` and `Accounts database`. No other candidate draws such a pair.
- The cause is two rules of the prototype together. A group has a marker in every rank that the group spans, and the place phase gives a marker a gap of 0 px (`place` in `src/own.rs`). The order phase can put the marker between two things of the group. Rank 4 of `07-compiler` is `Checks`, the marker of `Core`, `Lens`. Then no constraint holds the two things 24 px apart. The smallest input is in the README of the spike: 5 things, 1 group, 3 links.
- elk has the fewest crossings and the most compact pictures. `10-platform` is 636 px high with elk and 1128 px high with own.
- own has 2 crossings or fewer on seven of the ten diagrams. Six of the seven are acceptable. The seventh, `07-compiler`, has the touching pair. own is loose on `10-platform`. Its weak phase is the order phase, in two cases. The first case is a dense graph without groups: `08-module-deps` has 22 crossings with own, 3 with elk, 3 with graphlay and 14 with dagre. That diagram alone is 22 of the 80 crossings of own. The second case is nested groups: `06-cluster-request` has 9 crossings with own and 1 with elk.
- dagre sends 8 links through a box. It returns control points for a curve and has no router, so item 0033 would be needed with it too.
- graphlay has the most crossings on diagrams with groups: 137 on `10-platform`.

The pictures:

| diagram | | | | |
| --- | --- | --- | --- | --- |
| 01-checkout | [own](../../spikes/layered-layout/pictures/own/01-checkout.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/01-checkout.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/01-checkout.svg) | [elk](../../spikes/layered-layout/pictures/elk/01-checkout.svg) |
| 02-ci-pipeline | [own](../../spikes/layered-layout/pictures/own/02-ci-pipeline.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/02-ci-pipeline.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/02-ci-pipeline.svg) | [elk](../../spikes/layered-layout/pictures/elk/02-ci-pipeline.svg) |
| 03-order-states | [own](../../spikes/layered-layout/pictures/own/03-order-states.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/03-order-states.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/03-order-states.svg) | [elk](../../spikes/layered-layout/pictures/elk/03-order-states.svg) |
| 04-data-pipeline | [own](../../spikes/layered-layout/pictures/own/04-data-pipeline.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/04-data-pipeline.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/04-data-pipeline.svg) | [elk](../../spikes/layered-layout/pictures/elk/04-data-pipeline.svg) |
| 05-sign-in | [own](../../spikes/layered-layout/pictures/own/05-sign-in.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/05-sign-in.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/05-sign-in.svg) | [elk](../../spikes/layered-layout/pictures/elk/05-sign-in.svg) |
| 06-cluster-request | [own](../../spikes/layered-layout/pictures/own/06-cluster-request.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/06-cluster-request.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/06-cluster-request.svg) | [elk](../../spikes/layered-layout/pictures/elk/06-cluster-request.svg) |
| 07-compiler | [own](../../spikes/layered-layout/pictures/own/07-compiler.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/07-compiler.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/07-compiler.svg) | [elk](../../spikes/layered-layout/pictures/elk/07-compiler.svg) |
| 08-module-deps | [own](../../spikes/layered-layout/pictures/own/08-module-deps.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/08-module-deps.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/08-module-deps.svg) | [elk](../../spikes/layered-layout/pictures/elk/08-module-deps.svg) |
| 09-incident | [own](../../spikes/layered-layout/pictures/own/09-incident.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/09-incident.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/09-incident.svg) | [elk](../../spikes/layered-layout/pictures/elk/09-incident.svg) |
| 10-platform | [own](../../spikes/layered-layout/pictures/own/10-platform.svg) | [dagre](../../spikes/layered-layout/pictures/dagre/10-platform.svg) | [graphlay](../../spikes/layered-layout/pictures/graphlay/10-platform.svg) | [elk](../../spikes/layered-layout/pictures/elk/10-platform.svg) |

The README of the spike shows all forty pictures on one page, with the score of each.

### Evidence: earlier positions as a seed

How each candidate accepts earlier positions:

- **own: yes, in the order phase and in the place phase, but the seed of the prototype is too small.** The input is the earlier centre of each thing, and nothing for a new thing. The order phase pins every seeded thing: the seeded things under one parent keep the order of their earlier positions inside each layer, and only new things and bends look for a slot. The place phase starts every seeded thing at its earlier position and moves it only when a constraint pushes it. The rank phase does not read the seed. A bend has no earlier position, so the seeded run looks for a new slot for every bend, and a bend in a new slot pushes the pinned things beside it.
- **graphlay: yes without groups, no with groups.** The input is `Node::position`, with `SeedMode::Structure` to keep layers and order. The crate documents that it ignores positions in a graph with groups. The six diagrams with groups gave the same numbers with and without a seed.
- **elk: yes, but the result is poor.** The input is the `x` and `y` of each node, with the `INTERACTIVE` strategy for cycle breaking, layering, crossing minimisation and node placement. Every group must set the same four strategies, or ELK refuses the graph. A new thing needs a position too. The interactive order phase keeps the given order and does not reduce crossings.
- **dagre: no.** It has no input for positions.

The measurement, from `cargo run --release -- seed`: each of the ten diagrams gets one new thing with one link, and the new picture is compared with the earlier one. 189 things are in both pictures. "Changed sides" counts pairs of things where one was above the other and is now below it, or the same for left and right.

| candidate | things that moved | pairs that changed sides | crossings after the edit |
| --- | --- | --- | --- |
| own, no seed | 69 | 71 | 86 |
| own, seed | 68 | 18 | 110 |
| graphlay, no seed | 87 | 132 | 151 |
| graphlay, seed | 117 | 135 | 151 |
| elk, no seed | 128 | 434 | 37 |
| elk, seed | 141 | 232 | 273 |
| dagre | 134 | 708 | 88 |

With a seed, own keeps the arrangement: 18 pairs change sides, against 71 without a seed. The seed costs own 24 crossings. The 68 things that still move do not all move because of the edit. The next table shows that 33 things move with no edit at all. With a seed, elk goes from 37 crossings to 273 and still moves 141 things. SPIKE-75 decides the cost function. This spike shows only that the two phases can take the seed.

The same command then runs each candidate with no edit: the seed is the picture that the candidate drew for the same graph. A layout that holds a picture still moves nothing in this run.

| candidate | things that moved, of 189 | diagrams with a move, of 10 | crossings before | crossings after |
| --- | --- | --- | --- | --- |
| own | 33 | 5 | 80 | 99 |
| graphlay | 0 | 0 | 159 | 168 |
| elk | 131 | 9 | 36 | 252 |

The seeded layout of own is not a fixed point. The five diagrams, with the things that moved, the longest move and the crossings:

| diagram | things that moved | longest move | crossings |
| --- | --- | --- | --- |
| 04-data-pipeline | 6 of 19 | 24 px | 0 to 3 |
| 05-sign-in | 2 of 15 | 12 px | 0 to 4 |
| 07-compiler | 6 of 16 | 36 px | 1 to 4 |
| 08-module-deps | 4 of 22 | 24 px | 22 to 31 |
| 10-platform | 15 of 34 | 36 px | 44 to 44 |

Every longest move is a multiple of 12 px, which is the gap beside a bend (`BEND_GAP`). The prototype gives no seed to a bend (`seed: None` in `build`), so the order phase sorts the bends again from a different start and ends at a different order. The spike did not test a seed that carries the bends. graphlay moves no thing, and its crossings change on three diagrams without groups because its routes change. elk moves 131 things when it gets its own picture back.

### Evidence: size, dependencies and licence

One small program for each candidate, built for size and stripped. The numbers are the increase over a program with no layout.

| candidate | native | WebAssembly | crates added | lines of Rust |
| --- | --- | --- | --- | --- |
| own | 119 kB | 108 kB | 0 | 736 |
| dagre | 421 kB | 382 kB | 15 | 17329 |
| graphlay | 774 kB | 661 kB | 11 | 41935 |
| elk | 4648 kB | 4128 kB | 63 | 125499 |

- Every candidate builds for `wasm32-unknown-unknown`.
- Every candidate gave the same bytes on four runs.
- `elk-layout` is EPL-2.0 or GPL-3.0-or-later. horsegirl is MIT, and ADR-16 has not confirmed the licence yet. A binary that contains the port must carry the EPL terms for it. A port that we write from the Java source is a derived work under the same licence, and the layered phases alone are 76128 lines in the existing port.
- `dagre` 0.1.2 does not compile without its default `json` feature. The three crates are between one and seven months old.

### What item 0032 takes from the prototype

- Keep the group rule of the order phase. A group is present in every rank that it spans. Groups under one parent hold one order in every layer. This rule makes the group boxes disjoint without a repair step.
- Keep the constraint form of the place phase. The order and the gaps are difference constraints, and a seed is only a wish inside the constraints. The order constraints hold, so a box cannot overlap another box. The gap constraints of the prototype do not hold the node gap between every pair of things.
- Do not keep the gap of a group marker as it is. The two rules above together let two things touch: the marker of a group has a gap of 0 px, and the order phase can put the marker between two things. Item 0032 must do one of two things. Either the order phase never puts a marker between two things of one rank, or the place phase adds the node gap between neighbouring things across a marker. The check is a count of pairs of boxes closer than the node gap, and the count must be 0. The column "touching" of `cargo run --release -- draw` counts the pairs with less than one pixel between them, and the prototype has 4.
- Do not keep the shape of the seed. One optional centre for each thing cannot hold a picture still, because it says nothing about the bends. The seed must also carry the order of each layer, bends included, or the position of each bend. SPIKE-75 decides which.
- Hold this rule from the first version: a layout seeded with its own picture moves nothing. The no-edit run of `cargo run --release -- seed` is the check, and the prototype fails it.
- Replace the rank phase with network simplex. The prototype uses longest path with a local correction.
- Replace the choice of the links to reverse. `04-data-pipeline` puts `CRM dump` in the last rank, which stretches its group across the picture.
- Improve the order phase. Item 0122 holds the target.

### Where this answer is weak

- The ten diagrams were written for this spike. The corpus of public flowcharts from SPIKE-17 was not in the repository when the spike ran. The diagrams are in [`spikes/layered-layout/corpus`](../../spikes/layered-layout/corpus). Six have groups and three nest groups. The first one restates `examples/checkout.horse`.
- No diagram has a link label, and no link ends at a group. Items 0123 and 0124 hold those two cases. ELK and dagre place labels today, and own does not.
- The prototype fails the no-edit run, so the numbers of the edit run mix two effects: the edit, and the bends that have no seed. The edit run does not show how few things a correct seed would move.
- The seed measurement is one edit for each diagram. The interactive mode of ELK got one configuration inside the time box. A better configuration may exist.
- The score had no count of touching boxes until the second review, so the fault was found by a reviewer and not by the spike. The count sees a gap below 1 px only. A gap between 1 px and the node gap is not counted, and the spike did not look for one.
- The prototype routes links with one track for each source in each gap. Item 0033 replaces it.
- Until the third review, the elk adapter set the node gap and the layer gap on the root only. ELK does not pass the two gaps down to a group, so elk drew 33 pairs of boxes 20 px apart inside groups. The adapter now sets the two gaps on every group, and a test in `src/crates.rs` checks the two gaps inside an elk group. No test checks the gaps of dagre or graphlay inside a group.

### Follow-up items

- 0032 builds rank, order and place on this design. It existed.
- 0033 builds the router. It existed.
- SPIKE-75 and 0076 decide and build the cost of a moved thing. They existed. SPIKE-75 has a new criterion: a layout seeded with its own picture moves nothing.
- 0122: fewer crossings in the layered layout, measured against a reference.
- 0123: draw a link that starts or ends at a group.
- 0124: reserve room for link labels in the layered layout.
- 0125: check the choice on the corpus of public flowcharts. It depends on SPIKE-17.

## Acceptance criteria

- [x] The Answer section states a conclusion and the evidence for it.
- [x] Follow-up items exist for the work the answer implies.
- [x] At least two candidates draw the same ten corpus diagrams, and the pictures are in the item.
- [x] The answer says whether each candidate can take earlier positions as an input.

## 2026-11-25

Criterion 1: the Answer states the conclusion (build our own pipeline) with three sets of evidence. Ran 'cargo run --release -- draw' in spikes/layered-layout: it printed totals over ten diagrams of own 0 overlaps, 0 group faults, 0 links through a box, 80 crossings; dagre 0, 0, 8, 74; graphlay 0, 0, 0, 159; elk 0, 0, 0, 36. Four runs gave identical picture files (diff -r printed nothing).

## 2026-11-25

Criterion 2: 'cairn check' printed 'ok: 120 item(s), 0 warning(s)' after filing 0122 (fewer crossings), 0123 (links that end at a group) and 0124 (room for link labels). 0032, 0033, SPIKE-75 and 0076 already hold the rest. Searched with 'cairn search label', 'cairn search group' and 'cairn search crossings' before filing.

## 2026-11-25

Criterion 3: four candidates drew the same ten diagrams, and the Answer links the forty SVG files under spikes/layered-layout/pictures. The ten diagrams were written for this spike, because the SPIKE-17 corpus was not on main when the spike ran ('git ls-tree -r origin/main' showed no corpus). A reviewer who reads 'corpus' as the SPIKE-17 corpus must reject this tick.

## 2026-11-25

Criterion 4: the Answer says for each candidate how it accepts earlier positions: own yes in order and place; graphlay yes without groups and no with groups; elk yes through the INTERACTIVE strategies with a poor result; dagre no. Ran 'cargo run --release -- seed': over 189 things, pairs that changed sides were own 71 without a seed and 18 with a seed; graphlay 132 and 135; elk 448 and 215 with crossings rising from 37 to 273; dagre 708.

## 2026-11-25

Size evidence: built one small program for each candidate outside the repository, for x86_64 and for wasm32-unknown-unknown, with opt-level s, LTO and strip. Sizes in bytes, native then wasm: no layout 284648, 22170; own 403960, 130447; dagre 705304, 404457; graphlay 1058864, 683635; elk 4932680, 4150165.

## 2026-11-25

Review round 1. Ran 'cargo run --release -- seed' with the new no-edit run: seeded with its own picture, own moved 33 of 189 things on 5 diagrams (crossings 80 to 99), graphlay 0 (159 to 168), elk 136 (36 to 262). The Answer now says the seed must also carry the bends or the order of each layer, and that the order phase is weak on a dense graph without groups (08-module-deps: 22 against 3). Filed 0125 to check the choice on the SPIKE-17 corpus. Criterion 3 stays ticked on the ten spike diagrams; 0125 holds the check on public flowcharts. 'cairn check' printed 'ok: 121 item(s), 0 warning(s)'.

## 2026-11-25

Review round 2. Added the column 'touching' to measure.rs (pairs of boxes that do not overlap and have less than one pixel between them). Ran 'cargo run --release -- draw' in spikes/layered-layout: totals printed own 0 overlaps, 4 touching, 0 group faults, 0 links through a box, 80 crossings; dagre 0, 0, 0, 8, 74; graphlay 0, 0, 0, 0, 159; elk 0, 0, 0, 0, 36. The 4 pairs are 07-compiler Checks and Lens, and 10-platform Search index and Data lake, Analytics sink and Notifier, Cache and Accounts database. 'git status' showed no change to the forty pictures. Ran the five-thing input of the reviewer from a directory outside the repository: own printed touching 1 (e at y -128, a at y -92, both 36 high), the other three printed 0. The Answer names the pairs and the cause, and 0032 has a note that states the gap. The conclusion stands.

## 2026-11-25

Review round 3. The elk adapter set elk.spacing.nodeNode and elk.layered.spacing.nodeNodeBetweenLayers on the root only, so elk used 20 px for both gaps inside a group. Added the test elk_uses_the_same_gaps_inside_a_group to spikes/layered-layout/src/crates.rs: before the fix it failed with a gap of 20.00000076293945, and after the fix 'cargo test --release' printed 'test result: ok. 1 passed; 0 failed'. Ran 'cargo run --release -- draw' twice with identical output: six elk pictures changed (01, 02, 04, 06, 07, 10), 10-platform with elk is 2682 x 636, elk crossings stay 36 and bends stay 436. Ran 'cargo run --release -- seed': elk pairs that changed sides are 434 without a seed and 232 with a seed, things that moved with a seed are 141, and the no-edit run of elk is 131 things moved and crossings 36 to 252. No number of own, dagre or graphlay changed. The conclusion does not change.
