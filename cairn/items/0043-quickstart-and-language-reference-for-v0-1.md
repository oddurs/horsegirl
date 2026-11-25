---
id: 43
uid: ef6646d4-c363-4f3d-bbd4-eaee77a61e9a
title: Quickstart and language reference for v0.1
type: docs
status: planned
milestone: v0.1
depends_on:
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: m
area: language
track:
- language
- site
---

## Reader and question

An agent or a person who has never seen horsegirl. Question: how do I install it and draw one picture?

## Change

Write the quickstart and the language reference for v0.1: the five rules, `-x`, `?`, the flow lens, each command, and the diagnostic catalogue.

## Acceptance criteria

- [ ] A reader can complete the described task with the current program.
- [ ] Every example in the reference is a file in the golden tests.

## 2026-11-25

From item 23: the parser reads forms that the card does not spell out, and the reference must state them. A link label holds a comma, a colon and an arrow without quotes and runs to the end of the line; a brace or an @ in a label needs quotes. -x is a rule only as a whole word, so Dash-x is a name. A ? is a guess only as a final whole word. A citation is @ and a path with no space. A line that holds one of , : { -> and starts with show or story is not a show line or a story. A key: value pair is a fact only inside braces. Braces nest 64 deep at most (E0007).
