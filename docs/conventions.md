# Conventions

## Code

- Make it work, make it right, make it small, in that order.
- A comment says why. The code says what.
- No `TODO` stubs, no commented-out code, no abstraction for a second caller
  that does not exist.
- `unsafe` is forbidden in this crate.
- Handle an error where you can act on it. Otherwise return it. Never discard one.
- A dependency must earn its place. Prefer the standard library.
- Output is deterministic. The same input gives the same bytes on every
  machine, so never iterate a hash map into output and never read the clock.

## Diagnostics

- Every diagnostic has a stable code: `E` for an error, `W` for a warning,
  `N` for a note.
- Every diagnostic names the line and the cause. If a safe replacement exists,
  the diagnostic carries it as a fix.
- A message is written for an agent that has never seen the language. It says
  what to do next.

## Writing

- Short sentences. One idea in each.
- Active voice. The same word for the same thing.
- Technical names are exact: `-x`, `show`, `lens`, `story`.
- A document describes what the program does today. Plans belong in the backlog.

## The language

- The language card in `src/spec.txt` is the specification an agent reads.
  A change to it is a change to the language and needs a `decision` item.
- A new piece of syntax must not lower the benchmark score.
