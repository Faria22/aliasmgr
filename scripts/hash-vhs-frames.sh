#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

command -v ffmpeg >/dev/null || {
    echo "ffmpeg is required to hash the README recordings" >&2
    exit 1
}

for gif in docs/assets/*.gif; do
    digest=$(
        ffmpeg \
            -loglevel error \
            -sseof -0.1 \
            -i "$gif" \
            -frames:v 1 \
            -f hash \
            -hash sha256 \
            -
    )
    printf '%s  %s\n' "${digest#SHA256=}" "$gif"
done
