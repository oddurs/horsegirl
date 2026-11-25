#!/bin/sh
# Measures every label in labels.txt in headless Chromium and writes
# browser.tsv: the label, then its width in pixels in each face.
set -eu
cd "$(dirname "$0")"
browser="${BROWSER:-chromium}"
"$browser" --headless --no-sandbox --disable-gpu --allow-file-access-from-files \
  --virtual-time-budget=10000 --dump-dom "file://$PWD/measure.html" 2>/dev/null |
  sed -n '/<pre id="out">/,/<\/pre>/p' |
  sed 's/<pre id="out">//; s/<\/pre>.*//; s/&lt;/</g; s/&gt;/>/g; s/&amp;/\&/g' >browser.tsv
if grep -q "FONT NOT LOADED" browser.tsv || [ "$(wc -l <browser.tsv)" -ne "$(($(wc -l <labels.txt) + 1))" ]; then
  echo "measure: the browser did not measure every label in the reference font" >&2
  exit 1
fi
