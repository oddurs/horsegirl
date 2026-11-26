---
id: 134
uid: 34868b30-2b2c-421b-9173-54ae199d2cc7
title: Every text that needs quotes gets them in one step
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-26
closed_at: 2026-11-26
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

- [x] `Sensors: wind, B -> Hub` gives E0004 with a fix, and the fixed line parses with no diagnostic.
- [x] `api { note: GET /users/{id}, list }` gives one E0004 whose fix is `"GET /users/{id}, list"`, and the fixed line parses with no diagnostic.
- [x] A text that holds a double quote and gives E0004 gets a message that says to remove the double quote, and no message that says to add quotes. A bare label that holds a double quote and no `{`, `}` or `@` gives no diagnostic.
- [x] The fuzz test asserts that a fixed source has no E0004 at the place of the fix.
- [x] `A -> GET /users/{id}` gives E0004 whose fix is `A -> "GET /users/{id}"`, and the fixed line parses with no diagnostic.
- [x] `G{ A, B } -> C` and `Team@x{ A }` get no fix that puts a group inside quotes, and a test states this.
- [x] The language reference or a comment in the parser states when braces before an arrow are characters of a name and when they are a group, and a test holds one input for each case.

## 2026-11-25

Evidence. horsegirl check --json printed: 'Sensors: wind, B -> Hub' E0004 fix '"Sensors: wind"'; 'api { note: GET /users/{id}, list }' one E0004 fix '"GET /users/{id}, list"'; 'A -> GET /users/{id}' E0004 fix '"GET /users/{id}"' at columns 6-21; 'G{ A, B } -> C' E0001 at the arrow, fix null; 'Team@x{ A }' E0004 fix '"Team@x"' only. tests/syntax.rs asserts each fixed line parses with no diagnostic (a_name_or_a_value_that_needs_quotes_gets_them_from_the_fix, a_group_or_an_open_block_is_never_quoted_into_a_name), a_text_that_holds_a_double_quote_is_told_to_remove_it (message holds 'Remove each double quote', not 'Write the', no fix), braces_before_an_arrow_are_characters_only_around_one_word (A {x} -> B, GET /users/{id} -> B quoted; Services { Orders } -> Postgres and four others E0001 at the arrow). tests/fuzz.rs asserts no E0004 starts inside a fixed text; HORSEGIRL_FUZZ_ROUNDS=400000 cargo test --release --test fuzz printed 'test result: ok. 1 passed' in 700 s. docs/reference.md rule 1 and the doc comment of Parser::extent state the brace rule. scripts/task check: 314 tests run: 314 passed, exit 0.

## 2026-11-26

Fix round 1. The review returned no verdict, so the implementer probed the brace rule again. Team@x {A} and Team@x{A} got the fix '"Team@x {A}"', which lost the part A, although Team {A} is a group. Parser::in_link now limits one-word braces as characters to a line with an arrow in the same block. tests/syntax.rs a_group_or_an_open_block_is_never_quoted_into_a_name holds five such inputs, and each fixed line parses with no diagnostic. scripts/task check printed '314 tests run: 314 passed, 0 skipped', exit 0.

## 2026-11-26

Fix round 2: the review gave no verdict. A probe found that 'a: b{c} -> D' got the fix '"a: b"{c} -> D', which does not parse, because the colon path in the parser did not use the brace rule. Both paths now share name_braces. The fix is '"a: b{c}" -> D'. Test braces_before_an_arrow_are_characters_only_around_one_word failed before the change and passes after it; scripts/task check exited 0.

## 2026-11-26

Fix round 3: the review returned no verdict, so no finding was open. HORSEGIRL_FUZZ_ROUNDS=150000 cargo test --release -p horsegirl-core --test fuzz no_input printed 'test result: ok. 1 passed' in 271 s. A hand probe of 15 inputs with horsegirl fix and then horsegirl check, among them 'A -> GET /users/{id}/{name}', 'A -> B, C@d {e}', 'a: b{c}, d -> E' and 'A@b{c} -> D', left no E0004 and no new error in any fixed line. scripts/task check exited 0.

## 2026-11-26

Fix round (review of PR #72): a bare label holds a double quote, so the E0004 label message no longer says that a label cannot hold one. horsegirl check printed: 'A -> B: x "y"' and 'A -> C: "p" q' 0 errors; 'A -> D: say "hi" {x}' E0004 'holds `{`, `}` or `@`, so the label needs double quotes, and a label in double quotes cannot hold a double quote. Remove each double quote...'; 'A { note: a "b" c }' and 'x "y" -> B' E0004 'Remove each double quote', no fix. Criterion 3 reworded to state the label case. cargo test --test syntax double_quote: 2 passed (a_bare_label_holds_a_double_quote, a_text_that_holds_a_double_quote_is_told_to_remove_it). scripts/task check: 315 tests run: 315 passed, exit 0.
