#!/bin/bash
# PMAT-255 item 1 (PMAT-244) acceptance probe.
# Verifies SC2105 follows loop *structure*, not per-line keyword counts.
set -u
BASHRS="${BASHRS:-/mnt/nvme-raid0/targets/bashrs-251c/debug/bashrs}"
SCRATCH="$(dirname "$0")"
fail=0

check() {
    local name="$1" file="$2" expect="$3"  # expect: "clean" or "fires"
    out="$("$BASHRS" lint "$file" 2>&1)"
    hits="$(printf '%s\n' "$out" | grep -c 'SC2105' || true)"
    if [ "$expect" = "clean" ]; then
        if [ "$hits" -eq 0 ]; then
            echo "PASS: $name (clean, as expected)"
        else
            echo "FAIL: $name expected clean, got SC2105:"
            printf '%s\n' "$out"
            fail=1
        fi
    else
        if [ "$hits" -gt 0 ]; then
            echo "PASS: $name (fires, as expected)"
        else
            echo "FAIL: $name expected SC2105 to fire, got:"
            printf '%s\n' "$out"
            fail=1
        fi
    fi
}

# v1: one-line for-loop with && { ...; break; }
printf 'for m in a b; do [[ $m == b ]] && { x=1; break; }; done\n' > "$SCRATCH/v1.sh"
check "v1 inline &&-break" "$SCRATCH/v1.sh" clean

# v2: one-line for-loop with if/then/fi break
printf 'for m in a b; do if [[ $m == b ]]; then x=1; break; fi; done\n' > "$SCRATCH/v2.sh"
check "v2 inline if-break" "$SCRATCH/v2.sh" clean

# v3: same loop over several lines - must stay clean
cat > "$SCRATCH/v3.sh" <<'EOF'
for m in a b; do
    if [[ $m == b ]]; then
        x=1
        break
    fi
done
EOF
check "v3 multiline" "$SCRATCH/v3.sh" clean

# while true; do break; done - inline, must stay clean
printf 'while true; do break; done\n' > "$SCRATCH/v4.sh"
check "v4 while-true inline break" "$SCRATCH/v4.sh" clean

# v5: bare break at top level - true positive, must still fire
printf 'break\n' > "$SCRATCH/v5.sh"
check "v5 bare top-level break" "$SCRATCH/v5.sh" fires

rm -f "$SCRATCH/v1.sh" "$SCRATCH/v2.sh" "$SCRATCH/v3.sh" "$SCRATCH/v4.sh" "$SCRATCH/v5.sh"

if [ "$fail" -eq 0 ]; then
    echo "ALL PMAT-244 PROBES PASSED"
    exit 0
else
    echo "PMAT-244 PROBE FAILURES"
    exit 1
fi
