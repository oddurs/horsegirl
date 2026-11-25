---
id: 27
uid: d469ae36-cdf6-48e4-ad46-814c8f82f985
title: 'The first checks: typos, orphans, broken rules, guesses'
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 24
- 25
- 26
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: diagnostics
effort: m
area: check
track:
- agent-loop
---

## Problem

Things exist on first mention, so a typo makes a new thing silently.

## Proposal

Write the first checks: a name used once that is one edit from another name is error E0412 with a safe fix; a thing with no links is warning W0107; a link that breaks a `-x` rule is error E0500; a guess is note N0200; an unknown lens is error E0300 that lists the lenses.

## Acceptance criteria

- [x] Each check has a test with the file, the expected code and the expected fix.
- [x] `Paymnts` beside `Payments` gives E0412 with the replacement `Payments`.
- [x] A clean reference file gives zero diagnostics.

## 2026-11-25

From the review of item 23: `GET /users/{id}` alone on a line parses with no diagnostic as a group `GET /users/` with the part `id`. The parser cannot tell this from a group. A check can: a thing whose name ends in `/` and that has a block of one bare name is likely a path that needs quotes. Give it a warning with the quote fix.

## 2026-11-25

Claimed with --force: the model work of 0026 is merged (#32). Its open criterion is the formatter round trip, which waits on 0029 and does not affect the checks.

## 2026-11-25

Criteria 1-3 watched pass: cargo test --test check printed 20 passed, 0 failed. Criterion 1: each code has a test with its source, its codes and its fix or None (E0412 a_name_one_edit..., two_letters_swapped..., the_fix_writes_the_name_as_the_file_writes_it; W0107 a_thing_with_no_link_is_a_warning; E0500 a_link_that_breaks_a_rule_is_an_error; N0200 a_guess_is_a_note; E0300 an_unknown_lens... and a_lens_one_edit..., which applies the fix). Criterion 2: a_name_one_edit_from_a_known_name_is_a_typo_with_a_safe_fix gives E0412 on Paymnts with a safe edit Payments, and the fixed file checks clean. Criterion 3: check::of_model on examples/checkout.horse returns []; check::check returns only the parser's W0010 (stories arrive in v0.2, item 55). scripts/task check exit 0: nextest 96 tests run, 96 passed, and the wasm32-wasip1 core tests passed.

## 2026-11-25

Choices: E0412 fires only when the near name is written more than once, so the fix has a direction; a swap of two neighbours is one edit. W0107 counts a -x rule as a link and skips groups and things already reported as E0412. E0500 matches the same two things in the same direction only (group rules: 0143). Lenses are the six on the card. Follow-ups filed: 0143, 0144 (path parsed as a group, from the earlier note).
