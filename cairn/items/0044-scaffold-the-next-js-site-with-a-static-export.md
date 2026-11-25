---
id: 44
uid: 2a374060-be4a-4d0f-aaa5-c091e9690e67
title: Scaffold the Next.js site with a static export
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: site
track:
- site
---

## Problem

The project has no home. The concept page is a single private file.

## Proposal

Scaffold a Next.js site in `site/`: App Router, TypeScript in strict mode, static export, a base path of `/horsegirl`, a pinned Node version, and a dev server on a fixed port. Add `site` verbs to `scripts/task`: format, lint, type check, build.

## Acceptance criteria

- [x] `scripts/task check` builds the static site into `site/out`.
- [x] The exported site works when served from a sub-path.
- [x] No runtime dependency is added without a line in the pull request that says what it replaces.

## 2026-11-25

Criterion 1: removed site/out and site/.next, ran scripts/task check; it exited 0 and wrote site/out/index.html, site/out/404.html and site/out/_next/. Without pnpm on PATH, scripts/task fmt:check printed 'pnpm not found; skipped' and exited 0; with CI=1 it printed 'pnpm is required in CI' and exited 1.

## 2026-11-25

Criterion 2: the build runs site/scripts/check-export.mjs, which serves site/out under /horsegirl/ and requests every linked page and asset; it printed 'check-export: 7 paths served from /horsegirl/, all present'. A planted <a href="/escape/"> and a missing image failed the build with two 404 lines. Separately, python3 -m http.server with out/ linked at /horsegirl/ returned 200 for /horsegirl/, and a screenshot showed the page rendered.

## 2026-11-25

Criterion 3: runtime dependencies are next, react and react-dom, each stated in the pull request with what it replaces. Development dependencies: typescript, @types/react, @types/react-dom, @types/node, @biomejs/biome (one tool for format and lint, in place of ESLint plus Prettier). next.config.ts sets agentRules: false so next dev does not write site/AGENTS.md for some callers only.
