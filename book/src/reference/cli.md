# CLI Commands

This is the reference for all bashrs CLI commands.

## `bashrs bench` - Scientific Benchmarking

Benchmark shell scripts with scientific rigor, measuring execution time and optionally memory usage.

### Usage

```bash
bashrs bench [OPTIONS] <SCRIPT>...
```

### Arguments

- `<SCRIPT>...` - Shell script(s) to benchmark

### Options

- `-w, --warmup <N>` - Number of warmup iterations (default: 3)
- `-i, --iterations <N>` - Number of measured iterations (default: 10)
- `-o, --output <FILE>` - Output results to JSON file
- `-s, --strict` - Enable quality gates (lint + determinism checks)
- `--verify-determinism` - Verify script produces identical output
- `--show-raw` - Show raw iteration times
- `-q, --quiet` - Suppress progress output
- `-m, --measure-memory` - Measure memory usage (requires `/usr/bin/time`)

### Examples

Basic benchmark:
```bash
bashrs bench script.sh
```

With memory measurement:
```bash
bashrs bench script.sh --measure-memory
```

Custom iterations and warmup:
```bash
bashrs bench script.sh --iterations 20 --warmup 5
```

Compare multiple scripts:
```bash
bashrs bench fast.sh slow.sh --measure-memory
```

JSON output for automation:
```bash
bashrs bench script.sh --output results.json --quiet
```

With quality gates:
```bash
bashrs bench script.sh --strict --verify-determinism
```

### Output

The bench command provides:
- **Statistical metrics**: Mean, median, standard deviation, min, max
- **Memory statistics** (with `-m`): Mean, median, min, max, peak RSS in KB
- **Environment metadata**: CPU, RAM, OS, hostname
- **Console display**: Formatted output with results
- **JSON export**: Machine-readable format for automation

### Memory Measurement

When using `--measure-memory` / `-m`, bashrs measures the maximum resident set size (RSS) during script execution using `/usr/bin/time`. This provides accurate memory profiling:

```text
💾 Memory Usage
  Mean:    3456.00 KB
  Median:  3456.00 KB
  Min:     3456.00 KB
  Max:     3456.00 KB
  Peak:    3456.00 KB
```

**Requirements**:
- `/usr/bin/time` must be available (standard on Linux/Unix systems)
- Memory measurement adds negligible overhead (~1-2%)

### Quality Gates

Use `--strict` to run bashrs linter before benchmarking:
- Ensures scripts follow best practices
- Catches common errors before performance testing
- Fails benchmark if lint errors are found

Use `--verify-determinism` to check output consistency:
- Runs script multiple times
- Compares output across runs
- Fails if non-deterministic behavior detected (e.g., $RANDOM, timestamps)

## `bashrs build` - Transpile Rust to Shell Script

Transpiles Rust source code to deterministic POSIX shell scripts.

### Usage

```bash
bashrs build <INPUT> [OPTIONS]
```

### Arguments

- `<INPUT>` - Input Rust file

### Options

- `-o, --output <FILE>` - Output shell script file (default: `install.sh`)
- `--emit-proof` - Generate formal verification proof file
- `--no-optimize` - Disable code optimizations

### Examples

Basic transpilation:
```bash
bashrs build src/main.rs -o install.sh
```

With verification proof:
```bash
bashrs build src/install.rs -o install.sh --emit-proof
```

Without optimizations (for debugging):
```bash
bashrs build src/deploy.rs --no-optimize -o deploy.sh
```

### Output

The build command produces:
- **Shell script**: POSIX-compliant shell script at specified output path
- **Verification proof** (with `--emit-proof`): `.proof` file with formal verification evidence
- **Determinism**: Same Rust input always produces identical shell output
- **Safety**: No injection vectors in generated scripts

---

## `bashrs check` - Verify Rust Compatibility

Checks if Rust source is compatible with Rash transpiler (no unsupported features).

