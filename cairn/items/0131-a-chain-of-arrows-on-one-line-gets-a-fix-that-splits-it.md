---
id: 131
uid: 6636a6d9-1c8b-484a-a66e-c073482f1698
title: A chain of arrows on one line gets a fix that splits it
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 24
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] `A -> B -> C` gives one diagnostic with its own code, and its fix gives the lines `A -> B` and `B -> C`.
- [x] `A -> B -> C: label` gives the same diagnostic with a fix that is marked as suggested.
- [x] A name in quotes that holds `->` gives no diagnostic.

## 2026-11-25

From the review of item 23 (PR #27): 'A -> B: pay -> C: ship' parses clean as one link with the label 'pay -> C: ship'. This item should cover that chain too.

## 2026-11-25

Ran cargo test -p horsegirl-core --test syntax -- chain name_in_quotes_that_holds_an_arrow: 4 passed. a_chain_of_arrows_gets_a_safe_fix_that_splits_it: 'A -> B -> C' gives one E0010 with a safe fix that gives 'A -> B' and 'B -> C', and the result parses with no diagnostic. a_chain_with_a_label_gets_a_suggested_fix: 'A -> B -> C: label' gives one E0010 whose fix is Suggested. a_name_in_quotes_that_holds_an_arrow_is_not_a_chain: '"A -> B" -> C' gives no diagnostic. scripts/task check exit 0: nextest 253 tests run, 253 passed. CLI: horsegirl fix on a file with '  A -> B -> C' inside a block wrote the two links with the indent; the labelled chain stayed, as a suggested fix should.

## 2026-11-25

Follow-up filed: 0163, a label that holds a second link ('A -> B: pay -> C: ship') gets a warning that splits it. The review note on this item asked for that chain; it is a different code path (the label reader) and a possible false positive on labels such as 'calls -> returns', so it is its own item.
