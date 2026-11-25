---
id: 130
uid: a2c4b80e-00f3-4fd7-b5ca-801c04d2a966
title: A comment line is stated on the language card or leaves the language
type: decision
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
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

Decided by the owner: keep the form and state it on the card. A line that starts with `#`, after any indent, is a comment. The card states the form in one line: `A line that starts with # is a comment.`

Evidence:

- The parser of item 23, the reader on the concept page and the formatter of item 29 already read and keep comments in this form. No code changes.
- The card grows from 139 to 148 words. The test `the_card_fits_in_a_prompt` holds it under 200.
- The formatter already quotes a name that starts with `#` (ADR-119), so `"#1 priority" -> Plan` is the written form of such a name.

Consequences:

- The parser and its tests do not change. The test `the_comment_form_is_on_the_language_card_and_reads_as_written` in `crates/horsegirl-core/tests/check.rs` holds the card line and the parser behaviour together.
- A bare name at the start of a line that starts with `#` is still read as a comment without a diagnostic. An agent that reads the card now knows the rule. A diagnostic for a comment that looks like a link or a block (the fourth option) is not part of this decision.
- `//` is not a comment. It stays a character of a name.

## Revisit when

The benchmark (item 41) shows that agents write a comment form that the parser does not read, such as `//`.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
- [x] If the choice changes the language card, the card states the comment form in one line and still holds fewer than 200 words.
- [x] If the choice removes or changes the form, the parser and its tests follow in the same pull request or in a follow-up item.

## 2026-11-25

Decision recorded in the item: keep '#' comment lines and state them on the card. Evidence: scripts/task check exit 0 in the worktree; nextest printed PASS for horsegirl-core::check the_comment_form_is_on_the_language_card_and_reads_as_written, tests::the_card_fits_in_a_prompt, tests::the_card_has_five_rules and horsegirl::cli spec_prints_the_card. wc -w spec.txt printed 148 (was 139). The form is unchanged, so the parser and its tests do not change; criterion 3 holds with no follow-up.
