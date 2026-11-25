---
id: 141
uid: 3499bdb9-6ea1-4f6e-830c-62cac1ecba1c
title: A group box is wide enough for its name
type: feature
status: backlog
milestone: v0.1
depends_on:
- 32
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
---

## Problem

The layered layout gives a group box the width of the ranks that the group spans, plus a pad on each side. It does not read the name of the group. `Customer data warehouse { A }` gives a box that is about 50 px wide, and the name is about 170 px wide. The name then leaves the box and can cover a box beside it.

## Proposal

Give `horsegirl_core::layout::Group` the width of its name. The layout widens the first rank of the group, or the box of the group, until the name fits in the title row. The gap rules of the layout still hold: no box of another group and no node of another group is closer to the group box than the node gap.

## Acceptance criteria

- [ ] A group with one narrow node and a name of 30 characters has a box at least as wide as the name plus the pad on each side.
- [ ] No two boxes are closer than the node gap on the ten diagrams in `spikes/layered-layout/corpus` when each group has its name width.
- [ ] A group whose name is narrower than its box has the same position as before.

## 2026-11-25

From item 0040: corpus/horse/0050.horse is the one corpus file with Kind::LabelOnGroup. The name of the nested group 'Data Tier - Encrypted (CM-2)' lies across the border of its parent group 'VPC: fedramp-complex-vpc (10.30.0.0/16)'. Run 'cargo run -p horsegirl --example corpus' to see the count.
