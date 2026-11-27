#!/bin/sh
# Exercises the release flow against a scratch clone with a bare origin:
# `scripts/agent release`, a squash merge of its pull request, and
# scripts/release-tag as the release workflow runs it. Also exercises
# scripts/check-changelog. Run through scripts/task test.
set -eu
src=$(cd "$(dirname "$0")/.." && pwd)

# A hook sets GIT_DIR and friends; they must not leak into the scratch repos.
for var in $(env | sed -n 's/^\(GIT_[A-Z_]*\)=.*/\1/p'); do
  unset "$var"
done

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail() {
  echo "tests/release.sh: FAIL: $*" >&2
  exit 1
}
# Runs a command that must fail and prints what it said.
refused() {
  if out=$("$@" 2>&1); then
    fail "accepted: $*"
  fi
  printf '%s\n' "$out"
}

# The bare origin names main as its HEAD, as GitHub does, so a clone checks it out.
git init -q --bare -b main "$tmp/origin.git"
git init -q -b main "$tmp/repo"
cd "$tmp/repo"
git config user.name test
git config user.email test@example.invalid
mkdir -p scripts crates/one/src crates/two/src docs cairn/items
cp "$src/scripts/agent" "$src/scripts/attribution-patterns" "$src/scripts/commit-pattern" \
  "$src/scripts/release-tag" "$src/scripts/check-changelog" scripts/
printf '#!/bin/sh\nexit 0\n' >scripts/task
# The pinned toolchain: the scratch workspace has no other.
cp "$src/rust-toolchain.toml" .
chmod +x scripts/task
cat >Cargo.toml <<'EOF'
[workspace]
resolver = "3"
members = ["crates/one", "crates/two"]

[workspace.package]
version = "0.0.0"
edition = "2024"
EOF
for crate in one two; do
  printf '[package]\nname = "%s"\nversion.workspace = true\nedition.workspace = true\n' "$crate" >"crates/$crate/Cargo.toml"
  : >"crates/$crate/src/lib.rs"
done
cat >CHANGELOG.md <<'EOF'
# Changelog

## [Unreleased]

### Added

- The first command.
EOF
printf 'horsegirl 0.0.0\n' >docs/quickstart.md
echo target >.gitignore
cargo metadata --format-version 1 --offline >/dev/null
[ -f Cargo.lock ] || fail "the scratch workspace has no Cargo.lock"
git add -A
git commit -q -m "chore: initial commit"
git remote add origin "$tmp/origin.git"
git push -q -u origin main
git remote set-head origin main

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
    echo "https://example.invalid/pr/7"
    ;;
esac
GH
chmod +x "$tmp/bin/gh"
GH_LOG=$tmp/gh.log
export GH_LOG
PATH=$tmp/bin:$PATH
RELEASE_DATE=2026-11-27
export RELEASE_DATE

# The refusals, each before anything is created.
refused scripts/agent release | grep -q usage || fail "no version did not print the usage"
refused scripts/agent release 1.0 | grep -q "is not a version" || fail "'1.0' was not refused as a version"
refused scripts/agent release 0.0.0 | grep -q "not newer" || fail "the current version was accepted"
echo dirty >>CHANGELOG.md
refused scripts/agent release 0.1.0 | grep -q "uncommitted" || fail "a dirty tree was accepted"
git checkout -q CHANGELOG.md
git checkout -q -b feat/other
refused scripts/agent release 0.1.0 | grep -q "starts on main" || fail "a branch other than main was accepted"
git checkout -q main
git branch -q -D feat/other
[ ! -e "$tmp/.worktrees" ] || fail "a refused release created a worktree"
[ ! -s "$GH_LOG" ] || fail "a refused release called gh"

# The pull request merges as one squashed commit with the title of the
# pull request, as GitHub merges it.
merge() {
  git merge -q --squash "origin/chore/release-$1" >/dev/null 2>&1
  git commit -q -m "chore(release): v$1 (#$2)"
  git push -q origin main
}

# The release workflow checks out the pushed commit and runs release-tag.
ci() {
  rm -rf "$tmp/ci"
  git clone -q --depth 1 "file://$tmp/origin.git" "$tmp/ci"
  : >"$tmp/output"
  GITHUB_EVENT_NAME=push GITHUB_REF_TYPE=branch GITHUB_REF_NAME=main GITHUB_OUTPUT=$tmp/output \
    sh "$tmp/ci/scripts/release-tag"
}
# The other events never tag; they build, and publish only a tag.
route() {
  : >"$tmp/output"
  GITHUB_EVENT_NAME=$1 GITHUB_REF_TYPE=$2 GITHUB_REF_NAME=$3 DRY_RUN=$4 GITHUB_OUTPUT=$tmp/output \
    sh scripts/release-tag >/dev/null
  tr '\n' ' ' <"$tmp/output"
}
[ "$(route push tag v9.0.0 '')" = "build=true tag=v9.0.0 " ] || fail "a pushed tag is not published"
[ "$(route workflow_dispatch tag v9.0.0 false)" = "build=true tag=v9.0.0 " ] || fail "a manual run on a tag is not published"
[ "$(route workflow_dispatch tag v9.0.0 true)" = "build=true tag= " ] || fail "a dry run on a tag publishes"
[ "$(route workflow_dispatch branch main true)" = "build=true tag= " ] || fail "a manual run on a branch publishes"
[ -z "$(git ls-remote --tags origin)" ] || fail "a run that is not a push to the default branch pushed a tag"

