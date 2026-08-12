#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

command -v vhs >/dev/null || {
    echo "vhs is required to render the README recordings" >&2
    exit 1
}

cargo build --locked

if (( $# > 0 )); then
    tapes=("$@")
else
    tapes=(docs/vhs/*.tape)
fi

for tape in "${tapes[@]}"; do
    if [[ $(basename "$tape") == settings.tape ]]; then
        continue
    fi
    vhs "$tape"
done

for gif in docs/assets/*.gif; do
    perl -0777 -pi -e 's/\x21\xFF\x0BNETSCAPE2\.0\x03\x01..\x00//s' "$gif"
    if LC_ALL=C grep --quiet 'NETSCAPE2.0' "$gif"; then
        echo "failed to disable looping in $gif" >&2
        exit 1
    fi
done

./scripts/normalize-vhs-transcripts.sh
