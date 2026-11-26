---
id: 135
uid: 6e374b40-4507-422e-8dbc-b7094a16e585
title: Only a line feed, alone or after a carriage return, ends a line
type: bug
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-26
created: 2026-11-25
updated: 2026-11-26
priority: p2
risk: low
surface: language
area: language
effort: s
related:
- 23
- 24
---

## Problem

The lexer of item 23 reads a lone carriage return, U+2028 and U+0085 as spaces. `A\rB -> C\rD -> E\n` is one line and gives "this `->` is a second one". Item 24 computes a line and a column for each diagnostic, so the rule for a line end must be stated and held by a test before then.

## Proposal

State the rule in the language reference (item 43): only `\n` and `\r\n` end a line. Hold it with a test. If a lone `\r` must end a line instead, that is a change to the language and needs a decision item.

## Acceptance criteria

- [x] A test holds what the parser does with a lone `\r`, U+2028 and U+0085.
- [x] The language reference states which characters end a line.

## 2026-11-26

Ran: cargo test -p horsegirl-core --test syntax only_a_line printed 1 passed (lone CR, U+2028, U+0085 each give E0010, not a line end; CRLF parses clean). scripts/task check passed. docs/reference.md states the rule; spec.txt is unchanged, so no decision item.
