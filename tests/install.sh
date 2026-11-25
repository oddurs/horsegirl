#!/bin/sh
# install.sh must put the binary in ~/.local/bin, and must refuse an archive
# whose checksum does not match SHA256SUMS. A local directory stands in for the
# release, so the test needs no network. Run through scripts/task test.
set -eu
cd "$(dirname "$0")/.."

fail() {
  echo "tests/install.sh: FAIL: $1" >&2
  exit 1
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# One fake release with an archive for each target that the workflow builds.
# The binary is a script that names itself, so the test can run it.
mkdir "$tmp/release"
for target in x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
  x86_64-apple-darwin aarch64-apple-darwin; do
  mkdir -p "$tmp/stage/horsegirl-$target"
  printf '#!/bin/sh\necho "horsegirl test"\n' >"$tmp/stage/horsegirl-$target/horsegirl"
  chmod 755 "$tmp/stage/horsegirl-$target/horsegirl"
  tar -czf "$tmp/release/horsegirl-$target.tar.gz" -C "$tmp/stage" "horsegirl-$target"
done
if command -v sha256sum >/dev/null 2>&1; then
  sum() { sha256sum "$@"; }
else
  sum() { shasum -a 256 "$@"; }
fi
(cd "$tmp/release" && sum -- *.tar.gz) >"$tmp/release/SHA256SUMS"

run_install() {
  HOME="$1" HORSEGIRL_DOWNLOAD_URL="file://$tmp/release" sh ./install.sh
}

# A matching checksum installs to ~/.local/bin.
mkdir "$tmp/home-good"
run_install "$tmp/home-good" >"$tmp/out" 2>&1 || {
  cat "$tmp/out" >&2
  fail "the install failed with matching checksums"
}
bin="$tmp/home-good/.local/bin/horsegirl"
[ -x "$bin" ] || fail "no executable at ~/.local/bin/horsegirl"
[ "$("$bin")" = "horsegirl test" ] || fail "the installed file is not the binary from the archive"

# HORSEGIRL_INSTALL_DIR moves the binary.
mkdir "$tmp/home-dir"
HORSEGIRL_INSTALL_DIR="$tmp/elsewhere" run_install "$tmp/home-dir" >"$tmp/out" 2>&1 ||
  fail "the install failed with HORSEGIRL_INSTALL_DIR set"
[ -x "$tmp/elsewhere/horsegirl" ] || fail "HORSEGIRL_INSTALL_DIR was not used"

# A checksum mismatch stops the install, says why, and writes nothing.
zeros=$(printf '%064d' 0)
(cd "$tmp/release" && for f in *.tar.gz; do printf '%s  %s\n' "$zeros" "$f"; done) >"$tmp/release/SHA256SUMS"
mkdir "$tmp/home-bad"
if run_install "$tmp/home-bad" >"$tmp/out" 2>&1; then
  fail "the install passed with a checksum mismatch"
fi
grep -q 'checksum mismatch' "$tmp/out" || fail "the message does not name the checksum mismatch"
[ ! -e "$tmp/home-bad/.local/bin/horsegirl" ] || fail "a mismatched binary was installed"

# An archive that SHA256SUMS does not list stops the install.
printf '%s  horsegirl-other.tar.gz\n' "$zeros" >"$tmp/release/SHA256SUMS"
mkdir "$tmp/home-none"
if run_install "$tmp/home-none" >"$tmp/out" 2>&1; then
  fail "the install passed with no checksum for the archive"
fi
grep -q 'no checksum' "$tmp/out" || fail "the message does not name the missing checksum"
[ ! -e "$tmp/home-none/.local/bin/horsegirl" ] || fail "an unlisted binary was installed"
