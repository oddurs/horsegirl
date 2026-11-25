---
id: 47
uid: 28fe6a91-fe3e-44c4-933f-2fcc5d9566d2
title: Render the docs directory as site pages
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
depends_on:
- 44
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

Documentation in the repository must appear on the site without a second copy.

## Proposal

Render the Markdown files under `docs/` as site pages with a sidebar, heading anchors and syntax highlighting for horsegirl. A broken internal link fails the build.

## Acceptance criteria

- [ ] Adding a Markdown file under `docs/` adds a page with no other change.
- [ ] A link to a missing page fails `scripts/task check`.
