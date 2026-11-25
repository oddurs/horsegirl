# horsegirl

[![ci](https://github.com/oddurs/horsegirl/actions/workflows/ci.yml/badge.svg)](https://github.com/oddurs/horsegirl/actions/workflows/ci.yml)
[![license: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

An agent-first diagram language. An agent states the facts about a system once.
A lens turns the same facts into a flow, a sequence, a matrix, or another picture.

**Status: the start of the roadmap.** The binary checks a file, applies the
safe fixes, formats it and draws it as a flow in SVG. The rest of
[v0.1](ROADMAP.md) is in progress.

## Why it exists

Most diagrams are now written by an agent and read by a person. The tools were
designed for the opposite case. An agent that writes Mermaid cannot tell whether
the file parses, whether the picture overlaps, or whether the picture says what
it meant.

horsegirl is designed for that pair:

- The language is five rules, so the whole specification fits in a prompt.
- Every error names its fix, as text and as JSON.
- The tool reads a picture back as text, so an agent can check its own work.
- One file of facts draws as many pictures. A lens says what position means.
- A layout keeps its shape when one line of the source changes.

[docs/design.md](docs/design.md) holds the design. [ROADMAP.md](ROADMAP.md) holds
the plan.

## Install

There is no release yet. Build from a checkout with the Rust toolchain named in
`rust-toolchain.toml`:

```sh
git clone https://github.com/oddurs/horsegirl
cd horsegirl
cargo install --path crates/horsegirl --locked
```

## Quickstart

```sh
horsegirl spec                     # the language card: read it first
horsegirl check shop.horse         # every diagnostic, each with its fix
horsegirl check --json shop.horse  # the same, one JSON object per line
horsegirl fix shop.horse           # apply every safe fix in place
horsegirl fmt shop.horse           # print the one canonical form
horsegirl draw shop.horse          # write shop.svg beside the file
```

Each command reads standard input when the file is `-` or absent. It exits 0
when the file has no error, 1 when it has one, and 2 for a mistake in the
command line. `horsegirl <command> --help` names the options of one command.

## Development

```sh
scripts/setup            # once: git hooks, and merge support for the backlog
scripts/agent doctor     # check the machine
scripts/agent next       # what is ready to work on
scripts/agent start feat/0012-parse-links
```

Every change is one worktree, one branch and one pull request. `main` advances
only through a merged pull request. [CONTRIBUTING.md](CONTRIBUTING.md) has the
whole workflow, and [AGENTS.md](AGENTS.md) states it for agents.

The backlog lives in this repository as [cairn](https://github.com/oddurs/cairn)
items under `cairn/items`. [harrow](https://github.com/oddurs/harrow) shows it in
a terminal.

## License

Licensed under either of the [MIT licence](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

A contribution that you submit for inclusion in this project is licensed the
same way, with no additional terms.

The files under `corpus/` belong to their authors. `corpus/README.md` names the
source and the licence of each.
