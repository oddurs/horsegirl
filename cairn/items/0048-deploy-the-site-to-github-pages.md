---
id: 48
uid: ac23293c-44c2-45ce-8dca-68d134797ce4
title: Deploy the site to GitHub Pages
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 44
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] A merge to `main` updates https://oddurs.github.io/horsegirl within ten minutes.
- [x] The deploy job has only the permissions that Pages needs.
- [x] The `required` check covers the site build.

## 2026-11-25

Enabled Pages with gh api POST repos/oddurs/horsegirl/pages build_type=workflow; it printed html_url https://oddurs.github.io/horsegirl/ and build_type workflow.

## 2026-11-25

Criterion 2: deploy job sets permissions pages: write and id-token: write only; the workflow-level default stays contents: read, which a job-level block replaces. Read from .github/workflows/ci.yml.

## 2026-11-25

Criterion 3: required needs check; check runs scripts/task check, which runs build and site export. Ran scripts/task check locally: exit 0, 'check-export: 7 paths served from /horsegirl/, all present', site/out/index.html exists. Not yet seen on a CI run.

## 2026-11-25

Criterion 1 is open: it can only be watched after a merge to main.

## 2026-11-25

CI run 45556101850 on head 376a90c: check pass (steps 'Run scripts/task check' and 'Upload the site' pass), required pass, pr-title pass, deploy skipped (PR event). PR mergeable.

## 2026-11-25

Verified on main: the push for 510710e started CI at 06:21:12Z, the deploy job succeeded, and the run ended at 06:22:31Z. https://oddurs.github.io/horsegirl/ answers 200 with the page title horsegirl.