### Usage

```bash
bashrs check <INPUT>
```

### Arguments

- `<INPUT>` - Input Rust file to check

### Examples

Check compatibility:
```bash
bashrs check src/install.rs
```

Verify multiple files:
```bash
for f in src/*.rs; do bashrs check "$f"; done
```

### Output

- **Success**: "✓ Compatible with Rash transpiler"
- **Error**: List of incompatible features found with line numbers
- **Exit codes**: 0 for compatible, 1 for incompatible

---

## `bashrs init` - Initialize New Rash Project

Scaffolds a new Rash project with Cargo.toml and basic structure.

### Usage

```bash
bashrs init [PATH] [OPTIONS]
```

### Arguments

- `[PATH]` - Project directory (default: current directory `.`)

### Options

- `--name <NAME>` - Project name (defaults to directory name)

### Examples

Initialize in current directory:
```bash
bashrs init
```

Create new project:
```bash
bashrs init my-installer --name my-app
```

Initialize with custom name:
```bash
mkdir bootstrap && cd bootstrap
bashrs init --name deployment-tool
```

### Created Files

- `Cargo.toml` - Configured for Rash with proper dependencies
- `src/` - Source directory
- `src/main.rs` - Example Rust source file
- `.gitignore` - Standard Rust gitignore

---

## `bashrs verify` - Verify Shell Script Against Rust Source

Ensures generated shell script matches original Rust source behavior.

### Usage

```bash
bashrs verify <RUST_SOURCE> <SHELL_SCRIPT>
```

### Arguments

- `<RUST_SOURCE>` - Original Rust source file
- `<SHELL_SCRIPT>` - Generated shell script to verify

### Examples

Verify generated script:
```bash
bashrs build src/install.rs -o install.sh
bashrs verify src/install.rs install.sh
```

Verify with strict mode:
```bash
bashrs verify src/deploy.rs deploy.sh --strict
```

### Output

Verification report showing:
- **Behavioral equivalence**: Whether outputs match
- **Determinism check**: Whether script is deterministic
- **Safety validation**: Security issues detected
- **Discrepancies**: Any differences found with line numbers

---

## `bashrs inspect` - Generate Formal Verification Report

Generates detailed verification inspection report from AST.

### Usage

```bash
bashrs inspect <INPUT> [OPTIONS]
```

### Arguments

- `<INPUT>` - AST file (JSON) or inline AST specification

### Options

- `--format <FORMAT>` - Output format: `markdown`, `json`, `html` (default: `markdown`)
- `-o, --output <FILE>` - Output file (defaults to stdout)
- `--detailed` - Include detailed trace information

### Examples

Inspect AST:
```bash
bashrs build src/install.rs --emit-proof
bashrs inspect ast.json --format html -o report.html
```

Detailed markdown report:
```bash
bashrs inspect ast.json --detailed -o inspection.md
```

JSON output for automation:
```bash
bashrs inspect ast.json --format json -o report.json
```

### Output Sections

- **AST Analysis**: Abstract syntax tree structure
- **Verification Traces**: Detailed execution paths
- **Safety Checks**: Security validation results
- **Determinism Proof**: Mathematical proof of determinism
- **Transformation Log**: All applied transformations

---

## `bashrs compile` - Compile to Standalone Binary

Compiles Rust source to standalone executable or container image.

### Usage

```bash
bashrs compile <RUST_SOURCE> [OPTIONS]
```

### Arguments

- `<RUST_SOURCE>` - Input Rust source file

### Options

- `-o, --output <FILE>` - Output binary path (required)
- `--runtime <RUNTIME>` - Runtime: `dash`, `busybox`, `minimal` (default: `dash`)
- `--self-extracting` - Create self-extracting script instead of binary
- `--container` - Build distroless container
- `--container-format <FORMAT>` - Container format: `oci`, `docker` (default: `oci`)

### Examples

