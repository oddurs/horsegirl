---
id: 25
uid: 2052dca3-f995-47dc-b929-a8357ed35afa
title: Lower the tree into things, links, groups and facts
type: feature
status: planned
milestone: v0.1
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: model
track:
- language
---

## Problem

The parser gives a tree of text. The checks and the lenses need things, links, groups and facts.

## Proposal

Lower the tree into a model: things in order of first mention, links with labels, groups from braces, facts on things and links, the `show` request. A thing exists when it is first mentioned.

## Acceptance criteria

- [ ] The reference file lowers to 9 things, 9 links, 1 group and 5 owners, held by a test.
- [ ] Stating a link twice gives one link.
- [ ] The model never depends on hash-map order: two runs give identical debug output.
