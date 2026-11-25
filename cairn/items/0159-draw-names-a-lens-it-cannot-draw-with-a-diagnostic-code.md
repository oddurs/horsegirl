---
id: 159
uid: 49538270-cbfa-4256-bfb1-3bb3f725985c
title: draw names a lens it cannot draw with a diagnostic code
type: feature
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: cli
effort: s
related:
- 37
---

## Problem

`horsegirl draw` refuses a file whose first `show` line names a lens on the language card that this version does not draw, such as `show matrix`. The refusal is a line of prose with no diagnostic code. With `--json`, that line is the one line on standard error that is not a JSON object, so a reader that parses each line as JSON fails.

## Proposal

Give the refusal a code in the catalogue in `docs/diagnostics.md`, and print it as a diagnostic: the text form, or one JSON object with `--json`. The span is the lens name on the first `show` line.

## Acceptance criteria

- [ ] `printf 'A -> B\nshow matrix\n' | horsegirl draw --json` prints only JSON objects on standard error, and one of them has the new code.
- [ ] `docs/diagnostics.md` lists the code, and a test checks the catalogue against the code.
- [ ] The text form names the lenses this version draws.
