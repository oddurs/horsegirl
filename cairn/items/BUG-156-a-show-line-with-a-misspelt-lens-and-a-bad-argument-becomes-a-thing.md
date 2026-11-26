---
id: 156
uid: b3ead60b-c485-4824-85aa-dbf5a06741ad
title: A show line with a misspelt lens and a bad argument becomes a thing
type: bug
status: backlog
milestone: v0.2
created: 2026-11-25
updated: 2026-11-26
priority: p2
risk: low
surface: none
area: language
related:
- 136
effort: s
---

## Problem

Item 136 reads a line as a `show` line when it starts with `show`, a lens from `check::LENSES` and `of`, `from` or `by`. A misspelt lens is not in that list. So `show flwo from Sensors: wind` still gives E0009, and `show sequnce of S, T` still parses as two things with no diagnostic. E0300 already has a fix for a lens one edit from a known lens, but the line never reaches the checks as a `show` line.

The lens list stays strict on purpose: with any word, a link such as `show room of Mirrors -> Hall` would become a `show` line whose safe fix quotes `Mirrors -> Hall`.

## Acceptance criteria

- [ ] `show flwo from Sensors: wind` is read as a `show` line with a mistake, and gives a diagnostic that names the lens `flow`.
- [ ] `show room of Mirrors -> Hall` still parses as a link with no diagnostic.
- [ ] The fuzz test passes with both inputs in `FORMS`.
