---
id: 126
uid: d967415b-5e7e-4486-b954-1b4ad19fc7c9
title: The site sets its type in the concept page fonts
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 45
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] The exported site contains the three font files and their licences, and no page requests a font from another origin.
- [x] Headings, body text and code render in the concept page fonts in a screenshot of `/design/`.
- [x] `scripts/task check` passes, and the accessibility check in the site build still reports no serious finding.

## 2026-11-25

Watched pass: scripts/task check (all steps, a11y 36 page loads no serious finding). check-export now fails unless the export has 3 woff2 files, 3 OFL licences and no fonts.googleapis/gstatic reference. webshot of /horsegirl/design/ at 1280px and with -m shows Bricolage headings, Instrument Sans body, JetBrains Mono code.
