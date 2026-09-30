#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "==> Running cherry-pit tools/verify.py all..."
python3.12 -B tools/verify.py all

echo "==> Running cargo test..."
cargo test --workspace --all-features --locked

echo "==> Running cargo clippy..."
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "==> Running cargo fmt check..."
cargo fmt --all -- --check

echo "==> All cherry-pit verification checks passed."
