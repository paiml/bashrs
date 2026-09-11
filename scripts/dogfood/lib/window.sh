#!/usr/bin/env bash
# Shared helpers for the hermetic dogfood gates (D, G, K — PMAT-255/PMAT-253
# phase 3a). SOURCED, NOT RUN: sourcing this file defines functions and does
# nothing else — no output, no process launched, no side effect at source
# time. A helper with a side effect at source time would run before the
# caller's `fail()` exists and could die with no verdict line, which is the
# one outcome every gate in this directory exists to refuse (see forjar's
# scripts/dogfood/lib/window.sh, same rule, different question).
#
# The caller must define `fail()` (printing its own `GATE <letter> FAIL …`
# and exiting non-zero) BEFORE sourcing this file.

# The release binary this gate measures — NEVER `bashrs` resolved from PATH.
# A PATH lookup would silently grade whatever version a previous `cargo
# install` or `cargo publish --dry-run` left lying around, not the tree under
# test.
#
# The target directory is cargo's own answer for this workspace: CARGO_TARGET_DIR
# when the caller sets it, otherwise `cargo metadata`'s target_directory (which
# honours .cargo/config.toml as well). Neither is a guess; failing to get either
# is UNMEASURED, never ".". `make dogfood` builds the release binary into that
# directory first, so the binary graded is the tree under test.
dogfood_binary() {
  local dir="${CARGO_TARGET_DIR:-}"
  if [ -z "$dir" ]; then
    dir=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | jq -r '.target_directory // empty' 2>/dev/null)
  fi
  if [ -z "$dir" ]; then
    fail "cannot resolve the cargo target directory (CARGO_TARGET_DIR unset and cargo metadata failed) — the release binary this gate must run is UNMEASURED"
  fi
  DOGFOOD_BIN="${dir}/release/bashrs"
  if [ ! -x "$DOGFOOD_BIN" ]; then
    fail "${DOGFOOD_BIN} is missing or not executable — build it first with: cargo build --release -p bashrs --bin bashrs"
  fi
}
