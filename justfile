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
    just release-changelog "$NEW_VERSION" check

release-changelog version output:
    #!/usr/bin/env bash
    set -euo pipefail

    python3 - {{quote(version)}} {{quote(output)}} <<'PY'
    import argparse
    import re
    from pathlib import Path


    HEADING = re.compile(r"^##\s+([0-9]+\.[0-9]+\.[0-9]+)\s+-")


    def release_entry(version: str) -> tuple[str, str]:
        lines = Path("CHANGELOG.md").read_text().splitlines()
        headings = [
            (index, match.group(1))
            for index, line in enumerate(lines)
            if (match := HEADING.match(line))
        ]
        matches = [index for index, (_, heading) in enumerate(headings) if heading == version]

        if not matches:
            raise ValueError(f"No changelog entry for version {version}")
        if len(matches) > 1:
            raise ValueError(f"Multiple changelog entries found for version {version}")

        heading_index = matches[0]
        start = headings[heading_index][0] + 1
        end = headings[heading_index + 1][0] if heading_index + 1 < len(headings) else len(lines)
        notes = "\n".join(lines[start:end]).strip()
        if not notes:
            raise ValueError(f"Changelog entry for version {version} is empty")
        if heading_index + 1 >= len(headings):
            raise ValueError(f"No previous changelog entry found for version {version}")

        return notes, headings[heading_index + 1][1]


    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("output", choices=("check", "notes", "previous-version"))
    args = parser.parse_args()

    try:
        notes, previous_version = release_entry(args.version)
    except ValueError as error:
        parser.exit(1, f"{error}\n")

    if args.output == "notes":
        print(notes)
    elif args.output == "previous-version":
        print(previous_version)
    PY

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
