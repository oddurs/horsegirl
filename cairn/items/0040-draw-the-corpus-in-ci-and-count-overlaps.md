---
id: 40
uid: c1dddc48-cda1-443a-abb3-953b10aba464
title: Draw the corpus in CI and count overlaps
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 17
- 33
- 37
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: none
effort: m
area: bench
track:
- pictures
---

## Purpose

The release gate is 100 real flowcharts with no overlap. That needs the corpus converted and a test that counts.

## Approach

Convert the corpus from the grammar spike to horsegirl by script, draw each file, and run the overlap detector. Report the count in CI.

## Acceptance criteria

- [x] CI prints how many of the corpus files draw with zero overlaps.
- [x] The job fails when the count drops below the committed baseline.

## 2026-11-25

From item 0033: the detector is horsegirl_core::overlap::overlaps(&Scene). It returns one Overlap for each collision, with a Kind. The kind LinkThroughGroup is not zero today on 2 of the 11 test diagrams (item 0146), and the kind Labels is not zero when many links have labels (item 0124), so decide which kinds the baseline counts.

## 2026-11-25

Criterion 1: scripts/task check (exit 0) printed 'corpus: 198 of 239 files draw with zero overlaps (baseline 198)' from 'cargo run --locked --quiet -p horsegirl --example corpus', which scripts/task test now runs, so the CI check job prints it. Two runs gave the same output sha256 d0f17bd1574dec40. The count includes every overlap kind. Files with each kind: LinkThroughGroup 12 (item 0146), Labels 30 (item 0124), LabelOnGroup 1, all others 0. check refuses 2 files (0036, 0151) with E0412; 0151 is the plural case of item 0153.

## 2026-11-25

Criterion 2: with BASELINE set to 199 the example printed 'corpus: 198 files draw clean, fewer than the baseline of 199...' and exited 1; with 198 it exited 0. scripts/task test runs it under set -eu, so a drop fails the check job. Also: python3 corpus/convert.py --check exited 1 after a line was added to corpus/horse/0001.horse, and 0 after regeneration; scripts/task lint runs it.

## 2026-11-25

Review round 1: the count left out pictures that drop a link (item 0123). corpus.rs now compares the Role::Link shapes of each scene with model.links.len(). cargo run --locked --quiet -p horsegirl --example corpus printed 'corpus: 176 of 239 files draw with zero overlaps and every link (baseline 176)' and 'files with a link left out: 25'. cargo test -p horsegirl --example corpus: 2 passed.
