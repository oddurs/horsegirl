---
id: 68
uid: 98495692-f8fd-4ae6-a9cb-243279065c6b
title: The playground on the website
type: feature
status: backlog
milestone: v0.2
depends_on:
- 45
- 57
- 58
- 67
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: l
area: site
track:
- site
---

## Problem

The concept page has a toy editor. The site needs the real one.

## Proposal

Build the playground page: an editor with highlighting and inline diagnostics, the lens tabs, the theme menu, the check panel and the read-back panel, all on the WebAssembly build. The source lives in the URL hash.

## Acceptance criteria

- [ ] Every keystroke redraws in under 50 milliseconds for the reference file on a mid-range laptop.
- [ ] A link to a playground state reopens that state.
- [ ] The page works with a keyboard alone.
