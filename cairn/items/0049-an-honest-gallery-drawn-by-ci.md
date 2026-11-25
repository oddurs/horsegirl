---
id: 49
uid: df570902-2521-4676-a56c-ba293da8bd68
title: An honest gallery, drawn by CI
type: feature
status: planned
milestone: v0.1
depends_on:
- 37
- 47
- 48
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
effort: m
area: site
track:
- site
- pictures
---

## Problem

A gallery drawn by hand lies about what the program does.

## Proposal

Draw every example picture on the site in CI from a `.horse` file in the repository, with the released binary's code. Each picture shows its source and the command.

## Acceptance criteria

- [ ] No picture on the site is committed as a file; each is built.
- [ ] The gallery shows the flow lens on at least six files from the corpus.
