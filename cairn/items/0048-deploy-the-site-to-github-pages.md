---
id: 48
uid: ac23293c-44c2-45ce-8dca-68d134797ce4
title: Deploy the site to GitHub Pages
type: chore
status: planned
milestone: v0.1
depends_on:
- 44
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: low
surface: none
effort: s
area: workflow
track:
- site
- workflow
---

## Purpose

The site must publish itself.

## Approach

Add a workflow that builds the site on every pull request and deploys `site/out` to GitHub Pages on a push to `main`. The pull request build uploads the site as an artifact.

## Acceptance criteria

- [ ] A merge to `main` updates https://oddurs.github.io/horsegirl within ten minutes.
- [ ] The deploy job has only the permissions that Pages needs.
- [ ] The `required` check covers the site build.
