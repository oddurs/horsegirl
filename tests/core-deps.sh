#!/bin/sh
# scripts/check-core-deps must reject a core dependency outside the allow list
# and accept one inside it. Run through scripts/task test.
set -eu
cd "$(dirname "$0")/.."

fail() {
  echo "tests/core-deps.sh: FAIL: $1" >&2
  exit 1
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '# none\nokcrate\n' >"$tmp/allow"
export CORE_DEPS_ALLOW="$tmp/allow"

printf 'horsegirl-core v0.0.0 (/x)\nokcrate v1.0.0\n' | scripts/check-core-deps - 2>"$tmp/err" ||
  fail "an allowed dependency was rejected"

if printf 'horsegirl-core v0.0.0 (/x)\nokcrate v1.0.0\nreqwest v0.12.0\n' | scripts/check-core-deps - 2>"$tmp/err"; then
  fail "a dependency outside the allow list was accepted"
fi
grep -q "'reqwest'" "$tmp/err" || fail "the message does not name the dependency"

unset CORE_DEPS_ALLOW
scripts/check-core-deps || fail "the current core crate does not pass"
