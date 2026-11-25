#!/bin/sh
# The pnpm pinned for local runs in mise.toml must be the pnpm that CI installs
# from packageManager in site/package.json. Run through scripts/task test.
set -eu
cd "$(dirname "$0")/.."

[ -f site/package.json ] || exit 0
want=$(sed -n 's/^ *"packageManager": *"pnpm@\([^"]*\)".*/\1/p' site/package.json)
[ -f mise.toml ] || {
  echo "tests/pins.sh: FAIL: mise.toml is missing; it pins pnpm for local runs" >&2
  exit 1
}
have=$(sed -n 's/^pnpm *= *"\([^"]*\)".*/\1/p' mise.toml)
[ -n "$want" ] || {
  echo "tests/pins.sh: FAIL: site/package.json names no pnpm in packageManager" >&2
  exit 1
}
[ "$have" = "$want" ] || {
  echo "tests/pins.sh: FAIL: mise.toml pins pnpm '$have'; site/package.json names '$want'" >&2
  exit 1
}
