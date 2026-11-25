---
id: 136
uid: 256d4cc6-5bb3-42cb-9db7-edb8181151ba
title: A show line with a mistake in its argument is read as a show line
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
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

- [ ] A line that starts with `show`, a lens word and `of`, `from` or `by` is read as a `show` line with a mistake, and never as a thing or a group.
- [ ] `show rings from GET /users/{id}`, `show rings from Sensors: wind` and `show sequence of a@b` each give E0004 whose fix puts the argument in double quotes, and each fixed line parses as a `Show` node with no diagnostic.
- [ ] `show sequence of S, T` gives an error that says a `show` line takes one name.
- [ ] `show, A -> B` and `show { owner: X }` still read `show` as a word of a name.
- [ ] The fuzz test passes with these inputs in `FORMS`.
