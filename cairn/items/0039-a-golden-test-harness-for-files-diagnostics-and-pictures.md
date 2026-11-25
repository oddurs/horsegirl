---
id: 39
uid: 34963a4e-7154-4e13-af14-2ac579fb0141
title: A golden test harness for files, diagnostics and pictures
type: chore
status: planned
milestone: v0.1
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
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

- [ ] A new case needs one file and no Rust code.
- [ ] A changed expectation shows as a readable diff in a pull request.
