#!/usr/bin/env bash
# mutation: s/RELEASE_BAR=17942/RELEASE_BAR=999999999/
# Dogfood gate K — the transpiler corpus, run ONLY inside a sandbox.
#
# Exit code is the gate. The only line a caller must read is the final
# `GATE K PASS|FAIL <detail>`.
#
# WHY BWRAP, ALWAYS. `bashrs corpus run` executes every Bash entry's
# transpiled output with the CALLER's cwd, HOME and PATH (see corpus runner)
# — a corpus entry that shells out is then a corpus entry that touches the
# real machine. bwrap is not an optimisation here, it is the only thing that
# makes "run the corpus" and "run the corpus untrusted" the same sentence.
# bwrap missing is UNMEASURED, never a skip-and-pass.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

fail() {
  echo "GATE K FAIL $1"
  exit 1
}

# shellcheck source=lib/window.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/window.sh"
dogfood_binary
BIN="$DOGFOOD_BIN"

if ! command -v bwrap >/dev/null 2>&1; then
  fail "bwrap is not on PATH — 'bashrs corpus run' executes every entry's transpiled shell output, and running that outside a sandbox is not an option this gate takes; UNMEASURED"
fi

# The release bar (CLAUDE.md, v2 corpus scoring): the corpus must hold at
# least this many entries, or a truncated/broken corpus load would silently
# read as a clean run over nothing (bashrs#284 shipped v7.0.1 exactly that
# way: zero entries loaded scored 0 failures out of 0 and looked green).
RELEASE_BAR=17942

E="$(mktemp -d)"
trap 'rm -rf "${E:?}"' EXIT

rc=0
raw="$(bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp \
  --bind "$E" "$E" --chdir "$E" --unshare-net --die-with-parent \
  "$BIN" corpus run --format json 2>&1)" || rc=$?
if [ "$rc" -ne 0 ]; then
  printf '%s\n' "$raw" | tail -40
  fail "bwrap ... ${BIN} corpus run --format json exited ${rc}"
fi

# Stdout may carry a tracing INFO line (or several) before the JSON object;
# parse from the first line that opens a JSON object, never the whole blob.
json="$(printf '%s\n' "$raw" | sed -n '/^{/,$p')"
if [ -z "$json" ]; then
  printf '%s\n' "$raw" | tail -20
  fail "no line starting with '{' was found in the corpus runner's output — the JSON result is UNMEASURED"
fi

jrc=0
parsed="$(printf '%s\n' "$json" | jq -r '[.total, .passed, .failed] | @tsv' 2>&1)" || jrc=$?
if [ "$jrc" -ne 0 ]; then
  printf '%s\n' "$parsed"
  fail "jq could not read .total/.passed/.failed from the corpus runner's JSON — UNMEASURED"
fi
total="$(printf '%s' "$parsed" | cut -f1)"
passed="$(printf '%s' "$parsed" | cut -f2)"
failed="$(printf '%s' "$parsed" | cut -f3)"

case "$total" in ''|*[!0-9]*) fail "corpus runner reported a non-numeric total (${total}) — UNMEASURED" ;; esac
case "$passed" in ''|*[!0-9]*) fail "corpus runner reported a non-numeric passed count (${passed}) — UNMEASURED" ;; esac
case "$failed" in ''|*[!0-9]*) fail "corpus runner reported a non-numeric failed count (${failed}) — UNMEASURED" ;; esac

if [ "$total" -lt "$RELEASE_BAR" ]; then
  fail "corpus total is ${total}, release bar is ${RELEASE_BAR} — a corpus this small may have failed to load rather than shrunk"
fi
if [ "$failed" -ne 0 ]; then
  fail "${failed} of ${total} corpus entries failed"
fi
if [ "$passed" -ne "$total" ]; then
  fail "passed (${passed}) does not equal total (${total}) with 0 reported failures — the counts do not reconcile, UNMEASURED"
fi

echo "GATE K PASS ${passed}/${total} corpus entries pass inside bwrap (release bar ${RELEASE_BAR}, 0 failed)"