Compile to binary with dash runtime:
```bash
bashrs compile src/install.rs -o my-installer --runtime dash
```

Self-extracting script:
```bash
bashrs compile src/bootstrap.rs -o bootstrap.sh --self-extracting
```

OCI container image:
```bash
bashrs compile src/deploy.rs -o deploy-image --container --container-format oci
```

Minimal binary (smallest size):
```bash
bashrs compile src/tool.rs -o tool --runtime minimal
```

### Runtime Options

| Runtime | Size | Features | Use Case |
|---------|------|----------|----------|
| `dash` | ~180KB | Full POSIX | Production deployments |
| `busybox` | ~900KB | Extended utilities | Full-featured installers |
| `minimal` | ~50KB | Core only | Minimal footprint |

### Container Features

- **Distroless base**: Minimal attack surface
- **OCI/Docker compatible**: Works with all container runtimes
- **Single-file deployment**: No dependencies
- **Deterministic builds**: Same source = same binary

---

## `bashrs lint` - Lint Shell Scripts for Safety Issues

Analyzes shell scripts or Rust source for safety, determinism, and idempotency issues.

### Usage

```bash
bashrs lint <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Shell script or Rust source to lint

### Options

- `--format <FORMAT>` - Output format: `human`, `json`, `sarif` (default: `human`)
- `--fix` - Enable auto-fix suggestions (SAFE fixes only)
- `--fix-assumptions` - Apply fixes with assumptions (requires `--fix`)
- `-o, --output <FILE>` - Output file for fixed content

### Examples

Basic linting:
```bash
bashrs lint deploy.sh
```

JSON output for CI/CD:
```bash
bashrs lint script.sh --format json
```

Auto-fix safe issues:
```bash
bashrs lint deploy.sh --fix -o deploy-fixed.sh
```

Fix with assumptions (more aggressive):
```bash
bashrs lint src/install.rs --fix --fix-assumptions -o src/install-fixed.rs
```

SARIF output for GitHub Code Scanning:
```bash
bashrs lint script.sh --format sarif > results.sarif
```

### Detected Issues

**Security (SEC001-SEC008)**:
- Command injection via eval
- Insecure SSL/TLS
- Printf injection
- Unsafe symlinks
- And 4 more security rules

**Determinism (DET001-DET006)**:
- $RANDOM usage
- Timestamps (date, $(date))
- Process IDs ($$, $PPID)
- Hostnames
- UUIDs/GUIDs
- Network queries

**Idempotency (IDEM001-IDEM006)**:
- mkdir without -p
- rm without -f
- ln -s without cleanup
- Appending to files (>>)
- Creating files with >
- Database inserts without guards

### Fix Safety Levels

- **SAFE**: No assumptions needed (e.g., add -p to mkdir)
- **SAFE-WITH-ASSUMPTIONS**: Requires context (e.g., variable always set)
- **MANUAL**: Requires human review

---

## `bashrs purify` - Purify Bash Scripts

Transforms bash scripts into deterministic, idempotent, POSIX-compliant shell scripts.

### Usage

```bash
bashrs purify <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input bash script file

### Options

- `-o, --output <FILE>` - Output file (defaults to stdout)
- `--report` - Show detailed transformation report
- `--with-tests` - Generate test suite for purified script
- `--property-tests` - Generate property-based tests (100+ cases)

### Examples

Basic purification:
```bash
bashrs purify deploy.sh -o deploy-purified.sh
```

With detailed report:
```bash
bashrs purify messy.sh -o clean.sh --report
```

Generate test suite:
```bash
bashrs purify script.sh --with-tests --property-tests
```

Purify to stdout:
```bash
bashrs purify input.sh > output.sh
```

### Transformations Applied

**Determinism**:
- `$RANDOM` → version-based IDs
- `$(date +%s)` → fixed release tags
- `$$` (process ID) → deterministic IDs
- `$(hostname)` → configuration parameter

