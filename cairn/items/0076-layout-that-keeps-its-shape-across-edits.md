---
id: 76
uid: aa72a1ad-8690-4af0-bf31-c7c7744317a6
title: Layout that keeps its shape across edits
type: feature
status: backlog
milestone: v0.3
depends_on:
- 75
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: high
surface: svg
effort: l
area: layout
track:
- pictures
---

## Problem

One new node can rearrange a whole picture. Readers lose their place and picture diffs become noise.

## Proposal

Implement the design from the spike: ordering and placement take earlier positions as a seed and count each moved node as a cost.

## Acceptance criteria

- [ ] Adding one node to a 50-node corpus diagram moves fewer than three other nodes, measured over 100 random single edits.
- [ ] Widening a label moves no node to another rank or order.
- [ ] With no earlier positions the output equals the unseeded layout.
