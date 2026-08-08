#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${NEW_VERSION:-}" ]]; then
  echo "NEW_VERSION is not set; run this hook through cargo release." >&2
  exit 1
fi

cargo build --verbose
cargo test --verbose
cargo clippy
cargo fmt --check
python3 scripts/release-changelog.py "$NEW_VERSION" check
