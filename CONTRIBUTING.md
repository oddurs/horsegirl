# Contributing

## Set up

```sh
scripts/setup
scripts/agent doctor
```

`scripts/setup` points git at the hooks in `.githooks`. `scripts/agent doctor`
reports every problem with the machine, not only the first.

The backlog needs [cairn](https://github.com/oddurs/cairn) 1.0.0-alpha.1 or
later. Without it, the backlog check runs only in CI.

## The workflow

One unit of work is one worktree, one branch and one pull request.

1. Find work: `scripts/agent next`. Read the item: `cairn show <id>`.
2. Start: `scripts/agent take <id>`. This names the branch from the item,
   creates `../.worktrees/horsegirl/<branch>` from `origin/main`, claims the
   item and prints it. `scripts/agent start <type>/<id>-<slug>` does the same
   with a name you choose.
3. Work in that directory. Commit with `scripts/agent commit '<message>'`.
4. Tick the acceptance criteria that are now true: `cairn tick <id> <n>`.
5. Finish: `scripts/agent finish`. The command runs every check, closes the
   item, pushes, and opens the pull request with a body built from the item,
   set to merge when CI passes.
6. Land: `scripts/agent land`. It rebases when `main` has moved, pushes, and
   waits for the merge. A conflict in the generated roadmap is resolved for
   you; any other conflict stops with instructions.
7. After the merge: `scripts/agent done <branch>`.

Never commit to `main`. A hook refuses the push, and branch protection refuses
it on the server.

## Branches and commits

Branch names match `<type>/<slug>`, where the type is one of `feat`, `fix`,
`chore`, `docs`, `perf`, `refactor`, `test`. Start the slug with the item
number.

Commits follow [Conventional Commits](https://www.conventionalcommits.org).
The subject is imperative, at most 72 characters, with no trailing period. The
body says why. Add a `Refs: <id>` trailer for the backlog item.

A pull request is squashed, so its title becomes the commit on `main`. Write
the title as a Conventional Commit.

## Checks

```sh
scripts/task check
```

This runs the format check, the lints with warnings denied, the tests and the
build. CI runs the same command, so a branch that passes locally passes in CI.

The site in `site/` needs the Node version in `site/.node-version` and the pnpm
version in `site/package.json`. Without pnpm, `scripts/task` skips the site
verbs locally. CI always runs them.
Never use `--no-verify`.

## Golden tests

The parser and its diagnostics are contracts. The golden tests hold them as
text in `crates/horsegirl-core/tests/golden`. A case is one `.horse` file.
Beside it, `<case>.tree` holds the tree and `<case>.diagnostics.json` holds the
diagnostics.

To add a case, write the `.horse` file. To accept an intended change, or to
write the expectations of a new case, run:

```sh
HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test golden
```

The command writes every expectation and removes the expectations of a case
that is gone. Read the diff before you commit it: the diff is what a reviewer
reads.

## Review

This repository has one maintainer, so a pull request needs zero approvals and
one green check. State in the description what a reviewer should look at
sceptically.

## Licence

The project is licensed under MIT or Apache-2.0, at the user's option. A
contribution that you submit is licensed the same way, with no additional
terms.

## Attribution

Commits, pull requests, comments and documents name no assistant and no model.
The commit hook refuses a message that does.

Project conventions are in [docs/conventions.md](docs/conventions.md).
