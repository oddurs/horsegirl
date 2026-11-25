---
id: 25
uid: 2052dca3-f995-47dc-b929-a8357ed35afa
title: Lower the tree into things, links, groups and facts
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 23
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: model
track:
- language
---

## Problem

The parser gives a tree of text. The checks and the lenses need things, links, groups and facts.

## Proposal

Lower the tree into a model: things in order of first mention, links with labels, groups from braces, facts on things and links, the `show` request. A thing exists when it is first mentioned.

## Acceptance criteria

- [x] The reference file lowers to 9 things, 9 links, 1 group and 5 owners, held by a test.
- [x] Stating a link twice gives one link.
- [x] The model never depends on hash-map order: two runs give identical debug output.

## 2026-11-25

Criterion 1: cargo nextest (scripts/task check) printed PASS for horsegirl-core::model the_reference_file_lowers_to_its_counts; the wasm32-wasip1 run printed 16 passed for tests/model.rs. The test asserts 9 things that hold nothing, 1 group (Services), 9 links and the owners Apps, Platform, Commerce, Growth, External. A group is a thing (item 123 relies on this), so model.things holds 10 entries; the criterion's 9 things are the ones that are not groups. 9 links: the story restates 'Web app -> API gateway' with the label 'POST /checkout', and that is the same link; the -x rule makes no link.

## 2026-11-25

Criterion 2: a_link_stated_twice_is_one_link passed. Five statements of A -> B (bare, quoted, extra spaces, labels x and y) give one link with labels 'x | y' and the span of the first statement. the_reference_file_keeps_order_groups_and_facts holds the reference case.

## 2026-11-25

Criterion 3: two_runs_give_the_same_model passed; it lowers a 600-line file of 200 things and the reference file several times and compares the Debug output. the_reference_file_lowers_to_its_model compares the Debug output with tests/checkout.model, and it passed natively and in WebAssembly in scripts/task check, which exited 0. The model holds no map; the BTreeMaps in the lowering are lookups and never reach output.

## 2026-11-25

Left out on purpose: guesses and citations (item 26 adds guesses to the model), rule storage (26), story order and facts inside a story (55). A placement in a second group or in a loop is dropped silently; filed BUG-139.
