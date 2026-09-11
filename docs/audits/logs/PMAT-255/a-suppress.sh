#!/usr/bin/env bash
# A_5, PMAT-255 phase 5 (#265). PASS iff tests pass (>= 5); a bashrs code inside `# shellcheck disable=` is NOT honoured and IS
# reported with the bashrs-directive fix; a `# bashrs disable-line=` on a line with no code is reported, not silently ignored;
# SC codes in shellcheck directives, same-line disable-line and disable-file keep working.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-lint}; cd "$WT" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251c}
. /tmp/claude-1000/-home-noah-src-bashrs/bf151141-60ff-4e66-9794-31fe5c18982d/scratchpad/a-lib.sh
fail=0; tests_ok test_PMAT255_gh265_ 5 || fail=1
cargo build -p bashrs --bin bashrs -q 2>/dev/null || { echo "FAIL: build"; exit 1; }
B=$CARGO_TARGET_DIR/debug/bashrs; t=$(mktemp -d)
printf '#!/bin/bash\n# shellcheck disable=DET002\necho "$(date)" > VERSION\n' > $t/s1.sh
printf '#!/bin/bash\nv="a b"\n# shellcheck disable=SC2086\nls $v\n' > $t/s2.sh
printf '#!/bin/bash\n# bashrs disable-line=DET002\necho "$(date)" > VERSION\n' > $t/s3.sh
printf '#!/bin/bash\necho "$(date)" > VERSION # bashrs disable-line=DET002\n' > $t/s4.sh
printf '#!/bin/bash\n# bashrs disable-file=DET002\necho "$(date)" > VERSION\n' > $t/s5.sh
[ "$(n $t/s1.sh DET002)" -ge 1 ] || { echo "FAIL: s1 DET002 still suppressed by a shellcheck directive"; fail=1; }
msgs $t/s1.sh | grep -qiE 'bashrs disable' || { echo "FAIL: s1 no diagnostic pointing at the bashrs directive"; fail=1; }
[ "$(n $t/s2.sh SC2086)" = 0 ] || { echo "FAIL: s2 shellcheck disable=SC2086 no longer honoured"; fail=1; }
msgs $t/s3.sh | grep -qiE 'disable-line' || { echo "FAIL: s3 a disable-line with no code on its line is still silently ignored"; fail=1; }
[ "$(n $t/s4.sh DET002)" = 0 ] || { echo "FAIL: s4 same-line disable-line broke"; fail=1; }
[ "$(n $t/s5.sh DET002)" = 0 ] || { echo "FAIL: s5 disable-file broke"; fail=1; }
[ $fail = 0 ] && echo "A5: PASS"; exit $fail
