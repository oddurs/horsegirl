---
id: 161
uid: 15975225-8b07-4d53-b401-bda5a35117f6
title: A ? written against the name marks a guess, or says why not
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: s
related:
- 41
---

## Problem

The card says "A final ? marks a guess." `Report job -> Archive ?` is a guess.
`Report job -> Archive?`, with no space, is a firm link to a new thing named
`Archive?`. `check` gives no diagnostic. The hand-written run of the benchmark
(item 41, `bench/runs/hand/054.txt`) shows it: the reply is valid and states a
thing that the author did not mean.

## Proposal

Read a final `?` as the guess mark with or without a space before it. If a name
must end in `?`, it is written in quotes. Or, if the grammar keeps the space,
give a diagnostic with a safe fix that adds the space.

## Acceptance criteria

- [ ] `A -> B?` is a guess, or gives a diagnostic whose safe fix makes it one.
- [ ] `"B?"` in quotes is still a name that ends in `?`.
- [ ] The benchmark report for the hand-written run no longer shows `thing Archive` missing for task 054.
