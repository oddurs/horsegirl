---
id: 46
uid: 0571aefb-6bf0-4e14-855d-b275981004f5
title: The landing page
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
depends_on:
- 45
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: m
area: site
track:
- site
---

## Problem

A visitor needs to understand in one screen what horsegirl is and whether it is real yet.

## Proposal

Build the landing page from the concept page: headline, the tool window with the reference file and its picture, the three statements, the five rules, and an honest status line that says what works today.

## Acceptance criteria

- [x] The page states what the released binary does and links to the roadmap for the rest.
- [x] The first screen works at a width of 390 pixels with no horizontal scroll.
- [ ] The page scores at least 95 for performance and accessibility in Lighthouse on CI.

## 2026-11-25

Criterion 1: the status section of site/app/page.tsx lists the commands read from the USAGE text in crates/horsegirl/src/main.rs (spec, --version, --help), says there is no release, and links ROADMAP.md on GitHub for the rest. The hero pill says 'Not released. A build from source prints the language card.' site/tests/status.test.ts runs cargo run -- --help and compares its commands with the list the page reads: pnpm test printed pass 14, fail 0. The picture is captioned as drawn by hand; every statement about checks, pictures and rules is in the future tense.

## 2026-11-25

Criterion 2: check-a11y now fails when document scrollWidth exceeds clientWidth. scripts/task check printed 'check-a11y: 24 page loads, no serious finding, focus visible at every stop', with 16 focus stops on /horsegirl/ at 390px. With .hero { min-width: 600px } it printed 'the page is 620px wide in a 390px window' for light and dark and exited 1. webshot -m of the landing page: the status pill, headline, lede, and the sketch of the flow fit in the first screen with no sideways scroll; the source and the card scroll inside their own boxes.
