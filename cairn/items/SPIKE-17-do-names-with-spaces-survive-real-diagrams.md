---
id: 17
uid: 3c31376a-7080-4be4-b372-e90b7a3963b4
title: Do names with spaces survive real diagrams?
type: spike
status: planned
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: language
track:
- language
---

## Question

Can a grammar with spaces in bare names parse real diagrams without ambiguity?

## Why it matters

The parser, the formatter and the typo check all rest on this rule. If names need quotes often, ADR 'Names have spaces' must change before the parser is written.

## Time box

Three days.

## Answer

Not answered yet.

## Acceptance criteria

- [ ] The Answer section states a conclusion and the evidence for it.
- [ ] Follow-up items exist for the work the answer implies.
- [ ] A corpus of at least 200 public Mermaid flowcharts is in the repository under a licence that allows it.
- [ ] The answer reports the share of names that need quotes, with the five most common causes.
