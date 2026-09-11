#!/usr/bin/env bash
# publish-from-tag.sh -- publish bashrs-oracle, then bashrs, to crates.io from
# a clean checkout of a release tag.
#
# WHY (PMAT-255, PMAT-253 phase 5): the v7.0.4 release (2026-09-10, GitHub
# Actions run 34495173909) failed `cargo package -p bashrs --locked` on
# attempt 1 with "failed to select a version for the requirement
# bashrs-oracle = ^7.0", because the tag was pushed only 16s before
# bashrs-oracle 7.0.4 reached the crates.io index -- rash/Cargo.toml pins
# bashrs-oracle by path+version, and packaging/publishing bashrs resolves
# that dependency from the registry, not the path, so it needs bashrs-oracle
# already indexed. Attempt 2, unchanged, passed once the index caught up.
# The orchestrator then published by hand from a detached worktree of the
# tag: bashrs-oracle first, then bashrs. This script encodes that procedure.
#
# Usage:
#   scripts/publish-from-tag.sh vX.Y.Z [--worktree DIR]
#   DRY_RUN=1 scripts/publish-from-tag.sh vX.Y.Z [--worktree DIR]
#
# Refusals (exit 2, BEFORE any cargo call):
#   * no TAG, or TAG not shaped like vX.Y.Z
#   * TAG does not exist
#   * the workspace version (read from Cargo.toml's [workspace.package]
#     section AT THE TAG, never from the ambient working tree) disagrees
#     with the tag name
#   * the worktree is dirty (`git status --porcelain` non-empty) -- NEVER
#     `--allow-dirty`
# Refuses (exit 2) later if bashrs-oracle's published version never appears
# on the crates.io index within the bounded poll.
#
# --worktree DIR: the caller supplies an existing checkout already at TAG,
# and owns its lifecycle -- this script only reads it and never removes it.
# Without --worktree, a fresh detached worktree of TAG is created under
# $TMPDIR and removed on exit (never the ambient working tree: an untracked
# file anywhere in it would make `cargo publish` refuse without
# `--allow-dirty`, the one flag this script exists to avoid).
set -uo pipefail

# The sparse index cargo itself resolves against (not the web API, which answers
# before the index has caught up): https://index.crates.io/<a>/<b>/<name>.
CRATES_SPARSE_INDEX="${CRATES_SPARSE_INDEX:-https://index.crates.io}"
USER_AGENT="bashrs-publish-from-tag (https://github.com/paiml/bashrs)"

# Dependency order: bashrs-oracle before bashrs. Both share the workspace
# version (rash/Cargo.toml and bashrs-oracle/Cargo.toml both set
# `version.workspace = true`), so one version read covers both crates.
CRATES=(bashrs-oracle bashrs)

die() {
  printf '%s\n' "$*" >&2
  exit 2
}

# A failure AFTER a cargo call has already run (the publish/dry-run itself
# failing) is not one of this script's own pre-flight refusals, so it is
# reported with a distinct exit code (1) rather than the "refused before
# any cargo call" exit 2.
fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

usage() {
  printf 'usage: %s vX.Y.Z [--worktree DIR]\n' "${0##*/}" >&2
  printf '       DRY_RUN=1 %s vX.Y.Z [--worktree DIR]\n' "${0##*/}" >&2
  exit 2
}

# Every command this script runs is echoed to stderr before it runs, so a
# human -- or a test log -- can see the exact sequence without guessing.
run() {
  printf '+ %s\n' "$*" >&2
  "$@"
}

# --------------------------------------------------------------- arguments --

TAG=""
CALLER_WORKTREE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --worktree)
      [ $# -ge 2 ] || usage
      CALLER_WORKTREE="$2"
      shift 2
      ;;
    -h | --help)
      usage
      ;;
    -*)
      usage
      ;;
    *)
      [ -z "$TAG" ] || usage
      TAG="$1"
      shift
      ;;
  esac
done
[ -n "$TAG" ] || usage

# ------------------------------------------------------------ TAG refusals --

case "$TAG" in
  v[0-9]*.[0-9]*.[0-9]*) ;;
  *) die "refusing: '$TAG' is not shaped like vX.Y.Z" ;;
esac

git rev-parse -q --verify "refs/tags/$TAG" >/dev/null ||
  die "refusing: tag '$TAG' does not exist"

