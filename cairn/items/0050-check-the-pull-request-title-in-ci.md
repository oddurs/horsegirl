---
id: 50
uid: 492c4fe5-8ab1-4dac-9331-1dd7d872491e
title: Check the pull request title in CI
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
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

- [x] A pull request titled `Update stuff` fails the `required` check with a message that shows a valid example.
- [x] The pattern lives in one file that the hook and the job both read.

## 2026-11-25

PR 10 was first titled 'Update stuff'. Run 45553818878: pr-title failed with "'Update stuff' is not a Conventional Commit" and the example 'feat(parser): read fan-out links'; required failed. After retitling to 'ci(workflow): check the pull request title', run 45553899764: pr-title, check and required all passed. required needs [check, pr-title] in ci.yml.

## 2026-11-25

scripts/commit-pattern holds the regex. .githooks/commit-msg, scripts/agent and scripts/check-pr-title (the pr-title job) all cat it. tests/agent.sh checks the hook, and the title script with valid and invalid titles; scripts/task check passed.
