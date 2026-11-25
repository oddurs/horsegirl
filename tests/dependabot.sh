#!/bin/sh
# Tests for scripts/dependabot-update-type.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
fails=0

expect() {
  name=$1 want=$2 input=$3
  got=$(printf '%s\n' "$input" | "$root/scripts/dependabot-update-type")
  if [ "$got" = "$want" ]; then
    echo "ok   $name"
  else
    echo "FAIL $name: wanted $want, got $got"
    fails=$((fails + 1))
  fi
}

expect patch auto '---
updated-dependencies:
- dependency-name: serde
  update-type: version-update:semver-patch
...'
expect minor auto '  update-type: version-update:semver-minor'
expect group-of-patch-and-minor auto '  update-type: version-update:semver-patch
  update-type: version-update:semver-minor'
expect major manual '  update-type: version-update:semver-major'
expect group-with-one-major manual '  update-type: version-update:semver-patch
  update-type: version-update:semver-major'
expect unknown-kind manual '  update-type: version-update:semver-prerelease'
expect no-metadata manual 'chore: bump serde'
expect empty manual ''

[ "$fails" -eq 0 ]
