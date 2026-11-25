---
id: 142
uid: 2ae301a9-5240-4d93-8b9d-b6aa9eda286e
title: Say what to do with Mermaid arrows that horsegirl has no form for
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: s
---

### Problem

Item 28 reads `-->` as `->`. Mermaid has other arrows: `<->`, `---`, `-.->`, `==>` and `--x`. Today `A <-> B` gives a thing named `A <`, with no diagnostic.

### Proposal

The reader gives a diagnostic for each of these arrows and says what to write instead. A two-way link is two lines, and a line without an arrowhead has no horsegirl form.

## Acceptance criteria

- [ ] `A <-> B` gives a diagnostic with a code and a message that names the two-line form.
- [ ] `A --- B`, `A -.-> B` and `A ==> B` each give a diagnostic that says what to write.
