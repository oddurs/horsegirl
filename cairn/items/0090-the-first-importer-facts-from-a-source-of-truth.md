---
id: 90
uid: e42b5ffd-0795-4a5c-bcf9-cdaf07ada21c
title: 'The first importer: facts from a source of truth'
type: feature
status: backlog
milestone: v0.3
depends_on:
- 89
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: high
surface: cli
effort: l
area: interop
track:
- language
---

## Problem

Facts typed by hand go stale. A running system already knows its own shape.

## Proposal

Build the importer the spike chose. Each imported fact cites its source as evidence. Where the source has ordered calls, the importer writes a story.

## Acceptance criteria

- [ ] The sample from the spike imports into a file that passes `check`.
- [ ] Running the importer twice changes nothing.
