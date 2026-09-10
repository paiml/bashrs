#!/usr/bin/env bash
# A_2 for PMAT-251 phase 2 (corpus growth): every generated entry is accounted for — either appended to
# rash/src/corpus/registry/corpus_data.jsonl and passing every V2 dimension in `bashrs corpus run`, or listed in
# docs/audits/corpus-pending-PMAT-251.jsonl with the measured reason it was not added. No existing entry regresses.
set -uo pipefail
WTC=${WTC:-/mnt/nvme-raid0/wt/bashrs-PMAT-251-corpus}; cd "$WTC" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251c}
GEN=/run/user/1000/paiml-implement/agy/PMAT-251/bf151141-60ff-4e66-9794-31fe5c18982d/corpus/entries-all.jsonl
J=rash/src/corpus/registry/corpus_data.jsonl; P=docs/audits/corpus-pending-PMAT-251.jsonl
fail=0
gen=$(wc -l < "$GEN"); base=17942; now=$(wc -l < "$J"); added=$((now-base)); pending=$([ -f "$P" ] && wc -l < "$P" || echo 0)
echo "A: generated=$gen added=$added pending=$pending"
[ $((added+pending)) -ge $((gen-1)) ] || { echo "A: added+pending ($((added+pending))) does not account for the generated entries ($gen, one known cross-lane duplicate allowed)"; fail=1; }
[ "$added" -ge 150 ] || { echo "A: fewer than 150 entries added ($added)"; fail=1; }
jq -e . "$J" >/dev/null 2>&1 || { echo "A: corpus_data.jsonl has an invalid line"; fail=1; }
dups=$(jq -r .id "$J" | sort | uniq -d | wc -l); [ "$dups" = 0 ] || { echo "A: $dups duplicate ids"; fail=1; }
dupin=$(jq -r .input "$J" | sort | uniq -d | wc -l); echo "A: duplicate inputs in corpus (pre-existing count is informative): $dupin"
if [ -f "$P" ]; then jq -e . "$P" >/dev/null 2>&1 || { echo "A: pending file invalid"; fail=1; }; nores=$(jq -r 'select((.reason // "")|length<10)|.name' "$P" | wc -l); [ "$nores" = 0 ] || { echo "A: $nores pending entries without a measured reason"; fail=1; }; fi
cargo build -p bashrs --bin bashrs 2>/tmp/a-corpus.build.$$ || { echo "A: bashrs does not build"; tail -3 /tmp/a-corpus.build.$$; fail=1; }
rm -f /tmp/a-corpus.build.$$
BIN=$CARGO_TARGET_DIR/debug/bashrs
"$BIN" corpus run --format json > /tmp/a-corpus.run.$$ 2>/dev/null || { echo "A: corpus run failed"; fail=1; }
total=$(jq -r '.passed // 0' /tmp/a-corpus.run.$$); failed=$(jq -r '(.failed // (.entries_failed // 0))' /tmp/a-corpus.run.$$); score=$(jq -r '.score // 0' /tmp/a-corpus.run.$$)
echo "A: corpus run passed=$total failed=$failed score=$score (baseline: passed=17942 failed=0)"
[ "$total" -ge $((base+added)) ] 2>/dev/null || { echo "A: passed ($total) < base+added ($((base+added))) — an added entry fails or an existing one regressed"; fail=1; }
cp /tmp/a-corpus.run.$$ docs/audits/corpus-run-PMAT-251.json 2>/dev/null; rm -f /tmp/a-corpus.run.$$
cargo test -p bashrs --test corpus_registry_contract_tests 2>&1 | tail -1 | grep -q 'test result: ok' || { echo "A: corpus_registry_contract_tests failed"; fail=1; }
[ $fail = 0 ] && echo "A: PASS"
exit $fail
