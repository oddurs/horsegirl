---
id: 27
uid: d469ae36-cdf6-48e4-ad46-814c8f82f985
title: 'The first checks: typos, orphans, broken rules, guesses'
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
depends_on:
- 24
- 25
- 26
created: 2026-11-25
updated: 2026-11-25
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

- [ ] Each check has a test with the file, the expected code and the expected fix.
- [ ] `Paymnts` beside `Payments` gives E0412 with the replacement `Payments`.
- [ ] A clean reference file gives zero diagnostics.

## 2026-11-25

From the review of item 23: `GET /users/{id}` alone on a line parses with no diagnostic as a group `GET /users/` with the part `id`. The parser cannot tell this from a group. A check can: a thing whose name ends in `/` and that has a block of one bare name is likely a path that needs quotes. Give it a warning with the quote fix.

## 2026-11-25

Claimed with --force: the model work of 0026 is merged (#32). Its open criterion is the formatter round trip, which waits on 0029 and does not affect the checks.
