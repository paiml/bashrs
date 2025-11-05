# Analyzing Config Files

Shell configuration files like `.bashrc`, `.bash_profile`, `.zshrc`, and `.profile` are critical to your development environment, but they often accumulate issues over time:

- Duplicate PATH entries slowing down command lookup
- Unquoted variables creating security vulnerabilities
- Non-idempotent operations causing inconsistent behavior
- Non-deterministic constructs producing unpredictable results
- Performance bottlenecks from expensive operations

The `bashrs config analyze` command provides comprehensive analysis of your shell configuration files, detecting these issues and providing actionable recommendations.

## Quick Start

Analyze your shell configuration in seconds:

```bash
bashrs config analyze ~/.bashrc
```

This command:
1. Detects your configuration file type automatically
2. Analyzes for common issues (duplicate paths, unquoted variables, etc.)
3. Calculates complexity score
4. Reports performance bottlenecks
5. Provides specific suggestions for improvement

## What Config Analysis Detects

`bashrs config analyze` performs four core analyses, each corresponding to a specific rule:

### CONFIG-001: Duplicate PATH Entries

Detects when the same directory appears multiple times in PATH modifications:

```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # ⚠️ Duplicate detected!
```

**Why this matters**:
- Slower command lookup (shell searches each PATH entry in order)
- Confusion about which binary will execute
- Maintenance burden tracking which paths are active

**Detection**: Tracks all PATH modifications and identifies directories added more than once.

See [CONFIG-001: Deduplicate PATH Entries](./rules/config-001.md) for complete details.

### CONFIG-002: Unquoted Variable Expansions

Detects variables used without quotes, which can cause word splitting, glob expansion, and injection vulnerabilities:

```bash
export PROJECT_DIR=$HOME/my projects    # ⚠️ Unquoted - will break!
cd $PROJECT_DIR                         # ⚠️ Splits into: cd /home/user/my projects
cp $SOURCE $DEST                        # ⚠️ Vulnerable to injection
```

**Why this matters**:
- **Word splitting**: Spaces in values break arguments
- **Glob expansion**: Wildcards expand unexpectedly (`*.txt` → `file1.txt file2.txt`)
- **Security vulnerabilities**: Command injection through unquoted paths

**Detection**: Analyzes all variable expansions and identifies those without quotes.

See [CONFIG-002: Quote Variable Expansions](./rules/config-002.md) for complete details.

### CONFIG-003: Duplicate Alias Definitions

Detects when the same alias is defined multiple times (only the last definition is active):

```bash
alias ll='ls -la'
# ... 50 lines later ...
alias ll='ls -lah'        # ⚠️ Duplicate - this one wins
# ... 30 lines later ...
alias ll='ls -lAh'        # ⚠️ Duplicate - this one actually wins
```

**Why this matters**:
- **Confusing behavior**: Only the last definition takes effect
- **Maintenance burden**: Hard to track which aliases are active
- **Cluttered configs**: Unnecessary duplication

**Detection**: Tracks all alias definitions and identifies names appearing more than once.

See [CONFIG-003: Consolidate Duplicate Aliases](./rules/config-003.md) for complete details.

### CONFIG-004: Non-Deterministic Constructs

Detects constructs that produce different results on each execution:

```bash
SESSION_ID=$RANDOM                     # ⚠️ Random number
TIMESTAMP=$(date +%s)                  # ⚠️ Current timestamp
LOG_FILE="/tmp/log.$$"                 # ⚠️ Process ID
```

**Why this matters**:
- **Unpredictable behavior**: Different results across shell sessions
- **Testing difficulties**: Hard to write reproducible tests
- **Debugging challenges**: Behavior changes between runs

**Detection**: Identifies `$RANDOM`, timestamps (`date +%s`), process IDs (`$$`), and other non-deterministic patterns.

**Note**: Some timestamp usage is legitimate (e.g., measuring command execution time in `.zshrc`). Context matters.

### CONFIG-005: Performance Issues (Preview)

Detects operations that slow down shell startup:

