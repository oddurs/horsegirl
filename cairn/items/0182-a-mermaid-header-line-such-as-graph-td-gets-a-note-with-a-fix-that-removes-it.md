---
id: 182
uid: 2219a186-81fa-4f45-ba7e-62fb97c753b5
title: A Mermaid header line such as graph TD gets a note with a fix that removes it
type: feature
status: backlog
milestone: v0.2
created: 2026-11-27
updated: 2026-11-27
priority: p2
risk: low
surface: diagnostics
effort: s
area: check
related:
- 142
---

## Problem

A file pasted from Mermaid often starts with `graph TD` or `flowchart LR`. horsegirl reads that line as a thing named `graph TD`, and `check` gives W0107: the thing stands alone. The message tells the author to link the thing, which is the wrong advice. Seen with the v0.1.0 release binary.

## Proposal

Read a first line that is `graph` or `flowchart` with an optional direction (`TD`, `TB`, `BT`, `LR`, `RL`) as a Mermaid habit. Give the note N0101 with a safe fix that removes the line, and no W0107.

## Acceptance criteria

- [ ] `graph TD`, `flowchart LR` and `graph` alone on the first line give N0101 with a safe fix that deletes the line, and no W0107.
- [ ] A thing named `graph TD` on a later line, or in quotes, stays a thing.
- [ ] `fix` on a pasted Mermaid flowchart with a header leaves no header line.
