---
id: 138
uid: 91b31eb1-cca9-4296-9062-59541794218d
title: fmt writes a file in place
type: feature
status: backlog
milestone: v0.2
created: 2026-11-25
updated: 2026-11-26
priority: p1
risk: low
surface: cli
area: cli
effort: s
related:
- 29
- 37
---

## Problem

`horsegirl fmt` prints to standard output. An agent that formats a file must redirect the output to a temporary file and move it back. A redirect to the file itself empties it.

## Proposal

Add `fmt --write FILE`. It replaces the file only when the text changes, and it leaves a file with an error untouched.

## Acceptance criteria

- [ ] `fmt --write` of a formatted file does not touch the file.
- [ ] `fmt --write` of an unformatted file replaces it with the output of `fmt`.
- [ ] `fmt --write` of a file with an error leaves the file as it was and exits 1.

## 2026-11-26

Triage: v0.2 at p1. An agent that runs fmt today must redirect the output and move the file.
