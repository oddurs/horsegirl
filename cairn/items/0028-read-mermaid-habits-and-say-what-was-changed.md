---
id: 28
uid: 5cd7a3ca-c68f-445c-b662-af77d05b9e3a
title: Read Mermaid habits and say what was changed
type: feature
status: planned
milestone: v0.1
depends_on:
- 23
- 24
created: 2026-11-25
updated: 2026-11-25
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

- [ ] `"Web app" --> "API gateway" : HTTPS;` reads as one link with the label HTTPS.
- [ ] `fix` rewrites that line to `Web app -> API gateway: HTTPS`.

## 2026-11-25

From the review of item 23 (PR #27): the lexer splits '-->' into the word '-' and an arrow. 'Cafe --> Payments' parses with no diagnostic as a link from a thing named 'Cafe -', and 'A <-> B' gives a thing named 'A <'. Until this item lands, the result is a wrong thing and not an error.
