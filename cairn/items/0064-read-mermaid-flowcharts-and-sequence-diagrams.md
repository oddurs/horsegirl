---
id: 64
uid: 01e42e8b-e19c-44b9-8546-ff78ae83c181
title: Read Mermaid flowcharts and sequence diagrams
type: feature
status: backlog
milestone: v0.2
depends_on:
- 37
- 55
created: 2026-11-25
updated: 2026-11-25
priority: p0
risk: high
surface: cli
effort: l
area: interop
track:
- language
---

## Problem

A new language with no importer starts with no users.

## Proposal

`horsegirl convert <file.mmd>` reads Mermaid flowcharts and sequence diagrams and writes horsegirl. Node labels become names, and ids disappear. `draw` accepts a Mermaid file directly. A construct that cannot be converted is a warning that names the line.

## Acceptance criteria

- [ ] Every flowchart in the corpus converts, and the converted file passes `check` with no error.
- [ ] A sequence diagram becomes a story.
- [ ] Unsupported Mermaid diagram types fail with a message that lists the supported ones.
