---
id: 161
uid: 15975225-8b07-4d53-b401-bda5a35117f6
title: A ? written against the name marks a guess, or says why not
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
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

- [x] `A -> B?` is a guess, or gives a diagnostic whose safe fix makes it one.
- [x] `"B?"` in quotes is still a name that ends in `?`.
- [x] The benchmark report for the hand-written run no longer shows `thing Archive` missing for task 054.

## 2026-11-26

Chose the diagnostic, not a grammar change: the card stays as it is. W0111 is a warning on a bare name that ends a link or a rule and ends in ?, with a safe fix that adds the space. Quotes stay on a name that ends in ? (parser habit, fmt, corpus/convert.py), so a decision node such as "Trivial?" never becomes a guess. Evidence for 1: horsegirl check on 'Report job -> Archive?' prints warning[W0111] with 'safe fix: Write `Archive ?` to mark a guess.'; test a_question_mark_against_the_last_name_gets_a_fix_that_makes_a_guess passes and fails without the change.

## 2026-11-26

Evidence for 2: test a_name_that_ends_in_a_question_mark_in_quotes_stays_a_name passes: A -> "Archive?" gives no diagnostic, fix_all leaves it, and the model has the name Archive? with guess false. format test keeps "B?" quoted. Evidence for 3: horsegirl-bench score bench/runs/hand now shows '| 054 | One link is a guess | valid | valid | 5/5 | W0111 |  |' (was 3/5 with thing Archive missing); totals 10 of 100 with every expected fact. scripts/task check exits 0.
