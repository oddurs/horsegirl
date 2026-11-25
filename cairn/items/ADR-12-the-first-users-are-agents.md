---
id: 12
uid: 6a7f180d-75a1-4c41-9530-69ae43bd847b
title: The first users are agents
type: decision
status: done
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: none
effort: s
area: language
track:
- language
- agent-loop
---

## Context

Most diagrams are now written by an agent and read by a person. Existing tools were designed for a person who writes.

## Options and tradeoffs

People first: the larger audience today, but Mermaid already owns it. Agents first: a smaller audience that has no good tool and chooses on error rate.

## Decision

The first users are agents. The MCP server, the spec card, structured fixes and the read-back ship before anything that only a person uses.

## Revisit when

The benchmark shows that models do not write the language more reliably than they write Mermaid.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
