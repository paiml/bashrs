#!/usr/bin/env bash
# A_5, PMAT-255 (PMAT-253 phase 5). PASS iff scripts/publish-from-tag.sh: DRY_RUN=1 on v7.0.4 exits 0 and dry-runs bashrs-oracle
# before bashrs; it refuses with exit 2, before any cargo call, a tag that does not exist, a tag whose Cargo.toml version differs
# from the tag, and a --worktree that is not clean; `make publish-from-tag TAG=v7.0.4 DRY_RUN=1` is wired; release.yml waits for
# the tag's bashrs-oracle version on the crates.io index before `cargo package -p bashrs`.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-pub}; cd "$WT" || exit 2
S=scripts/publish-from-tag.sh; fail=0
[ -x "$S" ] || { echo "FAIL: $S missing or not executable"; exit 1; }
out=$(DRY_RUN=1 bash "$S" v7.0.4 2>&1); rc=$?; [ $rc = 0 ] || { echo "FAIL: dry run on v7.0.4 exit $rc: $(echo "$out" | tail -2)"; fail=1; }
o=$(echo "$out" | grep -n 'bashrs-oracle' | head -1 | cut -d: -f1); b=$(echo "$out" | grep -nE '(^|[^-])bashrs( |$|@|v7)' | grep -v oracle | head -1 | cut -d: -f1)
[ -n "$o" ] && [ -n "$b" ] && [ "$o" -lt "$b" ] || { echo "FAIL: the dry run does not name bashrs-oracle before bashrs (oracle line ${o:-none}, bashrs line ${b:-none})"; fail=1; }
out=$(DRY_RUN=1 bash "$S" v0.0.0-does-not-exist 2>&1); rc=$?; [ $rc = 2 ] || { echo "FAIL: a missing tag gave exit $rc, want 2"; fail=1; }
git tag -f pmat255-mismatch-probe HEAD >/dev/null 2>&1; out=$(DRY_RUN=1 bash "$S" pmat255-mismatch-probe 2>&1); rc=$?; git tag -d pmat255-mismatch-probe >/dev/null 2>&1
[ $rc = 2 ] || { echo "FAIL: a tag whose version is not the Cargo.toml version gave exit $rc, want 2"; fail=1; }
w=$(mktemp -d)/wt; git worktree add -q --detach "$w" v7.0.4 >/dev/null 2>&1 && touch "$w/untracked-probe"; out=$(DRY_RUN=1 bash "$S" v7.0.4 --worktree "$w" 2>&1); rc=$?; git worktree remove --force "$w" >/dev/null 2>&1
[ $rc = 2 ] || { echo "FAIL: a dirty --worktree gave exit $rc, want 2"; fail=1; }
make -n publish-from-tag TAG=v7.0.4 DRY_RUN=1 >/dev/null 2>&1 || { echo "FAIL: make publish-from-tag is not wired"; fail=1; }
grep -qE 'index\.crates\.io/ba/sh/bashrs-oracle|crates\.io/api/v1/crates/bashrs-oracle|cargo (search|info) bashrs-oracle' .github/workflows/release.yml || { echo "FAIL: release.yml does not wait for bashrs-oracle on the index"; fail=1; }
[ $fail = 0 ] && echo "A5: PASS"; exit $fail
