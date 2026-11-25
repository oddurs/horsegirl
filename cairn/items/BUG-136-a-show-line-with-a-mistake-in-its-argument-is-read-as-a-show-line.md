---
id: 136
uid: 256d4cc6-5bb3-42cb-9db7-edb8181151ba
title: A show line with a mistake in its argument is read as a show line
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: low
surface: diagnostics
area: language
effort: s
related:
- 23
---

## Problem

A line that starts with `show` is a `show` line only when it holds no arrow, comma, colon or brace (`is_show_line` in `crates/horsegirl-core/src/syntax/parser.rs`). A `show` line whose argument needs quotes therefore becomes another kind of line. The reviews of item 23 found these inputs:

1. `show rings from GET /users/{id}` parses with no diagnostic as a thing `show rings from GET /users/` with the part `id`.
2. `show rings from Sensors: wind` gives E0009 "A fact goes inside braces" and no fix.
3. `show sequence of a@b` gives E0008 with no fix, and the message does not name quotes.
4. `show sequence of S, T` parses with no diagnostic as two things, `show sequence of S` and `T`.

## Acceptance criteria

- [x] A line that starts with `show`, a lens word and `of`, `from` or `by` is read as a `show` line with a mistake, and never as a thing or a group.
- [x] `show rings from GET /users/{id}`, `show rings from Sensors: wind` and `show sequence of a@b` each give E0004 whose fix puts the argument in double quotes, and each fixed line parses as a `Show` node with no diagnostic.
- [x] `show sequence of S, T` gives an error that says a `show` line takes one name.
- [x] `show, A -> B` and `show { owner: X }` still read `show` as a word of a name.
- [x] The fuzz test passes with these inputs in `FORMS`.

## 2026-11-25

Test a_show_line_with_a_mistake_in_its_argument_is_still_a_show_line in crates/horsegirl-core/tests/syntax.rs: the three quote cases each give one E0004 whose fix gives show rings from "GET /users/{id}", show rings from "Sensors: wind" and show sequence of "a@b"; each fixed line parses to one Show node with no diagnostic. show sequence of S, T gives one E0008 whose message says a show line takes one name, with no fix. show, A -> B and show { owner: X } give no diagnostic and a Name "show". cargo test --test syntax printed 31 passed; scripts/task check exited 0 with the fuzz test (forms now hold the six lines) passing. The lens must be in check::LENSES, so show room of Mirrors -> Hall stays a link.

## 2026-11-25

Added show flow of A -> B (E0004) and show lanes by X { a, b } (E0008): each parses to one Error node and no Things or Link node. scripts/task check exited 0 after the change.
