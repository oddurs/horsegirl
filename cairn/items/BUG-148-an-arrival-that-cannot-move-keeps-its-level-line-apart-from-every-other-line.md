---
id: 148
uid: 57913748-7681-4dbc-8ba8-93a44c3b5fd2
title: An arrival that cannot move keeps its level line apart from every other line
type: bug
status: backlog
milestone: later
created: 2026-11-25
updated: 2026-11-25
priority: p2
risk: low
surface: none
area: layout
effort: m
related:
- 33
---

## Problem

`untie` in `crates/horsegirl-core/src/layout/route.rs` moves an arrival by 6 px or 12 px inside its box. When no free height exists, it marks the step `free` and continues without a report. Found in the review of item 0033.

- A node lower than 24 px has `reach = height / 2 - CLOSE` less than one `unit`. Four nodes of 60 x 19 with the links `(0,2) (0,3) (1,2) (1,3)` give two links that share the line `y 52.5` from `x 86` to `x 98`.
- `Graph { nodes: [60 x 22], links: [(0, 0)] }` gives a loop that starts above its box. `turn_back` uses `middle - BEND_GAP` and does not read the height.
- With heights from 36 px to 116 px that have fractions, about 1 of 500 generated graphs has no free height. 5 of 3000 graphs then have two level lines of links with no end in common 2.3 px apart.
- `Node::height` states no minimum. The tests use 36 px only.

## Acceptance criteria

- [ ] `Node::height` states its minimum, or the loop and the move of an arrival are a part of the height.
- [ ] A test holds the case of `untie` where no move is possible, and no two level lines of links with no end in common are closer than `CLOSE` there.
- [ ] `no_generated_graph_has_a_fault` uses node heights with fractions and passes.
