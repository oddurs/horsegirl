---
id: 144
uid: 5ff07c1a-3c6b-4ed8-bf23-af96a41ef423
title: A path that the parser reads as a group gets a warning with the quote fix
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
---

## Problem

`GET /users/{id}` alone on a line parses with no diagnostic as a group `GET /users/` with the part `id`. The parser cannot tell this from a group. The writer meant one name.

## Proposal

A check warns on a thing whose name ends in `/` and whose braces hold one bare name and nothing else. The fix writes the whole text in double quotes: `"GET /users/{id}"`.

## Acceptance criteria

- [x] `GET /users/{id}` gives one warning with a stable code at the line, with the fix `"GET /users/{id}"`.
- [x] `Services { Orders }` gives no such warning.
- [x] The reference file gives no such warning.

## 2026-11-25

Added W0108 in check.rs (reads the tree). cargo test -p horsegirl-core --test check printed 24 passed, 0 failed, including a_path_read_as_a_group_gets_a_warning_with_the_quote_fix (one W0108 at the line, suggested fix "GET /users/{id}", fix applied removes it), a_group_with_a_name_that_does_not_end_in_a_slash_gives_no_path_warning (Services { Orders } and 5 other forms give none), and the_reference_file_gives_no_diagnostic_from_the_checks (only W0010). scripts/task check exit 0.
