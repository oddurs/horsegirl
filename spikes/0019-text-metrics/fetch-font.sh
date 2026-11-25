#!/bin/sh
# Fetches the reference font from a pinned commit of the Google Fonts
# repository and checks its hash. The font is under the SIL Open Font Licence.
set -eu
cd "$(dirname "$0")"
commit=0b58fb370093f9a9f4ff785d94405710b79de67c
base="https://raw.githubusercontent.com/google/fonts/$commit/ofl/instrumentsans"
mkdir -p font
curl -sfLo font/InstrumentSans.ttf "$base/InstrumentSans%5Bwdth,wght%5D.ttf"
curl -sfLo font/OFL.txt "$base/OFL.txt"
echo "b24f1812584816958afcf22e22d08e44318c5e51651e25d2438efdde389b33b1  font/InstrumentSans.ttf" | sha256sum -c -