# A pre-release first, so that the release after it compares with one.
scripts/agent release 0.1.0-rc.1 >/dev/null 2>&1 || fail "the release of 0.1.0-rc.1 failed"
merge 0.1.0-rc.1 6
ci >/dev/null || fail "release-tag failed on the merged pre-release commit"
[ "$(cat "$tmp/output")" = "$(printf 'build=true\ntag=v0.1.0-rc.1')" ] || fail "release-tag did not tag the pre-release"
refused scripts/agent release 0.1.0-beta | grep -q "not newer" || fail "an older pre-release was accepted"
refused scripts/agent release 0.1.0-rc.1 | grep -q "not newer" || fail "the current pre-release was accepted"
refused scripts/agent release 0.1.0-2 | grep -q "not newer" || fail "a numeric pre-release was seen as newer than rc.1"
refused scripts/agent release 0.1.0-rc.2 | grep -q "is empty" || fail "rc.2 was not seen as newer than rc.1"
sed -i.bak 's/^## \[Unreleased\]$/## [Unreleased]\n\n### Fixed\n\n- The second command./' CHANGELOG.md
rm -f CHANGELOG.md.bak
git commit -q -am "fix: the second command"
git push -q origin main

# The refusals that need a releasable main: a commit origin lacks, and a tag
# that origin already has.
git commit -q --allow-empty -m "chore: not pushed"
refused scripts/agent release 0.1.0 | grep -q "not the same as origin/main" || fail "a main ahead of origin/main was accepted"
git reset -q --hard origin/main
git push -q origin HEAD:refs/tags/v0.1.0
refused scripts/agent release 0.1.0 | grep -q "already exists on origin" || fail "a version whose tag exists on origin was accepted"
git push -q origin :refs/tags/v0.1.0
[ ! -e "$tmp/.worktrees/repo/chore/release-0.1.0" ] || fail "a refused release created a worktree"
: >"$GH_LOG"

# The release: one command opens one pull request.
scripts/agent release v0.1.0 >/dev/null 2>&1 || fail "the release of 0.1.0 failed"
wt=$tmp/.worktrees/repo/chore/release-0.1.0
[ -d "$wt" ] || fail "the release did not create its worktree"
[ -z "$(git status --porcelain)" ] && [ "$(git rev-parse --abbrev-ref HEAD)" = main ] ||
  fail "the release changed the primary checkout"
[ -z "$(git -C "$wt" status --porcelain)" ] || fail "the release left uncommitted changes"
[ "$(git -C "$wt" log -1 --format=%s)" = "chore(release): v0.1.0" ] || fail "the release commit has the wrong subject"
expected=$(printf '## [Unreleased]\n\n## [0.1.0] - 2026-11-27\n\n### Fixed\n\n- The second command.\n\n## [0.1.0-rc.1] - 2026-11-27')
[ "$(sed -n '3,11p' "$wt/CHANGELOG.md")" = "$expected" ] || fail "the changelog section did not move under 0.1.0"
grep -q '^version = "0.1.0"$' "$wt/Cargo.toml" || fail "the workspace version was not bumped"
[ "$(grep -c '^version = "0.1.0"$' "$wt/Cargo.lock")" -eq 2 ] || fail "Cargo.lock does not hold 0.1.0 for both crates"
grep -q '^horsegirl 0.1.0$' "$wt/docs/quickstart.md" || fail "the quickstart still names the old version"
git ls-remote --exit-code origin refs/heads/chore/release-0.1.0 >/dev/null || fail "the release branch was not pushed"
grep -q "pr create --title chore(release): v0.1.0" "$GH_LOG" || fail "the release did not open its pull request"
grep -q "The second command" "$GH_LOG.body" || fail "the pull request body lacks the changes"
grep -q "The first command" "$GH_LOG.body" && fail "the pull request body repeats an earlier release"
refused scripts/agent release 0.1.0 | grep -q "already exists" || fail "a second release branch of 0.1.0 was accepted"

merge 0.1.0 7
merged=$(git rev-parse HEAD)
ci >/dev/null || fail "release-tag failed on the merged release commit"
[ "$(git ls-remote origin refs/tags/v0.1.0 | cut -f 1)" = "$merged" ] || fail "v0.1.0 does not point at the merge commit"
[ "$(cat "$tmp/output")" = "$(printf 'build=true\ntag=v0.1.0')" ] || fail "release-tag did not name the tag for the publish job"
ci >/dev/null || fail "a second run on the tagged commit failed"
[ "$(cat "$tmp/output")" = "$(printf 'build=true\ntag=v0.1.0')" ] || fail "a second run did not name the tag"

