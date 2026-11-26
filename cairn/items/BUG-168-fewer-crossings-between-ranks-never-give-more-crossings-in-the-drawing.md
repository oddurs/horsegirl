---
id: 168
uid: 8fef6d5c-d55c-4ca8-ba13-4d87d17772f5
title: Fewer crossings between ranks never give more crossings in the drawing
type: bug
status: backlog
milestone: later
created: 2026-11-26
updated: 2026-11-26
priority: p2
risk: low
surface: none
area: layout
effort: m
related:
- 122
- 166
---

## Problem

`untangle` in `crates/horsegirl-core/src/layout/order.rs` (item 0122) exchanges neighbouring entries of a rank until no exchange removes a crossing. It counts the crossings of straight steps between ranks. The drawing routes a link through dummy points and tracks, so the drawing can have more crossings than the order phase counts. On some graphs with no group and no label, `untangle` lowers the count of the order phase and raises the count of the drawing.

The review of pull request 67 found this graph. It has 18 nodes and no group. The widths are [110, 40, 40, 120, 100, 50, 40, 150, 140, 110, 120, 120, 140, 120, 110, 70, 60, 110] and every height is 36. The links are (5,15), (9,14), (6,8), (11,8), (4,9), (11,10), (6,17), (3,9), (13,16), (9,2), (2,8), (6,4), (4,2), (13,3), (11,12), (17,5), (0,8), (6,17), (13,2), (4,14), (10,17), (12,0), (16,12), (9,4).

- The drawing has 8 crossings on `main` before 0122 and 11 after it.
- With the call to `untangle` turned off, the drawing has 8.
- The order phase counts 8 before `untangle` and 7 after it.

On 2000 generated graphs with no group and no label (seed 5, up to 51 nodes, up to 3 links per node), 92 graphs draw more crossings than before 0122, and 41 of them by 5 or more. The worst goes from 604 to 646. The total falls from 187426 to 173280.

Item 0166 owns the rise on graphs with groups. This item owns the graphs with no group.

## Acceptance criteria

- [ ] The graph above draws with 8 crossings or fewer, and a test in `crates/horsegirl-core/tests/layout.rs` holds that number.
- [ ] On the 2000 generated graphs above, no graph draws more than a stated number of crossings above its count with `untangle` turned off, and the item states the number.
- [ ] The total on those 2000 graphs stays at 173280 or below.
- [ ] No number in `the_crossings_of_each_corpus_diagram_do_not_rise` rises.
