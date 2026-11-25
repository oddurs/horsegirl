---
id: 18
uid: 7e4d4ca9-0d39-4196-828f-bad060fa9dbb
title: Which layered layout design do we build on?
type: spike
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
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

Our own pipeline is the only candidate whose order phase and place phase take earlier positions on a diagram with groups. It is also the smallest candidate by two orders of magnitude. The port of ELK draws the best pictures today. If the stability rule did not exist, the port of ELK would be the answer.

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

| candidate | overlaps | group faults | links through a box | crossings |
| --- | --- | --- | --- | --- |
| own | 0 | 0 | 0 | 80 |
| dagre | 0 | 0 | 8 | 74 |
| graphlay | 0 | 0 | 0 | 159 |
| elk | 0 | 0 | 0 | 36 |

- No candidate overlaps two boxes or breaks a group.
- elk has the fewest crossings and the most compact pictures. `10-platform` is 604 px high with elk and 1128 px high with own.
- own is acceptable on the eight smaller diagrams and loose on `10-platform`. Its weak phase is the order phase where groups nest: `06-cluster-request` has 9 crossings with own and 1 with elk.
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

- **own: yes, in the order phase and in the place phase.** The input is the earlier centre of each thing, and nothing for a new thing. The order phase pins every seeded thing: the seeded things under one parent keep the order of their earlier positions inside each layer, and only new things and bends look for a slot. The place phase starts every seeded thing at its earlier position and moves it only when a constraint pushes it. The rank phase does not read the seed.
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
| elk, no seed | 128 | 448 | 37 |
| elk, seed | 143 | 215 | 273 |
| dagre | 134 | 708 | 88 |

With a seed, own keeps the arrangement: 18 pairs change sides, against 71 without a seed. The 68 things that still move are pushed by one slot to make room, or shift a few pixels when a layer gets wider. The seed costs own 24 crossings. With a seed, elk goes from 37 crossings to 273 and still moves 143 things. SPIKE-75 decides the cost function. This spike shows only that the two phases can take the seed.

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
- Keep the constraint form of the place phase. The order and the gaps are difference constraints, so a box cannot overlap another box, and a seed is only a wish inside the constraints.
- Keep the shape of the seed: one optional centre for each thing.
- Replace the rank phase with network simplex. The prototype uses longest path with a local correction.
- Replace the choice of the links to reverse. `04-data-pipeline` puts `CRM dump` in the last rank, which stretches its group across the picture.
- Improve the order phase. Item 0122 holds the target.

### Where this answer is weak

- The ten diagrams were written for this spike. The corpus of public flowcharts from SPIKE-17 was not in the repository when the spike ran. The diagrams are in [`spikes/layered-layout/corpus`](../../spikes/layered-layout/corpus). Six have groups and three nest groups. The first one restates `examples/checkout.horse`.
- No diagram has a link label, and no link ends at a group. Items 0123 and 0124 hold those two cases. ELK and dagre place labels today, and own does not.
- The seed measurement is one edit for each diagram. The interactive mode of ELK got one configuration inside the time box. A better configuration may exist.
- The prototype routes links with one track for each source in each gap. Item 0033 replaces it.

### Follow-up items

- 0032 builds rank, order and place on this design. It existed.
- 0033 builds the router. It existed.
- SPIKE-75 and 0076 decide and build the cost of a moved thing. They existed.
- 0122: fewer crossings in the layered layout, measured against a reference.
- 0123: draw a link that starts or ends at a group.
- 0124: reserve room for link labels in the layered layout.

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
