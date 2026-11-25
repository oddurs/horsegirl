---
id: 130
uid: a2c4b80e-00f3-4fd7-b5ca-801c04d2a966
title: A comment line is stated on the language card or leaves the language
type: decision
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
---

## Context

The parser of item 23 reads a line that starts with `#` as a comment, after any indent. It keeps the comment in the tree. The reader on the concept page does the same, and item 29 says that the formatter keeps comments. The language card in `crates/horsegirl-core/src/spec.txt` does not state the form. An agent that reads only the card does not know that a comment exists, and a name such as `#1 priority` at the start of a line is read as a comment without a diagnostic.

A `#` anywhere else on a line is a character of a name: `Deploy EC2 #1 -> Host` has no comment.

## Options and tradeoffs

- State the form on the card in one line. The card grows, and the card must stay on one page.
- Keep the form and leave it off the card. The card stays short, and a name that starts with `#` is lost in silence.
- Remove comments from the language. A file is facts only, and a person who wants a remark has no place for it.
- Keep the form, and add a diagnostic for a comment line that looks like a name with a link or braces.

## Decision

Not decided.

## Revisit when

The benchmark (item 41) shows that agents write a comment form that the parser does not read, such as `//`.

## Acceptance criteria

- [ ] The choice, its evidence and its consequences are recorded.
- [ ] If the choice changes the language card, the card states the comment form in one line and still holds fewer than 200 words.
- [ ] If the choice removes or changes the form, the parser and its tests follow in the same pull request or in a follow-up item.
