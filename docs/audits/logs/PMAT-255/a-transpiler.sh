#!/usr/bin/env bash
# A_4, PMAT-255 phase 4 (#293, #294). PASS iff the phase's tests pass (>= 6); array_join and array_len see the elements of a
# local array literal; a single-quoted literal holding a backtick or $( ) prints verbatim; exec strings holding them are STILL refused.
set -uo pipefail
WT=${WT:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-tr}; cd "$WT" || exit 2
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251f}
. /tmp/claude-1000/-home-noah-src-bashrs/bf151141-60ff-4e66-9794-31fe5c18982d/scratchpad/a-lib.sh
fail=0; tests_ok test_PMAT255_ 6 || fail=1
cargo build -p bashrs --bin bashrs -q 2>/dev/null || { echo "FAIL: build"; exit 1; }
B=$CARGO_TARGET_DIR/debug/bashrs; t=$(mktemp -d)
run() { printf '%s\n' "$3" > "$t/$1.rs"; if ! "$B" build "$t/$1.rs" -o "$t/$1.sh" > "$t/$1.log" 2>&1; then echo "FAIL: $1 does not transpile: $(grep -m1 -i error "$t/$1.log")"; fail=1; return; fi
  for sh in dash bash; do got=$($sh "$t/$1.sh" 2>&1); [ "$got" = "$2" ] || { echo "FAIL: $1 under $sh printed [$got], want [$2]"; fail=1; }; done; }
refuse() { printf '%s\n' "$2" > "$t/$1.rs"; if "$B" build "$t/$1.rs" -o "$t/$1.sh" >/dev/null 2>&1; then echo "FAIL: $1 transpiled; an exec string that would run a substitution must stay refused"; fail=1; fi; }
run join  'a,b'                  'fn main() { let items = ["a", "b"]; let joined = array_join(items, ","); println!("{}", joined); }'
run len   '2'                    'fn main() { let items = ["a", "b"]; let n = array_len(items); println!("{}", n); }'
run forin "$(printf 'a\nb')"     'fn main() { let items = ["a", "b"]; for x in items { println!("{}", x); } }'
run tick  'a `b` c'              'fn main() { let s = "a `b` c"; println!("{}", s); }'
run dollar 'cost $(x) and $HOME' 'fn main() { let s = "cost $(x) and $HOME"; println!("{}", s); }'
refuse exectick   'fn main() { exec("echo `date`"); }'
refuse execdollar 'fn main() { exec("echo $(date)"); }'
[ $fail = 0 ] && echo "A4: PASS"; exit $fail
