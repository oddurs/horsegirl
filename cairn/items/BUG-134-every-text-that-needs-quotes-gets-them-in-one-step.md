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
4. `A -> GET /users/{id}` gives E0001 and no fix. `A, GET /users/{id} -x B` gives E0004 with the fix, so the same name is corrected on one side of the arrow only.
5. `G{ A, B } -> C` and `Team@x{ A }` get a fix that puts the group inside the quotes: a brace with no space before it is read as a character of the name. The parts of the group are lost.
6. `Services { Orders } -> Postgres` gets the quote fix `"Services { Orders }"`, and `Services { Orders, Payments } -> Postgres` gets E0001 with no fix. The rule that a one-part group before an arrow is quoted and a larger group is not is written nowhere.

## Acceptance criteria

- [ ] `Sensors: wind, B -> Hub` gives E0004 with a fix, and the fixed line parses with no diagnostic.
- [ ] `api { note: GET /users/{id}, list }` gives one E0004 whose fix is `"GET /users/{id}, list"`, and the fixed line parses with no diagnostic.
- [ ] A text that holds a double quote gives a message that says to remove the double quote, and no message that says to add quotes.
- [ ] The fuzz test asserts that a fixed source has no E0004 at the place of the fix.
- [ ] `A -> GET /users/{id}` gives E0004 whose fix is `A -> "GET /users/{id}"`, and the fixed line parses with no diagnostic.
- [ ] `G{ A, B } -> C` and `Team@x{ A }` get no fix that puts a group inside quotes, and a test states this.
- [ ] The language reference or a comment in the parser states when braces before an arrow are characters of a name and when they are a group, and a test holds one input for each case.
