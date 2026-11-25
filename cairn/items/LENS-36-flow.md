---
id: 36
uid: 2bafa221-7bb2-44a5-b915-904571bee5f9
title: flow
type: lens
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 31
- 32
- 33
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: lens
track:
- pictures
---

## The question this picture answers

What depends on what.

## Dials

`x: depth` · `inside: group` · `colour: owner`

## Facts it needs

Links. With no links the compiler says that there is nothing to order.

## Solver

Layered.

## Example

```
Web app, Mobile app -> API gateway
API gateway -> Orders
show flow
```

## Acceptance criteria

- [x] The lens draws the example with no overlap.
- [x] Missing facts produce a diagnostic that names them.
- [x] The lens is in the language reference with its dials.
- [x] The lens is declared as data (field to channel), not as code paths in the solver.

## 2026-11-25

From the review of item 23: `show sequence of S, T` parses as two things with no message, and a `show` line whose argument needs quotes is not read as a `show` line. BUG-136 holds the inputs and the criteria. The flow lens must not rely on the parser to reject such a line until BUG-136 is done.

## 2026-11-25

From item 0032: the layered layout is horsegirl_core::layout::layout. It reads a Graph of boxes with sizes, groups and links, and knows nothing of the model. fn flow in crates/horsegirl-core/tests/layout.rs turns a Model into that Graph for the tests: a thing that holds something is a group, every other thing is a node as wide as its name at 14 px plus 28 px. The lens owns that step and the sizes.

## 2026-11-25

From item 0033: horsegirl_core::layout::layout now routes. Give Graph.labels the measured size of each link label, and read Layout.paths and Layout.labels into the Placement. fn scene and fn flow_of in crates/horsegirl-core/tests/layout.rs do both for the tests. Run horsegirl_core::overlap::overlaps on the scene to hold 'no overlap'.

## 2026-11-25

Criterion 1: ran 'scripts/task check' in the worktree, exit 0. The test the_example_draws_with_no_overlap passed natively (nextest) and on wasm32-wasip1: lens::draw(model of the example, &FLOW) gives 4 things and 3 links, and horsegirl_core::overlap::overlaps returns no collision of any kind. the_reference_file_draws_with_no_overlap holds the same for examples/checkout.horse. Drew both to SVG through lens::draw and looked at them in Chromium (webshot): the example reads Web app and Mobile app, then API gateway, then Orders, with no box or line on another. On checkout.horse the link labels sit on their own level lines and read as struck through: filed BUG-152.

## 2026-11-25

Criterion 2: check::needs gives E0301 for each show line whose lens needs a fact that the file does not state. 'Web app, Mobile app\nshow flow\n' gives one E0301 on the span 'flow' with the message: '`show flow` orders things by their links, and this file has no link, so there is nothing to order. Add a link with `->`, as in `Web app -> API gateway`.' Tests a_file_with_no_link_gives_an_error_that_names_links, a_rule_is_not_a_link_for_the_flow_lens and a_file_with_a_link_or_with_no_show_line_needs_nothing passed in scripts/task check, natively and on wasm32-wasip1. E0301 is in codes::CATALOGUE and docs/diagnostics.md.

## 2026-11-25

Criterion 3: docs/lenses.md is the language reference page for lenses; the site builds it as /docs/lenses (checked the table in site/out/docs/lenses/index.html). The table between the lenses markers is lens::reference_markdown(), and the test the_reference_lists_each_lens_with_its_dials holds the two equal and checks that the page shows 'x: depth', 'inside: group', 'colour: owner' and the example. Item 0043 writes the full reference and can take this page in.

## 2026-11-25

Criterion 4: horsegirl_core::lens::FLOW is a const Lens: name, question, dials [x: depth, inside: group, colour: owner], needs [links], solver layered. lens::bind reads the dials to build the layout Graph; horsegirl_core::layout has no change and knows no lens. lens::draw sets Scene::colour from the colour dial, and the SVG backend now colours by that key instead of the literal 'owner'. Tests: without_the_inside_dial_the_solver_gets_no_group (a lens with only x: depth gives the solver no group), the_colour_dial_picks_the_fact_that_colours_a_thing (colour: team gives slot-2 to B where colour: owner gives slot-1 to A), the_layered_solver_shows_every_dial_of_flow_and_refuses_x_by_group. All passed in scripts/task check. tests/layout.rs now binds through lens::bind and layout.positions did not change. Limit: a lens without inside: group still gets a thing that holds others drawn with the group role, because scene::build reads model.groups(); no layered lens drops the dial today.

## Result

horsegirl_core::lens::FLOW declares the flow lens as data: dials x: depth, inside: group, colour: owner, needs links, solver layered. lens::draw binds a model through the dials, runs the layered layout and gives a scene with no collision on the example and on examples/checkout.horse. show flow with no link gives E0301. docs/lenses.md lists the lens from the same constant.
