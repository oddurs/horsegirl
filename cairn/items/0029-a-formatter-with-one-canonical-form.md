---
id: 29
uid: 962335df-38e4-43f7-81d8-ee0c4a99d04f
title: A formatter with one canonical form
type: feature
status: planned
milestone: v0.1
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: language
effort: m
area: language
track:
- language
---

## Problem

A repository of files written by many agents drifts into many styles, and diffs become noise.

## Proposal

`horsegirl fmt` prints one canonical form: one fact per line, single spaces around arrows, facts in braces in a fixed order. Comments are kept.

## Acceptance criteria

- [ ] `fmt` of a formatted file changes nothing.
- [ ] `fmt` never changes the model: a property test compares the model before and after on the corpus.
- [ ] `fmt --check` exits 1 when a file would change.
