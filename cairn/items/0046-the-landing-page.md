---
id: 46
uid: 0571aefb-6bf0-4e14-855d-b275981004f5
title: The landing page
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 45
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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
- [x] The page scores at least 95 for performance and accessibility in Lighthouse on CI.

## 2026-11-25

Criterion 1: the status section of site/app/page.tsx lists the commands read from the USAGE text in crates/horsegirl/src/main.rs (spec, --version, --help), says there is no release, and links ROADMAP.md on GitHub for the rest. The hero pill says 'Not released. A build from source prints the language card.' site/tests/status.test.ts runs cargo run -- --help and compares its commands with the list the page reads: pnpm test printed pass 14, fail 0. The picture is captioned as drawn by hand; every statement about checks, pictures and rules is in the future tense.

## 2026-11-25

Criterion 2: check-a11y now fails when document scrollWidth exceeds clientWidth. scripts/task check printed 'check-a11y: 24 page loads, no serious finding, focus visible at every stop', with 16 focus stops on /horsegirl/ at 390px. With .hero { min-width: 600px } it printed 'the page is 620px wide in a 390px window' for light and dark and exited 1. webshot -m of the landing page: the status pill, headline, lede, and the sketch of the flow fit in the first screen with no sideways scroll; the source and the card scroll inside their own boxes.

## 2026-11-25

Criterion 3: site/scripts/check-lighthouse.mjs runs Lighthouse 13.6.0 on the landing page as a phone and as a desktop, three runs each, and fails when the median for performance or accessibility is under 95. It runs in the site build, so the CI check job runs it. CI run 45563394030 on f186337 (PR #23) succeeded and printed mobile performance 100 (runs: 100, 100, 100), mobile accessibility 100, desktop performance 100, desktop accessibility 100. With lang removed from <html> and the lede at low contrast, the check printed 'accessibility 92 (runs: 92, 92, 92), under 95', named color-contrast and html-has-lang, and exited 1. On a loaded local machine single mobile runs ranged 96 to 100, which is why the check takes a median.

## Result

The site root is a landing page that says there is no release, lists the commands read from the binary's USAGE text (held to --help by a test), shows the real language card and a picture drawn by hand that is labelled as such, and links the roadmap. The site build fails on a page wider than its window, and on a Lighthouse median under 95 for performance or accessibility on the landing page.
