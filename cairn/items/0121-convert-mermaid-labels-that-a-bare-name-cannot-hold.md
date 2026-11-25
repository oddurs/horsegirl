---
id: 121
uid: 2a3ad3e6-a4b6-4ca6-9b69-35abc3224785
title: Convert Mermaid labels that a bare name cannot hold
type: feature
status: backlog
milestone: v0.2
depends_on:
- 64
- 119
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: cli
area: interop
effort: m
related:
- 17
- 40
---

## Problem

SPIKE-17 found three kinds of Mermaid label that do not map to a bare horsegirl name. 169 of 3052 names are a title and a description joined by a line break. 93 nodes in 14 of 239 flowcharts share a label with another node of the same flowchart, such as `true`, `false` and `Description`; Mermaid keeps them apart by id, and names as identity would merge them. 10.5% of names hold a colon, a comma or another reserved character.

## Proposal

`convert` keeps the first line of a label as the name and writes the other lines as a `note` fact on the thing. Two nodes that share a label get distinct names, the second with its Mermaid id after it, and a warning that names both lines. A name or a fact value that still holds a reserved character is written in the quoted form that ADR-119 decides. SPIKE-17 found that 130 of the 133 names freed by the first-line rule hold a comma, a colon or a brace in the other lines.

## Acceptance criteria

- [ ] `api["api<br/><sub>django :8000</sub>"]` converts to the thing `api` with one `note` fact whose value is `django :8000`, written in the form ADR-119 decides, and the output parses back to that one fact.
- [ ] `wally["wally<br/>install, list"]` converts to the thing `wally` with one `note` fact whose value is `install, list`, and the output parses back to that one fact.
- [ ] Two nodes labelled `true` in one flowchart convert to two things, and `convert` prints one warning that names both source lines.
- [ ] Every flowchart in `corpus/flowcharts` converts, and no two Mermaid nodes of one flowchart become one thing.
