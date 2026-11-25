---
id: 140
uid: 8ed7f1cd-0b92-47f2-9811-f8c5efd7734d
title: A quote fix for a link target keeps the label outside the quotes
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
related:
- 134
---

## Problem

The safe E0004 fix for a link target that holds `@` puts the label inside the quotes. `A -> B@x: pay` gets the fix `A -> "B@x: pay"`. The fixed line parses with no diagnostic, but the label `pay` is now a part of the name of the target. The fix is marked safe, so `fix` would change the meaning of the file with no question. The source side is right: `A@b -> C: pay` gets `"A@b" -> C: pay`.

The cause is `Parser::extent` for a name: after an arrow, the scan of the target does not stop at the colon that starts the label. Found while item 24 wrote the documented JSON example.

## Acceptance criteria

- [ ] `A -> B@x: pay` gives E0004 whose fix gives `A -> "B@x": pay`, and the fixed line parses as a link with the label `pay`.
- [ ] A test holds a name with `:` inside it on the target side, such as `A -> "B: x"`, and states which form gets which fix.