```bash
eval "$(rbenv init -)"                 # ⚠️ Expensive - adds ~150ms
eval "$(pyenv init -)"                 # ⚠️ Expensive - adds ~200ms
eval "$(nodenv init -)"                # ⚠️ Expensive - adds ~100ms
```

**Why this matters**:
- **Slow shell startup**: Each eval adds 100-200ms
- **Compounding delays**: Multiple evals create noticeable lag
- **Unnecessary overhead**: Many tools can be lazy-loaded

**Suggestion**: Use lazy-loading patterns to defer expensive operations until needed.

## Supported Configuration Files

`bashrs config analyze` automatically detects and analyzes these configuration file types:

| File | Type | Shell | Purpose |
|------|------|-------|---------|
| `.bashrc` | Bashrc | bash | Interactive shell (non-login) |
| `.bash_profile` | BashProfile | bash | Login shell |
| `.profile` | Profile | sh | POSIX login shell (portable) |
| `.zshrc` | Zshrc | zsh | Interactive shell (non-login) |
| `.zprofile` | Zprofile | zsh | Login shell |
| `.zshenv` | Generic | zsh | All zsh sessions |

The tool understands shell-specific conventions and adjusts analysis accordingly.

## Command Usage

### Basic Analysis

```bash
bashrs config analyze <file>
```

Example:

```bash
bashrs config analyze ~/.bashrc
```

**Output**:

```text
Configuration Analysis: /home/user/.bashrc
===========================================

File Type: Bashrc (bash)
Lines: 157
Complexity: 5/10

Issues Found: 3

[CONFIG-001] Duplicate PATH entry
  → Line: 23
  → Path: /usr/local/bin
  → First occurrence: Line 15
  → Suggestion: Remove duplicate entry or use conditional addition

[CONFIG-002] Unquoted variable expansion
  → Line: 45
  → Variable: $HOME
  → Column: 18
  → Can cause word splitting and glob expansion
  → Suggestion: Quote the variable: "${HOME}"

[CONFIG-003] Duplicate alias definition: 'ls'
  → Line: 89
  → First occurrence: Line 67
  → Severity: Warning
  → Suggestion: Remove earlier definition or rename alias

PATH Entries:
  Line 15: /usr/local/bin
  Line 19: /opt/homebrew/bin
  Line 23: /usr/local/bin (DUPLICATE)
  Line 31: /home/user/.local/bin

Performance Issues: 1
  Line 52: eval "$(rbenv init -)" [~150ms]
  → Suggestion: Consider lazy-loading this version manager
```

### JSON Output

For integration with tools and CI/CD pipelines:

```bash
bashrs config analyze ~/.bashrc --format json
```

**Output**:

```json
{
  "file_path": "/home/user/.bashrc",
  "config_type": "Bashrc",
  "line_count": 157,
  "complexity_score": 5,
  "issues": [
    {
      "rule_id": "CONFIG-001",
      "severity": "Warning",
      "message": "Duplicate PATH entry",
      "line": 23,
      "column": 0,
      "suggestion": "Remove duplicate entry or use conditional addition"
    },
    {
      "rule_id": "CONFIG-002",
      "severity": "Warning",
      "message": "Unquoted variable expansion: $HOME",
      "line": 45,
      "column": 18,
      "suggestion": "Quote the variable: \"${HOME}\""
    },
    {
      "rule_id": "CONFIG-003",
      "severity": "Warning",
      "message": "Duplicate alias definition: 'ls'",
      "line": 89,
      "column": 0,
      "suggestion": "Remove earlier definition or rename alias"
    }
  ],
  "path_entries": [
    {"line": 15, "path": "/usr/local/bin", "is_duplicate": false},
    {"line": 19, "path": "/opt/homebrew/bin", "is_duplicate": false},
    {"line": 23, "path": "/usr/local/bin", "is_duplicate": true},
    {"line": 31, "path": "/home/user/.local/bin", "is_duplicate": false}
  ],
  "performance_issues": [
    {
      "line": 52,
      "command": "eval \"$(rbenv init -)\"",
      "estimated_cost_ms": 150,
      "suggestion": "Consider lazy-loading this version manager"
    }
  ]
}
```

