---
id: 55
uid: 79beeb54-06ff-4612-bce4-9b923a5ef9ef
title: 'Stories: links in the order they happen'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 25
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: model
track:
- language
---

## Problem

A link outside a story is always true. A sequence needs links in the order they happen.

## Proposal

Parse `story <name> { ... }` and keep its links as ordered steps in the model. A story link also counts as a plain link. `show <lens> of <story>` selects one.

## Acceptance criteria

- [ ] The checkout story lowers to seven ordered steps.
- [ ] A lens that needs a story reports E0301 with the fix text when the file has none.
