#!/usr/bin/env bash
# RA-1: replay the live shellcheck against the recorded expectations; print matched N/M per rule.
# Exit 0 only when every recorded (file, rule, count) matches and nothing new appears.
set -uo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
check_pin
[ -f "$EXPECTED" ] || { echo "UNMEASURED: no $EXPECTED; run record.sh"; exit 2; }
live="$(mktemp)"; trap 'rm -f "$live"' EXIT
measure | sort >"$live"
# Per rule: M = recorded occurrences; N = occurrences the live run reproduces (per file, min of the two).
awk -F'\t' '
  FNR==NR { want[$1"\t"$2]=$3; rules[$2]=1; next }
  { got[$1"\t"$2]=$3; rules[$2]=1 }
  END {
    bad=0
    for (k in want) { split(k, p, "\t"); m[p[2]]+=want[k]; n=(k in got)?got[k]:0; mn=(n<want[k])?n:want[k]; mat[p[2]]+=mn; if (n!=want[k]) bad++ }
    for (k in got) if (!(k in want)) { split(k, p, "\t"); extra[p[2]]+=got[k]; bad++ }
    for (r in rules) printf "%s matched %d/%d\n", r, mat[r]+0, m[r]+0
    for (r in extra) printf "%s unexpected %d (not recorded)\n", r, extra[r]
    printf "rows mismatched: %d\n", bad
    exit (bad>0)
  }' "$EXPECTED" "$live"
rc=$?
if [ "$rc" -eq 0 ]; then echo "PASS replay"; else echo "FAIL replay"; fi
exit "$rc"
