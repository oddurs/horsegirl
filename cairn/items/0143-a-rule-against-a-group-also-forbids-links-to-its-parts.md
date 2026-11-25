---
id: 143
uid: 29c4374c-2fe6-4f05-84f6-73427d9560ac
title: A rule against a group also forbids links to its parts
type: feature
status: backlog
milestone: later
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

E0500 compares a link with a `-x` rule only when both name the same two things. `Web app -x Services` and then `Web app -> Orders`, where `Services { Orders }`, gives no error. A writer who forbids a link to a group most likely means every part of the group.

## Proposal

A rule whose end is a group also matches a link whose end is a part of that group, at any depth. The message names the group and the rule's line.

## Acceptance criteria

- [ ] `Web app -x Services` with `Services { Orders }` and `Web app -> Orders` gives one E0500 that names `Services`.
- [ ] A rule between two parts does not match a link between their groups.
- [ ] The reference file gives no E0500.