**Idempotency**:
- `mkdir` → `mkdir -p`
- `rm` → `rm -f`
- `ln -s` → `rm -f` + `ln -s`
- `>>` (append) → check + append guards
- `>` (create) → idempotent alternatives

**Safety**:
- Unquoted variables → quoted variables
- `eval` with user input → safer alternatives
- Insecure SSL → verified SSL

**POSIX Compliance**:
- Bash arrays → space-separated lists
- `[[ ]]` → `[ ]`
- Bash string manipulation → POSIX commands
- `local` keyword → naming conventions

### Verification

All purified scripts:
- ✅ Pass `shellcheck -s sh`
- ✅ Run identically in sh, dash, ash, bash
- ✅ Safe to re-run multiple times
- ✅ Produce deterministic output

---

## `bashrs make parse` - Parse Makefile to AST

Parses Makefile into abstract syntax tree.

### Usage

```bash
bashrs make parse <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input Makefile

### Options

- `--format <FORMAT>` - Output format: `text`, `json`, `debug` (default: `text`)

### Examples

Parse Makefile to text:
```bash
bashrs make parse Makefile
```

JSON AST for tooling:
```bash
bashrs make parse Makefile --format json > makefile-ast.json
```

Debug output:
```bash
bashrs make parse Makefile --format debug
```

### Output

**Text format**: Human-readable AST
**JSON format**: Machine-readable structured data
**Debug format**: Full internal representation

---

## `bashrs make purify` - Purify Makefile

Transforms Makefile into deterministic, idempotent form.

### Usage

```bash
bashrs make purify <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input Makefile

### Options

- `-o, --output <FILE>` - Output file (defaults to stdout or in-place with `--fix`)
- `--fix` - Apply fixes in-place (creates `.bak` backup)
- `--report` - Show detailed transformation report
- `--format <FORMAT>` - Report format: `human`, `json`, `markdown` (default: `human`)
- `--with-tests` - Generate test suite
- `--property-tests` - Generate property-based tests (100+ cases)

### Examples

Purify Makefile:
```bash
bashrs make purify Makefile -o Makefile.purified
```

Fix in-place with backup:
```bash
bashrs make purify Makefile --fix
```

With detailed report:
```bash
bashrs make purify Makefile --fix --report --with-tests
```

### Transformations

- Non-deterministic timestamps → fixed versions
- Non-idempotent operations → idempotent alternatives
- Unsafe recipes → safe equivalents
- `.PHONY` declarations validated

---

## `bashrs make lint` - Lint Makefile

Analyzes Makefile for safety and quality issues.

### Usage

```bash
bashrs make lint <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input Makefile

### Options

- `--format <FORMAT>` - Output format: `human`, `json`, `sarif` (default: `human`)
- `--fix` - Apply automatic fixes
- `-o, --output <FILE>` - Output file (defaults to in-place with `--fix`)
- `--rules <RULES>` - Filter by specific rules (comma-separated: `MAKE001,MAKE003`)

### Examples

Lint Makefile:
```bash
bashrs make lint Makefile
```

JSON output:
```bash
bashrs make lint Makefile --format json
```

Auto-fix issues:
```bash
bashrs make lint Makefile --fix
```

Filter specific rules:
```bash
bashrs make lint Makefile --rules MAKE001,MAKE002
```

### Detected Issues

- **MAKE001**: Missing `.PHONY` declarations
- **MAKE002**: Non-deterministic recipes
- **MAKE003**: Non-idempotent operations
- **MAKE004**: Unsafe shell commands
- **MAKE005**: Missing dependencies

---

## `bashrs config analyze` - Analyze Shell Configuration File

Analyzes shell configuration files (.bashrc, .zshrc, .profile, etc.) for issues.

### Usage

```bash
bashrs config analyze <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input config file

### Options

- `--format <FORMAT>` - Output format: `human`, `json` (default: `human`)

