---
id: 125
uid: 8c12fe5f-d23b-4965-8314-6d9d4616f30a
title: Check the layered layout choice on the corpus of public flowcharts
type: chore
status: backlog
milestone: v0.1
depends_on:
- 17
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: s
related:
- 18
---

## Problem

SPIKE-18 chose our own Sugiyama pipeline from ten diagrams. The author of the prototype wrote those ten diagrams in the same spike that tuned the prototype. The corpus of public flowcharts from SPIKE-17 was not in the repository then. The ranking of the candidates is not checked on diagrams that the author did not write.

## Proposal

Run the `draw` command of `spikes/layered-layout` on the SPIKE-17 corpus. The spike reads a subset of horsegirl: links, and groups in braces. The corpus is Mermaid flowcharts, so convert each flowchart to that subset first, and count the flowcharts that do not convert. Compare own with elk by overlaps, group faults, links through a box and crossings. Write the totals and the verdict in a note on SPIKE-18.

On the ten spike diagrams own left 80 crossings and elk left 36, which is 2.2 times. Neither left an overlap, a group fault or a link through a box. The choice of SPIKE-18 rests on the seed and on the size, and not on the crossings. The verdict is therefore about faults: if own overlaps boxes or breaks a group on real flowcharts, the choice is open again.

Do not change the prototype to improve the numbers. Item 0122 improves the order phase in the crate.

## Acceptance criteria

- [ ] `draw` runs on at least 100 flowcharts from the SPIKE-17 corpus, and the number that did not convert is recorded.
- [ ] A note on SPIKE-18 gives the totals of own and elk on those flowcharts: overlaps, group faults, links through a box and crossings.
- [ ] The note says whether own still has no overlap and no group fault, and gives the ratio of the crossings of own to the crossings of elk beside the 2.2 from the spike.
- [ ] If own has an overlap or a group fault, or the ratio is above 3, a `decision` item exists that reopens the choice before item 0032 merges.

## Proposed priority: p2 -> p1 (Oddur Sigurdsson, 2026-11-25)

The check can reopen the choice that item 0032 (p0) builds on, so it must run before 0032 merges.
