---
id: 120
uid: 110dbedd-ed74-4995-8105-d9d58627683e
title: The typo check leaves sibling names alone
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 27
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 17
---

## Problem

Item 27 makes a name that is used once and is one edit from another name an error, E0412. In the corpus that rule fires on 23 labelled nodes in 10 of 239 flowcharts, and none of them is a typo. Each one is a sibling: `Service A` and `Service B`, `Request A` to `Request E`, `Deploy EC2 #1` and `Deploy EC2 #2`, a plural such as `Agents` beside two nodes called `Agent`, or a one-character name such as `1` and `2`. An error that is wrong this often teaches an agent to ignore it.

## Proposal

E0412 does not fire when the two names differ only in a final word that is a number or a single letter, or when both names are shorter than three characters. Measure the rule again with `python3 corpus/measure.py` after the change.

## Acceptance criteria

- [x] `Service A` beside `Service B` gives no E0412.
- [x] `Paymnts` beside `Payments` still gives E0412 with the replacement `Payments`.
- [x] A test holds the sibling pairs of `corpus/flowcharts/0143.mmd`, `0191.mmd` and `0198.mmd`, and none of them gives E0412.

## 2026-11-25

Watched: cargo test -p horsegirl-core --test check printed 24 passed, 0 failed. With the old check.rs the two new tests (sibling_names_are_not_typos, the_sibling_pairs_of_the_corpus_are_not_typos) failed, 2 failed. The existing test a_name_one_edit_from_a_known_name_is_a_typo_with_a_safe_fix still passes for Paymnts/Payments. python3 corpus/measure.py: E0412 labels fall from 23 in 10 flowcharts to 6 in 3; the rest are plurals, filed as 0153. scripts/task check exit 0.
