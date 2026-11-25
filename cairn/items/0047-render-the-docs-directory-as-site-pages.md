---
id: 47
uid: 28fe6a91-fe3e-44c4-933f-2fcc5d9566d2
title: Render the docs directory as site pages
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 44
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

Documentation in the repository must appear on the site without a second copy.

## Proposal

Render the Markdown files under `docs/` as site pages with a sidebar, heading anchors and syntax highlighting for horsegirl. A broken internal link fails the build.

## Acceptance criteria

- [x] Adding a Markdown file under `docs/` adds a page with no other change.
- [x] A link to a missing page fails `scripts/task check`.

## 2026-11-25

Criterion 1: added docs/probe.md and ran scripts/task build with no other change. It printed '/docs/probe' in the route list and 'check-export: 15 paths served ... all present'. The docs index linked to it and its horsegirl fence had hg-arrow, hg-key, hg-keyword and hg-punct spans. docs/concept is not in the routes. Test: 'a new Markdown file becomes a page'.

## 2026-11-25

Criterion 2: appended [bad](nowhere.md) to docs/probe.md and ran scripts/task check. Exit 1 with 'Broken links in docs: probe.md: link to "nowhere.md" names no page.' Tests for a missing page and a missing heading pass in scripts/task test. I removed probe.md; scripts/task check then exits 0.
