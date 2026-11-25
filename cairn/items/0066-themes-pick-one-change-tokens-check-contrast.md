---
id: 66
uid: 24181aa5-2e29-47f7-ac51-d1119752bdbc
title: 'Themes: pick one, change tokens, check contrast'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 35
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: theme
effort: m
area: render
track:
- pictures
---

## Problem

A team needs its own accent and font without painting single boxes.

## Proposal

Add `--theme`, a project setting, and `theme <name> { from: plain, ... }` blocks with twelve tokens. Ship rosette, blueprint and ink. `horsegirl theme check` measures contrast and palette separation for colour-blind readers.

## Acceptance criteria

- [ ] Switching theme never changes a position, held by a test.
- [ ] A theme with text contrast under 4.5 to 1 fails `theme check` with the measured ratio.
