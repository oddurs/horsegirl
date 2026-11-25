---
id: 131
uid: 6636a6d9-1c8b-484a-a66e-c073482f1698
title: A chain of arrows on one line gets a fix that splits it
type: feature
status: backlog
milestone: v0.1
depends_on:
- 24
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 23
---

## Problem

An agent that knows Mermaid or D2 writes `A -> B -> C`. The language card allows one arrow on a line. The parser of item 23 reports E0001 at the second arrow and says to write each link on its own line. The diagnostic carries no fix, so the agent must rewrite the line by hand.

## Proposal

Give the second arrow its own code. Carry a fix that rewrites `A -> B -> C` to `A -> B` and `B -> C` on two lines with the same indent. When the line has a label, a `?` or a citation, the fix is suggested and not safe, because the parser cannot know which link the label belongs to.

## Acceptance criteria

- [ ] `A -> B -> C` gives one diagnostic with its own code, and its fix gives the lines `A -> B` and `B -> C`.
- [ ] `A -> B -> C: label` gives the same diagnostic with a fix that is marked as suggested.
- [ ] A name in quotes that holds `->` gives no diagnostic.

## 2026-11-25

From the review of item 23 (PR #27): 'A -> B: pay -> C: ship' parses clean as one link with the label 'pay -> C: ship'. This item should cover that chain too.
