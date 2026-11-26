---
id: 171
uid: 6edaeea5-b8e5-4960-b2e6-b0600e75821f
title: W0107 leaves alone a thing whose group has a link
type: bug
status: backlog
milestone: v0.1
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: diagnostics
area: check
---

## Problem

Since item 0123 the flow lens draws a link to a group. In `API gateway -> Services` with `Services { Orders, Payments }`, `Orders` and `Payments` get W0107: "`Orders` has no link, so it stands alone in every picture." That is false: the picture draws each of them inside a box that a link reaches. The example in `docs/lenses.md` gives these two warnings.

## Acceptance criteria

- [ ] A thing inside a group, at any depth, that a link or a rule names gives no W0107.
- [ ] A thing in a group that no link or rule names, as in `Services { Orders, Spare }` with `Orders -> Postgres`, still gives W0107 for `Spare`.
- [ ] The W0107 row in `docs/diagnostics.md` states the rule.
