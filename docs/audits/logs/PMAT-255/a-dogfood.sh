#!/usr/bin/env bash
# A_3a, PMAT-255 (PMAT-253 phase 3a, gates D G K; gate B waits on the operator's decision 4). PASS iff `make dogfood` exits 0 and
# prints exactly one `GATE <L> PASS <detail>` line for each of D, G, K; each gate script carries a machine-applicable
# `# mutation: <sed expression>` and, applied to a copy, that mutation turns the gate FAIL with a non-zero exit;
# the falsification test that every gate script declares a mutation passes.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-dog}; cd "$WT" || exit 2
SP=/tmp/claude-1000/-home-noah-src-bashrs/bf151141-60ff-4e66-9794-31fe5c18982d/scratchpad; fail=0
make dogfood > "$SP/dogfood-255.log" 2>&1; rc=$?; grep -E '^GATE ' "$SP/dogfood-255.log"
[ $rc = 0 ] || { echo "FAIL: make dogfood exit $rc"; fail=1; }
for g in D G K; do c=$(grep -cE "^GATE $g PASS " "$SP/dogfood-255.log"); [ "$c" = 1 ] || { echo "FAIL: gate $g printed $c PASS line(s), want exactly 1"; fail=1; }; done
for f in scripts/dogfood/docs.sh scripts/dogfood/contracts.sh scripts/dogfood/corpus.sh; do
  [ -f "$f" ] || { echo "FAIL: $f missing"; fail=1; continue; }
  m=$(grep -m1 -E '^# mutation: ' "$f" | sed 's/^# mutation: //'); [ -n "$m" ] || { echo "FAIL: $f declares no mutation"; fail=1; continue; }
  cp "$f" "$f.mut"; sed -i -e "$m" "$f.mut"; if cmp -s "$f" "$f.mut"; then echo "FAIL: $f mutation [$m] changes nothing"; fail=1; rm -f "$f.mut"; continue; fi
  out=$(bash "$f.mut" 2>&1); r=$?; rm -f "$f.mut"
  echo "$out" | grep -qE '^GATE [A-Z] FAIL ' && [ $r != 0 ] || { echo "FAIL: $f under its own mutation did not go red (exit $r)"; fail=1; }
done
cargo test -p bashrs --test falsification_dogfood_scripts_declare_mutations 2>&1 | grep -E '^test result' | tail -1 | grep -q 'test result: ok' || { echo "FAIL: falsification_dogfood_scripts_declare_mutations"; fail=1; }
[ $fail = 0 ] && echo "A3a: PASS"; exit $fail
