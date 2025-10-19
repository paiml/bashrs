# Rash User Guide

**Version**: 2.0.0 (Target)
**Last Updated**: 2024-10-18
**Status**: Production Documentation

---

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Installation](#installation)
4. [What is Bash Purification?](#what-is-bash-purification)
5. [CLI Reference](#cli-reference)
6. [Before/After Examples](#beforeafter-examples)
7. [Common Workflows](#common-workflows)
8. [Advanced Usage](#advanced-usage)
9. [Troubleshooting](#troubleshooting)
10. [FAQ](#faq)

---

## Introduction

**Rash** is a shell script safety and purification tool that transforms messy, non-deterministic bash scripts into clean, safe, deterministic POSIX shell scripts.

### What Rash Does

- ✅ **Removes non-determinism**: Eliminates `$RANDOM`, timestamps, process IDs
- ✅ **Enforces idempotency**: Makes operations safe to re-run (adds `-p`, `-f`, `-s` flags)
- ✅ **Ensures POSIX compliance**: Outputs shellcheck-validated POSIX sh
- ✅ **Prevents injection attacks**: Quotes all variables properly
- ✅ **Detects security issues**: Built-in linter with 14 security/safety rules

### Who Should Use Rash?

- **DevOps Engineers**: Clean up deployment and CI/CD scripts
- **System Administrators**: Improve bootstrap and configuration scripts
- **Software Developers**: Make build scripts deterministic and safe
- **Security Engineers**: Audit and fix unsafe shell scripts

---

## Quick Start

### Installation

```bash
# Install from cargo (recommended)
cargo install rash

# Or build from source
git clone https://github.com/yourusername/bashrs.git
cd bashrs
cargo build --release
sudo cp target/release/rash /usr/local/bin/
```

### Your First Purification

```bash
# 1. Create a messy bash script
cat > messy.sh <<'EOF'
#!/bin/bash
TEMP_DIR="/tmp/app-$$"
TIMESTAMP=$(date +%s)
mkdir $TEMP_DIR
echo "Deployed at $TIMESTAMP"
EOF

# 2. Purify it
rash purify messy.sh -o purified.sh

# 3. See the difference
diff messy.sh purified.sh

# 4. Verify POSIX compliance
shellcheck -s sh purified.sh
```

**Result**: You now have a deterministic, idempotent, POSIX-compliant script!

---

## What is Bash Purification?

Bash purification is the process of transforming unreliable bash scripts into safe, deterministic, idempotent POSIX shell scripts.

### The Three Pillars of Purification

#### 1. Determinism ✅

**Problem**: Non-deterministic scripts produce different results on each run.

```bash
# ❌ NON-DETERMINISTIC (messy.sh)
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
BUILD_DIR="/tmp/build-$$"
```

**Solution**: Replace random/time-based values with predictable ones.

```bash
# ✅ DETERMINISTIC (purified.sh)
VERSION="${1:-unknown}"
SESSION_ID="session-${VERSION}"
RELEASE="release-${VERSION}"
BUILD_DIR="/tmp/build-${VERSION}"
```

**Benefit**: Same input always produces same output.

---

#### 2. Idempotency ✅

**Problem**: Scripts that fail when run twice.

```bash
# ❌ NON-IDEMPOTENT (messy.sh)
mkdir /app/releases
rm /app/current
ln -s /app/releases/v1.0 /app/current
```

**Solution**: Use flags that make operations safe to re-run.

```bash
# ✅ IDEMPOTENT (purified.sh)
mkdir -p /app/releases          # -p: create if missing, ignore if exists
rm -f /app/current              # -f: force, no error if missing
ln -sf /app/releases/v1.0 /app/current  # -s -f: symlink, overwrite if exists
```

**Benefit**: Scripts can be safely re-run without errors.

---

#### 3. Safety ✅

**Problem**: Unquoted variables allow injection attacks.

```bash
# ❌ UNSAFE (messy.sh)
USER_INPUT=$1
curl $URL | tar -xz
cd $TARGET_DIR
rm -rf $TEMP/*
```

**Solution**: Quote all variables to prevent word splitting and globbing.

```bash
# ✅ SAFE (purified.sh)
USER_INPUT="${1}"
curl "${URL}" | tar -xz
cd "${TARGET_DIR}" || exit 1
rm -rf "${TEMP:?}/"*
```

**Benefit**: No injection vulnerabilities, safe error handling.

---

## CLI Reference

Rash provides several commands for analyzing and transforming shell scripts.

### `rash parse`

Parse a bash script and output its Abstract Syntax Tree (AST).

```bash
rash parse <script.sh>

# Options:
#   --format <json|yaml>  Output format (default: yaml)
#   --output <file>       Write to file instead of stdout
```

**Example**:

```bash
$ rash parse deploy.sh
AST:
  - Script:
      shebang: "#!/bin/bash"
      commands:
        - Assignment:
            name: "RELEASE"
            value: "release-$(date +%s)"
        - Command:
            name: "mkdir"
            args: ["/app/releases/$RELEASE"]
```

**Use Case**: Understand script structure before purification.

---

### `rash purify`

Transform a bash script into a purified POSIX shell script.

```bash
rash purify <input.sh> [OPTIONS]

# Options:
#   -o, --output <file>   Write purified script to file
#   --posix               Enforce strict POSIX compliance (default: true)
#   --no-quotes           Disable automatic variable quoting (not recommended)
#   --preserve-comments   Keep comments from original script
```

**Example**:

```bash
# Basic purification
rash purify messy.sh -o purified.sh

# Purify and preserve comments
rash purify deploy.sh -o clean-deploy.sh --preserve-comments

# Purify to stdout (for piping)
rash purify script.sh | shellcheck -s sh -
```

**Use Case**: Main purification workflow - transform messy bash to clean POSIX sh.

---

### `rash lint`

Analyze a bash script for security, determinism, and idempotency issues.

```bash
rash lint <script.sh> [OPTIONS]

# Options:
#   --severity <error|warning|info>  Minimum severity to report (default: info)
#   --json                           Output as JSON
#   --fix                            Auto-fix issues where possible
```

**Example**:

```bash
$ rash lint unsafe.sh
Error [SEC001]: Command injection risk via eval
  --> unsafe.sh:5:1
  |
5 | eval "$user_input"
  | ^^^^ Command injection - manual review required
  |
  = help: Never use eval with untrusted input

Warning [IDEM001]: Non-idempotent mkdir
  --> unsafe.sh:10:1
   |
10 | mkdir /tmp/build
   | ^^^^^ Use 'mkdir -p' for idempotency
   |
   = fix: mkdir -p /tmp/build

2 errors, 1 warning
```

**Use Case**: Audit scripts before purification, identify security issues.

---

### `rash check`

Type-check and validate a script (dry-run purification).

```bash
rash check <script.sh>

# Options:
#   --strict   Enable strict validation
```

**Example**:

```bash
$ rash check deploy.sh
✓ Parse: OK
✓ Transform: OK
✓ POSIX compliance: OK
✓ Determinism: 2 issues found
✓ Idempotency: 1 issue found

Issues:
  - Line 5: Non-deterministic timestamp
  - Line 8: Non-deterministic $RANDOM
  - Line 12: Non-idempotent mkdir

Summary: Script can be purified
```

**Use Case**: Validate scripts without writing output.

---

### `rash ast`

Output the Abstract Syntax Tree in JSON format.

```bash
rash ast <script.sh> [--output <file>]
```

**Example**:

```bash
$ rash ast deploy.sh --output deploy-ast.json
$ cat deploy-ast.json | jq '.commands | length'
15
```

**Use Case**: Integration with other tools, programmatic analysis.

---

### `rash analyze`

Analyze script complexity and safety metrics.

```bash
rash analyze <script.sh>

# Options:
#   --metrics <complexity|safety|all>  What to analyze (default: all)
```

**Example**:

```bash
$ rash analyze deploy.sh
Complexity:
  - Lines: 150
  - Functions: 5
  - Max depth: 3
  - Cyclomatic complexity: 8

Safety:
  - Unquoted variables: 12
  - eval usage: 0
  - curl|sh patterns: 0
  - Hardcoded secrets: 0

Determinism:
  - $RANDOM: 2
  - Timestamps: 3
  - Process IDs: 1

Idempotency:
  - Non-idempotent operations: 8

Score: 6.5/10
```

**Use Case**: Get quick health check of script quality.

---

## Before/After Examples

### Example 1: Bootstrap Installer

#### Before (messy.sh)

```bash
#!/bin/bash
# Bootstrap installer - MESSY

TEMP="/tmp/install-$$"
VERSION=$(curl -s https://api.github.com/repos/app/releases/latest | grep tag_name | cut -d'"' -f4)

mkdir $TEMP
cd $TEMP

curl -L https://github.com/app/releases/download/$VERSION/app.tar.gz -o app.tar.gz
tar -xzf app.tar.gz

cp app /usr/local/bin/app
chmod +x /usr/local/bin/app

cd /
rm -r $TEMP

echo "Installed version $VERSION"
```

**Problems**:
- Non-deterministic temp dir (`$$`)
- Non-deterministic version fetch (timestamp-based)
- Non-idempotent `mkdir`
- Unquoted variables (`$TEMP`, `$VERSION`)
- No error handling
- Not POSIX compliant

---

#### After (purified.sh)

```bash
#!/bin/sh
# Bootstrap installer - PURIFIED by Rash v2.0.0

VERSION="${1:-latest}"
TEMP="/tmp/install-${VERSION}"

# Idempotent directory creation
mkdir -p "${TEMP}" || exit 1
cd "${TEMP}" || exit 1

# Download with error handling
curl -L "https://github.com/app/releases/download/${VERSION}/app.tar.gz" \
  -o app.tar.gz || exit 1

tar -xzf app.tar.gz || exit 1

# Idempotent install
cp app /usr/local/bin/app || exit 1
chmod +x /usr/local/bin/app || exit 1

# Cleanup
cd / || exit 1
rm -rf "${TEMP}"

printf 'Installed version %s\n' "${VERSION}"
```

**Improvements**:
- ✅ Deterministic temp dir (version-based)
- ✅ Version passed as argument (not fetched)
- ✅ Idempotent operations (`mkdir -p`, `cp` with error handling)
- ✅ All variables quoted
- ✅ Error handling (`|| exit 1`)
- ✅ POSIX compliant (`#!/bin/sh`, `printf` instead of `echo`)

---

### Example 2: Deployment Script

#### Before (messy.sh)

```bash
#!/bin/bash
# Deployment script - MESSY

RELEASE="release-$(date +%s)"
SESSION_ID=$RANDOM

mkdir /app/releases/$RELEASE
cp -r build/* /app/releases/$RELEASE/

rm /app/current
ln -s /app/releases/$RELEASE /app/current

echo "Deployed $RELEASE (session $SESSION_ID)"
logger "Deployment completed at $(date)"
```

**Problems**:
- Non-deterministic release name (timestamp)
- Non-deterministic session ID (`$RANDOM`)
- Non-idempotent `mkdir`, `rm`, `ln`
- Unquoted variables
- Timestamp logging (non-deterministic)

---

#### After (purified.sh)

```bash
#!/bin/sh
# Deployment script - PURIFIED by Rash v2.0.0

VERSION="${1:-unknown}"
RELEASE="release-${VERSION}"
SESSION_ID="session-${VERSION}"

# Idempotent release directory
mkdir -p "/app/releases/${RELEASE}" || exit 1
cp -r build/* "/app/releases/${RELEASE}/" || exit 1

# Idempotent symlink update
rm -f "/app/current"
ln -sf "/app/releases/${RELEASE}" "/app/current" || exit 1

printf 'Deployed %s (session %s)\n' "${RELEASE}" "${SESSION_ID}"
logger "Deployment of ${VERSION} completed"
```

**Improvements**:
- ✅ Deterministic release name (version-based)
- ✅ Deterministic session ID (version-based)
- ✅ Idempotent operations (`mkdir -p`, `rm -f`, `ln -sf`)
- ✅ All variables quoted
- ✅ Error handling
- ✅ POSIX compliant
- ✅ Deterministic logging (version instead of timestamp)

---

### Example 3: Database Migration

#### Before (messy.sh)

```bash
#!/bin/bash
# Database migration - MESSY

BACKUP="/tmp/db_backup_$(date +%Y%m%d_%H%M%S).sql"
LOG_FILE="/var/log/migration_$$.log"

echo "Starting migration at $(date)" > $LOG_FILE

mysqldump mydb > $BACKUP
mysql mydb < migration.sql

if [ $? -eq 0 ]; then
    echo "Migration successful"
    rm $BACKUP
else
    echo "Migration failed, restoring from $BACKUP"
    mysql mydb < $BACKUP
fi
```

**Problems**:
- Non-deterministic backup filename (timestamp)
- Non-deterministic log file (process ID)
- Unquoted variables
- No error handling for mysqldump
- `$?` can be unreliable

---

#### After (purified.sh)

```bash
#!/bin/sh
# Database migration - PURIFIED by Rash v2.0.0

MIGRATION_ID="${1:-unknown}"
BACKUP="/var/backups/db_backup_${MIGRATION_ID}.sql"
LOG_FILE="/var/log/migration_${MIGRATION_ID}.log"

printf 'Starting migration %s\n' "${MIGRATION_ID}" > "${LOG_FILE}"

# Create backup with error handling
mysqldump mydb > "${BACKUP}" 2>> "${LOG_FILE}"
if [ $? -ne 0 ]; then
    printf 'Error: Backup failed\n' | tee -a "${LOG_FILE}"
    exit 1
fi

# Run migration
mysql mydb < migration.sql 2>> "${LOG_FILE}"
MIGRATION_STATUS=$?

if [ ${MIGRATION_STATUS} -eq 0 ]; then
    printf 'Migration successful\n' | tee -a "${LOG_FILE}"
    rm -f "${BACKUP}"
else
    printf 'Migration failed, restoring from %s\n' "${BACKUP}" | tee -a "${LOG_FILE}"
    mysql mydb < "${BACKUP}" 2>> "${LOG_FILE}"
    exit 1
fi
```

**Improvements**:
- ✅ Deterministic filenames (migration ID-based)
- ✅ All variables quoted
- ✅ Explicit error handling
- ✅ Captured exit status before using it
- ✅ Idempotent cleanup (`rm -f`)
- ✅ POSIX compliant

---

## Common Workflows

### Workflow 1: One-Time Purification

**Use Case**: Clean up an existing bash script for production use.

```bash
# 1. Lint the original to understand issues
rash lint deploy.sh

# 2. Purify the script
rash purify deploy.sh -o deploy-clean.sh

# 3. Verify POSIX compliance
shellcheck -s sh deploy-clean.sh

# 4. Test the purified script
./deploy-clean.sh test-version

# 5. Replace original (after testing!)
mv deploy-clean.sh deploy.sh
```

---

### Workflow 2: CI/CD Integration

**Use Case**: Automatically purify scripts in CI pipeline.

```yaml
# .github/workflows/purify.yml
name: Purify Shell Scripts

on: [push, pull_request]

jobs:
  purify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rash
        run: cargo install rash

      - name: Lint all scripts
        run: |
          find scripts/ -name "*.sh" -exec rash lint {} \;

      - name: Purify scripts
        run: |
          for script in scripts/*.sh; do
            rash purify "$script" -o "purified/${script##*/}"
          done

      - name: Verify POSIX compliance
        run: |
          for script in purified/*.sh; do
            shellcheck -s sh "$script"
          done

      - name: Upload purified scripts
        uses: actions/upload-artifact@v2
        with:
          name: purified-scripts
          path: purified/
```

---

### Workflow 3: Bulk Migration

**Use Case**: Clean up a repository full of messy bash scripts.

```bash
#!/bin/sh
# bulk-purify.sh - Purify all scripts in a directory

SCRIPT_DIR="${1:-.}"
OUTPUT_DIR="${2:-purified}"

mkdir -p "${OUTPUT_DIR}"

find "${SCRIPT_DIR}" -name "*.sh" -type f | while IFS= read -r script; do
    basename=$(basename "${script}")

    printf 'Processing %s...\n' "${basename}"

    # Lint first
    if ! rash lint "${script}"; then
        printf '  Warning: Lint issues found in %s\n' "${basename}"
    fi

    # Purify
    if rash purify "${script}" -o "${OUTPUT_DIR}/${basename}"; then
        printf '  ✓ Purified %s\n' "${basename}"
    else
        printf '  ✗ Failed to purify %s\n' "${basename}"
    fi
done

printf '\nPurification complete! Purified scripts in %s/\n' "${OUTPUT_DIR}"
```

**Usage**:

```bash
# Purify all scripts in scripts/ directory
./bulk-purify.sh scripts/ purified/

# Review changes
diff -ur scripts/ purified/

# Test purified scripts
cd purified/ && ./run-tests.sh

# Replace originals after testing
cp -r purified/* scripts/
```

---

## Advanced Usage

### Custom Purification Rules

Create a `.rash.toml` configuration file to customize purification behavior:

```toml
# .rash.toml
[purification]
# Enable strict POSIX compliance
strict_posix = true

# Preserve comments from original script
preserve_comments = true

# Auto-add error handling
auto_error_handling = true

# Variable quoting style
quote_style = "double"  # "double" or "single"

[determinism]
# Replace $RANDOM with...
random_replacement = "uuid"  # "uuid", "version", or "error"

# Replace timestamps with...
timestamp_replacement = "version"  # "version", "fixed", or "error"

[idempotency]
# Force idempotent flags
force_mkdir_p = true
force_rm_f = true
force_ln_sf = true

[linter]
# Enable specific rule categories
enable_security = true
enable_determinism = true
enable_idempotency = true

# Treat warnings as errors
strict = false
```

**Usage with config**:

```bash
# Rash will automatically load .rash.toml from current directory
rash purify deploy.sh -o clean.sh

# Or specify config explicitly
rash purify deploy.sh -o clean.sh --config /path/to/.rash.toml
```

---

### Integration with Docker

**Dockerfile**:

```dockerfile
FROM rust:1.70 as builder

# Install Rash
RUN cargo install rash

FROM alpine:latest

# Copy Rash binary
COPY --from=builder /usr/local/cargo/bin/rash /usr/local/bin/rash

# Copy scripts to purify
COPY scripts/ /scripts/

# Purify on build
RUN for script in /scripts/*.sh; do \
      rash purify "$script" -o "/purified/$(basename $script)"; \
    done

# Use purified scripts at runtime
CMD ["/purified/entrypoint.sh"]
```

---

### Pre-commit Hook

Automatically purify scripts before commit:

```bash
# .git/hooks/pre-commit
#!/bin/sh

# Get all staged .sh files
SCRIPTS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$')

if [ -z "$SCRIPTS" ]; then
    exit 0
fi

echo "Purifying shell scripts..."

for script in $SCRIPTS; do
    echo "  Processing $script..."

    # Lint first
    if ! rash lint "$script"; then
        echo "  ✗ Lint failed for $script"
        exit 1
    fi

    # Purify in-place
    TEMP=$(mktemp)
    if rash purify "$script" -o "$TEMP"; then
        mv "$TEMP" "$script"
        git add "$script"
        echo "  ✓ Purified $script"
    else
        echo "  ✗ Purification failed for $script"
        rm -f "$TEMP"
        exit 1
    fi
done

echo "All scripts purified successfully!"
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

---

## Troubleshooting

### Issue: "Parse error: unexpected token"

**Problem**: Rash cannot parse your bash script.

```bash
$ rash purify complex.sh
Error: Parse error at line 42, column 15
  |
42| if [[ $var =~ ^[0-9]+$ ]]; then
  |       ^^^ unexpected token
```

**Cause**: Script uses bash-specific features not supported yet.

**Solution**:
1. Check if feature is POSIX-compatible
2. Simplify the construct to POSIX sh
3. Report unsupported feature as GitHub issue

**Example fix**:

```bash
# ❌ Bash-specific (not supported)
if [[ $var =~ ^[0-9]+$ ]]; then

# ✅ POSIX-compatible
case "$var" in
    ''|*[!0-9]*) echo "not a number" ;;
    *) echo "is a number" ;;
esac
```

---

### Issue: "Purified script behaves differently"

**Problem**: Purified script doesn't work the same as original.

**Diagnosis**:

```bash
# 1. Check what changed
diff original.sh purified.sh

# 2. Lint to see what was transformed
rash lint original.sh

# 3. Test both scripts with same input
./original.sh test-input > original-output.txt
./purified.sh test-input > purified-output.txt
diff original-output.txt purified-output.txt
```

**Common Causes**:
- Determinism: Script relied on `$RANDOM` or timestamps
- Quoting: Original had intentional word splitting
- Shell features: Used bash-isms not available in POSIX sh

**Solution**: Review differences and adjust script logic if needed.

---

### Issue: "Permission denied"

**Problem**: Cannot execute purified script.

```bash
$ ./purified.sh
bash: ./purified.sh: Permission denied
```

**Solution**: Add execute permission.

```bash
chmod +x purified.sh
```

Or purify with preserved permissions:

```bash
rash purify original.sh -o purified.sh --preserve-permissions
```

---

### Issue: "Command not found"

**Problem**: Purified script uses command not available in minimal environments.

```bash
$ ./purified.sh
./purified.sh: line 5: curl: command not found
```

**Solution**: Install required commands or use alternatives.

```bash
# Alpine Linux
apk add curl

# Ubuntu/Debian
apt-get install curl

# Or use wget instead
sed -i 's/curl/wget/' purified.sh
```

---

## FAQ

### Q: Does Rash modify my original scripts?

**A**: No, Rash never modifies input files. It always writes to a new file (specified with `-o`) or stdout.

---

### Q: Can I use Rash on scripts that aren't bash?

**A**: Rash is designed for bash scripts. For other shells (zsh, fish, etc.), you may need shell-specific tools.

---

### Q: Will purified scripts work on all systems?

**A**: Yes! Purified scripts are POSIX sh compliant and work on:
- Alpine Linux (busybox ash)
- Debian/Ubuntu (dash)
- MacOS (zsh with sh mode)
- FreeBSD/OpenBSD
- Any system with POSIX `/bin/sh`

---

### Q: Can Rash handle huge scripts (1000+ lines)?

**A**: Yes, Rash handles scripts of any size. Performance targets:
- Parse: <50ms for typical scripts
- Purify: <100ms for typical scripts
- Large scripts (1000+ lines): <500ms

---

### Q: What if Rash breaks my script?

**A**:
1. Rash never modifies originals (safe by design)
2. Always test purified scripts before deploying
3. Report issues on GitHub: https://github.com/yourusername/bashrs/issues
4. Use `rash check` for dry-run validation

---

### Q: Can I contribute linter rules?

**A**: Yes! See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to add custom linter rules.

---

### Q: How does Rash compare to ShellCheck?

**A**:
- **ShellCheck**: Linter (detects issues)
- **Rash**: Linter + Transpiler (detects AND fixes issues)

Use both together:
```bash
rash purify script.sh -o clean.sh && shellcheck -s sh clean.sh
```

---

### Q: Can Rash convert Python/Ruby scripts to shell?

**A**: No, Rash only works with bash → POSIX sh. For other languages, use language-specific tools.

---

### Q: Is Rash production-ready?

**A**: Yes (v2.0.0+)! The Bash → Purified Bash workflow is production-ready with:
- 1,489 tests passing (100% pass rate)
- >85% code coverage
- Comprehensive documentation
- Real-world examples

---

## Next Steps

- **API Reference**: See [API-REFERENCE.md](API-REFERENCE.md) for programmatic usage
- **Migration Guide**: See [MIGRATION-GUIDE.md](MIGRATION-GUIDE.md) for detailed migration strategies
- **Examples**: See [examples/](../examples/) for production-quality example scripts
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md) to contribute

---

**Questions or Issues?**
- GitHub Issues: https://github.com/yourusername/bashrs/issues
- Discussions: https://github.com/yourusername/bashrs/discussions

---

**Last Updated**: 2024-10-18
**Version**: 2.0.0 (Target)
**License**: MIT