### Examples

Analyze .bashrc:
```bash
bashrs config analyze ~/.bashrc
```

JSON output:
```bash
bashrs config analyze ~/.zshrc --format json
```

Check .profile:
```bash
bashrs config analyze ~/.profile
```

### Analysis Results

**PATH Issues**:
- Duplicate entries
- Non-existent directories
- Problematic order

**Environment Issues**:
- Non-deterministic variables
- Conflicting definitions
- Missing quotes

**Security Issues**:
- Command injection risks
- Insecure SSL usage
- Unsafe eval

**Idempotency Issues**:
- Non-idempotent sourcing
- Append-only operations
- Missing guards

---

## `bashrs config lint` - Lint Shell Configuration File

Lints shell configuration files for safety issues.

### Usage

```bash
bashrs config lint <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input config file

### Options

- `--format <FORMAT>` - Output format: `human`, `json` (default: `human`)

### Examples

Lint .bashrc:
```bash
bashrs config lint ~/.bashrc
```

JSON output for automation:
```bash
bashrs config lint ~/.zshrc --format json
```

### Detected Issues

- CONFIG-001: Duplicate PATH entry
- CONFIG-002: Non-existent PATH entry
- CONFIG-003: Non-deterministic environment variable
- CONFIG-004: Conflicting environment variable
- Plus all SEC, DET, IDEM rules

---

## `bashrs config purify` - Purify Shell Configuration File

Purifies and fixes shell configuration files automatically.

### Usage

```bash
bashrs config purify <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input config file

### Options

- `-o, --output <FILE>` - Output file (defaults to stdout, or in-place with `--fix`)
- `--fix` - Apply fixes in-place (creates timestamped backup)
- `--no-backup` - Don't create backup (dangerous!)
- `--dry-run` - Show what would be changed without applying

### Examples

Dry run (preview changes):
```bash
bashrs config purify ~/.bashrc --dry-run
```

Purify to new file:
```bash
bashrs config purify ~/.bashrc -o ~/.bashrc-purified
```

Fix in-place with backup:
```bash
bashrs config purify ~/.bashrc --fix
```

Fix without backup (dangerous):
```bash
bashrs config purify ~/.bashrc --fix --no-backup
```

### Safety Features

- **Timestamped backups**: `~/.bashrc.backup.20251104_143022`
- **Dry-run mode**: Preview changes without applying
- **Idempotent**: Safe to run multiple times
- **Validation**: All changes verified before applying

---

## `bashrs repl` - Interactive REPL

Starts interactive REPL for bash script analysis and debugging.

### Usage

```bash
bashrs repl [OPTIONS]
```

### Options

- `--debug` - Enable debug mode
- `--sandboxed` - Enable sandboxed execution
- `--max-memory <MB>` - Maximum memory usage in MB (default: 100)
- `--timeout <SECS>` - Timeout in seconds (default: 30)
- `--max-depth <DEPTH>` - Maximum recursion depth (default: 100)

### Examples

Start REPL:
```bash
bashrs repl
```

Debug mode:
```bash
bashrs repl --debug
```

Sandboxed with limits:
```bash
bashrs repl --sandboxed --max-memory 50 --timeout 10
```

### REPL Features

**Interactive Commands**:
- Parse bash expressions and view AST
- Purify scripts and see transformations
- Lint for issues with real-time feedback
- Explain bash constructs
- Debug execution flow
- View variable state
- Command completion
- Syntax highlighting

**Example Session**:
```text
bashrs REPL v6.32.1
>>> x=5
>>> echo $x
5
>>> echo ${x:-default}
5
>>> for i in 1 2 3; do echo $i; done
1
2
3
>>> :help
Available commands:
  :parse <script>   - Parse and show AST
  :purify <script>  - Purify and show result
  :lint <script>    - Lint and show issues
  :quit             - Exit REPL
```

---

## `bashrs test` - Run Bash Script Tests

