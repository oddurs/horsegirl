---
id: 178
uid: f773b63c-7782-407c-ac6e-81b32381bc36
title: Three or more loops of one node draw no line through the label of another loop
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: none
area: layout
---

### Problem

Since item 0169, the loops of a node stand in columns at the right side of its box, two in each column. From the third loop on, the level lines of a loop in an outer column pass through the labels of the loops in the nearer column. The overlap detector does not report this: both shapes belong to the same two ends, so `links_on_labels` skips them. The box has 18 px above the middle, which is too little for two rows of labels.

A file cannot state two links from one thing to itself today, so only a graph built by another lens or a generator reaches this.

### Done when

1. A node with three labelled loops draws no loop line through the label of another loop.
2. The overlap detector reports a link that crosses the label of another link with the same two ends, and the corpus and generated tests pass with it.
