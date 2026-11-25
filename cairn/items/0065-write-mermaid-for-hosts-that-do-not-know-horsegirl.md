---
id: 65
uid: 0cb41fed-2e3f-4cdc-bfb3-cb1e609df6d7
title: Write Mermaid for hosts that do not know horsegirl
type: feature
status: backlog
milestone: v0.2
depends_on:
- 64
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: cli
effort: m
area: interop
track:
- language
---

## Problem

GitHub renders Mermaid and does not render horsegirl. A README needs a picture today.

## Proposal

`draw --to mermaid` writes a Mermaid block for the flow and sequence lenses.

## Acceptance criteria

- [ ] The output of the reference file renders on GitHub with no error.
- [ ] Converting to Mermaid and back gives the same model.
