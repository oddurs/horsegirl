---
id: 73
uid: d64153e0-329b-469c-947b-84eccbbaf5ad
title: Select what a lens shows with where
type: feature
status: backlog
milestone: v0.2
depends_on:
- 36
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: language
effort: m
area: lens
track:
- language
---

## Problem

A lens says how to look. Nothing says at what.

## Proposal

Add `where <field> is <value>` to a `show` line to select things, and document the grammar.

## Acceptance criteria

- [ ] `show flow where owner is Commerce` draws only those things and the links among them.
- [ ] An unknown field is an error that lists the fields the file uses.
