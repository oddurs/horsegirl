---
id: 159
uid: 49538270-cbfa-4256-bfb1-3bb3f725985c
title: draw names a lens it cannot draw with a diagnostic code
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] `printf 'A -> B\nshow matrix\n' | horsegirl draw --json` prints only JSON objects on standard error, and one of them has the new code.
- [x] `docs/diagnostics.md` lists the code, and a test checks the catalogue against the code.
- [x] The text form names the lenses this version draws.

## 2026-11-25

Ran: printf 'A -> B\nshow matrix\n' | horsegirl draw --json. Stderr is one line, a JSON object with code E0302 and span 2:6-2:12; stdout is empty. Test: cli draw::a_lens_this_version_does_not_draw_is_one_json_object_with_json.

## 2026-11-25

docs/diagnostics.md lists E0302 (generated from codes.rs). The tests the_catalogue_in_the_docs_is_generated_from_the_code, every_code_in_the_source_is_in_the_catalogue and the_reference_names_every_diagnostic_code pass in scripts/task check.

## 2026-11-25

Text form prints error[E0302] with ' --> <stdin>:2:6' and 'It draws: flow.' Test: cli draw::a_lens_this_version_does_not_draw_gives_no_picture.