# The workspace version AT THE TAG, never from the ambient working tree --
# the two can disagree if main has moved since the tag. bashrs's own root
# Cargo.toml carries the shared version under [workspace.package], not
# [package] (the root [package] is bashrs-specs, publish = false).
workspace_version_at_tag() {
  git show "$TAG:Cargo.toml" | awk '
    /^\[workspace\.package\]/ { in_ws = 1; next }
    /^\[/ { in_ws = 0 }
    in_ws && /^version[[:space:]]*=/ {
      line = $0
      sub(/^version[[:space:]]*=[[:space:]]*"/, "", line)
      sub(/".*/, "", line)
      print line
      exit
    }
  '
}

tag_version="${TAG#v}"
manifest_version="$(workspace_version_at_tag)"
[ -n "$manifest_version" ] ||
  die "refusing: no [workspace.package] version found in $TAG:Cargo.toml"
[ "$manifest_version" = "$tag_version" ] ||
  die "refusing: Cargo.toml [workspace.package] version '$manifest_version' at $TAG does not match tag version '$tag_version'"

# --------------------------------------------------------------- worktree --

REMOVE_WORKTREE_ON_EXIT=0
if [ -n "$CALLER_WORKTREE" ]; then
  WT="$CALLER_WORKTREE"
  [ -d "$WT" ] || die "refusing: --worktree '$WT' does not exist"
else
  WT="$(mktemp -d "${TMPDIR:-/tmp}/bashrs-publish-XXXXXX")"
  REMOVE_WORKTREE_ON_EXIT=1
fi

cleanup() {
  if [ "$REMOVE_WORKTREE_ON_EXIT" = "1" ]; then
    printf '+ git worktree remove --force %s\n' "$WT" >&2
    git worktree remove --force "$WT" >/dev/null 2>&1 || true
    # `:?` is not decoration: an unset or empty WT here would make this
    # `rm -rf` a command about /, and this runs from a trap.
    rm -rf "${WT:?}" || true
  fi
}
trap cleanup EXIT

if [ "$REMOVE_WORKTREE_ON_EXIT" = "1" ]; then
  run git worktree add --detach "$WT" "$TAG"
fi

# NEVER `--allow-dirty`: a dirty worktree (caller-supplied or freshly
# created) refuses before any cargo call, whether the dirt is untracked
# files, unignored litter, or local modifications.
assert_worktree_clean() {
  local dirty
  dirty="$(git -C "$WT" status --porcelain)"
  if [ -n "$dirty" ]; then
    printf 'refusing: the worktree at %s is not clean:\n%s\n' "$WT" "$dirty" >&2
    exit 2
  fi
}

assert_worktree_clean

# A caller-supplied worktree must BE the tag: otherwise whatever it has checked
# out would be published under the tag's version.
tag_commit="$(git rev-parse "$TAG^{commit}")"
wt_commit="$(git -C "$WT" rev-parse HEAD 2>/dev/null || true)"
[ "$wt_commit" = "$tag_commit" ] ||
  die "refusing: the worktree at $WT is at ${wt_commit:-no commit}, not $TAG ($tag_commit)"

# ---------------------------------------------------------- crates.io poll --

# Bounded backoff over a delay sequence, never a single fixed sleep: the
# index is eventually consistent after a publish, and the dependent crate
# resolving against it too early is exactly the v7.0.4 failure this script
# exists to prevent. When the bound is exhausted the run refuses (exit 2)
# naming the crate and version, rather than proceeding against a stale
# index.
# Sparse-index path of a crate name, per the cargo registry layout.
index_path() {
  local n
  n="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
  case ${#n} in
    1) printf '1/%s' "$n" ;;
    2) printf '2/%s' "$n" ;;
    3) printf '3/%s/%s' "${n:0:1}" "$n" ;;
    *) printf '%s/%s/%s' "${n:0:2}" "${n:2:2}" "$n" ;;
  esac
}

already_on_index() {
  local name="$1" version="$2"
  curl -fsS -A "$USER_AGENT" --max-time 10 "$CRATES_SPARSE_INDEX/$(index_path "$name")" 2>/dev/null |
    grep -q "\"vers\":\"$version\""
}

poll_index() {
  local name="$1" version="$2" delay
  local -a delays=()
  IFS=' ' read -r -a delays <<<"${PUBLISH_POLL_DELAYS:-1 2 4 8 8 8 8 8}"
  if already_on_index "$name" "$version"; then
    printf 'waiting for the index: %s %s is already on crates.io\n' "$name" "$version" >&2
    return 0
  fi
  for delay in "${delays[@]}"; do
    sleep "$delay"
    if already_on_index "$name" "$version"; then
      printf 'waiting for the index: %s %s appeared on crates.io\n' "$name" "$version" >&2
      return 0
    fi
  done
  die "refusing: $name $version never appeared on the crates.io index after publishing (bounded poll exhausted)"
}

# --------------------------------------------------------------- publishing --

cd "$WT" || exit 2

version="$manifest_version"
prev=""
for crate in "${CRATES[@]}"; do
  printf '\n=== %s %s ===\n' "$crate" "$version" >&2

  # bashrs depends on bashrs-oracle by path+version; publishing (or
  # dry-run publishing) bashrs resolves that dependency from the registry,
  # so bashrs-oracle must already be indexed first.
  if [ -n "$prev" ] && [ "${DRY_RUN:-0}" = "1" ] && ! already_on_index "$prev" "$version"; then
    # A dry run publishes nothing, so $prev $version cannot appear on the index,
    # and $crate's dry run resolves it from the registry. Say so; do not poll.
    printf 'DRY_RUN: skipping %s %s — it resolves %s %s from the registry, which a dry run never publishes\n' \
      "$crate" "$version" "$prev" "$version" >&2
    continue
  fi
  if [ -n "$prev" ]; then
    poll_index "$prev" "$version"
  fi

  if [ "${DRY_RUN:-0}" = "1" ]; then
    run env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --locked -p "$crate" ||
      fail "cargo publish --dry-run failed for $crate $version"
  else
    run env -u CARGO_REGISTRY_TOKEN cargo publish --locked -p "$crate" ||
      fail "cargo publish failed for $crate $version"
  fi

  prev="$crate"
done
