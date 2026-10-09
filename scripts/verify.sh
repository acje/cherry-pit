#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "==> Running cherry-pit tools/verify.py all..."
python3.12 -B tools/verify.py all
