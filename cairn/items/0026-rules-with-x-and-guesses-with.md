---
id: 26
uid: fddc7b46-da30-4974-9716-6380d1c2ae0c
title: Rules with -x and guesses with ?
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 25
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] `A -x B: reason` is stored as a rule with its reason.
- [x] `A -> B ?` is stored as a link marked as a guess; a label that ends in a question mark with no space before it is not a guess.
- [x] Both forms survive the formatter unchanged.

## 2026-11-25

Criteria 1 and 2 watched pass: cargo test --test model, tests a_rule_is_stored_with_its_reason, a_final_question_mark_marks_a_guess, a_question_mark_inside_a_label_is_not_a_guess, one_firm_statement_makes_a_guess_firm; scripts/task check exited 0. The parser already read -x and a final ?; the change is in the model (Model.rules, Link.guess).

## 2026-11-25

Criterion 3 not ticked: no formatter exists on main (item 29 is not merged). The tree prints back the exact source, which the syntax tests hold, but that is not the formatter. Item 29 must keep -x and a final ? when it lands.

## 2026-11-25

Criterion 3: crates/horsegirl-core/tests/format.rs holds 'A -x B: reason' and 'A -> B ?' unchanged through the formatter (item 0029, pull request 29). scripts/task check passed on main at c8ebb1c.
