# Changelog

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `horsegirl_core::syntax::parse` reads a file into a lossless tree: names,
  links with `->`, rules with `-x`, braces with parts and facts, `show` lines,
  a final `?`, a citation after `@`, and names and values in double quotes. A
  line that starts with `#` is a comment. A line that it cannot read gives one
  error with a code from `E0001` to `E0009`, and the other lines still parse.
  A `story` block parses and gives the warning `W0010`: stories arrive in v0.2.
- `horsegirl_core::diagnostic` prints a diagnostic as text, with the source
  line and a mark under the span, and as one JSON object on one line.
  `docs/diagnostics.md` states the JSON shape and lists every code. A fix has a
  confidence, `safe` or `suggested`, a description and one or more edits.
- `horsegirl_core::check::check` gives every diagnostic for a file: the
  parser's and five checks on the model. `E0412`: a name written once that is
  one edit from a name written more often, with a safe fix that writes the
  other name; when two or more such names are one edit away, the message names
  each and there is no fix. `W0107`: a thing that no link and no rule names. `E0500`: a link
  that breaks a `-x` rule. `N0200`: a guess. `E0300`: a `show` line with an
  unknown lens; the message lists the lenses, and a lens one edit from a known
  lens gets a safe fix.

### Changed

- A missing name at the end of a line (`E0006`) has an empty span at the end of
  the line, and not the line break.
- When two fixes of one parse overlap, the outer one keeps its fix and the
  inner one has none, so a tool can apply every fix of one run.
- `horsegirl_core::text::measure` gives the width and height of a label in
  pixels, in Arial metrics, from a table bundled in the core. It gives the same
  result on Linux, macOS and in WebAssembly. A character outside the table
  measures as one em.
