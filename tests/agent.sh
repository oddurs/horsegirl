#!/bin/sh
# Exercises scripts/agent and the commit-msg hook against a scratch clone with
# a bare origin. Run through scripts/task test.
set -eu
src=$(cd "$(dirname "$0")/.." && pwd)

# A hook sets GIT_DIR and friends; they must not leak into the scratch repos.
for var in $(env | sed -n 's/^\(GIT_[A-Z_]*\)=.*/\1/p'); do
  unset "$var"
done

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail() {
  echo "tests/agent.sh: FAIL: $*" >&2
  exit 1
}

git init -q --bare "$tmp/origin.git"
git init -q -b main "$tmp/repo"
cd "$tmp/repo"
git config user.name test
git config user.email test@example.invalid
mkdir scripts
cp "$src/scripts/agent" "$src/scripts/attribution-patterns" scripts/
echo tracked >TRACKED
git add -A
git commit -q -m "chore: initial commit"
git remote add origin "$tmp/origin.git"
git push -q -u origin main
git remote set-head origin main

echo local >LOCAL.md
mkdir .localdir
echo inner >.localdir/file
printf '%s\n' '# comment' /LOCAL.md /.localdir '/*.glob' /absent /TRACKED >.git/info/exclude

scripts/agent start Bad/Name >/dev/null 2>&1 && fail "an invalid branch name was accepted"
scripts/agent start chore/links >/dev/null

wt=$tmp/.worktrees/repo/chore/links
[ -L "$wt/LOCAL.md" ] || fail "a literal file was not linked"
[ -L "$wt/.localdir" ] || fail "a directory was not linked"
[ "$(cat "$wt/.localdir/file")" = inner ] || fail "the linked directory does not resolve"
[ ! -e "$wt/absent" ] || fail "an absent path was created"
[ ! -L "$wt/TRACKED" ] || fail "a tracked file was replaced by a link"
[ -z "$(git -C "$wt" status --porcelain)" ] || fail "the new worktree is not clean"
scripts/agent start chore/links >/dev/null 2>&1 && fail "an existing branch was accepted"

git worktree remove "$wt"
[ -f LOCAL.md ] && [ -f .localdir/file ] || fail "removing the worktree removed the primary's files"

hook=$src/.githooks/commit-msg
msg=$tmp/msg
accept() {
  printf '%s\n' "$1" >"$msg"
  (cd "$src" && sh "$hook" "$msg") >/dev/null 2>&1 || fail "rejected a valid message: $1"
}
reject() {
  printf '%b\n' "$1" >"$msg"
  (cd "$src" && sh "$hook" "$msg") >/dev/null 2>&1 && fail "accepted an invalid message: $1"
  return 0
}
accept "feat(parser): read fan-out links"
accept "fix!: refuse a name that contains an arrow"
reject "Added the parser"
reject "feat: ends with a period."
reject "feat: $(printf 'x%.0s' $(seq 1 80))"
reject "feat: add the parser\n\nCo-authored-by: Claude <noreply@anthropic.com>"

echo "tests/agent.sh: ok"
