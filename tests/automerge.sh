#!/bin/sh
# Tests for scripts/dependabot-automerge, with a fake gh that logs its calls.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/gh" <<'GH'
#!/bin/sh
echo "$*" >> "$GH_LOG"
if [ "$1 $2" = "pr view" ]; then echo "$GH_ARMED"; fi
GH
chmod +x "$tmp/gh"
fails=0

run() {
  name=$1 kind=$2 armed=$3 want=$4
  : > "$tmp/log"
  GH_LOG="$tmp/log" GH_ARMED=$armed PATH="$tmp:$PATH" "$root/scripts/dependabot-automerge" "$kind" URL
  got=$(grep -v '^pr view' "$tmp/log" || true)
  if [ "$got" = "$want" ]; then
    echo "ok   $name"
  else
    echo "FAIL $name: wanted '$want', got '$got'"
    fails=$((fails + 1))
  fi
}

run auto-turns-on auto false 'pr merge --auto --squash URL'
run manual-turns-off-when-armed manual true 'pr merge --disable-auto URL'
run manual-leaves-alone-when-off manual false ''

[ "$fails" -eq 0 ]
