---
id: 137
uid: e455b4ed-a418-4087-a298-1718d4e94d91
title: An unclosed quote in a citation is an error
type: bug
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p1
risk: low
surface: diagnostics
area: language
effort: s
related:
- 23
---

## Problem

`A -> B @ "src/a.rs:3` parses with no diagnostic. The tree holds a `Link` with the `Cite` node `@ "src/a.rs:3`. `cite_end` in `crates/horsegirl-core/src/syntax/parser.rs` takes the `Unclosed` token, which is the rest of the line, as the path. An unclosed quote gives E0005 at every other place.

A first attempt during item 23 made `cite_end` return `None` for such a path. The line then gave E0004 from `cut_short`, not E0005. A scan in `cite` for the `Unclosed` token made `a_long_line_of_quoted_names_parses_in_linear_time` fail. The fix must keep the parser linear.

## Acceptance criteria

- [x] `A -> B @ "src/a.rs:3` gives one E0005 and no `Cite` node.
- [x] `A { owner: X @ "src/a.rs:3` followed by a line with `}` gives E0005.
- [x] `A -> B @ src/"a.rs:3` gives E0005.
- [x] `a_long_line_of_quoted_names_parses_in_linear_time` and the fuzz test still pass.

## 2026-11-25

scripts/task check exit 0. New test an_unclosed_quote_in_a_citation_is_an_error asserts one E0005, no Cite node, lossless tree for the three sources. a_long_line_of_quoted_names_parses_in_linear_time (with two new pieces) and the fuzz test pass.
