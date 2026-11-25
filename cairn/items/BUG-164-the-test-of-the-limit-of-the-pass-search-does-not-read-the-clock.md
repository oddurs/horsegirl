---
id: 164
uid: c1743a5f-9eb4-40cf-a9fa-f27ff4666367
title: The test of the limit of the pass search does not read the clock
type: bug
status: backlog
milestone: v0.1
created: 2026-11-25
updated: 2026-11-25
priority: p1
risk: low
surface: none
area: layout
effort: s
related:
- 146
- 155
---

## Problem

`a_dense_graph_stops_the_pass_search_at_its_limit` in `crates/horsegirl-core/tests/layout.rs` measures the `layout` call with `std::time::Instant` and fails above 120 s. The test does not show that `keep_out` stopped at `LIMIT`. A `LIMIT` ten times higher passes the test. A slow runner can fail the test with correct code.

## Reproduction

1. In `crates/horsegirl-core/src/layout/through.rs`, set `LIMIT` to `10_000_000`.
2. Run `cargo test -p horsegirl-core --test layout a_dense_graph`.
3. The test passes.

## Proposal

Let `keep_out` report whether the search reached the limit, or the work count, to a test in the crate. Assert on that value. Remove `Instant` from the test.

## Acceptance criteria

- [ ] The test fails when `LIMIT` is `10_000_000`.
- [ ] The test does not use `std::time`.
- [ ] `tests/layout.positions` does not change.
