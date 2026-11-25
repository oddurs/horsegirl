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

# The script must ask cargo for every target and every feature. A fake cargo
# records its arguments.
mkdir "$tmp/bin"
cat >"$tmp/bin/cargo" <<'EOF'
#!/bin/sh
echo "$*" >"$CARGO_ARGS"
echo "horsegirl-core v0.0.0 (/x)"
EOF
chmod +x "$tmp/bin/cargo"
CARGO_ARGS="$tmp/args" PATH="$tmp/bin:$PATH" scripts/check-core-deps ||
  fail "the fake cargo run failed"
grep -q -e '--target all' "$tmp/args" || fail "cargo tree does not use --target all"
grep -q -e '--all-features' "$tmp/args" || fail "cargo tree does not use --all-features"

unset CORE_DEPS_ALLOW
scripts/check-core-deps || fail "the current core crate does not pass"
