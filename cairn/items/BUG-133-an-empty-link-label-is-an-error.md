---
id: 133
uid: 884b644c-1743-4165-84c4-b7fd430f7c33
title: An empty link label is an error
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: s
related:
- 23
---

## Problem

`A -> B:` parses with no diagnostic. The tree holds the colon and no `Label`. `A { owner: }` gives E0006 for the same mistake in a fact. `A -> B: ?` is a link with a guess mark and no label.

## Proposal

Report E0006 when nothing follows the colon of a link.

## Acceptance criteria

- [x] `A -> B:` gives E0006 with a message that says to write the label or to remove the colon.
- [x] `A -> B: ?` gives the same error.
- [x] `A -> B: save` gives no diagnostic.

## 2026-11-25

scripts/task check passed. New tests in crates/horsegirl-core/tests/syntax.rs: 'A -> B:' and 'A -> B: ?' give E0006 with 'remove the colon'; 'A -> B: save' gives no diagnostic. Format test case 'A -> B  :' changed to 'A -> B  : save' because the old input is now an error.
