---
id: 165
uid: 1af8dd74-63a0-4bd8-b4a6-da33180c3ea1
title: corpus/convert.py gives two Mermaid nodes that share a label two names
type: chore
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: interop
related:
- 121
- 139
---

## Problem

`corpus/convert.py` uses a Mermaid label as the horsegirl name. Two Mermaid nodes with one label become one thing. When the two nodes are in two subgraphs, the converted file writes that thing into two groups, and `check` gives E0420 (item 139). Eleven corpus files give E0420 for this reason. Two of them, `0031.horse` and `0207.horse`, drew clean before item 139, so the corpus baseline fell from 176 to 174. In `0207.mmd`, `SNA`, `SNB` and `SNC` are three subnets that each have the label `Private subnet<br/>...`; the converted file has one `Private subnet`. In `0031.mmd`, a subgraph `Metrics` holds a node labelled `Metrics`, and the file writes `Metrics { Metrics }`.

## Approach

Follow the rule that item 121 proposes for `horsegirl convert`: the second node with a shared label gets its Mermaid id after the name. A node with the same name as its subgraph gets its id too. Regenerate `corpus/horse` and raise `BASELINE` in `crates/horsegirl/examples/corpus.rs` to the new count.

## Acceptance criteria

- [ ] No file in `corpus/horse` gives E0420.
- [ ] No two Mermaid nodes of one flowchart in `corpus/flowcharts` become one thing in `corpus/horse`.
- [ ] `cargo run -p horsegirl --example corpus` counts at least 176 clean files, and `BASELINE` holds the new count.
