---
id: 31
uid: 94fbc561-d5e3-40b7-9d64-3c3720b105e2
title: 'The scene: positioned shapes that every backend draws'
type: feature
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 25
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: render
track:
- pictures
---

## Problem

Each backend must draw the same picture. That needs one description of positioned shapes that every backend reads and no backend changes.

## Proposal

Define the scene: boxes, lines with bend points, labels, groups, each with a position, a size, a role (thing, link, group, label) and the facts that style reads, such as owner.

## Acceptance criteria

- [x] The scene serialises to stable JSON, held by a snapshot test.
- [x] A scene carries enough to draw the reference flow with no access to the model.

## 2026-11-25

Criterion 1: scripts/task check exited 0. cargo nextest printed PASS for horsegirl-core::scene the_reference_scene_serialises_to_its_json, which compares Scene::to_json() for the reference file byte for byte with crates/horsegirl-core/tests/checkout.scene.json, and the_same_input_gives_the_same_json. The wasm32-wasip1 run printed 'test result: ok. 7 passed' for tests/scene.rs, so the same bytes come out in WebAssembly. Numbers are rounded to two decimals; facts are ordered pairs, not a JSON object, so a repeated key survives.

## 2026-11-25

Criterion 2: the_reference_scene_draws_with_no_access_to_the_model passed natively and in WebAssembly. A miniature SVG backend in the test takes only &Scene and draws 10 rects (9 things, 1 group, the group first), 9 polylines and 17 texts. The test checks that every thing name, every link label and an owner-<value> class for each of the 5 owners appear in the SVG, and that each model link has a line with its two ends. The Scene holds owned strings and no reference into the model. The placement in that test is a stand-in for item 32: a column for each rank and one bend per link.

## 2026-11-25

Choices to revisit: a link with several labels shows one label 'a, b'; a group name sits GROUP_INSET (8 px) inside the top-left corner of the group, so the layout must leave that room; links carry no facts because the model gives links none yet; the arrow head is implied at the last point of a link.

## Result

The scene lives in crates/horsegirl-core/src/scene.rs: Scene { width, height, font_size, shapes } with each Shape holding a role (thing, group, link, label), the names it draws (of), a rect, the points of a link, the text of a label and the facts. A layout returns a Placement (a rect per thing, points per link, a label centre per link) and scene::build joins it with the model. Scene::to_json is the stable form; tests/checkout.scene.json holds the reference.
