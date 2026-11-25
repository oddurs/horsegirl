---
id: 134
uid: 34868b30-2b2c-421b-9173-54ae199d2cc7
title: Every text that needs quotes gets them in one step
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: m
related:
- 23
- 119
---

## Problem

ADR-119 says that a name or a value that needs quotes and lacks them is corrected in one step. The reviews of item 23 found inputs where the parser does not do that.

1. `Sensors: wind, B -> Hub` gives E0009 "A fact goes inside braces" and no fix. `A, Sensors: wind -> Hub` gives E0004 with the fix. `colon_outside_braces` stops its scan at the first comma.
2. `api { note: GET /users/{id}, list }` gets the fix `"GET /users/{id}", list`. The result gives E0004 again, now without a fix. `A { note: install, list {x} }` does the same.
3. `A { note: a "b" c }` gives E0004 "Write the value in double quotes" without a fix. A quoted value cannot hold a double quote, so the message asks for a form that the language does not have.

## Acceptance criteria

- [ ] `Sensors: wind, B -> Hub` gives E0004 with a fix, and the fixed line parses with no diagnostic.
- [ ] `api { note: GET /users/{id}, list }` gives one E0004 whose fix is `"GET /users/{id}, list"`, and the fixed line parses with no diagnostic.
- [ ] A text that holds a double quote gives a message that says to remove the double quote, and no message that says to add quotes.
- [ ] The fuzz test asserts that a fixed source has no E0004 at the place of the fix.
