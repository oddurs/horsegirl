---
id: 145
uid: 26b13352-a413-4e79-ae07-ec84222ed506
title: Each golden case that draws holds its SVG
type: chore
status: backlog
milestone: v0.1
depends_on:
- 32
- 34
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: render
effort: s
related:
- 39
---

## Problem

The golden harness in `crates/horsegirl-core/tests/golden.rs` holds the tree and the diagnostics of each case. It does not hold a picture. The SVG backend (0034) is held by one snapshot, `crates/horsegirl-core/tests/checkout.svg`, drawn from a stand-in for the layout in `tests/support/mod.rs`. No golden case draws until the layered layout (0032) is in the core.

## Proposal

Add `svg` to `OUTPUTS` in the golden harness. Write `<case>.svg` for each case that parses with no error, drawn with the layered layout. Draw `checkout.svg` with the real layout too, and remove the stand-in when no test needs it.

## Acceptance criteria

- [ ] Each golden case with no error has a `.svg` expectation that the golden test holds byte for byte.
- [ ] `HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden` writes the SVG expectations and removes an SVG expectation that has no case.
- [ ] `checkout.svg` is drawn with the layered layout, not the stand-in.
