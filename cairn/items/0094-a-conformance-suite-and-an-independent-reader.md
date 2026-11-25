---
id: 94
uid: fe6d484b-d7ba-48f5-b97d-3c9efd4cecd6
title: A conformance suite and an independent reader
type: chore
status: backlog
milestone: v1.0
depends_on:
- 93
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: medium
surface: none
effort: l
area: language
track:
- language
---

## Purpose

A specification that nothing tests is an opinion.

## Approach

Publish the conformance suite: files with the model and the diagnostics they must produce, in a format that needs no Rust. Write a small independent reader in another language that passes it.

## Acceptance criteria

- [ ] The independent reader passes every case in CI.
- [ ] The Rust core runs the same cases.