### SARIF Output (Planned)

For integration with GitHub Code Scanning and other security tools:

```bash
bashrs config analyze ~/.bashrc --format sarif > results.sarif
```

SARIF (Static Analysis Results Interchange Format) is an industry-standard format supported by GitHub, GitLab, and many CI/CD platforms.

## Real .bashrc Analysis Examples

### Example 1: Duplicate PATH Entries

**Input** (`messy.bashrc`):

```bash
# System paths
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/bin:$PATH"

# Homebrew
if [ -d "/opt/homebrew/bin" ]; then
    export PATH="/opt/homebrew/bin:$PATH"
fi

# Accidentally added again
export PATH="/usr/local/bin:$PATH"

# Python tools
export PATH="$HOME/.local/bin:$PATH"
```

**Analysis**:

```bash
bashrs config analyze messy.bashrc
```

**Output**:

```text
Configuration Analysis: messy.bashrc
====================================

File Type: Bashrc (bash)
Lines: 15
Complexity: 3/10

Issues Found: 1

[CONFIG-001] Duplicate PATH entry
  → Line: 12
  → Path: /usr/local/bin
  → First occurrence: Line 2
  → Suggestion: Remove duplicate entry or use conditional addition

PATH Entries:
  Line 2: /usr/local/bin
  Line 3: /usr/bin
  Line 7: /opt/homebrew/bin (conditional)
  Line 12: /usr/local/bin (DUPLICATE)
  Line 15: /home/user/.local/bin

Recommendation: Run `bashrs config purify messy.bashrc` to fix automatically
```

### Example 2: Unquoted Variables

**Input** (`unsafe.bashrc`):

```bash
# Project directory with space in name
export PROJECT_DIR=$HOME/my projects

# Backup directory
export BACKUP_DIR=$HOME/backups

# Aliases using unquoted variables
alias proj='cd $PROJECT_DIR'
alias backup='cp $PROJECT_DIR/file.txt $BACKUP_DIR/'

# Function with unquoted variables
deploy() {
    cd $PROJECT_DIR
    ./build.sh
    cp result.tar.gz $BACKUP_DIR
}
```

**Analysis**:

```bash
bashrs config analyze unsafe.bashrc
```

**Output**:

```text
Configuration Analysis: unsafe.bashrc
=====================================

File Type: Bashrc (bash)
Lines: 16
Complexity: 4/10

Issues Found: 8

[CONFIG-002] Unquoted variable expansion
  → Line: 2
  → Variable: $HOME
  → Column: 18
  → Can cause word splitting and glob expansion
  → Suggestion: Quote the variable: "${HOME}"

[CONFIG-002] Unquoted variable expansion
  → Line: 5
  → Variable: $HOME
  → Column: 18
  → Suggestion: Quote the variable: "${HOME}"

[CONFIG-002] Unquoted variable expansion
  → Line: 8
  → Variable: $PROJECT_DIR
  → Column: 16
  → Suggestion: Quote the variable: "${PROJECT_DIR}"

[CONFIG-002] Unquoted variable expansion
  → Line: 9
  → Variable: $PROJECT_DIR
  → Column: 19
  → Suggestion: Quote the variable: "${PROJECT_DIR}"

[CONFIG-002] Unquoted variable expansion
  → Line: 9
  → Variable: $BACKUP_DIR
  → Column: 47
  → Suggestion: Quote the variable: "${BACKUP_DIR}"

[CONFIG-002] Unquoted variable expansion
  → Line: 13
  → Variable: $PROJECT_DIR
  → Column: 8
  → Suggestion: Quote the variable: "${PROJECT_DIR}"

[CONFIG-002] Unquoted variable expansion
  → Line: 15
  → Variable: $BACKUP_DIR
  → Column: 24
  → Suggestion: Quote the variable: "${BACKUP_DIR}"

Security Risk: HIGH
Unquoted variables can cause:
- Word splitting (spaces break arguments)
- Glob expansion (wildcards expand unexpectedly)
- Command injection vulnerabilities

Recommendation: Run `bashrs config purify unsafe.bashrc` to fix automatically
```