Runs test suite for bash scripts.

### Usage

```bash
bashrs test <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input bash script file

### Options

- `--format <FORMAT>` - Output format: `human`, `json`, `junit` (default: `human`)
- `--detailed` - Show detailed test results
- `--pattern <PATTERN>` - Run only tests matching pattern

### Examples

Run all tests:
```bash
bashrs test script.sh
```

Filter tests:
```bash
bashrs test script.sh --pattern "test_deploy*"
```

JUnit output for CI:
```bash
bashrs test script.sh --format junit > results.xml
```

---

## `bashrs score` - Score Bash Script Quality

Scores bash script quality across multiple dimensions.

### Usage

```bash
bashrs score <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input bash script file

### Options

- `--format <FORMAT>` - Output format: `human`, `json`, `markdown` (default: `human`)
- `--detailed` - Show detailed dimension scores

### Examples

Score script:
```bash
bashrs score deploy.sh
```

Detailed breakdown:
```bash
bashrs score script.sh --detailed
```

Markdown report:
```bash
bashrs score script.sh --format markdown > QUALITY.md
```

### Scoring Dimensions

- **Safety**: Security issues (0-100)
- **Determinism**: Non-deterministic patterns (0-100)
- **Idempotency**: Re-run safety (0-100)
- **POSIX Compliance**: Portability (0-100)
- **Code Quality**: Complexity, style (0-100)
- **Overall**: Weighted average

---

## `bashrs audit` - Comprehensive Quality Audit

Runs comprehensive quality audit with all checks.

### Usage

```bash
bashrs audit <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input bash script file

### Options

- `--format <FORMAT>` - Output format: `human`, `json`, `sarif` (default: `human`)
- `--strict` - Fail on warnings
- `--detailed` - Show detailed check results
- `--min-grade <GRADE>` - Minimum required grade (A+, A, B+, B, C+, C, D, F)

### Examples

Full audit:
```bash
bashrs audit script.sh --detailed
```

Strict mode with minimum grade:
```bash
bashrs audit deploy.sh --strict --min-grade A
```

SARIF for GitHub:
```bash
bashrs audit script.sh --format sarif > audit.sarif
```

### Audit Checks

- Linting (all rules)
- Security scanning
- Determinism verification
- Idempotency validation
- POSIX compliance
- Code complexity
- Best practices
- Documentation quality

---

## `bashrs coverage` - Generate Coverage Report

Generates code coverage report for bash scripts.

### Usage

```bash
bashrs coverage <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Input bash script file

### Options

- `--format <FORMAT>` - Output format: `terminal`, `json`, `html`, `lcov` (default: `terminal`)
- `--min <PERCENT>` - Minimum coverage percentage required
- `--detailed` - Show detailed coverage breakdown
- `-o, --output <FILE>` - Output file for HTML/LCOV format

### Examples

Terminal coverage:
```bash
bashrs coverage script.sh
```

HTML report:
```bash
bashrs coverage script.sh --format html -o coverage.html
```

With minimum threshold:
```bash
bashrs coverage script.sh --min 80 --detailed
```

LCOV for CI integration:
```bash
bashrs coverage script.sh --format lcov -o coverage.lcov
```

---

## `bashrs format` - Format Bash Scripts

Formats bash scripts according to style guidelines.

### Usage

```bash
bashrs format <FILE>... [OPTIONS]
```

### Arguments

- `<FILE>...` - Input bash script file(s) (one or more)

### Options

- `--check` - Check if files are formatted without applying changes
- `--dry-run` - Show diff without applying changes
- `-o, --output <FILE>` - Output file (for single input file)

### Examples

Format single file:
```bash
bashrs format script.sh -o script-formatted.sh
```

Format in-place:
```bash
bashrs format script.sh deploy.sh install.sh
```

Check formatting:
```bash
bashrs format script.sh --check
```

