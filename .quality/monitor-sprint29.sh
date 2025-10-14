#!/bin/bash
# Sprint 29 Mutation Testing Monitor
# Usage: bash .quality/monitor-sprint29.sh

echo "═══════════════════════════════════════════════════════════"
echo "  Sprint 29 - Mutation Testing Progress Monitor"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Check if cargo-mutants is running
RUNNING=$(ps aux | grep -c "[c]argo-mutants")
echo "🔄 Active cargo-mutants processes: $RUNNING"
echo ""

# Check log files
echo "📊 Log File Status:"
echo "-----------------------------------------------------------"
for log in /tmp/mutants-ast-final.log /tmp/mutants-emitter-final.log /tmp/mutants-bash-parser-final.log; do
    if [ -f "$log" ]; then
        SIZE=$(ls -lh "$log" | awk '{print $5}')
        LINES=$(wc -l < "$log")
        BASENAME=$(basename "$log")
        echo "  $BASENAME: $SIZE ($LINES lines)"
    fi
done
echo ""

# Show recent progress from AST
echo "📈 AST Module Progress (last 10 lines):"
echo "-----------------------------------------------------------"
if [ -f /tmp/mutants-ast-final.log ]; then
    tail -10 /tmp/mutants-ast-final.log
else
    echo "  Log file not found yet..."
fi
echo ""

# Quick summary
echo "📋 Quick Status Check:"
echo "-----------------------------------------------------------"
for log in /tmp/mutants-*-final.log; do
    if [ -f "$log" ]; then
        MODULE=$(basename "$log" | sed 's/mutants-//;s/-final.log//')
        CAUGHT=$(grep -c "caught" "$log" 2>/dev/null || echo "0")
        MISSED=$(grep -c "MISSED" "$log" 2>/dev/null || echo "0")
        TOTAL=$(grep "Found.*mutants" "$log" | grep -oE '[0-9]+' | head -1)

        if [ -n "$TOTAL" ]; then
            echo "  $MODULE: $CAUGHT caught, $MISSED missed (of $TOTAL total)"
        fi
    fi
done
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "💡 Tip: Run this script again to see updated progress"
echo "   Estimated completion: 2-3 hours from start"
echo "═══════════════════════════════════════════════════════════"
