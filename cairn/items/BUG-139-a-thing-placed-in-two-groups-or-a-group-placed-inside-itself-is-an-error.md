---
id: 139
uid: 3fb9b40e-d2cd-40a5-bfbd-8c8c2217e216
title: A thing placed in two groups, or a group placed inside itself, is an error
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 24
- 25
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p2
risk: low
surface: diagnostics
area: check
effort: s
related:
- 27
---

## Problem

The model gives each thing one parent: the first group whose braces hold it. `G { A }` and then `H { A }` leaves `A` in `G` and `H` empty, with no diagnostic. `A { B }` and then `B { A }` would make a loop, so the model drops the second placement, also with no diagnostic. `A, B { C }` places `C` in `A` only. An agent that writes one of these lines learns nothing.

## Proposal

Report the placement that the model drops. Give it a stable code, the span of the name in the later braces, and a message that names the group that holds the thing already. Where removing the later name is safe, the fix removes it.

## Acceptance criteria

- [x] `G { A }` then `H { A }` gives one error at the second `A` that names `G`.
- [x] `A { B }` then `B { A }` gives one error at the second `A` that says the group would hold itself.
- [x] `A, B { C }` gives one error that says a part belongs to one group.
- [x] The reference file gives no such error.

## 2026-11-25

Evidence: cargo test -p horsegirl-core --test check (34 passed) includes a_thing_in_a_second_group_is_an_error_that_names_the_first, a_group_placed_inside_itself_is_an_error, a_part_in_the_braces_after_several_names_is_an_error and the_reference_file_and_sound_groups_give_no_misplaced_part. horsegirl check on 'G { A } / H { A } / A -> B' printed one E0420 at 2:5: 'A is in the group G already'. On 'A { B } / B { A } / A -> C' one E0420 at the second A: 'would make the group A hold itself'. On 'A, B { C } / C -> D' one E0420: 'a part belongs to one group'. On examples/checkout.horse only W0010. scripts/task check exit 0: 297 tests passed, corpus 174 of 239 (baseline lowered from 176; 0031 and 0207 drew clean only because the second group was dropped; item 165 filed).

## 2026-11-25

Fix round 1: E0420 fix is now suggested; fix_all leaves corpus 0197 and the E0412 chain case unchanged (test fix_leaves_a_thing_in_two_groups_for_the_writer). Shared message reads the final parent (test a_part_after_several_names_names_the_group_that_holds_it). Merged main; cargo run -p horsegirl --example corpus printed 181 of 239 (baseline 181). scripts/task check exited 0.
