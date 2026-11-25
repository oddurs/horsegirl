---
id: 132
uid: 4184876c-d493-409d-8fdd-addaab8ce969
title: A bad line with a brace keeps the lines after it, and deep braces stay paired
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: language
effort: m
related:
- 23
---

## Problem

The reviews of item 23 found two faults in how the parser recovers at a brace.

1. `A -> B: uses {braces\nC -> D\nshow flow\n` gives one `Error` node that holds a `Block` with `C -> D` and `show flow` inside. The recovery in `statement` opens a block for every `{`, also for one that does not close on its line. The model skips an `Error` node, so the two good lines are lost.
2. 70 nested groups, correctly closed, give six E0007 and six E0003. Past 64 levels the `}` of a group that was too deep closes an outer group, so the parts that follow sit in the wrong group.

## Proposal

In the recovery of `statement`, open a block only when its `}` is on the same line or the `{` ends its line. The parser already knows the `}` of each `{` on a line (`closes` in `parser.rs`). At the depth limit, skip the too-deep block as a balanced unit.

## Acceptance criteria

- [x] `A -> B: uses {braces\nC -> D\nshow flow\n` gives a tree in which `C -> D` is a `Link` and `show flow` is a `Show` at the top level.
- [x] 70 nested groups that close correctly give E0007 and no E0003, and a thing after the last `}` is at the top level.
- [x] The fuzz test passes with both inputs in its seed file.

## 2026-11-25

From the review of item 23: `story s {} x` gives E0001 for `x`, and the line is an Error node. The parser still gives W0010 for the story on that line. The model skips an Error node, so the warning names a story that does not exist. Give W0010 only when the story line parses, and add this input to the tests.

## 2026-11-25

Criterion 1: scripts/task check exits 0 on 9209f7b and prints 'PASS horsegirl-core::syntax a_bad_line_with_an_open_brace_keeps_the_lines_after_it' natively and 'test a_bad_line_with_an_open_brace_keeps_the_lines_after_it ... ok' in WebAssembly. The test asserts top-level kinds [Error, Link, Show] and the one diagnostic E0001. On the parser before the change the test failed with 'left: [Error] right: [Error, Link, Show]'.

## 2026-11-25

Criterion 2: scripts/task check prints 'PASS horsegirl-core::syntax braces_past_the_depth_limit_stay_paired'. 70 nested groups, one per line and all on one line, then 'after': diagnostics are exactly [E0007] and the top-level kinds are [Things, Things]. Before the change: six E0007 and six E0003. A new test, a_block_past_the_depth_limit_that_never_closes_is_reported, gives one E0007 and 65 E0002 for 64 groups and an unclosed H {.

## 2026-11-25

Criterion 3: the fuzz seed moved from the FORMS constant to crates/horsegirl-core/tests/fuzz-seed.horse and ends with both inputs. scripts/task check prints 'PASS horsegirl-core::fuzz no_input_makes_the_parser_panic_or_lose_a_byte'. HORSEGIRL_FUZZ_ROUNDS=300000 cargo test --release --locked --test fuzz printed 'test result: ok. 2 passed; 0 failed; finished in 182.99s'. The E0004 property changed: it counted the errors of the whole file, and now counts the errors up to the end of the fixed text. A bad line now hides the rest of its line, and a fix that makes the text good can reveal errors there. The old property failed on the soup input 'of>src/a.rs:3{"a, b""a, b"?-x{' (2 errors to 3).

## 2026-11-25

Story note from the review of item 23: a_story_line_that_fails_after_its_block_gives_no_story_note passes; 'story s {} x' gives only E0001. Before the change it gave [W0010, E0001].

## 2026-11-25

Round 1 review: criterion 2 held only for two layouts. 'G { A' repeated 70 times, then 70 closing braces, gave six E0007 and six E0003. statement now pairs the brace at which open stopped with skip_block when the error is E0007. cargo test -p horsegirl-core --test syntax braces_past: 1 passed, with the 'G { A' layout added; the same test without the parser change fails with six E0007 and six E0003. scripts/task check: exit 0.

## 2026-11-25

Review round 2: a name that needs quotes, a space and a { with a part after it (G@x { A / B / }) now keeps its group in the recovery; a { after a label stays a stray mark and its later } gives E0003, held by a test and stated in the changelog. scripts/task check exited 0: 297 tests run, 297 passed, including a_name_that_needs_quotes_keeps_its_group_across_lines and the fuzz test with G@x { A in the seed. The new test failed without the parser change (left [Error, Things, Error, Things], right [Error, Things]).
