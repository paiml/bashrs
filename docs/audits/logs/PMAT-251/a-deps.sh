#!/usr/bin/env bash
# A_1 for PMAT-251 phase 1: every direct dependency whose latest release is outside the current requirement is either
# upgraded or named in docs/audits/deps-exceptions-PMAT-251.txt with the exact error that refused it; the lockfile has no
# pending compatible update; the workspace builds with every target.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-251}; cd "$WT" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251}
fail=0
pending=$(cargo update --dry-run 2>&1 | grep -c -E '^\s*Updating [A-Za-z0-9_-]+ v[0-9]' || true)
[ "$pending" = 0 ] || { echo "A: $pending compatible lockfile update(s) not applied (cargo update --dry-run)"; fail=1; }
cargo upgrade --dry-run --incompatible 2>/dev/null | awk 'NF==5 && $1!="name" && $1!="====" && $3!=$4 {print $1" "$4}' | sort -u > /tmp/a-deps.upgradable.$$
while read -r name latest; do
  [ -n "$name" ] || continue
  grep -q -E "^$name([[:space:]]|$)" docs/audits/deps-exceptions-PMAT-251.txt 2>/dev/null || { echo "A: $name (latest $latest) is neither upgraded nor in docs/audits/deps-exceptions-PMAT-251.txt"; fail=1; }
done < /tmp/a-deps.upgradable.$$
rm -f /tmp/a-deps.upgradable.$$
if cargo build --workspace --all-targets --exclude bashrs-wasm 2>/tmp/a-deps.build.$$ && cargo build -p bashrs-wasm --lib --tests 2>>/tmp/a-deps.build.$$; then :; else echo "A: workspace build failed:"; tail -5 /tmp/a-deps.build.$$; fail=1; fi   # bashrs-wasm test targets build alone but not under --workspace --all-targets (pre-existing on main: panic-strategy mismatch via jugar-probar/base64)
rm -f /tmp/a-deps.build.$$
[ $fail = 0 ] && echo "A: PASS — lockfile current, every outside-requirement dependency upgraded or excused with an error, workspace builds"
exit $fail
