#!/usr/bin/env bash
# RA-1: record shellcheck (pinned, printed) over the committed corpus into expected.tsv.
set -uo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
check_pin
measure | sort >"$EXPECTED"
echo "recorded $(wc -l <"$EXPECTED") (file, rule) rows over $(ls "$CORPUS"/*.sh | wc -l) scripts -> $EXPECTED"
