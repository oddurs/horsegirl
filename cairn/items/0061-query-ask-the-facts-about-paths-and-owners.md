---
id: 61
uid: 334851c8-15b5-47b9-8819-7662e899d9d2
title: 'query: ask the facts about paths and owners'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 25
- 37
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: cli
effort: m
area: describe
track:
- agent-loop
---

## Problem

A horsegirl file is a small map of a system. An agent should ask it questions without reading every line.

## Proposal

`horsegirl query` answers a fixed set of questions: paths between two things, what reaches a thing, what a thing reaches, and the owners along a story. The questions are a small grammar, not free text.

## Acceptance criteria

- [ ] `query 'reaches Postgres'` on the reference file prints the two paths.
- [ ] An unknown question lists the questions that exist.
