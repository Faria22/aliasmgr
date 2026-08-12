#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

for transcript in docs/assets/*.txt; do
    transcript_tmp=$(mktemp "${transcript}.tmp.XXXXXX")
    awk '
        /^─+$/ {
            delete final
            final_last = current_last
            for (line = 1; line <= current_last; line++) {
                final[line] = current[line]
            }
            delete current
            current_count = 0
            current_last = 0
            next
        }
        {
            sub(/, [0-9]+ bytes \| [^,]+, done\.$/, ", done.")
            current[++current_count] = $0
            if ($0 != "") {
                current_last = current_count
            }
        }
        END {
            if (current_last > 0) {
                delete final
                final_last = current_last
                for (line = 1; line <= current_last; line++) {
                    final[line] = current[line]
                }
            }
            for (line = 1; line <= final_last; line++) {
                print final[line]
            }
        }
    ' "$transcript" > "$transcript_tmp"
    mv "$transcript_tmp" "$transcript"
done
