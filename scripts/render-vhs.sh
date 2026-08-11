#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

command -v vhs >/dev/null || {
    echo "vhs is required to render the README recordings" >&2
    exit 1
}

cargo build --locked

for tape in docs/vhs/*.tape; do
    if [[ $(basename "$tape") == settings.tape ]]; then
        continue
    fi
    vhs "$tape"
done

./scripts/hash-vhs-frames.sh > docs/assets/frame-hashes.txt
