---
id: 32
uid: 72b5f672-35bd-446b-ad4a-01cea1ea5842
title: 'Layered layout: rank, order and place'
type: feature
status: planned
milestone: v0.1
depends_on:
- 18
- 25
- 30
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

## Problem

A flow needs things placed in layers by dependency, with few crossings.

## Proposal

Build the first three phases of the layered solver the spike chose: rank, order, place. Groups stay together. The phases take an optional seed of earlier positions and ignore it for now.

## Acceptance criteria

- [ ] The reference flow places 9 things in ranks that follow the links.
- [ ] No two boxes overlap on any corpus diagram.
- [ ] The same input gives the same positions on every run.

## 2026-11-25

From the review of SPIKE-18: a seed of one optional centre for each thing cannot hold a picture still. The prototype in spikes/layered-layout fails the no-edit run of 'cargo run --release -- seed': 33 of 189 things move when a diagram is seeded with its own picture, each by a multiple of 12 px, the gap beside a bend. The bends have no seed. Give the seed type room for the order of each layer, bends included, or for the position of each bend. Rule for the first version: a layout seeded with its own picture moves nothing. SPIKE-75 decides what the seed carries.

## 2026-11-25

From the second review of SPIKE-18: the criterion 'No two boxes overlap' passes when two boxes touch, so read it with a gap. No two boxes of one rank are closer than the node gap (24 px in the prototype) on any corpus diagram. The prototype in spikes/layered-layout breaks this rule on 4 pairs: 'cargo run --release -- draw' prints touching 1 for 07-compiler (Checks and Lens share the edge y -92) and touching 3 for 10-platform, and 0 for dagre, graphlay and elk. Cause: 'place' in src/own.rs gives a pair of neighbours a gap of 0 when one of the two is a group marker (Kind::Border), and the order phase can put the marker between two things of one rank. Do one of two things: the order phase never puts a marker between two things, or the place phase adds the node gap between neighbouring things across a marker. Smallest input: 'G {' / 'a, c, d, e' / '}' / 'b -> e' / 'd -> e' / 'b -> a' draws e at y -128 to -92 and a at y -92 to -56. Test it with a count of pairs closer than the node gap, not with an overlap count.
