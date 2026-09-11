#!/bin/bash
# PMAT-255 item 2 (PMAT-250) acceptance probe.
# Verifies shell_words carries command substitutions as a new Expansion kind,
# SC2046 consumes it, the private scanner is gone, and no SC2046 verdict
# changed (six CLI probes + >= 4 new unit tests).
set -u
REPO="${REPO:-/mnt/nvme-raid0/wt/bashrs-PMAT-255-lint}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/mnt/nvme-raid0/targets/bashrs-251c}"
BASHRS="${BASHRS:-$CARGO_TARGET_DIR/debug/bashrs}"
SCRATCH="$(dirname "$0")"
fail=0

check() {
    local name="$1" file="$2" expect_count="$3"
    out="$("$BASHRS" lint "$file" 2>&1)"
    hits="$(printf '%s\n' "$out" | grep -c 'SC2046' || true)"
    if [ "$hits" -eq "$expect_count" ]; then
        echo "PASS: $name (SC2046 x$hits, as expected)"
    else
        echo "FAIL: $name expected $expect_count SC2046, got $hits:"
        printf '%s\n' "$out"
        fail=1
    fi
}

# Probe 1: plain unquoted command substitution in argument position - fires.
printf 'echo $(date)\n' > "$SCRATCH/p1.sh"
check "p1 echo \$(date)" "$SCRATCH/p1.sh" 1

# Probe 2: assignment RHS - the shell never word-splits it - clean.
printf 'x=$(date)\n' > "$SCRATCH/p2.sh"
check "p2 assignment RHS" "$SCRATCH/p2.sh" 0

# Probe 3: inside double quotes - clean.
printf 'echo "$(date)"\n' > "$SCRATCH/p3.sh"
check "p3 double-quoted" "$SCRATCH/p3.sh" 0

# Probe 4: case operand - clean.
printf 'case $(uname) in\n  Linux) echo l ;;\n  *) echo o ;;\nesac\n' > "$SCRATCH/p4.sh"
check "p4 case operand" "$SCRATCH/p4.sh" 0

# Probe 5: nested substitutions - both levels reported.
printf 'echo $(echo $(date))\n' > "$SCRATCH/p5.sh"
check "p5 nested" "$SCRATCH/p5.sh" 2

# Probe 6: backtick form still reported.
printf 'echo `date`\n' > "$SCRATCH/p6.sh"
check "p6 backtick" "$SCRATCH/p6.sh" 1

rm -f "$SCRATCH/p1.sh" "$SCRATCH/p2.sh" "$SCRATCH/p3.sh" "$SCRATCH/p4.sh" "$SCRATCH/p5.sh" "$SCRATCH/p6.sh"

# Private scanner must be gone from sc2046.rs.
SC2046_FILE="$REPO/rash/src/linter/rules/sc2046.rs"
for banned in scan_word find_close read_backtick skip_quoted find_matching_paren; do
    if grep -q "fn $banned" "$SC2046_FILE"; then
        echo "FAIL: fn $banned still present in sc2046.rs"
        fail=1
    else
        echo "PASS: fn $banned absent from sc2046.rs"
    fi
done

# >= 4 new unit tests, plus the existing sc2046/lexer_context suites, all green.
cd "$REPO" || exit 1
test_out="$(CARGO_TARGET_DIR="$CARGO_TARGET_DIR" command cargo test -p bashrs --lib -- \
    linter::shell_words::tests_pmat250 \
    linter::rules::sc2046 \
    linter::lexer_context_tests 2>&1)"
echo "$test_out" | tail -5

new_test_count="$(printf '%s\n' "$test_out" | grep -c 'test_PMAT255_pmat250_.* ok')"
if [ "$new_test_count" -ge 4 ]; then
    echo "PASS: $new_test_count test_PMAT255_pmat250_* tests passed (>= 4)"
else
    echo "FAIL: only $new_test_count test_PMAT255_pmat250_* tests passed (need >= 4)"
    fail=1
fi

if printf '%s\n' "$test_out" | grep -q "FAILED"; then
    echo "FAIL: some test in the sc2046/shell_words/lexer_context suites failed"
    fail=1
else
    echo "PASS: sc2046, shell_words and lexer_context suites all green"
fi

if [ "$fail" -eq 0 ]; then
    echo "ALL PMAT-250 PROBES PASSED"
    exit 0
else
    echo "PMAT-250 PROBE FAILURES"
    exit 1
fi
