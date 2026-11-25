---
id: 41
uid: e4a34f6a-eec8-4ef3-bc40-dacb183cd647
title: 'The first benchmark: can a model write it from the card?'
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 37
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: high
surface: none
effort: l
area: bench
track:
- agent-loop
- language
---

## Problem

The claim is that a model writes this language correctly from one page. Nothing measures it.

## Proposal

Build the first benchmark: 100 tasks, each a description of a system and a checker. A runner gives a model the spec card and the task, then scores validity on the first try and after one `fix`. The runner takes any model behind a small adapter.

## Acceptance criteria

- [x] The runner scores a recorded transcript with no network access, so CI can run it.
- [x] A report lists the score for each task and the diagnostics that models hit most often.
- [x] The tasks and the checkers are in the repository.

## 2026-11-25

Criterion 1: ran 'unshare -rn sh -c "target/debug/horsegirl-bench score bench/runs/hand | diff - bench/runs/hand/report.md"' in a namespace with only a down loopback; it printed no diff. The golden test horsegirl-bench::bench the_hand_run_matches_its_report runs in scripts/task check: 'Summary 253 tests run: 253 passed, 0 skipped', exit 0.

## 2026-11-25

Criterion 2: bench/runs/hand/report.md, made by horsegirl-bench score, has a row for each of the 100 tasks (first try, after one fix, facts met, first-try codes, first missing fact) and a table of codes by replies hit, times and left after fix. The hand run gives 8/12/9 of 100 and N0101, E0004, W0107, E0008, E0009, E0300, E0412, W0010. The replies are hand-written, not model output.

## 2026-11-25

Criterion 3: bench/tasks holds 100 task files, each a description and a checker in horsegirl. Tests there_are_one_hundred_tasks_with_ids_in_order, every_checker_is_a_valid_file_with_no_lone_thing, every_checker_names_its_things_in_its_description and the_checker_of_each_task_scores_full_marks pass. No real model was run: there is no model access in this run. Follow-ups filed: 161 (a ? against a name is not a guess), 162 (E0412 on Billing v1 / v2).

## Result

bench/tasks holds 100 tasks, each a description and a checker in horsegirl. horsegirl-bench scores replies for validity on the first try, after one fix, and against the checker, and reports the diagnostics hit most often. A model joins through a command adapter; recorded replies score with no network. Only a hand-written run exists; no real model has been scored.
