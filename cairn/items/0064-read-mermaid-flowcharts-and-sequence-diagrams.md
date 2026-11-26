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
updated: 2026-11-26
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

## 2026-11-25

From item 0040: corpus/convert.py is a stand-in converter that writes corpus/horse/*.horse for the corpus count in CI. When 'horsegirl convert' exists, make the corpus count read its output and remove the stand-in.

## 2026-11-26

From 0165, dropped: two Mermaid nodes that share a label must stay two things after conversion. The stand-in corpus/convert.py merged them, which hid E0420 in 11 corpus files and cost two clean files (0031, 0207). Check the converted corpus for this case.
