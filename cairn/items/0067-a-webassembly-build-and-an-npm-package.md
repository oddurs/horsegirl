---
id: 67
uid: 820c29d5-03cd-4267-bbb5-4137f9eaadbe
title: A WebAssembly build and an npm package
type: feature
status: backlog
milestone: v0.2
depends_on:
- 37
- 59
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: high
surface: none
effort: m
area: wasm
track:
- release
- site
---

## Problem

The same core must run in a browser.

## Proposal

Build the core to WebAssembly and publish an npm package with a typed API: `check`, `fix`, `draw`, `describe`. CI records the compressed size and fails over the budget.

## Acceptance criteria

- [ ] The npm package draws the reference file in Node and in a browser test.
- [ ] The compressed size is under the budget that this item sets and records.
- [ ] Native and WebAssembly builds give byte-identical SVG for the corpus.