### Example 3: Duplicate Aliases

**Input** (`aliases.bashrc`):

```bash
# Initial aliases (2019)
alias ls='ls --color=auto'
alias ll='ls -la'
alias grep='grep --color=auto'

# Experimentation (2020)
alias ll='ls -lah'

# macOS migration (2021)
alias ls='ls -G'

# Current preferences (2024)
alias ll='ls -lAh'
alias grep='grep -i --color=auto'
```

**Analysis**:

```bash
bashrs config analyze aliases.bashrc
```

**Output**:

```text
Configuration Analysis: aliases.bashrc
======================================

File Type: Bashrc (bash)
Lines: 15
Complexity: 3/10

Issues Found: 4

[CONFIG-003] Duplicate alias definition: 'll'
  → Line: 8
  → First occurrence: Line 3
  → Severity: Warning
  → Suggestion: Remove earlier definition. Last definition wins.

[CONFIG-003] Duplicate alias definition: 'ls'
  → Line: 11
  → First occurrence: Line 2
  → Severity: Warning
  → Suggestion: Remove earlier definition. Last definition wins.

[CONFIG-003] Duplicate alias definition: 'll'
  → Line: 14
  → First occurrence: Line 3
  → Severity: Warning
  → Suggestion: Remove earlier definition. Last definition wins.

[CONFIG-003] Duplicate alias definition: 'grep'
  → Line: 15
  → First occurrence: Line 4
  → Severity: Warning
  → Suggestion: Remove earlier definition. Last definition wins.

Active Aliases (last definition wins):
  ls='ls -G' (line 11)
  ll='ls -lAh' (line 14)
  grep='grep -i --color=auto' (line 15)

Recommendation: Run `bashrs config purify aliases.bashrc` to consolidate
```

### Example 4: Non-Deterministic Content

**Input** (`random.bashrc`):

```bash
# Session ID using random number
export SESSION_ID=$RANDOM

# Timestamped log file
export LOG_FILE="/tmp/bash-$(date +%s).log"

# Process-specific temp directory
export TEMP_DIR="/tmp/bash-$$"

# Cache with timestamp
export CACHE_KEY="cache-$(date +%Y%m%d%H%M%S)"
```

**Analysis**:

```bash
bashrs config analyze random.bashrc
```

**Output**:

```text
Configuration Analysis: random.bashrc
=====================================

File Type: Bashrc (bash)
Lines: 11
Complexity: 3/10

Issues Found: 4

[CONFIG-004] Non-deterministic construct: $RANDOM
  → Line: 2
  → Variable: $RANDOM generates different values on each execution
  → Suggestion: Use a deterministic value or parameter

[CONFIG-004] Non-deterministic construct: $(date +%s)
  → Line: 5
  → Timestamp generates different values on each execution
  → Suggestion: Accept timestamp as parameter for determinism

[CONFIG-004] Non-deterministic construct: $$
  → Line: 8
  → Process ID differs on each shell invocation
  → Suggestion: Use mktemp or accept directory as parameter

[CONFIG-004] Non-deterministic construct: $(date +%Y%m%d%H%M%S)
  → Line: 11
  → Timestamp generates different values on each execution
  → Suggestion: Accept timestamp as parameter for determinism

Determinism: POOR
Non-deterministic constructs make behavior unpredictable and testing difficult.

Recommendation: Run `bashrs config purify random.bashrc` to remove non-determinism
```

### Example 5: Performance Issues

**Input** (`slow-startup.bashrc`):

```bash
# Version managers
eval "$(rbenv init -)"
eval "$(pyenv init -)"
eval "$(nodenv init -)"

# NVM
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Completions
eval "$(gh completion -s bash)"
eval "$(kubectl completion bash)"
```

**Analysis**:

```bash
bashrs config analyze slow-startup.bashrc
```

**Output**:

```text
Configuration Analysis: slow-startup.bashrc
===========================================

File Type: Bashrc (bash)
Lines: 11
Complexity: 4/10

Issues Found: 0

Performance Issues: 5

Line 2: eval "$(rbenv init -)" [~150ms]
  → Suggestion: Consider lazy-loading this version manager

Line 3: eval "$(pyenv init -)" [~200ms]
  → Suggestion: Consider lazy-loading this version manager

Line 4: eval "$(nodenv init -)" [~100ms]
  → Suggestion: Consider lazy-loading this version manager

Line 10: eval "$(gh completion -s bash)" [~80ms]
  → Suggestion: Consider lazy-loading completions

Line 11: eval "$(kubectl completion bash)" [~120ms]
  → Suggestion: Consider lazy-loading completions

Total Estimated Startup Cost: ~650ms

Recommendation: Implement lazy-loading pattern:
```bash
# Lazy-load rbenv
rbenv() {
    unset -f rbenv
    eval "$(command rbenv init -)"
    rbenv "$@"
}
```

See: https://bashrs.dev/docs/config/performance
```text

## Output Formats

### Human-Readable (Default)

Best for interactive use and reading:

```bash
bashrs config analyze ~/.bashrc
```

Features:
- Color-coded severity levels (errors in red, warnings in yellow)
- Clear section headers
- Actionable recommendations
- Summary statistics

### JSON

Best for programmatic analysis and CI/CD integration:

```bash
bashrs config analyze ~/.bashrc --format json
```

Features:
- Structured data for parsing
- All issue details included
- Machine-readable format
- Easy to filter/query with `jq`

Example with `jq`:

```bash
# Count issues by severity
bashrs config analyze ~/.bashrc --format json | jq '.issues | group_by(.severity) | map({severity: .[0].severity, count: length})'

# Extract only CONFIG-001 issues
bashrs config analyze ~/.bashrc --format json | jq '.issues[] | select(.rule_id == "CONFIG-001")'

# Get all duplicate PATH entries
bashrs config analyze ~/.bashrc --format json | jq '.path_entries[] | select(.is_duplicate == true)'
```

### SARIF (Planned - v6.32.0+)

Best for security scanning and GitHub Code Scanning:

```bash
bashrs config analyze ~/.bashrc --format sarif > results.sarif
```

Features:
- Industry-standard format
- GitHub Code Scanning integration
- GitLab Security Dashboard support
- Rich metadata and remediation guidance

## Integration with CI/CD Pipelines

### GitHub Actions

```yaml
name: Config Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install bashrs
        run: cargo install bashrs

      - name: Analyze shell configs
        run: |
          bashrs config analyze .bashrc --format json > results.json
          bashrs config analyze .zshrc --format json >> results.json

      - name: Check for errors
        run: |
          if jq -e '.issues[] | select(.severity == "Error")' results.json; then
            echo "❌ Config errors found"
            exit 1
          fi
```

### GitLab CI

```yaml
config_analysis:
  stage: test
  image: rust:latest
  script:
    - cargo install bashrs
    - bashrs config analyze .bashrc --format json > results.json
    - |
      if jq -e '.issues[] | select(.severity == "Error")' results.json; then
        echo "❌ Config errors found"
        exit 1
      fi
  artifacts:
    reports:
      dotenv: results.json
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Analyze shell configs before commit

configs=(".bashrc" ".bash_profile" ".zshrc")

for config in "${configs[@]}"; do
    if [ -f "$config" ]; then
        echo "Analyzing $config..."
        if ! bashrs config analyze "$config" --format json > /tmp/analysis.json; then
            echo "❌ Analysis failed for $config"
            exit 1
        fi

        # Check for errors
        if jq -e '.issues[] | select(.severity == "Error")' /tmp/analysis.json > /dev/null; then
            echo "❌ Config errors found in $config"
            jq '.issues[] | select(.severity == "Error")' /tmp/analysis.json
            exit 1
        fi
    fi
done

echo "✅ All configs analyzed successfully"
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Best Practices for Config Analysis

### 1. Analyze Regularly

Run analysis regularly, especially:
- Before committing config changes
- After installing new tools
- During system migrations
- When shell startup feels slow

```bash
# Add to your workflow
alias analyze-config='bashrs config analyze ~/.bashrc && bashrs config analyze ~/.zshrc'
```

### 2. Use JSON Output for Automation

```bash
# Check if any errors exist
bashrs config analyze ~/.bashrc --format json | jq -e '.issues[] | select(.severity == "Error")'

