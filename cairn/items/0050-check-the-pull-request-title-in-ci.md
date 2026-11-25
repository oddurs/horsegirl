---
id: 50
uid: 492c4fe5-8ab1-4dac-9331-1dd7d872491e
title: Check the pull request title in CI
type: feature
status: doing
milestone: v0.1
assignee: Oddur Sigurdsson
claimed: 2026-11-25
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
effort: s
area: workflow
track:
- workflow
---

## Problem

A pull request is squashed, so its title becomes the commit on main. Nothing checks the title.

## Proposal

Add a job that validates the pull request title as a Conventional Commit with the same pattern as the commit hook, and rejects attribution in the title and the body.

## Acceptance criteria

- [ ] A pull request titled `Update stuff` fails the `required` check with a message that shows a valid example.
- [ ] The pattern lives in one file that the hook and the job both read.
