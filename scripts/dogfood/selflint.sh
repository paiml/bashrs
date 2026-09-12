#!/usr/bin/env bash
# mutation: s/"\$n" -gt "\$recorded"/"\$n" -lt "\$recorded"/
#
# Dogfood gate S — bashrs self-lint, promoted from an advisory report to a
# GATE with a PER-FILE RATCHET (PMAT-263, PMAT-253 phase 4, decided 3-0 by
# blind quorum 2026-09-11, decision D4), in forjar's `legacy bashrs errors
# N <= N` shape: a pull request pays only for the files it touches, not for
# the whole tree's pre-existing debt at once.
#
# Exit code is the gate. The only line a caller must read is the final
# `GATE S PASS|FAIL <detail>`.
#
# THE RATCHET, scripts/dogfood/selflint-ratchet.tsv, is two tab-separated
# columns (`errors<TAB>path`, sorted by path) recording the ERROR-severity
# diagnostic count `bashrs lint --format json` reports for every tracked
# *.sh file this gate measures, on the day it was written or last lowered.
# A file over its recorded count is new debt and FAILS. A file under its
# recorded count IMPROVED and the ratchet should be lowered to match — this
# gate does that for you in --write-ratchet mode. A file with errors and NO
# row is new debt with no baseline and FAILS ("add it to the ratchet
# deliberately or fix it") rather than silently passing.
#
# --write-ratchet REGENERATES the file from today's measurement. This is
# the ONLY thing --write-ratchet is for. Running it to make a FAILING gate
# pass — lowering the bar to meet the ball instead of raising the ball to
# meet the bar — is exactly the failure mode this gate exists to catch, and
# it is not hidden by the regenerated file: `git diff
# scripts/dogfood/selflint-ratchet.tsv` shows every number that moved, in
# either direction, on the review that must approve it.
set -euo pipefail

if [ -n "${SELFLINT_ROOT:-}" ]; then
  ROOT="$SELFLINT_ROOT"
else
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fi
cd "$ROOT"

fail() {
  echo "GATE S FAIL $1"
  exit 1
}

# The linter under test. BASHRS_BIN names the binary the caller built from
# this tree (make dogfood-selflint passes it; the integration tests pass
# CARGO_BIN_EXE_bashrs); without it the gate falls back to whatever `bashrs`
# is on PATH, which may be an older release, so the Makefile never relies on
# that fallback.
BASHRS="${BASHRS_BIN:-bashrs}"
if ! command -v "$BASHRS" >/dev/null 2>&1; then
  fail "${BASHRS} is not an executable — this gate lints every tracked *.sh file with it, and that is UNMEASURED, never a pass"
fi

RATCHET="scripts/dogfood/selflint-ratchet.tsv"

# EXCLUDE — files this gate does not measure, one reason each. The bench
# fixtures are first: they are deliberately messy shell used as PERFORMANCE
# INPUT (rash/benches/*.rs reads them as byte data), not scripts this
# repository runs, and ratcheting them would ratchet the input's noise
# rather than this repository's debt.
EXCLUDE=(
  "rash/benches/fixtures/large.sh"
  "rash/benches/fixtures/medium.sh"
  "rash/benches/fixtures/small.sh"
)

is_excluded() {
  local candidate="$1" ex
  for ex in "${EXCLUDE[@]}"; do
    [ "$candidate" = "$ex" ] && return 0
  done
  return 1
}

# Tracked files when $ROOT is a git work tree (the real repository); a
# plain recursive `find` otherwise, so a hermetic fixture copy (this gate's
# own integration test) that is never `git init`-ed still enumerates.
list_sh_files() {
  if git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$ROOT" ls-files '*.sh'
  else
    find . -type f -name '*.sh' | sed 's#^\./##' | LC_ALL=C sort
  fi
}

# Error-severity diagnostic count for one file. Prints a bare integer on
# success; prints nothing and returns 1 if `bashrs lint --format json`
# produced no parseable diagnostics at all (a crash, a missing file) —
# UNMEASURED is the caller's problem to report, never silently 0.
count_errors() {
  local file="$1" json n
  json="$("$BASHRS" lint --format json "$file" 2>/dev/null || true)"
  n="$(printf '%s' "$json" | jq '[.diagnostics[]? | select(.severity == "error")] | length' 2>/dev/null || true)"
  case "$n" in
    ''|*[!0-9]*) return 1 ;;
  esac
  printf '%s' "$n"
}

mapfile -t all_files < <(list_sh_files)

# --write-ratchet: regenerate $RATCHET from today's measurement and stop.
# Not a gate run — no PASS/FAIL line, because this mode does not judge
# anything, it only records what is true right now.
if [ "${1:-}" = "--write-ratchet" ]; then
  tmp="$(mktemp)"
  trap 'rm -f "$tmp"' EXIT
  for f in "${all_files[@]}"; do
    is_excluded "$f" && continue
    n="$(count_errors "$f")" || fail "bashrs lint --format json ${f} produced no parseable diagnostics while writing the ratchet — UNMEASURED, refusing to write"
    [ "$n" -gt 0 ] && printf '%s\t%s\n' "$n" "$f" >> "$tmp"
  done
  LC_ALL=C sort -k2,2 -t "$(printf '\t')" "$tmp" > "$RATCHET"
  echo "wrote $(wc -l < "$RATCHET") row(s) to ${RATCHET}"
  exit 0
fi

declare -A ratchet
if [ -f "$RATCHET" ]; then
  while IFS=$'\t' read -r errs path; do
    [ -n "$path" ] || continue
    ratchet["$path"]="$errs"
  done < "$RATCHET"
fi

files_checked=0
files_at_ratchet=0
files_improved=0
total_errors=0
failures=()
improvements=()

for f in "${all_files[@]}"; do
  is_excluded "$f" && continue
  files_checked=$((files_checked + 1))

  n="$(count_errors "$f")" || {
    failures+=("${f}: bashrs lint --format json produced no parseable diagnostics — UNMEASURED, not a pass")
    continue
  }
  total_errors=$((total_errors + n))

  recorded="${ratchet[$f]:-}"
  if [ -z "$recorded" ]; then
    if [ "$n" -gt 0 ]; then
      failures+=("${f}: new file with ${n} error(s): add it to the ratchet deliberately or fix it")
    else
      files_at_ratchet=$((files_at_ratchet + 1))
    fi
    continue
  fi

  if [ "$n" -gt "$recorded" ]; then
    failures+=("${f}: ${n} error(s), ratchet allows ${recorded} — regression")
  elif [ "$n" -lt "$recorded" ]; then
    files_improved=$((files_improved + 1))
    improvements+=("${f}: improved from ${recorded} to ${n} error(s) — lower the ratchet with --write-ratchet")
  else
    files_at_ratchet=$((files_at_ratchet + 1))
  fi
done

for line in "${improvements[@]}"; do
  echo "  IMPROVED ${line}"
done
for line in "${failures[@]}"; do
  echo "  ${line}"
done

echo "checked ${files_checked} file(s); ${files_at_ratchet} at ratchet; ${files_improved} improved; ${total_errors} error(s) total"

if [ "${#failures[@]}" -gt 0 ]; then
  fail "${#failures[@]} of ${files_checked} file(s) exceed the selflint ratchet: $(IFS='; '; echo "${failures[*]}")"
fi

echo "GATE S PASS ${files_checked} file(s) checked, ${files_at_ratchet} at ratchet, ${files_improved} improved, ${total_errors} error(s) total"
