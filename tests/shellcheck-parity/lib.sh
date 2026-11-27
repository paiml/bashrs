# shared by record.sh and replay.sh. Sourced, not run.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORPUS="${SC_CORPUS:-$HERE/corpus}"
EXPECTED="${SC_EXPECTED:-$HERE/expected.tsv}"
PIN="$(tr -d '[:space:]' <"$HERE/PIN")"

# Print the shellcheck version line and fail (exit 2, UNMEASURED) unless it is the pinned version.
check_pin() {
  command -v shellcheck >/dev/null 2>&1 || { echo "UNMEASURED: shellcheck not on PATH"; exit 2; }
  got="$(shellcheck --version | sed -n 's/^version: //p')"
  echo "shellcheck version: ${got} (pinned: ${PIN})"
  [ "$got" = "$PIN" ] || { echo "UNMEASURED: shellcheck ${got} is not the pinned ${PIN}; re-record on purpose with a new PIN"; exit 2; }
}

# Emit "file<TAB>SCcode<TAB>count" for every corpus file, sorted, from the live shellcheck.
measure() {
  for f in "$CORPUS"/*.sh; do
    name="$(basename "$f")"
    shellcheck -f gcc -s bash "$f" 2>/dev/null | grep -o '\[SC[0-9]*\]' | tr -d '[]' \
      | sort | uniq -c | awk -v n="$name" '{print n "\t" $2 "\t" $1}'
  done
}
