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
# CARGO_TARGET_DIR must already be set by the caller (the orchestrator sets
# it; a gate that guessed a default here could grade a stale binary in a
# different target dir without saying so). Its absence is UNMEASURED, not "."
dogfood_binary() {
  if [ -z "${CARGO_TARGET_DIR:-}" ]; then
    fail "CARGO_TARGET_DIR is unset — the release binary this gate must run cannot be located, and guessing a default could silently grade a stale binary in a different target dir"
  fi
  DOGFOOD_BIN="${CARGO_TARGET_DIR}/release/bashrs"
  if [ ! -x "$DOGFOOD_BIN" ]; then
    fail "${DOGFOOD_BIN} is missing or not executable — build it first with: cargo build --release -p bashrs --bin bashrs"
  fi
}
