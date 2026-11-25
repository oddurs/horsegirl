---
id: 34
uid: 6f396b4b-c038-4575-a22d-299fd2580048
title: Draw the scene as SVG with CSS variables
type: feature
status: planned
milestone: v0.1
depends_on:
- 30
- 31
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: svg
effort: m
area: render
track:
- pictures
---

## Problem

The scene has no visible form yet.

## Proposal

Write the SVG backend. Colours and fonts are CSS variables with fallbacks. Each shape has a class for its role and for its facts, such as `owner-commerce`. The output has a `title` and a `desc`.

## Acceptance criteria

- [ ] The reference flow renders to an SVG that a snapshot test holds byte for byte.
- [ ] The SVG is valid XML and uses no script and no external resource.
- [ ] Setting `--hg-accent` in a host page changes the accent with no rebuild.
