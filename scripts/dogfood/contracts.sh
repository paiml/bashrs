#!/usr/bin/env bash
# mutation: s/errors" -ne 0 \]; then/errors" -eq 0 ]; then/
# Dogfood gate G — the contract corpus, validated against `pv` itself.
#
# Exit code is the gate. The only line a caller must read is the final
# `GATE G PASS|FAIL <detail>`.
#
# Scope (PMAT-255 / PMAT-253 phase 3a): `pv validate` for every top-level
# contracts/*.yaml. contracts/work/ is PMAT-246's working set (ad-hoc,
# per-issue contracts under review) and is EXCLUDED here — named below, not
# silently skipped, so a reader of this gate's PASS line knows the boundary
# without reading the script.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

fail() {
  echo "GATE G FAIL $1"
  exit 1
}

if ! command -v pv >/dev/null 2>&1; then
  fail "pv (provable-contracts CLI) is not on PATH — the contract corpus cannot be validated, and that is UNMEASURED, never a pass"
fi

# The floor below which the corpus would have been emptied out from under
# this gate. 15 top-level contracts today (some symlinks into
# ../provable-contracts, some regular files — both are contracts, only their
# storage differs).
MIN_CONTRACTS=10

shopt -s nullglob
all=(contracts/*.yaml)
shopt -u nullglob
if [ "${#all[@]}" -lt "$MIN_CONTRACTS" ]; then
  fail "only ${#all[@]} contract(s) under contracts/*.yaml (excluding contracts/work/, which this gate does not read), floor is ${MIN_CONTRACTS} — the corpus may have been emptied"
fi

n_errors=0
n_ok=0
for c in "${all[@]}"; do
  rc=0
  out="$(pv validate "$c" 2>&1)" || rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$out"
    echo "GATE G FAIL pv validate ${c} exited ${rc}"
    n_errors=$((n_errors + 1))
    continue
  fi
  case "$out" in
    *"Contract is valid."*) n_ok=$((n_ok + 1)) ;;
    *)
      printf '%s\n' "$out"
      echo "  ${c}: pv validate exited 0 without saying the contract is valid — unmeasured is not valid"
      n_errors=$((n_errors + 1))
      ;;
  esac
done

if [ "$n_errors" -ne 0 ]; then
  fail "${n_errors} of ${#all[@]} contract(s) under contracts/*.yaml do not validate (contracts/work/ excluded — that corpus is PMAT-246's)"
fi

echo "GATE G PASS ${n_ok} contract(s) under contracts/*.yaml validate via pv (contracts/work/ excluded — PMAT-246)"
