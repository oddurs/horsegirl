---
id: 13
uid: ac86a5c2-5a6d-4126-8c54-ff475eaa6227
title: Names have spaces and no separate ids
type: decision
status: done
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: language
effort: s
area: language
track:
- language
---

## Context

A Mermaid node needs an id and a label. Agents confuse the two, and the ids carry no meaning.

## Options and tradeoffs

Names with spaces and no ids: shorter and readable, but the grammar must forbid arrows, colons, commas and braces inside a bare name. Ids with labels: safe to parse, worse to write.

## Decision

A name is the thing. Names may contain spaces. A name that appears once and is one letter from another name is an error.

## Revisit when

The grammar spike finds real diagrams that the rule cannot express without quotes in more than a few percent of names.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
