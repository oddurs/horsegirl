---
id: 177
uid: 72e6d327-6f19-480f-a070-38d44b498f39
title: The SVG of a wide labelled fan grows with the square of the fan
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: svg
area: render
related:
- 170
---

## Problem

Links from one thing share the line that leaves it. When a label sits on that shared line, the SVG backend stops the line of every link that runs along it at the label (BUG-170). Each link path is drawn whole, so in a fan every link carries a cut for each label on the shared line. A fan of 400 labelled links out of one thing and 400 into another draws 2.2 MB of SVG, against 0.3 MB when only the own label cuts a line. Writing that SVG takes about 50 ms of the 0.3 s draw.

## Proposal

Draw a part that two links share once, or cut a link only at the labels on the parts that no earlier link has drawn. Keep each link path in the SVG whole enough that a reader can select one link.

## Acceptance criteria

- [ ] The SVG of a fan of 400 labelled links out of one thing and 400 into another is at most twice the size of the same fan with no label on a shared line.
- [ ] No link line is drawn across the text of a link label, by the test of tests/svg.rs.
