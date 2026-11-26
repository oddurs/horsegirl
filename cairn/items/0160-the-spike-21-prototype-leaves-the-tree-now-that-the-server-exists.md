---
id: 160
uid: 78a6dc66-bb5e-42c3-be6b-902d80120a6c
title: The SPIKE-21 prototype leaves the tree now that the server exists
type: chore
status: backlog
milestone: v0.2
created: 2026-11-25
updated: 2026-11-26
priority: p2
surface: none
area: mcp
effort: s
related:
- 38
risk: low
---

## Problem

`spikes/mcp/` holds the throwaway prototype of SPIKE-21. `horsegirl mcp` (0038) now serves the same tools from the binary, with tests. The prototype has its own `Cargo.lock` with a second copy of the rmcp tree, so a dependency update must touch it or let it go stale. A reader can take it for the real server.

## Proposal

Delete `spikes/mcp/`. Change the Evidence section of SPIKE-21 to say that the prototype was removed and that 0038 holds its tests, so the answer does not point at a path that is gone.

## Acceptance criteria

- [ ] `spikes/mcp/` is not in the tree.
- [ ] No document or item points at `spikes/mcp/` as a path that exists.
- [ ] `scripts/task check` passes.

## Proposed risk:  -> low (Oddur Sigurdsson, 2026-11-25)

It deletes a prototype that no build, test or document depends on.
