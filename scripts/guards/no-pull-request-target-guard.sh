#!/bin/sh
# no-pull-request-target-guard.sh -- fail if a workflow uses the pull_request_target trigger.
#
# pull_request_target runs with a write token and secrets in the BASE repo context, on
# PRs from forks: the "pwn request" hole. paiml accepts no fork contributions (NO-FORK-1),
# so every PR-triggered workflow uses `pull_request` and guards on a same-repo head.
#
# Usage: no-pull-request-target-guard.sh [WORKFLOW_DIR] [ALLOWLIST_FILE]
#   Allowlist lines: `<workflow file basename>  # reason`. An allowlisted file must not
#   contain a checkout step (the fork-pr-closer case: metadata only, never PR code).
# Exit: 0 clean, 1 violation, 2 usage, an unreadable entry, or no workflows scanned
#   (0 files is not a pass; a file the guard could not read is not a clean file).
#
# Matching fails closed. Only full-line comments are dropped: a `#` later on a line may sit
# inside a value (`branches: [feat#1]`, `name: "PR #1"`), so a mention in a trailing comment
# counts as a hit. Before matching, the YAML double-quoted spellings that could carry a name
# past a literal match are undone: escaped line breaks (a backslash at end of line) and
# \x, \u and \U escapes of letters, digits and `_ / . - @`. Matching ignores case.
set -eu

dir="${1:-.github/workflows}"
allow="${2:-.github/pull-request-target-allowlist.txt}"

[ -d "$dir" ] || { echo "guard: no such directory: $dir" >&2; exit 2; }

bs="\\"
# hexpat HEX -> a BRE matching HEX with each hex letter in either case
hexpat() {
  printf '%s' "$1" | sed -e 's/a/[aA]/g; s/b/[bB]/g; s/c/[cC]/g; s/d/[dD]/g; s/e/[eE]/g; s/f/[fF]/g'
}
decode=''
for c in a b c d e f g h i j k l m n o p q r s t u v w x y z \
         A B C D E F G H I J K L M N O P Q R S T U V W X Y Z \
         0 1 2 3 4 5 6 7 8 9 _ / . - @; do
  p=$(hexpat "$(printf '%02x' "'$c")")
  for pre in "${bs}${bs}x" "${bs}${bs}u00" "${bs}${bs}U000000"; do
    decode="${decode}s|${pre}${p}|${c}|g
"
  done
done

# normalize FILE -> the file's text with full-line comments dropped, escaped line breaks
# joined, and the escapes above decoded. Exits non-zero if FILE cannot be read.
normalize() {
  sed -e '/^[[:space:]]*#/d' \
      -e ':a' -e '/\\$/{N;s/\\\n[[:space:]]*//;ba' -e '}' \
      -e "$decode" "$1"
}

scanned=0
bad=0
for f in "$dir"/*.yml "$dir"/*.yaml; do
  # an unmatched glob stays literal and names nothing
  if [ ! -e "$f" ] && [ ! -L "$f" ]; then continue; fi
  if [ ! -f "$f" ] || [ ! -r "$f" ]; then
    echo "guard: cannot read $f (not a readable regular file)" >&2
    exit 2
  fi
  norm=$(normalize "$f") || { echo "guard: cannot read $f" >&2; exit 2; }
  scanned=$((scanned + 1))
  rc=0
  hits=$(printf '%s\n' "$norm" | grep -ci 'pull_request_target') || rc=$?
  [ "$rc" -le 1 ] || { echo "guard: grep failed on $f (rc=$rc)" >&2; exit 2; }
  [ "$hits" -gt 0 ] || continue
  base=$(basename "$f")
  if [ -f "$allow" ] && sed -e 's/[[:space:]]*#.*$//' -e 's/[[:space:]]*$//' "$allow" | grep -qxF -- "$base"; then
    if printf '%s\n' "$norm" | grep -qi 'actions/checkout'; then
      echo "FAIL $f: allowlisted pull_request_target workflow contains actions/checkout" >&2
      bad=$((bad + 1))
    fi
    continue
  fi
  echo "FAIL $f: uses pull_request_target ($hits line(s)); use pull_request with a same-repo head guard" >&2
  bad=$((bad + 1))
done

echo "no-pull-request-target-guard: scanned $scanned workflow file(s), $bad violation(s)"
[ "$scanned" -gt 0 ] || exit 2
[ "$bad" -eq 0 ]
