pre-release:
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

render-vhs:
    #!/usr/bin/env bash
    set -euo pipefail

    command -v vhs >/dev/null || {
        echo "vhs is required to render the README recording" >&2
        exit 1
    }

    cargo build --locked
    vhs docs/vhs/quick-start.tape

    gif=docs/assets/quick-start.gif
    perl -0777 -pi -e 's/\x21\xFF\x0BNETSCAPE2\.0\x03\x01..\x00//s' "$gif"
    if LC_ALL=C grep --quiet 'NETSCAPE2.0' "$gif"; then
        echo "failed to disable looping in $gif" >&2
        exit 1
    fi
