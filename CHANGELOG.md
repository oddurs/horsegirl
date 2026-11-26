# Changelog

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The flow lens draws a link that starts or ends at a group, as in
  `API gateway -> Services`. The link meets the border of the group box and
  enters no box inside it, and the group ranks as one unit for the link.
  `horsegirl_core::layout::Graph` names each end of a link as an `End`: a
  node or a group.
- `W0110`: a link between a group and itself or a thing inside it. The flow
  lens leaves such a link out.
- `W0111`: a link or a rule that ends in a bare name with a `?` against it, as
  in `A -> Archive?`. The line states a firm link to `Archive?`. The safe fix
  adds the space and makes the link a guess. A name that the file also writes
  with its `?` elsewhere, as in `Archive? -> Store`, gives no warning. `fmt`
  and `corpus/convert.py` keep the quotes on a name that ends in `?`.
- `horsegirl mcp` serves the tools `spec`, `check`, `fix` and `draw` over MCP
  on standard input and standard output. Each tool takes the whole text of a
  file. `check`, `fix` and `draw` return `structuredContent` with an
  `outputSchema`, and a diagnostic has the shape of the `--json` line.
  `horsegirl mcp --config` prints the entry for the configuration of a client.
  `docs/mcp.md` states each shape. The tool schemas are committed in
  `crates/horsegirl/tests/golden/tools.json`, and a test holds the server to
  them for clients of `2025-11-25` and of `2026-07-28`.
- A version tag publishes static binaries for Linux and macOS on x86_64 and
  aarch64, with `SHA256SUMS` and a build attestation. `install.sh` installs
  the binary to `~/.local/bin` and refuses an archive whose checksum does not
  match.
- The first benchmark, in `bench/` and the `horsegirl-bench` crate: 100 tasks,
  each a description of a system and a checker. `horsegirl-bench run` asks a
  model through any command, and `horsegirl-bench score` scores recorded
  replies with no network. The report gives validity on the first try, after
  one `fix`, and against the checker, and the diagnostics hit most often.
  `horsegirl_core::check::fix_all` applies every safe fix as `horsegirl fix`
  does.
- `docs/quickstart.md` and `docs/reference.md`: the way to a first picture
  and the five rules, each command and each code. A test holds every
  example of both pages to a golden case.
- `horsegirl check [--json] [FILE]` prints every diagnostic of a file, as text
  or as one JSON object per line. `horsegirl fix [--json] [FILE]` applies every
  safe fix in place, changes no other byte, and prints what is left.
  `horsegirl draw [--to svg] [-o OUT] [--json] [FILE]` writes the picture
  beside the file or to `OUT`, and refuses a file with an error. Each command
  exits 0 when the file has no error, 1 when it has one, and 2 for a usage
  mistake. `horsegirl_core::diagnostic::apply_safe` makes the safe fixes.
- `horsegirl fmt` prints the errors of a file in the text form of a diagnostic.
- `E0010`: a link with a second arrow, as in `A -> B -> C`. It was `E0001`
  with no fix. The fix writes `A -> B` and `B -> C` on two lines with the same
  indent. The fix is safe when the line ends at the last name. When the line
  ends in a label, a `?` or a citation, the fix gives that end to each link and
  is suggested, because the parser cannot know which link it belongs to.
- `E0420`: a name in braces that the model does not place in their group. A
  thing in a second group names the group that holds it. A group placed
  inside itself, and a part in the braces after several names, each say why.
  Before, the model dropped the placement with no diagnostic. The suggested
  fix removes the name when the line names other things and says nothing
  else. It is never safe, because only the writer knows whether the two
  mentions mean one thing.
- `W0108`: a name that ends in `/` followed by braces that hold one bare name,
  as in `GET /users/{id}`. The parser reads a group; the warning suggests
  writing the whole text in double quotes.
- `W0109`: a label that holds an arrow, a name and a colon, as in
  `A -> B: pay -> C: ship`. The parser reads one link with a long label; the
  warning suggests writing the second link on its own line. A label with an
  arrow and no colon after it, and a label in double quotes, give no warning.
