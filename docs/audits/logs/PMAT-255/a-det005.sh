#!/usr/bin/env bash
# A_6, PMAT-255 (#232). PASS iff tests pass (>= 6); the four #232 examples raise DET005 at warning and no DET002;
# the duration example raises neither; DET002 still fires on a timestamp written to a file; the book documents DET005.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-lint}; cd "$WT" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251c}
. /tmp/claude-1000/-home-noah-src-bashrs/bf151141-60ff-4e66-9794-31fe5c18982d/scratchpad/a-lib.sh
fail=0; tests_ok test_PMAT255_gh232_ 6 || fail=1
cargo build -p bashrs --bin bashrs -q 2>/dev/null || { echo "FAIL: build"; exit 1; }
B=$CARGO_TARGET_DIR/debug/bashrs; t=$(mktemp -d)
printf '#!/bin/sh\nif [ "$(date +%%H)" -lt 6 ]; then echo early; fi\n' > $t/d1.sh
printf '#!/bin/sh\ndeadline=5\nwhile [ "$(date +%%s)" -lt "$deadline" ]; do sleep 1; done\n' > $t/d2.sh
printf '#!/bin/sh\ncase "$(date +%%u)" in 6|7) echo weekend ;; esac\n' > $t/d3.sh
printf '#!/bin/sh\nexpiry=1\n[ "$(date +%%s)" -gt "$expiry" ] && exit 1\n' > $t/d4.sh
printf '#!/bin/sh\nstart=$(date +%%s)\nsleep 1\nend=$(date +%%s)\nelapsed=$(( end - start ))\necho "$elapsed"\n' > $t/d5.sh
printf '#!/bin/sh\necho "$(date)" > VERSION\n' > $t/d6.sh
for d in d1 d2 d3 d4; do [ "$(n $t/$d.sh DET005)" -ge 1 ] || { echo "FAIL: $d no DET005"; fail=1; }; [ "$(n $t/$d.sh DET002)" = 0 ] || { echo "FAIL: $d also DET002"; fail=1; }
  has_level $t/$d.sh DET005 warning || { echo "FAIL: $d DET005 missing or not at warning"; fail=1; }; done
[ "$(n $t/d5.sh DET005)" = 0 ] && [ "$(n $t/d5.sh DET002)" = 0 ] || { echo "FAIL: d5 duration reported (DET005=$(n $t/d5.sh DET005) DET002=$(n $t/d5.sh DET002))"; fail=1; }
[ "$(n $t/d6.sh DET002)" -ge 1 ] || { echo "FAIL: d6 DET002 true positive lost"; fail=1; }
grep -q 'DET005' book/src/linting/determinism.md 2>/dev/null || { echo "FAIL: book/src/linting/determinism.md does not document DET005"; fail=1; }
[ $fail = 0 ] && echo "A6: PASS"; exit $fail
