#!/bin/bash
# SC2154 Fix Demonstration (Issue #24)
# This script demonstrates proper function parameter handling
# that previously triggered false positive SC2154 warnings.

set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly VERSION="1.0.0"

# Export declarations (no longer false positive)
export LOG_LEVEL="${LOG_LEVEL:-INFO}"
export OUTPUT_DIR="${OUTPUT_DIR:-/tmp/output}"

#
# Function with local parameters (Issue #24 fix)
#
validate_input() {
    local input_file="$1"
    local output_file="$2"

    echo "Validating: ${input_file} -> ${output_file}"

    if [[ -z "${input_file}" ]]; then
        echo "Error: Input file required" >&2
        return 1
    fi

    if [[ ! -f "${input_file}" ]]; then
        echo "Error: File '${input_file}' does not exist" >&2
        return 1
    fi

    echo "✅ Validation passed"
}

#
# Function with default parameters (Issue #24 fix)
#
process_data() {
    local input="${1:-}"
    local format="${2:-json}"
    local verbose="${3:-false}"

    echo "Processing: ${input}"
    echo "Format: ${format}"

    if [[ "${verbose}" == "true" ]]; then
        echo "Verbose mode enabled"
    fi
}

#
# Function with declare -i (integer) (Issue #24 fix)
#
calculate_size() {
    declare -i total="$1"
    declare -i count="$2"

    if [[ "${count}" -eq 0 ]]; then
        echo "0"
        return
    fi

    declare -i average=$((total / count))
    echo "${average}"
}

#
# Function with readonly in function scope (Issue #24 fix)
#
setup_constants() {
    local config_dir="$1"
    readonly MAX_RETRIES=3
    readonly TIMEOUT=30

    echo "Config directory: ${config_dir}"
    echo "Max retries: ${MAX_RETRIES}"
    echo "Timeout: ${TIMEOUT}s"
}

#
# Function with typeset (ksh/bash compatibility) (Issue #24 fix)
#
legacy_function() {
    typeset name="$1"
    typeset -i age="$2"

    echo "Name: ${name}, Age: ${age}"
}

#
# Main entry point
#
main() {
    local input_file="${1:-/etc/hosts}"
    local output_file="${2:-/tmp/output.txt}"

    echo "=== SC2154 Fix Demonstration (Issue #24) ==="
    echo "Script: ${SCRIPT_NAME}"
    echo "Version: ${VERSION}"
    echo ""

    echo "[1/5] Testing local parameter validation..."
    validate_input "${input_file}" "${output_file}"
    echo ""

    echo "[2/5] Testing parameters with defaults..."
    process_data "sample.txt" "xml" "true"
    echo ""

    echo "[3/5] Testing declare -i (integer variables)..."
    local avg
    avg=$(calculate_size 100 4)
    echo "Average: ${avg}"
    echo ""

    echo "[4/5] Testing readonly in function scope..."
    setup_constants "/etc/config"
    echo ""

    echo "[5/5] Testing typeset (legacy compatibility)..."
    legacy_function "Alice" 30
    echo ""

    echo "✅ All tests passed - No SC2154 false positives!"
}

# Run main with script arguments
main "$@"