- `horsegirl fmt [--check] [FILE]` and `horsegirl_core::format::format` print a
  file in one canonical form: one statement on each line, one space around an
  arrow, the facts of a statement in the order of their keys, and quotes only
  on a name that needs them (ADR-119). Comments stay. `--check` exits 1 when
  the file would change. A file with an error has no canonical form, so `fmt`
  names each error and exits 1. A bare name that needs quotes and holds a `"`
  has no canonical form either: `fmt` names it and exits 1, and
  `horsegirl_core::format::unquotable_name` gives it.
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
  each and there is no fix. `E0412` leaves sibling names alone: names that differ only in a final number or one-letter word (`Service A`, `Service B`), and two names shorter than three characters. `W0107`: a thing that no link and no rule names. `E0500`: a link
  that breaks a `-x` rule. `N0200`: a guess. `E0300`: a `show` line with an
  unknown lens; the message lists the lenses, and a lens one edit from a known
  lens gets a safe fix.
- `horsegirl_core::svg::draw` draws a scene as one SVG document with a `title`
  and a `desc`, no script and no external resource. Each colour and the font
  is a CSS variable with a fallback. The fallbacks are the plain theme, in
  light and dark mode: twelve colour tokens (`--hg-ink`, `--hg-paper`,
  `--hg-line`, `--hg-muted`, `--hg-accent`, `--hg-accent-ink`, `--hg-c0` to
  `--hg-c3`, `--hg-s2` and `--hg-s3`) and `--hg-font`. `docs/theme.md` lists
  them. A page that holds the SVG inline sets them to restyle it, or sets
  `--hg-scheme` to `light` or `dark`. Each shape has the class of its role
  (`thing`, `group`, `link`, `label`) and one class for each fact, such as
  `owner-commerce`. When the scene names a colour fact, as the flow lens names
  `owner`, a thing with that fact also has `slot-1`, `slot-2` or `slot-3`,
  the palette slot of its value. Each label has `textLength` set to its
  measured width and `lengthAdjust="spacingAndGlyphs"`, so a face other than
  the measured one draws the label at the same width. A label with a
  character outside the metrics table has neither attribute and keeps its
  natural width.
- The parser reads habits from Mermaid and says what it changed with the note
  `N0101`. It reads `-->` as `->`, a final `;` at the end of a line, double
  quotes around a name that does not need them, and a space before the colon
  of a label. Each note carries a safe fix. A quoted name stays quoted when a
  bare name would join the next character, as in `"a-"->B`.
- `horsegirl_core::overlap::overlaps` reads a scene and gives each collision
  in it: two boxes that overlap, a link that crosses the box of a thing, a
  link that passes through a group that holds neither of its ends, a link that
  crosses the border of its own group more than once, two links with no end
  in common that lie on each other, a label on the box of another thing, a
  label across the border of a group, a link label that is not on its link,
  and two labels that overlap.
- The flow lens. `horsegirl_core::lens::FLOW` declares it as data: the
  question it answers, its dials (`x: depth`, `inside: group`,
  `colour: owner`), the facts it needs and its solver. `lens::draw` reads the
  dials, places the model with the layered layout and gives the scene.
  `docs/lenses.md` lists each lens with its dials, from the same data.
- The error `E0301`: a `show` line picks a lens that needs facts the file does
  not state. `show flow` in a file with no link gives it, because there is
  nothing to order.
- `Scene::colour` names the fact whose value picks the palette slot of a
  thing, and the scene JSON holds it as `"colour"`. The flow lens sets it to
  `owner`.

### Changed

- A link line stops 3 px before the label of its own link and goes on after
  it, so the line no longer runs through the text. The SVG backend draws a
  link as a `<path>` in place of a `<polyline>`, with one `M` for each part
  of the line. The label keeps its place and its colour.
- Two link labels between the same two ranks no longer cover each other. A
  label that would cover another moves along its upright line, or onto a
  level line of its link, and the room between the two ranks grows when the
  label needs it. A picture with no label does not change.
- A label keeps its white space. The style rule for `.label` sets
  `white-space: pre`, so the drawn text has the width that the measure counts
  and `textLength` no longer stretches a label with leading, trailing or
  repeated spaces.
