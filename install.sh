#!/bin/sh
# Installs the horsegirl binary from a GitHub release:
#
#   curl -fsSL https://raw.githubusercontent.com/oddurs/horsegirl/main/install.sh | sh
#
# The script checks the archive against the SHA256SUMS of the same release
# before it writes to the install directory.
#
#   HORSEGIRL_VERSION       a release tag, such as v0.1.0. Unset: the latest release.
#   HORSEGIRL_INSTALL_DIR   the directory for the binary. Unset: ~/.local/bin.
#   HORSEGIRL_DOWNLOAD_URL  the directory that holds the archives and SHA256SUMS.
#                           Unset: the release on GitHub.
set -eu

repo=https://github.com/oddurs/horsegirl

say() {
  echo "install.sh: $*" >&2
}

die() {
  say "$*"
  exit 1
}

detect_target() {
  case "$(uname -m)" in
  x86_64 | amd64) arch=x86_64 ;;
  aarch64 | arm64) arch=aarch64 ;;
  *) die "no release binary for the CPU $(uname -m). Build from source: cargo install --git $repo horsegirl" ;;
  esac
  case "$(uname -s)" in
  Linux) echo "$arch-unknown-linux-musl" ;;
  Darwin) echo "$arch-apple-darwin" ;;
  *) die "no release binary for $(uname -s). Build from source: cargo install --git $repo horsegirl" ;;
  esac
}

download() {
  if command -v curl >/dev/null 2>&1; then
    curl --proto '=https,file' -fsSL "$1" -o "$2" || die "could not download $1"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$2" "$1" || die "could not download $1"
  else
    die "curl or wget is necessary to download the release"
  fi
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    die "sha256sum or shasum is necessary to verify the download"
  fi
}

target=$(detect_target)
asset="horsegirl-$target.tar.gz"
if [ -n "${HORSEGIRL_DOWNLOAD_URL:-}" ]; then
  base=$HORSEGIRL_DOWNLOAD_URL
elif [ -n "${HORSEGIRL_VERSION:-}" ]; then
  base="$repo/releases/download/$HORSEGIRL_VERSION"
else
  base="$repo/releases/latest/download"
fi
dir=${HORSEGIRL_INSTALL_DIR:-$HOME/.local/bin}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

say "downloading $base/$asset"
download "$base/$asset" "$tmp/$asset"
download "$base/SHA256SUMS" "$tmp/SHA256SUMS"

# sha256sum marks a binary-mode file with '*' before the name.
expected=$(awk -v f="$asset" '$2 == f || $2 == "*" f { print $1; exit }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || die "SHA256SUMS has no checksum for $asset. Nothing was installed."
actual=$(sha256 "$tmp/$asset")
[ "$actual" = "$expected" ] ||
  die "checksum mismatch for $asset: SHA256SUMS says $expected, the download is $actual. Nothing was installed."

tar -xzf "$tmp/$asset" -C "$tmp"
[ -f "$tmp/horsegirl-$target/horsegirl" ] || die "$asset holds no horsegirl-$target/horsegirl. Nothing was installed."

# A copy beside the target, then a rename, so a running horsegirl never sees
# half a file.
mkdir -p "$dir"
cp "$tmp/horsegirl-$target/horsegirl" "$dir/.horsegirl.$$"
chmod 755 "$dir/.horsegirl.$$"
mv -f "$dir/.horsegirl.$$" "$dir/horsegirl"
say "installed $dir/horsegirl"

case ":$PATH:" in
*":$dir:"*) ;;
*) say "$dir is not on PATH. Add it to run horsegirl by name." ;;
esac
