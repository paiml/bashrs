# SC2154 Fix Demonstration (Issue #24)

## Overview

This example demonstrates the fix for Issue #24 - SC2154 false positive warnings for function parameters.

## What Was Fixed

Before the fix, bashrs incorrectly flagged function parameters as "referenced but not assigned" when using proper bash declaration keywords. This produced 20+ false positive warnings on well-written scripts.

## Demonstration

The `sc2154-demo.sh` script showcases all patterns that are now correctly recognized:

### 1. Local Parameters
```bash
validate_input() {
    local input_file="$1"
    local output_file="$2"

    if [[ -z "${input_file}" ]]; then
        echo "Error" >&2
    fi
}
```

### 2. Parameters with Defaults
```bash
process_data() {
    local input="${1:-}"
    local format="${2:-json}"
    local verbose="${3:-false}"
}
```

### 3. Integer Variables (declare -i)
```bash
calculate_size() {
    declare -i total="$1"
    declare -i count="$2"
    declare -i average=$((total / count))
}
```

### 4. Readonly in Function Scope
```bash
setup_constants() {
    local config_dir="$1"
    readonly MAX_RETRIES=3
    readonly TIMEOUT=30
}
```

### 5. Export Declarations
```bash
export LOG_LEVEL="${LOG_LEVEL:-INFO}"
export OUTPUT_DIR="${OUTPUT_DIR:-/tmp/output}"
```

### 6. Typeset (Legacy Compatibility)
```bash
legacy_function() {
    typeset name="$1"
    typeset -i age="$2"
}
```

## Running the Example

### Lint the script (verify no SC2154 false positives)
```bash
bashrs lint examples/sc2154-demo.sh
```

**Expected output:**
```
Summary: 0 error(s), 4 warning(s), 8 info(s)
```

Note: The warnings are legitimate style suggestions (SC2016, SC2089, etc.), **NOT** SC2154 false positives.

### Execute the script
```bash
bash examples/sc2154-demo.sh
```

**Expected output:**
```
=== SC2154 Fix Demonstration (Issue #24) ===
Script: sc2154-demo.sh
Version: 1.0.0

[1/5] Testing local parameter validation...
✅ Validation passed

[2/5] Testing parameters with defaults...
Processing: sample.txt
Format: xml
Verbose mode enabled

[3/5] Testing declare -i (integer variables)...
Average: 25

[4/5] Testing readonly in function scope...
Config directory: /etc/config
Max retries: 3
Timeout: 30s

[5/5] Testing typeset (legacy compatibility)...
Name: Alice, Age: 30

✅ All tests passed - No SC2154 false positives!
```

## Before vs After

### Before Fix (bashrs v6.34.0)
```bash
$ bashrs lint examples/sc2154-demo.sh
⚠ SC2154: Variable 'input_file' is referenced but not assigned (x3)
⚠ SC2154: Variable 'output_file' is referenced but not assigned (x2)
⚠ SC2154: Variable 'input' is referenced but not assigned
⚠ SC2154: Variable 'format' is referenced but not assigned
⚠ SC2154: Variable 'verbose' is referenced but not assigned
... (20+ false positives)
```

### After Fix (bashrs v6.34.1+)
```bash
$ bashrs lint examples/sc2154-demo.sh
Summary: 0 error(s), 4 warning(s), 8 info(s)
✅ No SC2154 false positives!
```

## Testing

This example is covered by:
- 8 unit tests in `rash/src/linter/rules/sc2154.rs`
- 300+ property-based test cases
- Full integration test in CI/CD pipeline

## Related

- Issue: https://github.com/paiml/bashrs/issues/24
- Commit: 92859053f
- CHANGELOG: See CHANGELOG.md (Unreleased section)
