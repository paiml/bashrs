#!/usr/bin/env bash
# A_3, PMAT-255 (corpus growth). PASS iff every generated entry is appended or pending (with a reason); the seven entries whose
# B3 execution has side effects are pending and never appended; appended lines carry exactly the canonical keys; a SANDBOXED
# corpus run (bwrap: read-only root, private /tmp, no network, empty cwd) passes every entry with the new ones included and
# leaves no file the base run did not; corpus_registry_contract_tests pass.
set -uo pipefail
WTC=${WTC:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-corpus}; cd "$WTC" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-255c}
SP=/tmp/claude-1000/-home-noah-src-bashrs/bf151141-60ff-4e66-9794-31fe5c18982d/scratchpad
GEN=/run/user/1000/paiml-implement/agy/PMAT-255/bf151141-60ff-4e66-9794-31fe5c18982d/ph2/entries-all.jsonl
J=rash/src/corpus/registry/corpus_data.jsonl; P=docs/audits/corpus-pending-PMAT-255.jsonl
UNSAFE="make-gnu-bash-build-j make-gnu-bash-var-override make-gnu-bash-recursive-env make-gnu-bash-dry-run-check make-gnu-bash-keep-going-ci polyglot-inline-bash-uv-python agentic-ops-agent-cli-probes"
KEYS='["deterministic","description","expected_output","format","id","idempotent","input","name","sets","shellcheck","tier"]'
fail=0; base=18178
gen=$(wc -l < "$GEN"); now=$(wc -l < "$J"); added=$((now-base)); pending=$([ -f "$P" ] && wc -l < "$P" || echo 0)
echo "A: generated=$gen added=$added pending=$pending"
[ $((added+pending)) -eq "$gen" ] || { echo "FAIL: added+pending ($((added+pending))) != generated ($gen)"; fail=1; }
[ "$added" -ge 180 ] || { echo "FAIL: fewer than 180 entries added ($added)"; fail=1; }
jq -e . "$J" >/dev/null 2>&1 || { echo "FAIL: corpus_data.jsonl has an invalid line"; fail=1; }
[ "$(jq -r .id "$J" | sort | uniq -d | wc -l)" = 0 ] || { echo "FAIL: duplicate ids"; fail=1; }
bad=$(tail -n "$added" "$J" | jq -c --argjson k "$KEYS" 'select((keys|sort) != ($k|sort))' | wc -l); [ "$bad" = 0 ] || { echo "FAIL: $bad appended lines without exactly the canonical keys"; fail=1; }
for n in $UNSAFE; do
  jq -es --arg n "$n" 'any(.[]; .name==$n)' "$J" >/dev/null 2>&1 && { echo "FAIL: unsafe entry $n was appended"; fail=1; }
  [ -f "$P" ] && jq -es --arg n "$n" 'any(.[]; .name==$n)' "$P" >/dev/null 2>&1 || { echo "FAIL: unsafe entry $n is not in pending"; fail=1; }
done
if [ -f "$P" ]; then nores=$(jq -r 'select((.reason // "")|length<10)|.name' "$P" | wc -l); [ "$nores" = 0 ] || { echo "FAIL: $nores pending entries without a measured reason"; fail=1; }; fi
cargo build --release -p bashrs --bin bashrs -q 2>/dev/null || { echo "FAIL: release build"; exit 1; }
E=$(mktemp -d)
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --bind "$E" "$E" --chdir "$E" --unshare-net --die-with-parent "$CARGO_TARGET_DIR/release/bashrs" corpus run --format json 2>/dev/null | sed -n '/^{/,$p' > "$SP/corpus-run-255.json"
passed=$(jq -r '.passed' "$SP/corpus-run-255.json"); failed=$(jq -r '.failed' "$SP/corpus-run-255.json"); total=$(jq -r '.total' "$SP/corpus-run-255.json")
echo "A: sandboxed corpus run passed=$passed failed=$failed total=$total score=$(jq -r .score "$SP/corpus-run-255.json") (base: see corpus-run-base-255.json)"
[ "$total" = "$now" ] && [ "$passed" = "$total" ] && [ "$failed" = 0 ] || { echo "FAIL: sandboxed run is not clean over all $now entries"; fail=1; }
find "$E" -mindepth 1 | sed "s#^$E/##" | sort > "$SP/corpus-debris-255.txt"
newdebris=$(comm -13 "$SP/corpus-debris-base.txt" "$SP/corpus-debris-255.txt" | wc -l); [ "$newdebris" = 0 ] || { echo "FAIL: the run left $newdebris file(s) the base run did not:"; comm -13 "$SP/corpus-debris-base.txt" "$SP/corpus-debris-255.txt" | head -5; fail=1; }
cargo test -p bashrs --test corpus_registry_contract_tests 2>&1 | grep -E '^test result' | tail -1 | grep -q 'test result: ok' || { echo "FAIL: corpus_registry_contract_tests"; fail=1; }
[ $fail = 0 ] && echo "A3: PASS"; exit $fail
