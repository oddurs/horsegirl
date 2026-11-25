---
id: 17
uid: 3c31376a-7080-4be4-b372-e90b7a3963b4
title: Do names with spaces survive real diagrams?
type: spike
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: language
effort: m
area: language
track:
- language
---

## Question

Can a grammar with spaces in bare names parse real diagrams without ambiguity?

## Why it matters

The parser, the formatter and the typo check all rest on this rule. If names need quotes often, ADR 'Names have spaces' must change before the parser is written.

## Time box

Three days.

## Answer

Mostly yes. Bare names with spaces hold 89.5% of the names in 239 real
Mermaid flowcharts. They do not hold the other 10.5%, which is more than the
"few percent" that ADR-13 named as its limit. Keep bare names as the normal
form, and add one quoted form for a name that holds a reserved character.
Do not go back to ids. Ids would solve the 10.5% by adding an id to every
one of the 3052 names.

### Evidence

The corpus is in `corpus/`: 239 flowcharts from 187 public repositories, each
under MIT, Apache-2.0, CC-BY-4.0, CC0-1.0 or BSD-3-Clause. `corpus/manifest.tsv`
names the repository, the path, the commit, the licence and a permanent URL for
each file. `python3 corpus/measure.py` reads every flowchart and prints the
numbers below. It understood every line of the corpus.

A name is the label of a Mermaid node, or its id when it has no label, or the
title of a subgraph. The corpus has 3052 names: 2693 nodes and 359 subgraph
titles. A converter rewrites line breaks to spaces and removes HTML tags and
icons before it measures a name, because that rewrite is safe.

320 names (10.5%) need quotes. They occur in 81 of the 239 flowcharts (33.9%).
A name can have more than one cause. The five most common causes:

| Cause | Names | Share of all names | Shortest examples |
| --- | ---: | ---: | --- |
| colon | 163 | 5.3% | `:8000 API`, `up:Disease`, `包格式: .jupk` |
| comma | 159 | 5.2% | `true, true`, `false, true` |
| empty label | 10 | 0.3% | `" "`, used to draw an invisible node |
| `->` inside the name | 6 | 0.2% | `apply trial -> new current` |
| `@` inside the name | 4 | 0.1% | `Renderer PixiJS / @pixi/react` |

Braces (4 names) and the keyword `show` as a whole name (1) are the only other
causes. No name contains `-x`, and no name ends in ` ?` with a space before
the mark.

### What the numbers mean

- Colon and comma are nearly the whole problem: 301 of the 320 names hold one
  or both. Every other cause is under 0.5% of names.
- 169 of the 320 names are a title and a description joined by a line break,
  for example `api<br/><sub>django :8000</sub>` or `wally<br/>install, list`.
  When the name is
  only the first line, 187 names (6.1%) still need quotes. A converter that
  keeps the first line as the name and the rest as a fact halves the problem.
  It does not remove it.
- Only 27 of the colon names have no space after the colon, as in
  `up:Disease`. A rule that only `: ` with a space starts a label would save
  those 27 and no more. That is not worth a subtle rule.
- 56 names (1.8%) end in `?` with no space, mostly decisions such as
  `Is valid?`. Item 26 already reads only ` ?` with a space as a guess, so
  these names stay bare. That rule must hold.
- 694 names (22.7%) hold a line break and 101 (3.3%) hold HTML or an icon. A
  converter rewrites these without quotes.
- 93 nodes (3.5%) in 14 flowcharts share a label with another node of the
  same flowchart: `true`, `false`, `Description`, and the empty label. Mermaid
  keeps them apart by id. Names as identity merges them, so a converter must
  give them distinct names.
- The typo check E0412 as item 27 states it fires on 23 labelled nodes (0.9%)
  in 10 flowcharts. None is a typo. Each one is a sibling such as `Service A`
  and `Service B`, `Request A` to `Request E`, `Deploy EC2 #1` and
  `Deploy EC2 #2`, a plural such as `Agents` beside `Agent`, or a
  one-character name.
- Link labels are safe. Of 788 link labels, none ends in ` ?` and none holds `@`.

### Limits of the evidence

- GitHub code search ranks its results in its own way, and the sample is
  the first repositories after a shuffle with a fixed seed. It is not a random
  sample of all flowcharts.
- Several repositories write long generated labels. The share of names is
  therefore weighted towards them; the share of flowcharts (33.9%) is the more
  stable figure.
- `measure.py` is a reader written for this spike, not Mermaid's own parser.
  It understood every line of this corpus, but it is not a conformance test.
- The authors of these flowcharts wrote Mermaid, not horsegirl. An agent that
  writes from the card can avoid a colon where a Mermaid author did not. The
  10.5% is the figure for converted files; new files may need quotes less often.

### Follow-up

- ADR-119: a quoted form for names that hold a reserved character. The parser
  (item 23) now depends on it.
- Item 121: `convert` keeps the first line of a label as the name, and gives
  distinct names to nodes that share a label.
- Item 120: the typo check E0412 leaves sibling names alone.

## Acceptance criteria

- [x] The Answer section states a conclusion and the evidence for it.
- [x] Follow-up items exist for the work the answer implies.
- [x] A corpus of at least 200 public Mermaid flowcharts is in the repository under a licence that allows it.
- [x] The answer reports the share of names that need quotes, with the five most common causes.

## 2026-11-25

Corpus: corpus/flowcharts holds 239 Mermaid flowcharts from 187 public repositories, none a fork. GitHub reports MIT (191), Apache-2.0 (37), CC-BY-4.0 (5), CC0-1.0 (4) or BSD-3-Clause (2) for each. corpus/manifest.tsv gives repository, path, line, commit, licence and URL; corpus/licences holds 162 licence texts. One repository (prime-radiant-inc/evener) was removed because its only licence file, LICENSE-kilroy, looks like an upstream licence. Total size 692 KB.

## 2026-11-25

Ran: python3 -I -B corpus/measure.py. It printed: 3052 names, 0 lines not understood; 320 names need quotes (10.5%) in 81 flowcharts (33.9%); causes colon 163 (5.3%), comma 159 (5.2%), empty 10 (0.3%), arrow -> 6 (0.2%), @ 4 (0.1%), brace 4, keyword 1; first line only 187 (6.1%); E0412 would flag 23 labelled nodes in 10 flowcharts; 93 nodes share a name in 14 flowcharts. Two runs gave the same sha256 of the report.

## 2026-11-25

Follow-ups filed: ADR-119 (quoted form for names with a reserved character; 0023 now depends on it), 0120 (typo check leaves sibling names alone), 0121 (convert Mermaid labels that a bare name cannot hold). cairn check printed: ok: 117 item(s), 0 warning(s).
