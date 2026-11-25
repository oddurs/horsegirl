---
id: 144
uid: 5ff07c1a-3c6b-4ed8-bf23-af96a41ef423
title: A path that the parser reads as a group gets a warning with the quote fix
type: feature
status: backlog
milestone: v0.1
depends_on:
- 27
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
---

## Problem

`GET /users/{id}` alone on a line parses with no diagnostic as a group `GET /users/` with the part `id`. The parser cannot tell this from a group. The writer meant one name.

## Proposal

A check warns on a thing whose name ends in `/` and whose braces hold one bare name and nothing else. The fix writes the whole text in double quotes: `"GET /users/{id}"`.

## Acceptance criteria

- [ ] `GET /users/{id}` gives one warning with a stable code at the line, with the fix `"GET /users/{id}"`.
- [ ] `Services { Orders }` gives no such warning.
- [ ] The reference file gives no such warning.