# Count warnings
bashrs config analyze ~/.bashrc --format json | jq '[.issues[] | select(.severity == "Warning")] | length'

# Extract performance impact
bashrs config analyze ~/.bashrc --format json | jq '[.performance_issues[].estimated_cost_ms] | add'
```

### 3. Fix Issues Incrementally

Don't try to fix everything at once:

```bash
# Start with errors only
bashrs config analyze ~/.bashrc --format json | jq '.issues[] | select(.severity == "Error")'

# Then fix high-priority warnings
bashrs config analyze ~/.bashrc --format json | jq '.issues[] | select(.rule_id == "CONFIG-001" or .rule_id == "CONFIG-002")'

# Finally address info-level issues
bashrs config analyze ~/.bashrc --format json | jq '.issues[] | select(.severity == "Info")'
```

### 4. Track Improvements Over Time

```bash
# Baseline
bashrs config analyze ~/.bashrc --format json > baseline.json

# After improvements
bashrs config analyze ~/.bashrc --format json > improved.json

# Compare
echo "Before: $(jq '.issues | length' baseline.json) issues"
echo "After: $(jq '.issues | length' improved.json) issues"
echo "Fixed: $(( $(jq '.issues | length' baseline.json) - $(jq '.issues | length' improved.json) )) issues"
```

### 5. Combine with Linting

`bashrs config analyze` focuses on configuration-specific issues. Combine with `bashrs lint` for comprehensive analysis:

```bash
# Config-specific analysis
bashrs config analyze ~/.bashrc

# General shell linting
bashrs lint ~/.bashrc

# Combined analysis
bashrs audit ~/.bashrc  # Runs both + more
```

### 6. Understand Your Complexity Score

Complexity scores (0-10):

| Score | Grade | Description |
|-------|-------|-------------|
| 0-2   | A+    | Minimal - Very simple config |
| 3-4   | A     | Low - Simple, maintainable |
| 5-6   | B     | Moderate - Reasonable complexity |
| 7-8   | C     | High - Consider simplifying |
| 9-10  | D/F   | Very High - Refactor recommended |

```bash
# Check complexity trend
for config in ~/.bashrc ~/.bash_profile ~/.zshrc; do
    score=$(bashrs config analyze "$config" --format json | jq '.complexity_score')
    echo "$config: $score/10"
done
```

## Troubleshooting

### Issue: False Positive for CONFIG-004 (Timestamps)

**Problem**: Timestamp usage flagged as non-deterministic, but it's legitimate for measuring command execution time.

**Example**:

```bash
# In .zshrc - measures command execution time
preexec() { timer=$(($(date +%s))) }
precmd() {
    elapsed=$(($(date +%s) - timer))
    echo "Took ${elapsed}s"
}
```

**Solution**: This is expected behavior. CONFIG-004 flags all non-deterministic constructs. For timing measurements in interactive shells, this is acceptable and can be ignored.

**Future**: CONFIG-004 will gain context awareness to distinguish legitimate timestamp usage (v6.33.0+).

### Issue: Large Number of CONFIG-002 Warnings

**Problem**: Many unquoted variables flagged, making output overwhelming.

**Solution**: Use JSON output with `jq` to filter:

```bash
# Count CONFIG-002 issues
bashrs config analyze ~/.bashrc --format json | jq '[.issues[] | select(.rule_id == "CONFIG-002")] | length'

# Group by line
bashrs config analyze ~/.bashrc --format json | jq '.issues[] | select(.rule_id == "CONFIG-002") | .line' | sort -n | uniq -c
```

Then fix with automatic purification:

```bash
bashrs config purify ~/.bashrc --fix
```

### Issue: Performance Issues Reported for NVM

**Problem**: NVM initialization flagged as expensive, but it's necessary.

**Example**:

```bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
```

**Solution**: Implement lazy-loading pattern:

```bash
# Lazy-load NVM
nvm() {
    unset -f nvm node npm
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm "$@"
}

