---
id: 49
uid: df570902-2521-4676-a56c-ba293da8bd68
title: An honest gallery, drawn by CI
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 37
- 47
- 48
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] No picture on the site is committed as a file; each is built.
- [x] The gallery shows the flow lens on at least six files from the corpus.

## 2026-11-25

Criterion 1: site/lib/gallery.ts runs the horsegirl binary at build time and inlines each SVG; the hand-drawn CheckoutSketch component is deleted and the home page draws checkout.horse the same way. A site test finds no .svg or .png under site/. scripts/task check exits 0.

## 2026-11-25

Criterion 2: after scripts/task check, site/out/gallery/index.html holds 11 svg elements with class horsegirl (examples/checkout.horse and the ten files of spikes/layered-layout/corpus, all flow). check-export now fails below six. I looked at a screenshot of /gallery/. a11y: 0 findings on /gallery/.

## 2026-11-25

Fix round 1. scripts/task check passed. site/out/gallery/index.html holds 9 svg elements with class horsegirl: examples/checkout.horse and corpus/horse/0001 to 0008 (each exits 0). site/out/design/index.html has no 'Drawn by hand' text; its figure is drawn from examples/checkout.horse. The gallery test asserts the named list holds six or more corpus/horse files.
