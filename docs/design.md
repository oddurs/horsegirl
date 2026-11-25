# Design

This document states what horsegirl is and the decisions already made. The
backlog in `cairn/items` holds the plan. `docs/concept/index.html` is the
concept page: a prototype with a toy editor, every proposed picture, and the
visual system for the website. Open it in a browser.

## The idea

Write what is true. Choose how to look.

A horsegirl file states facts about a system: things, the links between them,
what is inside what, and who owns what. A lens turns those facts into a picture.
The same file draws as a flow, a sequence, lanes, a matrix, and more. No other
diagram language separates the two.

The first users are agents. An agent writes the file, the tool checks it and
names each fix, the tool reads the picture back as text, and a person looks only
at the end.

## The language

The whole language is the card in `crates/horsegirl-core/src/spec.txt`. `examples/checkout.horse` is
the reference file that the tests and the documents use.

1. A name is a thing. Names can have spaces. A thing exists when it is first
   mentioned.
2. An arrow is a link. Commas fan out. Words after the colon label the link.
   `-x` is a link that must not exist. A final `?` marks a guess.
3. Braces hold what is inside (`Services { Orders, Payments }`) and what is
   known (`Payments { owner: Commerce }`).
4. A story is links in the order they happen.
5. A `show` line picks a lens.

A fact can cite its evidence: `Orders -> Postgres: save  @ src/repo.ts:17`.

## Lenses

A lens binds a field to a channel. That is the whole mechanism.

| Lens | Dials | Solver |
| --- | --- | --- |
| flow | `x: depth`, `inside: group` | layered |
| sequence | `x: thing`, `y: step` | grid |
| lanes | `x: step`, `y: owner` | grid |
| matrix | `x: to`, `y: from` | grid |
| rings | `radius: hops from focus` | radial |
| seams | `thing: owner`, `thickness: crossing links` | radial |
| weight | `inside: parent`, `area: size` | nested |
| storyline | `x: step`, `y: closeness` | its own |

Four solvers serve every lens: layered, grid, nested, radial.

## The pipeline

```
source -> parse -> model -> lens -> layout -> scene -> backend
```

- **parse**: a lossless tree that recovers after a bad line.
- **model**: things, links, groups, facts, stories, rules.
- **lens**: binds fields to channels and picks a solver.
- **layout**: places and routes. It takes earlier positions as an input, so a
  small edit moves few nodes.
- **scene**: positioned shapes. It is the only thing a backend reads.
- **backend**: SVG, text, and later a web viewer.

The core is one Rust library with no I/O. It compiles to a native binary and to
WebAssembly, and gives the same bytes in each.

## What the tool gives an agent

| Command | Returns |
| --- | --- |
| `spec` | The one-page language card |
| `check` | Every diagnostic, each with a code, a span and a fix |
| `fix` | The file with every safe fix applied |
| `describe` | The picture as text, with its size and overlap count |
| `suggest` | The lenses these facts support |
| `query` | Paths and owners |
| `draw` | SVG or text |
| `diff` | One picture of what changed between two versions |

Each command is also an MCP tool.

## Decisions

Each decision is an item of type `decision`. List them with
`cairn list --view decisions`.

- The first users are agents.
- Names have spaces and no separate ids.
- The core is Rust, compiled to native and to WebAssembly.
- Diagrams are plain by default, and colour follows meaning. The language has
  no syntax to paint one box.

## What horsegirl will not do

- Let a file place a thing by hand. A lens decides position.
- Have variables or logic. A file of facts is not a program.
- Run a server.
