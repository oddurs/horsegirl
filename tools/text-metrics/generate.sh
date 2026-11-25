#!/bin/sh
# Regenerates the metrics table of the core from Liberation Sans 2.1.5 Regular.
# Usage: tools/text-metrics/generate.sh <LiberationSans-Regular.ttf>
#
# The checksum pins the font: a different file gives a different table, and
# the widths in the core tests would then move without a reason in the diff.
set -eu
font=${1:?usage: generate.sh <LiberationSans-Regular.ttf>}
here=$(cd "$(dirname "$0")" && pwd)
out="$here/../../crates/horsegirl-core/src/text/metrics.rs"
expected=bade59d822652f76e6941aa87b40a87c13d1cc70db98ededb5011127efafd1d3

actual=$(sha256sum "$font" | cut -d ' ' -f 1)
if [ "$actual" != "$expected" ]; then
  echo "generate.sh: $font is not Liberation Sans 2.1.5 Regular (sha256 $actual)" >&2
  exit 1
fi

cargo run --quiet --release --locked --manifest-path "$here/Cargo.toml" -- "$font" >"$out.tmp"
rustfmt --edition 2024 --config max_width=100 "$out.tmp"
mv "$out.tmp" "$out"
