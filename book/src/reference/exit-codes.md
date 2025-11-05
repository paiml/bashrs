# Exit Codes

This chapter provides a complete reference for bashrs v6.31.0 exit codes, their meanings, and how to handle them in scripts and CI/CD pipelines.

## Table of Contents

- [Exit Code Reference](#exit-code-reference)
- [Success Codes](#success-codes)
- [Error Codes](#error-codes)
- [Using Exit Codes in Scripts](#using-exit-codes-in-scripts)
- [CI/CD Integration](#cicd-integration)
- [Exit Code Ranges](#exit-code-ranges)
- [Best Practices](#best-practices)

## Exit Code Reference

bashrs follows standard Unix conventions for exit codes:

### Lint Command Exit Codes (Issue #6 - Updated)

**IMPORTANT**: The `bashrs lint` command uses a simplified exit code scheme aligned with industry standards (shellcheck, eslint, gcc):

| Exit Code | Meaning | When Returned | CI/CD Behavior |
|-----------|---------|---------------|----------------|
| **0** | **No errors** | No errors found (warnings/info are OK) | ✅ **PASS** - Pipeline continues |
| **1** | **Errors found** | Actual lint failures (ERROR severity) | ❌ **FAIL** - Pipeline blocked |
| **2** | **Tool failure** | File not found, invalid arguments, I/O errors | 🚫 **FAIL** - Tool malfunction |

**Why This Matters**:
- **Warnings don't block CI/CD**: Only actual errors (ERROR severity) cause exit 1
- **Tool failures are distinct**: Exit 2 indicates tool problems (not lint issues)
- **Industry standard**: Matches shellcheck, eslint, gcc behavior

**Examples**:

```bash
# Clean script - exits 0
$ bashrs lint clean.sh
✓ No errors found
$ echo $?
0

# Script with warnings only - exits 0 (warnings are non-blocking)
$ bashrs lint script-with-warnings.sh
warning[SC2086]: Quote to prevent globbing
  --> script.sh:3:5
1 warning(s), 0 error(s)
$ echo $?
0  # ✅ CI/CD passes

# Script with errors - exits 1
$ bashrs lint script-with-errors.sh
error[SC2188]: Redirection without command
  --> script.sh:5:1
0 warning(s), 1 error(s)
$ echo $?
1  # ❌ CI/CD fails

# File not found - exits 2
$ bashrs lint nonexistent.sh
error: No such file or directory
$ echo $?
2  # 🚫 Tool failure
```

**CI/CD Integration** (Recommended Pattern):

```bash
#!/bin/bash
# lint-check.sh - CI/CD linting script

bashrs lint scripts/*.sh
exit_code=$?

case $exit_code in
    0)
        echo "✅ All checks passed (warnings are OK)"
        exit 0
        ;;
    1)
        echo "❌ Lint errors found - fix before merging"
        exit 1
        ;;
    2)
        echo "🚫 Tool failure - check bashrs installation or file paths"
        exit 2
        ;;
    *)
        echo "Unexpected exit code: $exit_code"
        exit $exit_code
        ;;
esac
```

### General Exit Codes (All Commands)

For other bashrs commands (purify, parse, check, etc.), the following exit codes apply:

| Exit Code | Category | Meaning | Common Causes |
|-----------|----------|---------|---------------|
| 0 | Success | Operation completed successfully | All checks passed, no errors |
| 1 | General Error | Generic failure | Command execution failed, invalid arguments |
| 2 | Parse Error | Failed to parse input | Syntax errors in shell scripts |
| 3 | Validation Error | Validation checks failed | Linter rules violated, ShellCheck errors |
| 4 | Configuration Error | Invalid configuration | Bad bashrs.toml, invalid options |
| 5 | I/O Error | File system or I/O failure | Cannot read/write files, permission denied |
| 6 | Not Implemented | Feature not yet implemented | Unsupported operation |
| 7 | Dependency Error | Missing dependencies | External tools not found |
| 64 | Command Line Error | Invalid command line usage | Missing required arguments, bad flags |
| 65 | Input Data Error | Invalid input data | Malformed input files |
| 66 | Cannot Open Input | Cannot open input file | File not found, no read permission |
| 67 | User Does Not Exist | Invalid user reference | User lookup failed (rare) |
| 68 | Host Does Not Exist | Invalid host reference | Host lookup failed (rare) |
| 69 | Service Unavailable | Service temporarily unavailable | Network issues, rate limiting |
| 70 | Internal Software Error | Internal error in bashrs | Bug in bashrs (please report) |
| 71 | System Error | Operating system error | OS-level failure |
| 72 | Critical OS File Missing | Required OS file missing | Missing system files |
| 73 | Cannot Create Output | Cannot create output file | No write permission, disk full |
| 74 | I/O Error | Input/output error | Read/write failed |
| 75 | Temporary Failure | Temporary failure | Retry may succeed |
| 76 | Protocol Error | Protocol error | Network protocol issue |
| 77 | Permission Denied | Insufficient permissions | No access to required resources |
| 78 | Configuration Error | Configuration error | Invalid configuration file |

## Success Codes

### Exit Code 0: Success

**Meaning:** Operation completed successfully with no errors or warnings.

**When returned:**
- All linter checks passed
- Purification completed successfully
- Parsing succeeded
- Validation passed

**Examples:**

```bash
$ bashrs lint clean-script.sh
$ echo $?
0
```

```bash
$ bashrs purify script.sh -o purified.sh
Purified script written to purified.sh
$ echo $?
0
```

**CI/CD Usage:**
```bash
#!/bin/bash
if bashrs lint scripts/*.sh; then
    echo "All scripts passed linting"
    # Continue deployment
else
    echo "Linting failed"
    exit 1
fi
```

## Error Codes

### Exit Code 1: General Error

**Meaning:** Generic failure not covered by more specific error codes.

**Common Causes:**
- Command execution failed
- Unknown error occurred
- General validation failure

**Examples:**

```bash
$ bashrs nonexistent-command
error: 'nonexistent-command' is not a bashrs command
$ echo $?
1
```

```bash
$ bashrs purify /nonexistent/script.sh
error: Failed to read file: No such file or directory
$ echo $?
1
```

**Handling:**
```bash
#!/bin/bash
if ! bashrs purify script.sh; then
    echo "Purification failed with exit code $?"
    exit 1
fi
```

### Exit Code 2: Parse Error

**Meaning:** Failed to parse input file (syntax errors).

**Common Causes:**
- Bash syntax errors
- Unclosed quotes
- Mismatched braces
- Invalid command structure

**Examples:**

```bash
$ cat bad-script.sh
#!/bin/bash
if [ "$x" = "foo"  # Missing closing bracket
    echo "bar"
fi

$ bashrs lint bad-script.sh
error: Parse error at line 2: Unexpected end of file
$ echo $?
2
```

**Handling:**
```bash
#!/bin/bash
bashrs lint script.sh
exit_code=$?

case $exit_code in
    0) echo "Success" ;;
    2) echo "Parse error - fix syntax first" ;;
    *) echo "Other error: $exit_code" ;;
esac
```

### Exit Code 3: Validation Error

**Meaning:** Linter rules or validation checks failed.

**Common Causes:**
- Security violations (SEC001-SEC008)
- Determinism issues (DET001-DET003)
- Idempotency problems (IDEM001-IDEM003)
- ShellCheck rule violations

**Examples:**

```bash
$ cat insecure.sh
#!/bin/bash
eval "$USER_INPUT"  # SEC001 violation

$ bashrs lint insecure.sh
error[SEC001]: Command injection risk via eval
  --> insecure.sh:2:1
$ echo $?
3
```

**Handling:**
```bash
#!/bin/bash
bashrs lint scripts/*.sh
exit_code=$?

if [ $exit_code -eq 3 ]; then
    echo "Validation failed - review linter output"
    bashrs lint scripts/*.sh --format json > lint-report.json
    exit 1
fi
```

### Exit Code 4: Configuration Error

**Meaning:** Invalid bashrs configuration.

**Common Causes:**
- Malformed bashrs.toml
- Invalid configuration values
- Conflicting options

**Examples:**

```bash
$ cat bashrs.toml
[bashrs]
target = "invalid-shell"  # Invalid value

$ bashrs purify script.sh
error: Invalid configuration: Unknown target 'invalid-shell'
$ echo $?
4
```

**Handling:**
```bash
#!/bin/bash
if ! bashrs config validate bashrs.toml; then
    echo "Configuration error - fix bashrs.toml"
    exit 4
fi
```

### Exit Code 5: I/O Error

**Meaning:** File system or I/O operation failed.

**Common Causes:**
- Permission denied
- Disk full
- File system error
- Cannot read/write files

**Examples:**

```bash
$ bashrs purify readonly.sh -o /readonly/output.sh
error: Cannot write to /readonly/output.sh: Permission denied
$ echo $?
5
```

**Handling:**
```bash
#!/bin/bash
bashrs purify script.sh -o output.sh
exit_code=$?

if [ $exit_code -eq 5 ]; then
    echo "I/O error - check permissions and disk space"
    df -h .
    ls -l script.sh
    exit 5
fi
```

### Exit Code 64: Command Line Error

**Meaning:** Invalid command line usage.

**Common Causes:**
- Missing required arguments
- Invalid flags
- Conflicting options

**Examples:**

```bash
$ bashrs lint
error: Missing required argument: <FILE>
$ echo $?
64
```

```bash
$ bashrs purify --invalid-flag script.sh
error: Unknown flag: --invalid-flag
$ echo $?
64
```

**Handling:**
```bash
#!/bin/bash
bashrs purify script.sh -o output.sh || {
    exit_code=$?
    if [ $exit_code -eq 64 ]; then
        echo "Usage error - check command syntax"
        bashrs purify --help
    fi
    exit $exit_code
}
```

### Exit Code 70: Internal Software Error

**Meaning:** Internal error in bashrs (bug).

**Common Causes:**
- Unexpected condition in bashrs code
- Panic or assertion failure
- Unhandled edge case

**Examples:**

```bash
$ bashrs lint complex-script.sh
thread 'main' panicked at 'internal error'
$ echo $?
70
```

**Handling:**
```bash
#!/bin/bash
bashrs lint script.sh
exit_code=$?

if [ $exit_code -eq 70 ]; then
    echo "Internal error in bashrs - please report bug"
    echo "bashrs version: $(bashrs --version)"
    echo "Script: script.sh"
    # Create bug report with context
    exit 70
fi
```

### Exit Code 77: Permission Denied

**Meaning:** Insufficient permissions to perform operation.

**Common Causes:**
- No read permission on input file
- No write permission for output
- No execute permission for dependency

**Examples:**

```bash
$ chmod 000 secret.sh
$ bashrs lint secret.sh
error: Permission denied: secret.sh
$ echo $?
77
```

**Handling:**
```bash
#!/bin/bash
bashrs purify script.sh -o output.sh
exit_code=$?

case $exit_code in
    77)
        echo "Permission denied - check file permissions"
        ls -l script.sh
        exit 77
        ;;
esac
```

## Using Exit Codes in Scripts

### Basic Error Handling

```bash
#!/bin/bash

# Check if script passes linting
if bashrs lint deploy.sh; then
    echo "Linting passed"
else
    echo "Linting failed with exit code $?"
    exit 1
fi
```

### Detailed Error Handling

```bash
#!/bin/bash

bashrs purify script.sh -o purified.sh
exit_code=$?

case $exit_code in
    0)
        echo "Success: Script purified"
        ;;
    1)
        echo "General error occurred"
        exit 1
        ;;
    2)
        echo "Parse error: Fix syntax errors in script.sh"
        exit 2
        ;;
    3)
        echo "Validation error: Review linter warnings"
        bashrs lint script.sh
        exit 3
        ;;
    4)
        echo "Configuration error: Check bashrs.toml"
        bashrs config validate bashrs.toml
        exit 4
        ;;
    5)
        echo "I/O error: Check file permissions and disk space"
        exit 5
        ;;
    *)
        echo "Unexpected error: $exit_code"
        exit $exit_code
        ;;
esac
```

### Retry Logic for Temporary Failures

```bash
#!/bin/bash

max_retries=3
retry_count=0

while [ $retry_count -lt $max_retries ]; do
    bashrs lint script.sh
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "Success"
        exit 0
    elif [ $exit_code -eq 75 ]; then
        # Temporary failure - retry
        echo "Temporary failure, retrying..."
        retry_count=$((retry_count + 1))
        sleep 2
    else
        # Permanent failure
        echo "Permanent failure: $exit_code"
        exit $exit_code
    fi
done

echo "Max retries exceeded"
exit 75
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Shell Script Quality Check

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install bashrs
        run: cargo install bashrs

      - name: Lint shell scripts
        run: |
          bashrs lint scripts/*.sh
          exit_code=$?
          if [ $exit_code -eq 3 ]; then
            echo "::error::Linting failed - validation errors detected"
            exit 1
          elif [ $exit_code -ne 0 ]; then
            echo "::error::bashrs failed with exit code $exit_code"
            exit $exit_code
          fi

      - name: Purify scripts
        run: |
          for script in scripts/*.sh; do
            bashrs purify "$script" -o "purified/$(basename $script)"
            if [ $? -ne 0 ]; then
              echo "::error::Failed to purify $script"
              exit 1
            fi
          done

      - name: Upload purified scripts
        uses: actions/upload-artifact@v3
        with:
          name: purified-scripts
          path: purified/
```

### GitLab CI

```yaml
shell-quality:
  stage: test
  script:
    - cargo install bashrs
    - bashrs lint scripts/*.sh
    - |
      exit_code=$?
      if [ $exit_code -eq 3 ]; then
        echo "Validation errors detected"
        exit 1
      elif [ $exit_code -ne 0 ]; then
        echo "bashrs failed with exit code $exit_code"
        exit $exit_code
      fi
  artifacts:
    reports:
      codequality: lint-report.json
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    stages {
        stage('Install bashrs') {
            steps {
                sh 'cargo install bashrs'
            }
        }

        stage('Lint Scripts') {
            steps {
                script {
                    def exitCode = sh(
                        script: 'bashrs lint scripts/*.sh --format json > lint-report.json',
                        returnStatus: true
                    )

                    if (exitCode == 3) {
                        error("Validation errors detected")
                    } else if (exitCode != 0) {
                        error("bashrs failed with exit code ${exitCode}")
                    }
                }
            }
        }

        stage('Purify Scripts') {
            steps {
                sh '''
                    for script in scripts/*.sh; do
                        bashrs purify "$script" -o "purified/$(basename $script)" || exit $?
                    done
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'lint-report.json', allowEmptyArchive: true
            archiveArtifacts artifacts: 'purified/*', allowEmptyArchive: true
        }
    }
}
```

### CircleCI

```yaml
version: 2.1

jobs:
  quality-check:
    docker:
      - image: rust:latest
    steps:
      - checkout

      - run:
          name: Install bashrs
          command: cargo install bashrs

      - run:
          name: Lint scripts
          command: |
            bashrs lint scripts/*.sh --format json > lint-report.json
            exit_code=$?

            case $exit_code in
              0) echo "All checks passed" ;;
              3) echo "Validation errors" && exit 1 ;;
              *) echo "Error: $exit_code" && exit $exit_code ;;
            esac

      - store_artifacts:
          path: lint-report.json

workflows:
  version: 2
  quality:
    jobs:
      - quality-check
```

## Exit Code Ranges

bashrs exit codes follow standard Unix conventions:

### Standard Ranges

| Range | Category | Usage |
|-------|----------|-------|
| 0 | Success | Operation succeeded |
| 1-2 | Standard Errors | Generic and parse errors |
| 3-63 | bashrs Specific | Custom error codes |
| 64-78 | BSD sysexits.h | Standard Unix error codes |
| 126-127 | Shell Reserved | Command not executable, not found |
| 128-255 | Signal-based | Process terminated by signal |

### bashrs-Specific Range (3-63)

| Code | Meaning |
|------|---------|
| 3 | Validation error (linter rules) |
| 4 | Configuration error |
| 5 | I/O error |
| 6 | Not implemented |
| 7 | Dependency error |
| 8-63 | Reserved for future use |

### BSD sysexits.h Range (64-78)

bashrs uses standard BSD error codes for compatibility:

| Code | Constant | Meaning |
|------|----------|---------|
| 64 | EX_USAGE | Command line usage error |
| 65 | EX_DATAERR | Data format error |
| 66 | EX_NOINPUT | Cannot open input |
| 67 | EX_NOUSER | Addressee unknown |
| 68 | EX_NOHOST | Host name unknown |
| 69 | EX_UNAVAILABLE | Service unavailable |
| 70 | EX_SOFTWARE | Internal software error |
| 71 | EX_OSERR | System error |
| 72 | EX_OSFILE | Critical OS file missing |
| 73 | EX_CANTCREAT | Cannot create output |
| 74 | EX_IOERR | Input/output error |
| 75 | EX_TEMPFAIL | Temporary failure |
| 76 | EX_PROTOCOL | Protocol error |
| 77 | EX_NOPERM | Permission denied |
| 78 | EX_CONFIG | Configuration error |

## Best Practices

### 1. Always Check Exit Codes

```bash
#!/bin/bash

# BAD: Ignoring exit code
bashrs lint script.sh

# GOOD: Checking exit code
if ! bashrs lint script.sh; then
    echo "Linting failed"
    exit 1
fi
```

### 2. Use Specific Error Handling

```bash
#!/bin/bash

bashrs purify script.sh -o output.sh
exit_code=$?

# GOOD: Specific handling
case $exit_code in
    0) echo "Success" ;;
    2) echo "Parse error" && exit 2 ;;
    3) echo "Validation error" && exit 3 ;;
    *) echo "Other error: $exit_code" && exit $exit_code ;;
esac

# BAD: Generic handling
if [ $exit_code -ne 0 ]; then
    echo "Something failed"
    exit 1
fi
```

### 3. Preserve Exit Codes in Pipelines

```bash
#!/bin/bash

# GOOD: Preserve exit code
bashrs lint script.sh | tee lint.log
exit ${PIPESTATUS[0]}

# BAD: Loses exit code
bashrs lint script.sh | tee lint.log
# $? is exit code of 'tee', not 'bashrs lint'
```

### 4. Document Expected Exit Codes

```bash
#!/bin/bash
# This script lints shell scripts and returns:
#   0 - All checks passed
#   3 - Validation errors (non-blocking)
#   Other - Fatal errors (blocking)

bashrs lint scripts/*.sh
exit_code=$?

case $exit_code in
    0) echo "All checks passed" ;;
    3) echo "Validation warnings (non-blocking)" && exit 0 ;;
    *) echo "Fatal error: $exit_code" && exit $exit_code ;;
esac
```

### 5. Use `set -e` Carefully with Exit Codes

```bash
#!/bin/bash
set -e  # Exit on error

# This will exit immediately on non-zero
bashrs lint script.sh

# Use explicit checks when you need custom handling
set +e
bashrs purify script.sh -o output.sh
exit_code=$?
set -e

if [ $exit_code -ne 0 ]; then
    echo "Purification failed: $exit_code"
    exit $exit_code
fi
```

### 6. Test Exit Codes in CI/CD

```bash
#!/bin/bash

# Test that linting actually catches errors
echo 'eval "$bad"' > test.sh
if bashrs lint test.sh; then
    echo "ERROR: Linting should have failed"
    exit 1
fi

# Test that clean scripts pass
echo '#!/bin/sh' > clean.sh
echo 'echo "hello"' >> clean.sh
if ! bashrs lint clean.sh; then
    echo "ERROR: Clean script should pass"
    exit 1
fi

echo "Exit code tests passed"
```

### 7. Log Exit Codes for Debugging

```bash
#!/bin/bash

log_exit_code() {
    local exit_code=$1
    local command=$2
    echo "[$(date)] $command exited with code $exit_code" >> bashrs.log
}

bashrs lint script.sh
exit_code=$?
log_exit_code $exit_code "bashrs lint script.sh"

if [ $exit_code -ne 0 ]; then
    echo "Check bashrs.log for details"
    exit $exit_code
fi
```

### 8. Use Trap for Cleanup on Exit

```bash
#!/bin/bash

cleanup() {
    local exit_code=$?
    echo "Cleaning up (exit code: $exit_code)"
    rm -f /tmp/bashrs-$$-*
    exit $exit_code
}

trap cleanup EXIT

bashrs purify script.sh -o /tmp/bashrs-$$-output.sh
# cleanup will run automatically with correct exit code
```

## Summary

bashrs exit codes follow Unix conventions:

- **0**: Success
- **1-2**: Standard errors (general, parse)
- **3-7**: bashrs-specific errors (validation, config, I/O, etc.)
- **64-78**: BSD sysexits.h standard codes

**Key Points:**
1. Always check exit codes in scripts and CI/CD
2. Use specific error handling for different exit codes
3. Preserve exit codes through pipelines
4. Document expected exit codes in your scripts
5. Test that your error handling works correctly

For more information, see:
- [Configuration Reference](./configuration.md)
- [Linter Rules Reference](./rules.md)
- [CLI Commands Reference](./cli.md)
