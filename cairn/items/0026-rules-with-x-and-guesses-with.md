---
id: 26
uid: fddc7b46-da30-4974-9716-6380d1c2ae0c
title: Rules with -x and guesses with ?
type: feature
status: planned
milestone: v0.1
depends_on:
- 25
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: language
effort: s
area: model
track:
- language
---

## Problem

An agent must be able to say that a link must not exist, and that a fact is a guess.

## Proposal

Add the crossed arrow `-x` as a rule and a final `?` as a guess, in the parser and the model.

## Acceptance criteria

- [ ] `A -x B: reason` is stored as a rule with its reason.
- [ ] `A -> B ?` is stored as a link marked as a guess; a label that ends in a question mark with no space before it is not a guess.
- [ ] Both forms survive the formatter unchanged.
