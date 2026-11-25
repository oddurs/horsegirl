---
id: 151
uid: ea6add05-0d59-4eda-a39d-b90341028c4a
title: A comment line that reads as a link or a block gets a warning
type: feature
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 130
---

## Problem

ADR-130 keeps `#` comment lines and states them on the card. A line such as `#1 priority -> Plan` is still read as a comment with no diagnostic, so the link is lost in silence.

## Approach

A warning for a comment line whose text after `#` parses as a link, a rule or a block. Its suggested fix writes the name in quotes: `"#1 priority" -> Plan`.

## Acceptance criteria

- [ ] `#1 priority -> Plan` gives one warning with a stable code on line 1, with a suggested fix that gives `"#1 priority" -> Plan`.
- [ ] `# a plain remark` gives no diagnostic.
- [ ] `docs/diagnostics.md` lists the new code.
