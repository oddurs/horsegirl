---
id: 24
uid: bc2edf35-ff31-4884-9cb6-bc7a1ddf28ab
title: Diagnostics with stable codes, spans and fixes
type: feature
status: planned
milestone: v0.1
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: diagnostics
effort: m
area: check
track:
- agent-loop
---

## Problem

An error that an agent cannot act on is noise. Every diagnostic needs a stable code, a place, a cause and, where one exists, the exact replacement.

## Proposal

Define the diagnostic type: severity, code (`E`, `W`, `N` plus four digits), line and column span, message, and an optional fix with a confidence of safe or suggested. Render it as text for a terminal and as one JSON object per line.

## Acceptance criteria

- [ ] The JSON shape is documented and held by a snapshot test.
- [ ] The text form shows the source line and marks the span.
- [ ] Every code in the code base appears in a generated catalogue, and a test fails when one is missing.
