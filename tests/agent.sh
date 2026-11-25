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

# take and finish need the backlog tool. CI has it; a contributor may not.
if command -v cairn >/dev/null 2>&1; then
  cp "$src/cairn.toml" .
  mkdir -p cairn/items scripts docs
  cp "$src/docs/roadmap-intro.md" docs/
  printf '#!/bin/sh\nexit 0\n' >scripts/task
  chmod +x scripts/task
  body='## Problem\n\nNothing reads a file.\n\n## Proposal\n\nRead it.\n\n## Acceptance criteria\n\n- [ ] It reads a file.\n- [ ] It says so.\n'
  printf '%b' "$body" | cairn new "Read a File, please!" -t feature -s planned -q --no-hooks --stdin >/dev/null
  cairn render -q
  git add -A
  git commit -q -m "chore: add a backlog"
  git push -q origin main

  # A stand-in for gh that records how it was called.
  mkdir "$tmp/bin"
  cat >"$tmp/bin/gh" <<'GH'
#!/bin/sh
echo "$*" >>"$GH_LOG"
case "$1 $2" in
  "pr view") exit 1 ;;
  "pr create")
    while [ $# -gt 0 ]; do
      [ "$1" = "--body-file" ] && cp "$2" "$GH_LOG.body"
      shift
    done
    echo "https://example.invalid/pr/1"
    ;;
esac
GH
  chmod +x "$tmp/bin/gh"
  GH_LOG=$tmp/gh.log
  export GH_LOG
  PATH=$tmp/bin:$PATH

  prompt=$(scripts/agent take 2>/dev/null)
  wt=$tmp/.worktrees/repo/feat/0001-read-a-file-please
  [ -d "$wt" ] || fail "take did not create the worktree named from the item"
  echo "$prompt" | grep -q "It reads a file" || fail "take did not print the item as a prompt"
  grep -q '^assignee:' "$wt"/cairn/items/0001-*.md || fail "take did not claim the item"

  cd "$wt"
  echo change >feature.txt
  git add -A
  git commit -q -m "feat: read a file"
  out=$(scripts/agent finish 2>&1) && fail "finish accepted an item with unticked criteria"
  echo "$out" | grep -q "It says so" || fail "finish did not name the unticked criterion"
  cairn tick 1 --all >/dev/null
  scripts/agent finish >/dev/null 2>&1 || fail "finish failed on a complete item"
  grep -q '^status: done' cairn/items/0001-*.md || fail "finish did not close the item"
  [ -z "$(git status --porcelain)" ] || fail "finish left uncommitted changes"
  grep -q "Nothing reads a file" "$GH_LOG.body" || fail "the pull request body lacks the problem"
  grep -q "^Refs: 0001" "$GH_LOG.body" || fail "the pull request body lacks the Refs line"
  grep -q "Acceptance criteria" "$GH_LOG.body" && fail "the pull request body repeats the criteria"
  grep -q "pr merge --auto --squash" "$GH_LOG" || fail "finish did not set the pull request to merge itself"

  # The default branch moves and touches the generated roadmap. sync must end
  # with a finished rebase, a current roadmap and a clean tree.
  cd "$tmp/repo"
  printf '%b' "$body" | cairn new "A second item" -t feature -s planned -q --no-hooks --stdin >/dev/null
  cairn render -q
  git add -A
  git commit -q -m "docs: add a second item"
  git push -q origin main
  cd "$wt"
  scripts/agent sync >/dev/null 2>&1 || fail "sync failed when the default branch had moved"
  git merge-base --is-ancestor origin/main HEAD || fail "sync reported success without a finished rebase"
  [ -z "$(git status --porcelain)" ] || fail "sync left uncommitted changes"
  cairn check --render --strict >/dev/null 2>&1 || fail "sync left a stale roadmap or an invalid backlog"
  [ -f feature.txt ] || fail "sync lost the branch's work"

  echo dirty >>feature.txt
  scripts/agent sync >/dev/null 2>&1 && fail "sync accepted a dirty working tree"
  git checkout -q feature.txt
  cd "$tmp/repo"
elif [ -n "${CI:-}" ]; then
  fail "cairn is required in CI, and it is missing"
else
  echo "tests/agent.sh: cairn not found; take, finish and sync were not tested" >&2
fi

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
