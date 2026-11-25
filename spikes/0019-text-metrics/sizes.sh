#!/bin/sh
# Prints the size each way of measuring adds to a release binary, native and
# WebAssembly, raw and compressed. The baseline binary reads a label and
# prints its length, so the difference is the cost of measuring.
set -eu
cd "$(dirname "$0")"
for target in x86_64-unknown-linux-gnu wasm32-unknown-unknown; do
  cargo build --quiet --release --target "$target" --no-default-features --bin baseline --bin table
  cargo build --quiet --release --target "$target" --bin shape
  ext=""
  [ "$target" = wasm32-unknown-unknown ] && ext=".wasm"
  for bin in baseline table shape; do
    file="target/$target/release/$bin$ext"
    raw=$(wc -c <"$file")
    gz=$(gzip -9n <"$file" | wc -c)
    printf '%-26s %-9s %9s bytes %9s gzip\n' "$target" "$bin" "$raw" "$gz"
  done
done
