#!/bin/bash
# Proof-of-concept: Score with file type detection
# Demonstrates improvements from zshrc-refactor-spec-improvements.md

set -euo pipefail

FILE="${1:?Usage: $0 <file>}"

# Detect file type
detect_file_type() {
    local filename
    filename="$(basename "$1")"

    case "$filename" in
        *rc|*profile|.bash_profile|.bash_login)
            echo "Config"
            ;;
        *.sh)
            echo "Script"
            ;;
        *)
            echo "Library"
            ;;
    esac
}

FILE_TYPE="$(detect_file_type "$FILE")"

echo "==> Analyzing: $FILE"
echo "==> File Type: $FILE_TYPE"
echo ""

# Run standard scoring
echo "==> Running bashrs score..."
SCORE_OUTPUT=$(cargo run --quiet --bin bashrs -- score "$FILE" 2>&1 | grep -A 2 "Overall")

# Extract current score and grade
CURRENT_SCORE=$(echo "$SCORE_OUTPUT" | grep "Overall Score" | grep -oP '\d+\.\d+' | head -1)
CURRENT_GRADE=$(echo "$SCORE_OUTPUT" | grep "Overall Grade" | awk '{print $3}')

echo "$SCORE_OUTPUT"
echo ""

# Calculate adjusted score based on file type
calculate_adjusted_grade() {
    local score="$1"
    local file_type="$2"

    # Use bc for floating point comparison
    case "$file_type" in
        Config)
            # More lenient thresholds for config files
            if (( $(echo "$score >= 9.0" | bc -l) )); then
                echo "A+"
            elif (( $(echo "$score >= 8.5" | bc -l) )); then
                echo "A"
            elif (( $(echo "$score >= 8.0" | bc -l) )); then
                echo "A-"
            elif (( $(echo "$score >= 7.5" | bc -l) )); then
                echo "B+"
            elif (( $(echo "$score >= 7.0" | bc -l) )); then
                echo "B"
            else
                echo "$CURRENT_GRADE"
            fi
            ;;
        Script)
            # Strict thresholds (current behavior)
            echo "$CURRENT_GRADE"
            ;;
        Library)
            # Medium thresholds
            if (( $(echo "$score >= 9.3" | bc -l) )); then
                echo "A+"
            elif (( $(echo "$score >= 8.8" | bc -l) )); then
                echo "A"
            elif (( $(echo "$score >= 8.3" | bc -l) )); then
                echo "A-"
            else
                echo "$CURRENT_GRADE"
            fi
            ;;
    esac
}

ADJUSTED_GRADE=$(calculate_adjusted_grade "$CURRENT_SCORE" "$FILE_TYPE")

echo "==> File Type-Aware Scoring:"
echo "    Current (Script thresholds): $CURRENT_GRADE ($CURRENT_SCORE/10.0)"
echo "    Adjusted ($FILE_TYPE thresholds): $ADJUSTED_GRADE ($CURRENT_SCORE/10.0)"

if [ "$ADJUSTED_GRADE" != "$CURRENT_GRADE" ]; then
    echo ""
    echo "🎉 IMPROVEMENT: Grade improved from $CURRENT_GRADE to $ADJUSTED_GRADE!"
    echo "   This reflects appropriate standards for $FILE_TYPE files."
fi

echo ""
echo "==> Smart Suppression Analysis:"

# Count SC2154 warnings
SC2154_COUNT=$(cargo run --quiet --bin bashrs -- lint "$FILE" 2>&1 | grep -c "SC2154" || true)

echo "    Current SC2154 warnings: $SC2154_COUNT"

if [ "$SC2154_COUNT" -gt 50 ]; then
    ESTIMATED_AFTER=$((SC2154_COUNT / 10))
    echo "    Estimated after smart suppression: ~$ESTIMATED_AFTER (90% reduction)"
    echo "    Known external variables (NVM_DIR, BUN_INSTALL, etc.) would be suppressed"
fi

echo ""
echo "==> Summary of Improvements (from spec):"
echo "    ✅ File type detection: $FILE_TYPE"
echo "    ✅ Appropriate grade thresholds applied"
if [ "$ADJUSTED_GRADE" != "$CURRENT_GRADE" ]; then
    echo "    ✅ Grade improved: $CURRENT_GRADE → $ADJUSTED_GRADE"
fi
if [ "$SC2154_COUNT" -gt 50 ]; then
    echo "    🔜 Smart suppression would reduce SC2154 by ~90%"
fi

echo ""
echo "📄 Full specification: docs/specifications/zshrc-refactor-spec-improvements.md"
