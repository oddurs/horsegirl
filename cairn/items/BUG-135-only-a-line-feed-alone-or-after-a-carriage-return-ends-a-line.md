---
id: 135
uid: 6e374b40-4507-422e-8dbc-b7094a16e585
title: Only a line feed, alone or after a carriage return, ends a line
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
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

- [ ] A test holds what the parser does with a lone `\r`, U+2028 and U+0085.
- [ ] The language reference states which characters end a line.
