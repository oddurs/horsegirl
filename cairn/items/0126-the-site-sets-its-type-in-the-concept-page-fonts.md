---
id: 126
uid: d967415b-5e7e-4486-b954-1b4ad19fc7c9
title: The site sets its type in the concept page fonts
type: feature
status: backlog
milestone: v0.1
depends_on:
- 45
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: site
---

## Problem

The concept page sets its type in Bricolage Grotesque, Instrument Sans and JetBrains Mono, loaded from Google Fonts. The site has the same font stacks in `site/app/tokens.css`, but it loads no font file. A visitor sees the fallback fonts: Avenir Next, Helvetica Neue, Arial, Menlo.

## Proposal

Serve the three fonts from the site itself. Either check in the WOFF2 files under `site/public` with their licences, or use `next/font` with local files. A build must not fetch fonts from the network, so that two builds of one commit give the same bytes.

## Acceptance criteria

- [ ] The exported site contains the three font files and their licences, and no page requests a font from another origin.
- [ ] Headings, body text and code render in the concept page fonts in a screenshot of `/design/`.
- [ ] `scripts/task check` passes, and the accessibility check in the site build still reports no serious finding.
