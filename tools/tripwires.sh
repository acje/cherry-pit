#!/usr/bin/env bash
set -euo pipefail
ROOT="$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
  async-trait|pardosa-dep-deny|forbid-unsafe-total) check=graph ;;
  dead-code-suppression) check=dead-code ;;
  non-exhaustive) check=non-exhaustive ;;
  gate-citation) check=citations ;;
  adr-number-collision) check=adr-collision ;;
  deny-ignore-lifecycle) check=deny-lifecycle ;;
  all) check=all ;;
  *) printf 'usage: bash tools/tripwires.sh <source-check-name>|all\n' >&2; exit 2 ;;
esac
exec python3.12 -B "$ROOT/tools/verify.py" "$check"
