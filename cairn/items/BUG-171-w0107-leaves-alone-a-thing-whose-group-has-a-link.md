---
id: 171
uid: 6edaeea5-b8e5-4960-b2e6-b0600e75821f
title: W0107 leaves alone a thing whose group has a link
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-26
updated: 2026-11-26
closed_at: 2026-11-26
priority: p2
risk: low
surface: diagnostics
area: check
---

## Problem

Since item 0123 the flow lens draws a link to a group. In `API gateway -> Services` with `Services { Orders, Payments }`, `Orders` and `Payments` get W0107: "`Orders` has no link, so it stands alone in every picture." That is false: the picture draws each of them inside a box that a link reaches. The example in `docs/lenses.md` gives these two warnings.

## Acceptance criteria

- [x] A thing inside a group, at any depth, that a link or a rule names gives no W0107.
- [x] A thing in a group that no link or rule names, as in `Services { Orders, Spare }` with `Orders -> Postgres`, still gives W0107 for `Spare`.
- [x] The W0107 row in `docs/diagnostics.md` states the rule.

## 2026-11-26

Watched pass: cargo test -p horsegirl-core --test check printed 41 passed, 0 failed, including a_thing_in_a_linked_group_is_not_alone_at_any_depth (group link, depth two, rule on group) and a_group_needs_no_link_of_its_own_and_its_parts_do (Spare still W0107). scripts/task check passed. W0107 text in codes.rs, docs/diagnostics.md and bench/runs/hand/report.md now states the rule.

## 2026-11-26

Round 1: ancestor walk in orphans now skips links W0110 reports. Test a_link_that_w0110_reports_does_not_reach_the_parts_of_its_group passes; scripts/task check passes.
