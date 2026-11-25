---
id: 95
uid: fcd46bc6-584c-4796-a293-0199cce0ec14
title: What 1.0 promises not to break
type: decision
status: backlog
milestone: v1.0
depends_on:
- 93
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: cli
effort: m
area: cli
track:
- release
---

## Context

At 1.0 people build on the output. Each public contract needs a stated promise: the language, the diagnostic codes and JSON, the command line, the MCP tools, the SVG structure and class names, and the theme tokens.

## Options and tradeoffs

Promise everything: simple to state, and it freezes mistakes. Promise by contract: more words, and each promise can be tested.

## Decision

Not decided.

## Revisit when

Before the 1.0 release candidate.

## Acceptance criteria

- [ ] The choice, its evidence and its consequences are recorded.
- [ ] Each promise names the test that holds it.
- [ ] The promise for layout says exactly when a picture may change between versions.
