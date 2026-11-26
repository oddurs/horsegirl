---
id: 175
uid: 1086052a-4aa3-4154-857f-2955ba7da604
title: A ? written against the last word of a label marks a guess, or says why not
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: none
area: check
related:
- 161
---

## Problem

BUG-161 gave W0111 for `A -> B?`, where a `?` against the last name reads as
part of the name. A label has the same trap: `A -> B: maybe?` is a firm link
with the label `maybe?`, and `check` says nothing. The writer may mean
`A -> B: maybe ?`, a guess. A Mermaid label such as `valid?` is common and is
no guess, so a safe fix is wrong here.

## Proposal

Measure how often a label ends in `?` in the corpus and in benchmark replies.
If writers mean a guess, give a warning with a suggested fix that adds the
space. A label that must end in `?` stays as it is or is written in quotes.

## Acceptance criteria

- [ ] The count of labels that end in `?` in `corpus/horse` and in the benchmark replies is recorded in a note.
- [ ] `A -> B: maybe?` gives a warning with a suggested fix, or a note on this item says why not.
- [ ] `A -> B: "maybe?"` gives no warning.
