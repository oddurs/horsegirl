---
id: 78
uid: ecc67c30-03e2-4402-86de-1260b04cdf2a
title: 'diff: one picture of what changed'
type: feature
status: backlog
milestone: v0.3
depends_on:
- 59
- 76
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: cli
effort: m
area: render
track:
- pictures
- workflow
---

## Problem

A reviewer reads a pull request that changes a system and sees only text.

## Proposal

`horsegirl diff <a> <b>` draws one picture through any lens over two versions of a file: added things and links in the accent colour, removed ones dotted.

## Acceptance criteria

- [ ] The diff of a file with itself is the plain picture.
- [ ] The read-back lists what was added and removed.
