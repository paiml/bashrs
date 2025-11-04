#!/bin/bash
# SEC008 Baseline Completion Monitor
# Auto-triggers iteration tests when SEC008 completes
# Following EXTREME TDD - automated monitoring

LOG_FILE="mutation_sec008_baseline_v3.log"
CHECK_INTERVAL=30  # Check every 30 seconds

echo "=== SEC008 Baseline Monitor ==="
echo "Watching: $LOG_FILE"
echo "Check interval: ${CHECK_INTERVAL}s"
echo ""

while true; do
    if [ -f "$LOG_FILE" ]; then
        # Check for completion marker
        if grep -q "mutants tested in" "$LOG_FILE"; then
            # Extract results
            RESULT=$(grep "mutants tested in" "$LOG_FILE" | tail -1)
            echo ""
            echo "✅ SEC008 BASELINE COMPLETE!"
            echo ""
            echo "$RESULT"
            echo ""

            # Parse kill rate
            MISSED=$(echo "$RESULT" | grep -oP '\d+(?= missed)')
            CAUGHT=$(echo "$RESULT" | grep -oP '\d+(?= caught)')
            TOTAL=$((MISSED + CAUGHT))

            if [ $TOTAL -gt 0 ]; then
                KILL_RATE=$(awk "BEGIN {printf \"%.1f\", ($CAUGHT / $TOTAL) * 100}")
                echo "Kill Rate: $KILL_RATE% ($CAUGHT/$TOTAL caught, $MISSED MISSED)"
            fi

            echo ""
            echo "🚀 READY FOR ITERATION TESTS"
            echo ""
            echo "Next action: ./run_sec_iteration_tests.sh"
            echo ""

            exit 0
        else
            # Show progress
            LINES=$(wc -l < "$LOG_FILE")
            MISSED_COUNT=$(grep -c "MISSED" "$LOG_FILE" 2>/dev/null || echo "0")
            CAUGHT_COUNT=$(grep -c "ok      " "$LOG_FILE" 2>/dev/null || echo "0")

            echo -ne "\r⏳ Testing... Log lines: $LINES | MISSED: $MISSED_COUNT | Processing...    "
        fi
    else
        echo -ne "\r⏳ Waiting for log file to appear...    "
    fi

    sleep $CHECK_INTERVAL
done