- A `{` in the label or the value of a bad line is a mark of that line, so
  the lines after it are read as their own facts. Before, `A -> B: uses
  {braces` held the rest of the file in its error. Such a `{` still opens a
  block when it closes on its line or ends the line. A `{` with a part after
  it, as in `A -> B: x { C`, is a mark, so the `}` on a later line gives
  `E0003`. A `{` in any other place of a bad line opens a block, as in
  `G@x { A` and `G@x, H { A`.
- A group that nests past 64 levels gives one `E0007`, and the parser reads
  the group to its own `}`. Each `}` after it closes the group it belongs to,
  so 70 nested groups give no `E0003`.
- The note `W0010` comes only for a story line that reads to its end. In
  `story s {} x` the line is an error, and it gives no note.
- An unclosed double quote in a citation path (`A -> B @ "src/a.rs:3`) gives
  `E0005` and no `Cite` node. A path that holds an unpaired double quote is no
  citation.
- The language card states the comment form: a line that starts with `#` is a
  comment (ADR-130). The parser reads comments as before.
- A missing name at the end of a line (`E0006`) has an empty span at the end of
  the line, and not the line break.
- `A -> B:` and `A -> B: ?` give `E0006`: a link with a colon needs a label.
  Before, the parser accepted both with no diagnostic.
- When two fixes of one parse overlap, the outer one keeps its fix and the
  inner one has none, so a tool can apply every fix of one run.
- `E0004` corrects a text in one step in more places. `Sensors: wind, B -> Hub`
  gets the quotes, as `A, Sensors: wind -> Hub` does. A value runs on past a
  comma until a fact follows, so `note: GET /users/{id}, list` gets one fix
  for the whole value. The name after an arrow gets the fix that the name
  before one gets: `A -> GET /users/{id}`.
- Braces in a name that needs quotes are characters of the name only when
  they hold one word and no space, as in `{id}`, and an arrow is on the line.
  Other braces are a group, so `G{ A, B } -> C` and
  `Services { Orders } -> Postgres` give `E0001` at the arrow. The rule holds
  for a name with a colon too: `a: b{c} -> D` gets `"a: b{c}" -> D`. On a line
  with no arrow, all braces after a name are a group, so
  `Team@x{ A }` and `Team@x {A}` get the quotes around `Team@x` only. Before,
  the fix put such a group inside the quotes and lost its parts.
- A bare name, value or label can hold a double quote, as in `x"y" -> A`.
  A text in double quotes cannot hold one. So a text that holds a double
  quote and also a character that needs quotes, as in `x"y"@z -> A` or
  `say "hi" {x}`, gives `E0004` with no fix. The message says that a text in
  double quotes cannot hold a double quote, and no longer says to add quotes.
- `horsegirl_core::text::measure` gives the width and height of a label in
  pixels, in Arial metrics, from a table bundled in the core. It gives the same
  result on Linux, macOS and in WebAssembly. A character outside the table
  measures as one em.
- `horsegirl_core::layout::layout` places the boxes and the groups of a flow
  in ranks from left to right. A link goes to a higher rank, and the link that
  closes a loop is the one that turns. Two boxes of one rank are at least
  24 px apart, and a group box holds its own boxes and no other. It takes an
  earlier picture as a seed and does not read it yet.
- The layout routes each link as a path at right angles. A path leaves the
  right side of a box, turns only between two ranks, and enters the left side
  of the next box, so it crosses no box of a thing. A link out of a group
  crosses the border of the group once. A path can still pass through a group
  that holds neither end of its link. A link from a thing to itself is a
  loop at the right side of its box. `Graph::labels` gives the size of the
  label of each link, and the layout gives the centre of each label on its
  path, with a gap between two ranks that is wide enough for the label. A
  label that would lie across the border of a group, or on the band that
  holds the name of the group, moves along its upright line to 3 px from that
  border or that band. When the line has more than one such height, the
  label takes one outside every group that holds neither end of its link.
  The paths are in `Layout::paths` and the labels in `Layout::labels`.
