---
id: 119
uid: cb05580b-48ff-424d-a94b-34e8a620616e
title: Names that hold a reserved character are written in quotes
type: decision
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: language
area: language
effort: s
related:
- 17
- 13
- 28
---

## Context

SPIKE-17 measured 3052 names in 239 real Mermaid flowcharts. 10.5% of them hold a character that a bare name cannot hold: a colon (5.3%), a comma (5.2%), an empty label, `->`, `@` or a brace. One flowchart in three has at least one such name. ADR-13 said to revisit bare names when this share is more than a few percent.

The same characters reach fact values. Item 121 keeps the first line of a Mermaid label as the name and writes the other lines as a `note` fact. That frees 133 names, but 130 of them hold a comma, a colon or a brace in the other lines, as in `wally { note: install, list }` and `api { note: django :8000 }`. Braces separate parts with commas and a fact key from its value with a colon, so these fact values have the same problem as the names.

## Options and tradeoffs

- Quotes as an escape: `"Weather Telemetry: Crosswind" -> Sensors`. A bare name stays the normal form. The card gains one line. The formatter writes quotes only when a name needs them.
- Ids with labels, as in Mermaid: safe to parse, but they spoil the 89.5% of names that are fine today, and ADR-13 records why agents confuse the two.
- A narrower grammar, for example only `: ` with a space starts a label: it saves 27 names of 320, and the rule is hard to explain on the card.
- Reject such names and tell the agent to rename: one file in three would fail on conversion.

## Decision

Decided: quotes as an escape. A name in double quotes may hold any character except a line break and a double quote. The same quoted form applies to a fact value: `api { note: "django :8000" }`. `fmt` removes quotes from a name or a fact value that does not need them, and adds them to one that does. Item 28 rewrites a Mermaid quoted name to a bare name only when the bare name is safe.

A bare name stays the normal form: nine names in ten need no quotes. A name or a fact value that needs quotes and lacks them is an error whose fix adds the quotes, so an agent that forgets them is corrected in one step. The language card states the quoted form in one line.

The two examples from the corpus are written `wally { note: "install, list" }` and `api { note: "django :8000" }`. Each parses as one fact.

## Revisit when

The benchmark (item 41) shows that agents write quotes where they are not needed, or forget them where they are.

## Acceptance criteria

- [x] The choice, its evidence and its consequences are recorded.
- [x] If the choice changes the language card, the card states the quoted form in one line.
- [x] The decision states whether the quoted form also holds a fact value, and how `{ note: install, list }` and `{ note: django :8000 }` are written so that they parse as one fact each.