# Placeholder for node/npm
node() {
    unset -f nvm node npm
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    node "$@"
}

npm() {
    unset -f nvm node npm
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    npm "$@"
}
```

This defers NVM initialization until first use, improving shell startup by ~200ms.

### Issue: Complexity Score Seems High

**Problem**: Complexity score is 8/10 but config seems reasonable.

**Cause**: Complexity calculation considers:
- Line count (>200 lines = higher score)
- Function count and length
- Conditional nesting depth
- Comment density

**Solution**:

1. **Extract functions to separate files**:

```bash
# ~/.bashrc
source ~/.bash_functions
source ~/.bash_aliases
```

2. **Remove unused code**:

```bash
# Use git to track what you remove
git add ~/.bashrc
bashrs config analyze ~/.bashrc --format json | jq '.issues'
# Remove unused sections
git diff ~/.bashrc
```

3. **Simplify conditionals**:

```bash
# Before (nested)
if [ "$OS" = "Darwin" ]; then
    if [ -d "/opt/homebrew" ]; then
        export PATH="/opt/homebrew/bin:$PATH"
    fi
fi

# After (flat)
[ "$OS" = "Darwin" ] && [ -d "/opt/homebrew" ] && export PATH="/opt/homebrew/bin:$PATH"
```

### Issue: Can't Analyze Symlinked Config

**Problem**: `bashrs config analyze ~/.bashrc` fails when `.bashrc` is a symlink.

**Cause**: Tool follows symlinks but may have permission issues.

**Solution**:

```bash
# Analyze the real file
bashrs config analyze "$(readlink -f ~/.bashrc)"

# Or fix permissions
chmod +r ~/.bashrc
```

### Issue: JSON Output Truncated

**Problem**: JSON output appears incomplete.

**Cause**: Large configs generate large JSON. Shell may truncate output.

**Solution**:

```bash
# Write to file instead
bashrs config analyze ~/.bashrc --format json > analysis.json

# Then analyze
jq '.' analysis.json
```

## Advanced Usage

### Analyze Multiple Configs

```bash
# Analyze all config files
for config in ~/.bashrc ~/.bash_profile ~/.zshrc ~/.profile; do
    [ -f "$config" ] && echo "=== $config ===" && bashrs config analyze "$config"
done
```

### Compare Configs Before/After

```bash
# Before
bashrs config analyze ~/.bashrc --format json > before.json

# Make changes...
vim ~/.bashrc

# After
bashrs config analyze ~/.bashrc --format json > after.json

# Compare
diff <(jq -S '.' before.json) <(jq -S '.' after.json)
```

### Extract Specific Metrics

```bash
# Total issues
bashrs config analyze ~/.bashrc --format json | jq '.issues | length'

# Issues by severity
bashrs config analyze ~/.bashrc --format json | jq '.issues | group_by(.severity) | map({severity: .[0].severity, count: length})'

# Average performance cost
bashrs config analyze ~/.bashrc --format json | jq '[.performance_issues[].estimated_cost_ms] | add / length'

# Complexity trend
bashrs config analyze ~/.bashrc --format json | jq '.complexity_score'
```

### Integration with Other Tools

```bash
# Combine with shellcheck
bashrs config analyze ~/.bashrc --format json > bashrs.json
shellcheck -f json ~/.bashrc > shellcheck.json

# Merge results
jq -s '.[0] + .[1]' bashrs.json shellcheck.json > combined.json

# Analyze combined
jq '.issues | length' combined.json
```

## See Also

- [Configuration Overview](./overview.md) - Understanding shell configuration files
- [Purifying Configs](./purifying.md) - Automatically fixing detected issues
- [CONFIG-001](./rules/config-001.md) - PATH deduplication details
- [CONFIG-002](./rules/config-002.md) - Variable quoting details
- [CONFIG-003](./rules/config-003.md) - Alias consolidation details
- [Complete Workflow Example](../example_zshrc_workflow.md) - Real-world .zshrc analysis
- [CLI Reference](../reference/cli.md) - All `bashrs config` commands
