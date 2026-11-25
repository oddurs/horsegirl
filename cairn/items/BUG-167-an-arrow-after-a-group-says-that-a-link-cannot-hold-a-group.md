---
id: 167
uid: c3c7aff8-46d1-4c83-afe2-15c7e875810a
title: An arrow after a group says that a link cannot hold a group
type: bug
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
---

## Problem

`Services { Orders } -> Postgres` and `G{ A, B } -> C` give E0001 "Cannot read `->` here. Write one fact on a line". The message does not say why: a link cannot hold a group. BUG-134 made these lines give E0001 at the arrow instead of a fix that quoted the group into a name.

## Acceptance criteria

- [ ] An arrow after a group gives a message that says a link cannot hold a group, and to write the group on its own line.
- [ ] A test holds `Services { Orders } -> Postgres` and `G{ A, B } -> C` to that message.
