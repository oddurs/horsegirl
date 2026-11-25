---
id: 39
uid: 34963a4e-7154-4e13-af14-2ac579fb0141
title: A golden test harness for files, diagnostics and pictures
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: low
surface: none
effort: m
area: workflow
track:
- workflow
- language
---

## Purpose

Parser, diagnostics and SVG output are contracts. A change to any of them must show up as a diff in review.

## Approach

Add a golden test harness: each case is a `.horse` file with its expected diagnostics JSON and, where it draws, its expected SVG. One command updates the expectations.

## Acceptance criteria

- [x] A new case needs one file and no Rust code.
- [x] A changed expectation shows as a readable diff in a pull request.

## 2026-11-25

Criterion 1: added tests/golden/new-case.horse (one line, 'A -> -> B') with no Rust change. 'cargo test -p horsegirl-core --test golden' failed with 'new-case.tree is missing' and 'new-case.diagnostics.json is missing'; 'HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden' printed 4 passed and wrote both files (E0006 at line 1 column 6); the plain run then printed 4 passed. Probe removed afterwards. A stale gone.tree failed as 'gone.tree has no case' and bless removed it.

## 2026-11-25

Criterion 2: changed three-bad-lines.horse and blessed. git diff showed each expectation as a one-line change, for example '-    "span": [140, 142],' / '+    "span": [150, 152],' in three-bad-lines.diagnostics.json, with code, line, column, text and message as context lines. Without bless the test prints 'three-bad-lines.diagnostics.json changed at line 14' with the expected and actual lines and the bless command. The expectations are plain text with LF endings and are not marked generated in .gitattributes, so GitHub shows them expanded.

## 2026-11-25

Scope: no SVG backend exists yet, so no case has a picture. The harness writes each output from a fixed list (OUTPUTS in crates/horsegirl-core/tests/golden.rs); 0034 adds '<case>.svg' there for the cases that draw. The golden test is native only, because the WebAssembly runner gives tests no directory. scripts/task check exited 0 with the 4 golden tests passing.

## Result

Golden harness in crates/horsegirl-core/tests/golden.rs: each .horse case under tests/golden has its .tree and .diagnostics.json beside it; HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden writes them. SVG joins as one more output when 0034 lands.