Preview changes:
```bash
bashrs format script.sh --dry-run
```

Format all scripts:
```bash
bashrs format *.sh
```

### Formatting Rules

- Consistent indentation (2 spaces)
- Proper quoting
- Aligned assignments
- Standard shebang
- Function formatting
- Comment style

---

## Global Options

All commands accept these global options:

### Verification Level
```bash
--verify <LEVEL>
```
- `none` - No verification
- `basic` - Basic checks
- `strict` - Strict validation (default)
- `paranoid` - Maximum validation

### Target Shell Dialect
```bash
--target <DIALECT>
```
- `posix` - POSIX sh (default)
- `bash` - GNU Bash
- `dash` - Debian Almquist Shell
- `ash` - Almquist Shell

### Validation Level
```bash
--validation <LEVEL>
```
- `none` - No validation
- `minimal` - Minimal checks (default)
- `strict` - Strict validation
- `paranoid` - Maximum validation

### Other Global Options
- `--strict` - Fail on warnings
- `-v, --verbose` - Enable verbose debug output

### Examples

Paranoid verification:
```bash
bashrs build src/install.rs --verify paranoid --target posix
```

Minimal validation:
```bash
bashrs lint script.sh --validation minimal
```

---

## Common Workflows

### Workflow 1: Transpile Rust to Shell
```bash
# Check compatibility
bashrs check src/install.rs

# Build shell script
bashrs build src/install.rs -o install.sh --emit-proof

# Verify correctness
bashrs verify src/install.rs install.sh
```

### Workflow 2: Purify Existing Bash Script
```bash
# Lint first
bashrs lint deploy.sh

# Purify with report
bashrs purify deploy.sh -o deploy-purified.sh --report --with-tests

# Verify it works
bash deploy-purified.sh
```

### Workflow 3: Complete Quality Audit
```bash
# Full audit
bashrs audit script.sh --detailed --strict

# Score quality
bashrs score script.sh --detailed

# Coverage report
bashrs coverage script.sh --format html -o coverage.html

# Format code
bashrs format script.sh
```

### Workflow 4: Config File Management
```bash
# Analyze issues
bashrs config analyze ~/.bashrc

# Lint for problems
bashrs config lint ~/.bashrc

# Dry run purification
bashrs config purify ~/.bashrc --dry-run

# Apply fixes with backup
bashrs config purify ~/.bashrc --fix
```

### Workflow 5: Interactive Development
```bash
# Start REPL
bashrs repl --debug

# Inside REPL:
# > x=5
# > echo $x
# > :lint echo $x
# > :purify echo $x
# > :quit
```

### Workflow 6: CI/CD Integration
```bash
# Lint in CI
bashrs lint script.sh --format json > lint-results.json

# Quality gates
bashrs audit script.sh --strict --min-grade B --format sarif > audit.sarif

# Coverage requirements
bashrs coverage script.sh --min 80 --format lcov -o coverage.lcov

# Benchmark performance
bashrs bench script.sh --verify-determinism -o bench.json
```

---

## Exit Codes

All commands follow standard exit code conventions:

- `0` - Success
- `1` - Error (linting issues, compilation failure, etc.)
- `2` - Invalid usage (missing arguments, invalid options)

---

## Environment Variables

### `RASH_DEBUG`
Enable debug logging:
```bash
RASH_DEBUG=1 bashrs build src/main.rs
```

### `RASH_NO_COLOR`
Disable colored output:
```bash
RASH_NO_COLOR=1 bashrs lint script.sh
```

### `RASH_STRICT`
Enable strict mode globally:
```bash
RASH_STRICT=1 bashrs audit script.sh
```

---

## See Also

- [Getting Started Guide](../getting-started/installation.md)
- [Purification Concepts](../concepts/purification.md)
- [Security Linting](../linting/security.md)
- [Configuration Management](../config/overview.md)
- [REPL Usage](../getting-started/repl.md)

