#!/usr/bin/env bash
# mutation: s/if \[ "\$rc" -ne 0 \]; then/if [ "$rc" -eq 0 ]; then/
# Dogfood gate D — README.md's documented `bashrs` invocations, and the
# pre-release book check, both run for real against the built binary.
#
# Exit code is the gate. The only line a caller must read is the final
# `GATE D PASS|FAIL <detail>`.
#
# Two things this gate proves and one it does not (same honesty boundary as
# forjar's docs.sh): the argv SHAPE of every documented `bashrs` line — this
# verb, these flags, in this order, against a fixture of this kind — exits 0
# against THIS binary. It does not prove the reader's own files exist; that
# is out of scope for a hermetic gate that runs offline.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

fail() {
  echo "GATE D FAIL $1"
  exit 1
}

# shellcheck source=lib/window.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/window.sh"
dogfood_binary
BIN="$DOGFOOD_BIN"

# ------------------------------------------------------- check-book-updated.sh
#
# check-book-updated.sh has FIVE checks; the fourth ("book/ touched since the
# last tag") is a pre-RELEASE gate, not a pre-COMMIT one — `dogfood` (unlike
# `dogfood-release`) is documented to run on every commit (Makefile), and a
# tree with no book work in flight would fail check 4 on every single commit
# that is not a book change, which is not what check 4 is for (its own
# docstring: "Enforce book updates before release"). SKIP_BOOK_CHECK=1 is the
# script's own documented escape for exactly this: it is not this gate's
# --skip, it is check-book-updated.sh's, and checks 1-3 (book still BUILDS
# and its examples still PASS) run unskipped below and are the ones that
# catch doc rot.
if [ ! -x scripts/check-book-updated.sh ] && [ ! -f scripts/check-book-updated.sh ]; then
  fail "scripts/check-book-updated.sh is missing — the pre-release book check is UNMEASURED"
fi
book_out=""
book_rc=0
book_out="$(SKIP_BOOK_CHECK=1 bash scripts/check-book-updated.sh 2>&1)" || book_rc=$?
if [ "$book_rc" -ne 0 ]; then
  printf '%s\n' "$book_out" | tail -20
  fail "scripts/check-book-updated.sh exited ${book_rc} (SKIP_BOOK_CHECK=1 set for check 4 only; checks 1-3 still ran) — the book does not build or an example does not pass"
fi

# --------------------------------------------------- README invocations extraction
#
# Only fenced blocks tagged bash/sh/shell/console (or untagged) hold runnable
# lines; a `toml`/`text`/`rust` fence is prose or source, not a command. A
# line's LEADING `$ ` (the reader-facing prompt) is stripped before the
# first-word test, same as forjar.
SHELL_FENCES="bash sh shell console "
EXTRACT="$(mktemp)"
trap 'rm -f "$EXTRACT"' EXIT
python3 - "$EXTRACT" <<'PY'
import re, sys
out_path = sys.argv[1]
inside, lang = False, ""
rows = []
for lineno, raw in enumerate(open("README.md", encoding="utf-8").read().splitlines(), 1):
    if raw.startswith("```"):
        inside, lang = (not inside), (raw[3:].strip() if not inside else "")
        continue
    if not inside:
        continue
    if lang not in ("bash", "sh", "shell", "console", ""):
        continue
    text = re.sub(r"^\$\s+", "", raw.strip())
    if re.match(r"^bashrs(\s|$)", text):
        rows.append(f"{lineno}\t{text}")
with open(out_path, "w", encoding="utf-8") as fh:
    fh.write("\n".join(rows))
    if rows:
        fh.write("\n")
PY

n_inv="$(grep -c . "$EXTRACT" || true)"
# Vacuity floor: today's count of fenced `bashrs` invocations in README.md. A
# README that stopped documenting anything would satisfy every loop below.
MIN_INVOCATIONS=7
if [ "$n_inv" -lt "$MIN_INVOCATIONS" ]; then
  fail "README.md holds ${n_inv} fenced bashrs invocation(s), floor is ${MIN_INVOCATIONS} — a README that documents nothing passes every check below"
fi

# ------------------------------------------------------------------- fixtures
#
# One sandbox per run. Fixtures are named EXACTLY as README's operands
# (install.rs, input.rs, messy.sh, script.sh) so the documented lines run
# UNCHANGED — no rebind table, because running from inside the sandbox with
# those names present already IS running the line as printed.
SANDBOX="$(mktemp -d)"
trap 'rm -rf "${SANDBOX:?}" "$EXTRACT"' EXIT

cat > "${SANDBOX}/install.rs" <<'RS'
fn main() {
    let version = env_var_or("VERSION", "1.0.0");
    echo("Installing {version}");
}
RS
cp "${SANDBOX}/install.rs" "${SANDBOX}/input.rs"

cat > "${SANDBOX}/messy.sh" <<'SH'
#!/bin/bash
mkdir /app/releases/foo
rm /app/current
SH

cat > "${SANDBOX}/script.sh" <<'SH'
#!/bin/sh
set -eu
name="${1:-world}"
echo "Hello, ${name}!"
SH

# Documented invocations that DO NOT WORK against this binary, each with the
# defect that stops it. A ratchet, not an excuse list: an entry here that
# starts passing is a failure too ("remove it"), so the exception cannot
# outlive the defect it names. Empty today — every documented line runs.
declare -A KNOWN_BROKEN=()

known_broken_reason() {
  local text="$1" prefix
  for prefix in "${!KNOWN_BROKEN[@]}"; do
    case "$text" in
      "$prefix"*) printf '%s' "$prefix"; return 0 ;;
    esac
  done
  return 1
}

failures=0
ran=0
broken=0
while IFS=$'\t' read -r lineno text; do
  [ -n "$text" ] || continue
  ran=$((ran + 1))
  # shellcheck disable=SC2206 — word-splitting the documented line is the point
  argv=($text)
  verb_argv=("${argv[@]:1}")
  # `repl` reads a command loop from stdin; a gate has no one to type into it,
  # so stdin is closed and the REPL's own EOF handling is what is measured —
  # the same "stdin from /dev/null" every other invocation gets here.
  rc=0
  # $SANDBOX is our own mktemp -d, never reader input.
  # bashrs disable-next-line=SEC010
  out="$(cd "$SANDBOX" && "$BIN" "${verb_argv[@]}" < /dev/null 2>&1)" || rc=$?
  if prefix="$(known_broken_reason "$text")"; then
    if [ "$rc" -eq 0 ]; then
      failures=$((failures + 1))
      echo "  README:${lineno}: \`${text}\` now SUCCEEDS but is still listed in KNOWN_BROKEN — remove it"
    else
      broken=$((broken + 1))
      echo "  README:${lineno}: KNOWN-BROKEN \`${text}\` — ${KNOWN_BROKEN[$prefix]}"
    fi
    continue
  fi
  if [ "$rc" -ne 0 ]; then
    failures=$((failures + 1))
    echo "  README:${lineno}: \`${text}\` exited ${rc}"
    printf '%s\n' "$out" | tail -5 | sed 's/^/    /'
  else
    echo "  README:${lineno}: RUN \`${text}\`"
  fi
done < "$EXTRACT"

if [ "$failures" -ne 0 ]; then
  fail "${failures} documented bashrs invocation(s) in README.md do not hold against ${BIN}"
fi

echo "GATE D PASS ${ran} documented bashrs invocation(s) from README.md run against ${BIN} (${broken} pinned known-broken); scripts/check-book-updated.sh passes (checks 1-3; check 4 is release-only, see comment above)"
