---
id: 15
uid: aa3cdd8d-3ad2-4ff5-a201-a82eb673114a
title: Diagrams are plain by default, and colour follows meaning
type: decision
status: done
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: theme
effort: s
area: render
track:
- pictures
---

## Context

A diagram sits inside other people's documents. A branded default makes every embed look foreign.

## Options and tradeoffs

A branded default theme: recognisable, but clashes with every host. A plain default that follows the host: invisible, and correct in light and dark mode.

## Decision

The default theme is plain and follows the page around it. Colour binds to a fact such as owner. The language has no syntax to paint one box; a stylesheet can.

## Revisit when

Users repeatedly need a single highlighted box and the rule-based emphasis does not cover it.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
