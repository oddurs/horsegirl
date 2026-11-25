---
id: 96
uid: a6f67f9c-63d8-43fd-8542-1076b6c0dc9d
title: One install command on Linux, macOS and Windows
type: chore
status: backlog
milestone: v1.0
depends_on:
- 42
- 67
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: release
track:
- release
---

## Purpose

An install that needs a toolchain reaches few people.

## Approach

Publish to Homebrew, npm and crates.io, add Windows binaries, and verify each install path in CI on its own platform.

## Acceptance criteria

- [ ] CI installs from each channel on Linux, macOS and Windows and runs `horsegirl spec`.
- [ ] The README lists one command for each platform.
