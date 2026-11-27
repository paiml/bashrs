#!/usr/bin/env bash
# Planted falsifier: a wrong recorded expectation must make the replay fail; the true one must pass.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
T="$(mktemp -d)"; trap 'rm -rf "${T:?}"' EXIT
R="$HERE/replay.sh"
SC_EXPECTED="$HERE/expected.tsv" bash "$R" >/dev/null 2>&1 || { echo "FAIL selftest: replay does not pass on the recorded expectations"; exit 1; }
echo "ok   replay passes on the recorded expectations"
# plant: bump one recorded count (SC2086 in unquoted-var.sh) from its true value to a wrong one
awk -F'\t' 'BEGIN{OFS="\t"} $1=="unquoted-var.sh" && $2=="SC2086" {$3=$3+1} {print}' "$HERE/expected.tsv" >"$T/planted.tsv"
out="$(SC_EXPECTED="$T/planted.tsv" bash "$R" 2>&1)"; rc=$?
if [ "$rc" -ne 0 ] && grep -q 'FAIL replay' <<<"$out"; then echo "ok   planted wrong expectation FAILS the replay"; else echo "FAIL selftest: planted expectation was not caught"; exit 1; fi
# plant: a rule recorded as absent that the live run finds
awk -F'\t' '$2!="SC2034"' "$HERE/expected.tsv" >"$T/dropped.tsv"
out="$(SC_EXPECTED="$T/dropped.tsv" bash "$R" 2>&1)"; rc=$?
if [ "$rc" -ne 0 ]; then echo "ok   a dropped expectation FAILS the replay"; else echo "FAIL selftest: dropped expectation was not caught"; exit 1; fi
echo "PASS selftest"
