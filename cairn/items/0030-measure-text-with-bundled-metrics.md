---
id: 30
uid: e66b6f6e-e7e6-4f25-9bdc-89a216afec01
title: Measure text with bundled metrics
type: feature
status: planned
milestone: v0.1
depends_on:
- 19
- 22
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

A node must be as wide as its label, and the answer must be the same on every machine.

## Proposal

Implement the text measurement that the spike chose, behind one function: text and style in, width and height out.

## Acceptance criteria

- [ ] Widths for 100 reference labels are within 3 percent of the browser measurement recorded in the spike.
- [ ] The function gives the same result on Linux, macOS and in WebAssembly, held by CI.
