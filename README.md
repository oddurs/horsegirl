# horsegirl

[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

An agent-first diagram language. An agent states the facts about a system once.
A lens turns the same facts into a flow, a sequence, a matrix, or another picture.

**Status: the start of the roadmap.** The binary prints the language card and
nothing else. The parser, the checks and the first picture are the work of
[v0.1](ROADMAP.md).

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
cargo install --path .
```

## Quickstart

```sh
horsegirl spec
```

This prints the one-page language card: the five rules an agent reads before it
writes a file.

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

MIT. See [LICENSE](LICENSE).
