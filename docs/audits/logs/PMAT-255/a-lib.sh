# shared helpers for the PMAT-255 acceptance scripts (sourced)
# bashrs 7.0.4 prints a tracing INFO line on STDOUT before --format json output; start at the first "{" line.
json() { "$B" lint --format json "$1" 2>/dev/null | sed -n '/^{/,$p'; }
codes() { local out; out=$(json "$1"); if printf '%s' "$out" | jq -e . >/dev/null 2>&1; then printf '%s' "$out" | jq -r '.. | objects | select(has("code")) | .code'; else "$B" lint "$1" 2>&1 | grep -oE '\b(DET|SC|SEC|IDEM|REL|MAKE|DOCKER|BASHRS)[0-9]+\b'; fi; }
levels() { json "$1" | jq -r --arg c "$2" '.. | objects | select(.code? == $c) | (.severity // .level // "?") | ascii_downcase' 2>/dev/null; }
msgs() { local out; out=$(json "$1"); if printf '%s' "$out" | jq -e . >/dev/null 2>&1; then printf '%s' "$out" | jq -r '.. | objects | select(has("message")) | .message'; else "$B" lint "$1" 2>&1; fi; }
n() { codes "$1" | grep -cx "$2"; }
has_level() { local l; l=$(levels "$1" "$2"); [ -n "$l" ] && ! printf '%s\n' "$l" | grep -qvx "$3"; }
tests_ok() { # filter min
  local r; r=$(cargo test -p bashrs --lib -- "$1" 2>&1 | grep -E '^test result' | tail -1); echo "tests[$1]: $r"
  case "$r" in *"test result: ok"*) local k; k=$(printf '%s' "$r" | sed -E 's/.* ([0-9]+) passed.*/\1/'); [ "${k:-0}" -ge "$2" ] || { echo "FAIL: fewer than $2 $1 tests ran ($k)"; return 1; };; *) echo "FAIL: $1 tests missing or failing"; return 1;; esac; }