# After the release, the same version and an empty section are refused.
refused scripts/agent release 0.1.0 | grep -q "not newer" || fail "the released version was accepted again"
refused scripts/agent release 0.1.0-rc.2 | grep -q "not newer" || fail "a pre-release of the released version was accepted"
refused scripts/agent release 0.0.9 | grep -q "not newer" || fail "an older version was accepted"
refused scripts/agent release 0.1.1 | grep -q "Unreleased section of CHANGELOG.md is empty" ||
  fail "an empty Unreleased section was accepted"
refused scripts/agent release 0.10.0 | grep -q "is empty" || fail "0.10.0 was not seen as newer than 0.1.0"

# Any other commit gets no tag.
echo more >>docs/quickstart.md
git commit -q -am "docs: say more"
git push -q origin main
ci >/dev/null || fail "release-tag failed on a commit that is not a release"
[ "$(cat "$tmp/output")" = "$(printf 'build=false\ntag=')" ] || fail "release-tag builds a commit that is not a release"
[ "$(git ls-remote --tags origin | wc -l)" -eq 2 ] || fail "release-tag pushed a tag for a commit that is not a release"

# A release title with files that state another version is refused.
git commit -q --allow-empty -m "chore(release): v0.2.0 (#8)"
git push -q origin main
refused ci | grep -q "Cargo.toml states version '0.1.0'" || fail "a title without a version bump was tagged"
[ -z "$(git ls-remote origin refs/tags/v0.2.0)" ] || fail "v0.2.0 was pushed"

# An existing tag at another commit is never moved.
git commit -q --allow-empty -m "chore(release): v0.1.0 (#9)"
git push -q origin main
refused ci | grep -q "already exists" || fail "an existing tag at another commit was accepted"
[ "$(git ls-remote origin refs/tags/v0.1.0 | cut -f 1)" = "$merged" ] || fail "v0.1.0 moved"

# A release title whose Cargo.toml states the version and whose changelog has
# no section for it is refused.
sed -i.bak 's/^version = "0.1.0"$/version = "0.2.0"/' Cargo.toml
rm -f Cargo.toml.bak
git commit -q -am "chore(release): v0.2.0 (#10)"
git push -q origin main
refused ci | grep -q "has no section" || fail "a release with no changelog section was tagged"
[ -z "$(git ls-remote origin refs/tags/v0.2.0)" ] || fail "v0.2.0 was pushed"

# check-changelog: an item whose surface is not none needs a new entry.
item() {
  printf -- '---\nid: %s\ntitle: An item\nsurface: %s\n---\n\n## Purpose\n\n---\n\nsurface: none\n' "$1" "$2" \
    >"cairn/items/$(printf '%04d' "$1")-an-item.md"
}
item 7 cli
item 8 none
git add -A
git commit -q -m "chore: add items"
base=$(git rev-parse HEAD)
check() { BRANCH=$1 BASE=$base sh scripts/check-changelog; }
refused check feat/0007-an-item | grep -q "surface: cli" || fail "a cli change with no entry passed"
check feat/0008-an-item >/dev/null || fail "an item with surface none needed an entry"
check chore/release-0.2.0 >/dev/null || fail "a branch with no item needed an entry"
refused check feat/0099-gone | grep -q "no item 99" || fail "a branch naming a missing item passed"
sed -i.bak 's/^## \[Unreleased\]$/## [Unreleased]\n\n### Changed/' CHANGELOG.md
rm -f CHANGELOG.md.bak
refused check feat/0007-an-item >/dev/null || fail "a heading alone counted as an entry"
git checkout -q CHANGELOG.md
sed -i.bak 's/^## \[Unreleased\]$/## [Unreleased]\n\n### Changed\n\n- The command reads a file./' CHANGELOG.md
rm -f CHANGELOG.md.bak
check feat/0007-an-item >/dev/null || fail "a cli change with an entry failed"
# An entry in an old release section is not an entry.
git checkout -q CHANGELOG.md
echo "- A late note." >>CHANGELOG.md
refused check feat/0007-an-item >/dev/null || fail "an entry outside the Unreleased section passed"
git checkout -q CHANGELOG.md
# An entry that the base already holds is not a new entry.
sed -i.bak 's/^## \[Unreleased\]$/## [Unreleased]\n\n### Added\n\n- The first command./' CHANGELOG.md
rm -f CHANGELOG.md.bak
git commit -q -am "docs: an unreleased entry"
base=$(git rev-parse HEAD)
refused check feat/0007-an-item | grep -q "surface: cli" || fail "an entry of the base counted as a new entry"

# A crate with its own version is refused before the release commit.
sed -i.bak 's/^version.workspace = true$/version = "0.0.0"/' crates/two/Cargo.toml
rm -f crates/two/Cargo.toml.bak
cargo metadata --format-version 1 --offline >/dev/null
git commit -q -am "chore: give crate two its own version"
git push -q origin main
refused scripts/agent release 0.3.0 | grep -q "do not take the workspace version" ||
  fail "a crate with its own version was accepted"
git ls-remote --exit-code origin refs/heads/chore/release-0.3.0 >/dev/null && fail "a refused release was pushed"

echo "tests/release.sh: ok"
