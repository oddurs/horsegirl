---
id: 71
uid: 75208f60-3db7-4e40-be8a-d2cd3f8c5c0a
title: A GitHub Action that checks and draws every file
type: feature
status: backlog
milestone: v0.2
depends_on:
- 42
- 64
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: medium
surface: none
effort: m
area: workflow
track:
- release
- workflow
---

## Problem

A team should adopt horsegirl by changing CI, not by changing habits.

## Proposal

Publish a GitHub Action that checks every horsegirl and Mermaid file in a repository and writes the pictures as SVG, as a commit or as an artifact.

## Acceptance criteria

- [ ] A test repository with ten Mermaid files gets ten pictures from a five-line workflow.
- [ ] A file with an error fails the job with the diagnostic as an annotation on the line.
