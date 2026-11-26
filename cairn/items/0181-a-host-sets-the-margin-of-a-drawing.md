---
id: 181
uid: ef11a454-5196-488e-9211-fddc3c831cf1
title: A host sets the margin of a drawing
type: feature
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: cli
area: cli
effort: s
---

## Problem

The margin round a drawing is the constant `theme::MARGIN`. The CLI and MCP cannot change it. Refs: BUG-179.

## Acceptance criteria

- [ ] The CLI and the MCP server accept a margin and pass it to `svg::draw_with_margin`.
- [ ] A test draws one case with a custom margin and checks the view box.
