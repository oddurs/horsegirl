---
id: 28
uid: 5cd7a3ca-c68f-445c-b662-af77d05b9e3a
title: Read Mermaid habits and say what was changed
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 23
- 24
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: low
surface: language
effort: s
area: check
track:
- language
- agent-loop
---

## Problem

Models carry habits from Mermaid: `-->`, quoted names, a semicolon at the end of a line.

## Proposal

The reader accepts these forms, treats them as the canonical form, and emits note N0101 with a safe fix that rewrites the line.

## Acceptance criteria

- [x] `"Web app" --> "API gateway" : HTTPS;` reads as one link with the label HTTPS.
- [x] `fix` rewrites that line to `Web app -> API gateway: HTTPS`.

## 2026-11-25

From the review of item 23 (PR #27): the lexer splits '-->' into the word '-' and an arrow. 'Cafe --> Payments' parses with no diagnostic as a link from a thing named 'Cafe -', and 'A <-> B' gives a thing named 'A <'. Until this item lands, the result is a wrong thing and not an error.

## 2026-11-25

Ran cargo test -p horsegirl-core --test syntax: 27 passed, including a_mermaid_link_reads_as_one_link_with_its_label (things "Web app" and "API gateway", one link, labels ["HTTPS"]).

## 2026-11-25

The same run passed a_mermaid_link_gets_one_note_whose_fix_rewrites_the_line: one N0101 note, safe fix, Fix::apply gives "Web app -> API gateway: HTTPS". The fix command does not exist yet, so the test applies the fix through the library. scripts/task check exit 0.
