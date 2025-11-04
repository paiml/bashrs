#!/bin/bash
# SEC Batch Results Analysis Script
# Analyzes all baseline and iteration test results for SEC002-SEC008

set -e

echo "=== SEC Batch Mutation Testing Results Analysis ==="
echo ""

# Function to extract kill rate from log file
extract_results() {
    local log_file="$1"
    local rule_name="$2"
    
    if [ ! -f "$log_file" ]; then
        echo "  ❌ Log file not found: $log_file"
        return 1
    fi
    
    # Extract final line with results
    local result_line=$(grep "mutants tested in" "$log_file" 2>/dev/null | tail -1)
    
    if [ -z "$result_line" ]; then
        echo "  ⏳ $rule_name: Test in progress or failed"
        return 1
    fi
    
    # Parse: "X mutants tested in Xm Ys: Y missed, Z caught"
    local total=$(echo "$result_line" | grep -oP '\d+(?= mutants tested)')
    local missed=$(echo "$result_line" | grep -oP '\d+(?= missed)')
    local caught=$(echo "$result_line" | grep -oP '\d+(?= caught)')
    
    # Calculate kill rate
    local kill_rate=$(awk "BEGIN {printf \"%.1f\", ($caught / $total) * 100}")
    
    echo "  ✅ $rule_name: $kill_rate% ($caught/$total caught, $missed missed)"
    
    return 0
}

echo "📊 Baseline Results (RED Phase):"
echo "================================"
extract_results "mutation_sec002_baseline.log" "SEC002"
extract_results "mutation_sec003_baseline.log" "SEC003"
extract_results "mutation_sec004_baseline_v2.log" "SEC004"
extract_results "mutation_sec005_baseline.log" "SEC005"
extract_results "mutation_sec006_baseline.log" "SEC006"
extract_results "mutation_sec007_baseline.log" "SEC007"
extract_results "mutation_sec008_baseline.log" "SEC008"
echo ""

echo "🚀 Iteration Results (GREEN Phase):"
echo "==================================="
extract_results "mutation_sec002_iter1.log" "SEC002 Iteration 1"
extract_results "mutation_sec003_iter2.log" "SEC003 Iteration 2"
extract_results "mutation_sec004_iter1.log" "SEC004 Iteration 1"
extract_results "mutation_sec005_iter1.log" "SEC005 Iteration 1"
extract_results "mutation_sec006_iter1.log" "SEC006 Iteration 1"
extract_results "mutation_sec007_iter1.log" "SEC007 Iteration 1"
extract_results "mutation_sec008_iter1.log" "SEC008 Iteration 1"
echo ""

echo "📈 Summary Statistics:"
echo "======================"
echo "Target: 90%+ kill rate across all SEC rules"
echo "Methodology: EXTREME TDD with universal pattern recognition"
echo ""
echo "Run 'grep \"mutants tested\" mutation_sec*.log' for full details"
