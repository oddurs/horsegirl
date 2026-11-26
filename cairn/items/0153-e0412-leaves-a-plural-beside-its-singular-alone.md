---
id: 153
uid: 7e3e9205-c151-4537-84df-8cc7026aa106
title: E0412 leaves a plural beside its singular alone
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
priority: p1
risk: low
surface: none
area: check
effort: s
pr: '89'
---

## Purpose

Item 120 left plurals alone: `Agents` beside `Agent` still gives E0412 in 3 corpus flowcharts (6 labels). Decide whether a plural is a typo, then change the rule or record why not.

## Approach

Run `python3 corpus/measure.py` before and after. Related: 0120.

## Acceptance criteria

- [x] `Agent` beside `Agents` gives no E0412, or the item records why it does.
- [x] A test holds the case.

## 2026-11-26

Triage: moved to v0.1 at p1, for the same reason as BUG-162: a safe E0412 fix must never merge two names that the author meant as two things. If a plural stays an E0412, its fix must not be safe.

## 2026-11-26

Decision: a plural is not a typo. A name and the same name plus a final s are two things (a group and its members), so E0412 no longer fires and no fix merges them. python3 corpus/measure.py before: 'labels that E0412 would call a typo 6 ... in 3 flowcharts'; after: '5 ... in 2 flowcharts'. The 5 left are 'a','b','c' in 0017 and two Chinese theme names in 0232, which measure.py counts but check.rs does not (neither is written more than once). New test a_plural_beside_its_singular_is_not_a_typo failed without the change and passes with it; cargo test -p horsegirl-core --test check: 51 passed. scripts/task check ran to the end.
