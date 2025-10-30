# Specification: Shell Configuration File Management

**Feature**: `bashrs config` - Purify, lint, and manage shell configuration files

**Version**: 1.0.0
**Status**: Draft
**Created**: 2024-10-22
**Author**: Claude + Noah Gift

---

## Executive Summary

Shell configuration files (`~/.bashrc`, `~/.zshrc`, `~/.profile`, etc.) are critical developer infrastructure that accumulates technical debt over years of use. Developers copy-paste snippets from StackOverflow, add hacky workarounds, and rarely clean up obsolete configurations. This leads to:

- **Non-deterministic behavior** (different shells, different results)
- **Performance issues** (unnecessary path scans, duplicate entries)
- **Security vulnerabilities** (unquoted variables, injection risks)
- **Maintenance nightmares** (can't remember what each line does)

**Solution**: `bashrs config` provides automatic purification, linting, and management of shell configuration files using Rash's full AST understanding and transformation capabilities.

---

## Problem Space

### Common Issues in Shell Config Files

1. **Duplicate PATH entries**
   ```bash
   # ~/.bashrc becomes bloated over time
   export PATH="/usr/local/bin:$PATH"
   export PATH="/opt/homebrew/bin:$PATH"
   export PATH="/usr/local/bin:$PATH"  # Duplicate!
   export PATH="$HOME/.cargo/bin:$PATH"
   export PATH="/usr/local/bin:$PATH"  # Duplicate again!
   ```

2. **Non-idempotent operations**
   ```bash
   # Run every shell launch - not idempotent
   alias ls='ls --color=auto'
   alias ls='ls -G'  # Overrides previous - which wins?
   ```

3. **Non-deterministic commands**
   ```bash
   # Different results each time
   export SESSION_ID=$RANDOM
   export BUILD_TAG="dev-$(date +%s)"
   ```

4. **Performance killers**
   ```bash
   # Expensive operations on every shell launch
   eval "$(rbenv init -)"
   eval "$(pyenv init -)"
   eval "$(nodenv init -)"
   [ -f ~/.fzf.bash ] && source ~/.fzf.bash
   # Each eval can take 50-200ms!
   ```

5. **Unquoted variables**
   ```bash
   export PROJECT_DIR=$HOME/my projects  # Word splitting!
   export FILES=$(ls *.txt)               # Glob expansion!
   ```

6. **Shell-specific syntax mixed together**
   ```bash
   # Bash-isms in .profile (should be POSIX)
   export PATH+=:/usr/local/bin  # += is bash-only
   [[ -f ~/.env ]] && source ~/.env  # [[ ]] is bash-only
   ```

7. **Obsolete/dead code**
   ```bash
   # From 5 years ago, no longer used
   export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk
   alias myproject="cd ~/projects/oldcompany/deadproject"
   ```

8. **Security issues**
   ```bash
   # Dangerous patterns
   eval "$(curl -s https://untrusted.com/setup.sh)"
   source $DOWNLOAD_DIR/config.sh  # Unquoted!
   ```

---

## User Personas

### 1. **Sarah - Senior Developer**
- Uses zsh, heavily customized config
- Config file is 800+ lines, accumulated over 8 years
- "I'm afraid to touch it - something might break"
- **Needs**: Safe cleanup, duplicate removal, performance analysis

### 2. **Mike - DevOps Engineer**
- Manages dotfiles across 20+ servers
- Uses Ansible to deploy configs
- **Needs**: Deterministic, idempotent configs that work on bash/zsh/dash

### 3. **Emma - Junior Developer**
- Just started, copied .zshrc from a tutorial
- Doesn't understand half of what's in there
- **Needs**: Explanation of what each section does, safety validation

### 4. **Alex - Security Engineer**
- Audits developer workstations
- **Needs**: Detect security issues, enforce safe defaults

---

## Feature Requirements

### Core Capabilities

1. **Purify** - Fix common issues automatically
   - Remove duplicate PATH entries (preserve order)
   - Quote all variable expansions
   - Convert non-idempotent operations
   - Remove non-deterministic constructs
   - Fix shell-specific syntax issues

2. **Lint** - Detect issues with clear explanations
   - Performance problems (expensive evals)
   - Security vulnerabilities
   - Obsolete patterns
   - Cross-shell compatibility issues

3. **Analyze** - Provide insights
   - Performance profile (which commands are slow)
   - Dependency analysis (what's required vs optional)
   - Complexity score
   - Dead code detection

4. **Manage** - Organize and maintain
   - Split large configs into modular sections
   - Generate structured, commented output
   - Create portable configs (POSIX-compliant)
   - Version control friendly

---

## CLI Interface

### Basic Commands

```bash
# Analyze current shell config
bashrs config analyze ~/.bashrc

# Lint config file
bashrs config lint ~/.zshrc

# Purify config file (creates backup)
bashrs config purify ~/.bashrc

# Apply purification (with backup)
bashrs config purify ~/.bashrc --fix

# Check cross-shell compatibility
bashrs config check ~/.profile --target posix

# Performance profile
bashrs config profile ~/.zshrc

# Explain what config does
bashrs config explain ~/.bashrc

# Modularize large config
bashrs config modularize ~/.bashrc --output ~/.config/bash/
```

### Options

```bash
bashrs config <COMMAND> [OPTIONS]

COMMANDS:
    analyze     Analyze config file structure and issues
    lint        Lint config for safety and compatibility
    purify      Purify config (fix issues automatically)
    check       Validate cross-shell compatibility
    profile     Performance profile (find slow operations)
    explain     Generate human-readable documentation
    modularize  Split large config into organized modules

OPTIONS:
    --fix                Apply fixes automatically (creates .bak)
    --dry-run            Show what would be changed
    --target <SHELL>     Target shell: posix, bash, zsh, ash
    --strict             Enable strict mode (error on warnings)
    --no-backup          Don't create backup (dangerous!)
    --explain            Show explanations for each change
    --output <PATH>      Output directory (for modularize)

EXAMPLES:
    # Safe purification with preview
    bashrs config purify ~/.bashrc --dry-run

    # Apply fixes with backup
    bashrs config purify ~/.bashrc --fix

    # Check if .profile is POSIX compliant
    bashrs config check ~/.profile --target posix

    # Find performance bottlenecks
    bashrs config profile ~/.zshrc

    # Split large config into modules
    bashrs config modularize ~/.bashrc --output ~/.config/bash/
```

---

## Purification Rules

### CONFIG-001: Deduplicate PATH Entries

**Issue**: PATH accumulates duplicates over time

**Input**:
```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate
export PATH="$HOME/.cargo/bin:$PATH"
```

**Output**:
```bash
# PATH entries (deduplicated, order preserved)
export PATH="/usr/local/bin:${PATH}"
export PATH="/opt/homebrew/bin:${PATH}"
export PATH="${HOME}/.cargo/bin:${PATH}"
```

---

### CONFIG-002: Quote Variable Expansions

**Issue**: Unquoted variables can cause word splitting and glob expansion

**Input**:
```bash
export PROJECT_DIR=$HOME/my projects
export BACKUP_DIR=$HOME/backups
cd $PROJECT_DIR
```

**Output**:
```bash
export PROJECT_DIR="${HOME}/my projects"
export BACKUP_DIR="${HOME}/backups"
cd "${PROJECT_DIR}"
```

---

### CONFIG-003: Convert Non-Idempotent Aliases

**Issue**: Aliases that build on themselves cause issues

**Input**:
```bash
alias ls='ls --color=auto'
alias ls='ls -G'  # Overwrites previous
```

**Output**:
```bash
# Consolidated aliases (later definition wins)
alias ls='ls -G'
```

---

### CONFIG-004: Remove Non-Deterministic Constructs

**Issue**: $RANDOM, timestamps, $$ make configs non-reproducible

**Input**:
```bash
export SESSION_ID=$RANDOM
export BUILD_TAG="dev-$(date +%s)"
export TEMP_DIR="/tmp/work-$$"
```

**Output**:
```bash
# Deterministic alternatives
export SESSION_ID="${USER}-${HOSTNAME}"
export BUILD_TAG="dev-local"
export TEMP_DIR="${HOME}/.cache/work"
```

---

### CONFIG-005: Optimize Expensive Operations

**Issue**: eval and source on every shell launch is slow

**Input**:
```bash
eval "$(rbenv init -)"
eval "$(pyenv init -)"
eval "$(nodenv init -)"
```

**Output**:
```bash
# Lazy-load version managers (only when needed)
rbenv() {
    unset -f rbenv
    eval "$(command rbenv init -)"
    rbenv "$@"
}

pyenv() {
    unset -f pyenv
    eval "$(command pyenv init -)"
    pyenv "$@"
}
```

---

### CONFIG-006: Fix Shell-Specific Syntax

**Issue**: Bash-isms in POSIX shell scripts

**Input** (~/.profile):
```bash
[[ -f ~/.env ]] && source ~/.env
export PATH+=:/usr/local/bin
```

**Output**:
```sh
# POSIX-compliant
[ -f "${HOME}/.env" ] && . "${HOME}/.env"
export PATH="${PATH}:/usr/local/bin"
```

---

### CONFIG-007: Security - Validate Source Paths

**Issue**: Sourcing unquoted or untrusted files

**Input**:
```bash
source $CONFIG_DIR/settings.sh
eval "$(curl -s https://example.com/setup.sh)"
```

**Output**:
```bash
# Safe sourcing with validation
if [ -f "${CONFIG_DIR}/settings.sh" ]; then
    . "${CONFIG_DIR}/settings.sh"
fi

# WARNING: eval of remote content is dangerous - removed
# Original: eval "$(curl -s https://example.com/setup.sh)"
# Security: Remote code execution risk
```

---

### CONFIG-008: Dead Code Detection

**Issue**: Obsolete paths and aliases clutter configs

**Input**:
```bash
export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk
alias oldproject="cd ~/projects/2015/oldproject"
export PYTHONPATH="/opt/python2.7/lib"
```

**Output** (with warnings):
```bash
# WARNING: Path does not exist: /Library/Java/JavaVirtualMachines/1.6.0.jdk
# export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk

# WARNING: Directory does not exist: ~/projects/2015/oldproject
# alias oldproject="cd ~/projects/2015/oldproject"

# WARNING: Path does not exist: /opt/python2.7/lib
# export PYTHONPATH="/opt/python2.7/lib"

# Note: Commented out lines reference non-existent paths
# Review and remove if no longer needed
```

---

## Safety Guarantees

### Backup Strategy

1. **Always create backups** (unless `--no-backup` explicitly specified)
2. **Timestamped backups**: `~/.bashrc.bak.2024-10-22T14-30-45`
3. **Keep last 10 backups** (configurable)
4. **Atomic writes**: Write to temp file, then rename

### Validation Before Apply

1. **Syntax validation**: Parse config to AST, ensure valid
2. **Shell compatibility check**: Verify target shell can execute
3. **Dry-run mode**: Show diff before applying
4. **Rollback command**: `bashrs config rollback ~/.bashrc`

### User Consent

```bash
# Interactive mode (default)
$ bashrs config purify ~/.bashrc --fix

Analyzing ~/.bashrc...
Found 12 issues:
  - 3 duplicate PATH entries
  - 5 unquoted variables
  - 2 non-idempotent aliases
  - 1 security warning (eval of remote URL)
  - 1 dead code (path does not exist)

Apply fixes? [y/N/preview] preview

=== Preview of changes ===
[Shows unified diff]

Apply fixes? [y/N] y

Creating backup: ~/.bashrc.bak.2024-10-22T14-30-45
Applying 12 fixes...
✓ Done!

To rollback: bashrs config rollback ~/.bashrc
```

---

## Performance Analysis

### Profiling Output

```bash
$ bashrs config profile ~/.zshrc

Performance Profile: ~/.zshrc
==============================

Shell startup time: 847ms

Slowest operations:
  1. eval "$(rbenv init -)"          312ms  (37%)
  2. eval "$(pyenv init -)"          198ms  (23%)
  3. eval "$(nodenv init -)"         145ms  (17%)
  4. source ~/.fzf.zsh                89ms  (10%)
  5. nvm lazy-load script             67ms   (8%)

Recommendations:
  ✓ Lazy-load rbenv (save ~300ms)
  ✓ Lazy-load pyenv (save ~190ms)
  ✓ Lazy-load nodenv (save ~140ms)
  ✓ Consider removing unused version managers

Estimated speedup: 630ms → 217ms (74% faster)

Apply optimizations? bashrs config purify ~/.zshrc --optimize
```

---

## Modularization

### Before (Monolithic)

```bash
# ~/.bashrc - 800 lines, hard to maintain
export PATH="..."
export EDITOR="vim"
# ... 750 more lines ...
```

### After (Modular)

```bash
# ~/.bashrc - clean entry point
# Generated by bashrs v6.0.0

# Load modules in order
for module in "${HOME}/.config/bash/modules"/*.sh; do
    [ -f "${module}" ] && . "${module}"
done
```

**Generated modules**:
```
~/.config/bash/
├── modules/
│   ├── 01-environment.sh    # Environment variables
│   ├── 02-path.sh           # PATH configuration
│   ├── 03-aliases.sh        # Command aliases
│   ├── 04-functions.sh      # Shell functions
│   ├── 05-completions.sh    # Tab completions
│   ├── 06-prompt.sh         # Prompt configuration
│   ├── 07-tools.sh          # Tool integrations (fzf, etc)
│   └── 99-local.sh          # Machine-specific overrides
└── README.md                # Documentation
```

---

## Cross-Shell Compatibility

### Compatibility Matrix

| Feature | POSIX sh | bash | zsh | ash | fish |
|---------|----------|------|-----|-----|------|
| Variables | ✅ | ✅ | ✅ | ✅ | ⚠️ Different syntax |
| Aliases | ✅ | ✅ | ✅ | ✅ | ⚠️ Different syntax |
| Functions | ✅ | ✅ | ✅ | ✅ | ⚠️ Different syntax |
| Arrays | ❌ | ✅ | ✅ | ❌ | ⚠️ Different syntax |
| `[[` test | ❌ | ✅ | ✅ | ❌ | ❌ |
| `local` keyword | ⚠️ Not POSIX | ✅ | ✅ | ✅ | ⚠️ Different |

### Compatibility Checks

```bash
$ bashrs config check ~/.profile --target posix

Checking ~/.profile for POSIX compliance...

❌ Line 12: [[ -f ~/.env ]] - [[ ]] is not POSIX
   Fix: Use [ -f "${HOME}/.env" ]

❌ Line 34: export PATH+=:/usr/local/bin - += is bash-only
   Fix: export PATH="${PATH}:/usr/local/bin"

⚠️  Line 56: local var="value" - 'local' is not in POSIX
   Note: Widely supported but not standardized

Summary: 2 errors, 1 warning
Compliance: ❌ Not POSIX compliant

Apply fixes: bashrs config purify ~/.profile --target posix --fix
```

---

## Example Workflow

### Initial State (~/.bashrc - Messy)

```bash
# My bashrc - accumulated over 5 years

export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"

export EDITOR=vim
export PROJECT_DIR=$HOME/my projects

alias ls='ls --color=auto'
alias ll='ls -lah'
alias ls='ls -G'

eval "$(rbenv init -)"
eval "$(pyenv init -)"

export SESSION=$RANDOM
export BUILD="build-$(date +%s)"

source $HOME/.local/config.sh

export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk
```

### Step 1: Analyze

```bash
$ bashrs config analyze ~/.bashrc

Analysis: ~/.bashrc
===================

Statistics:
  - Lines: 19
  - Complexity score: 7/10 (moderate)
  - Estimated load time: ~340ms

Issues Found: 11
  ✗ 2 duplicate PATH entries (CONFIG-001)
  ✗ 3 unquoted variables (CONFIG-002)
  ✗ 1 duplicate alias (CONFIG-003)
  ✗ 2 non-deterministic constructs (CONFIG-004)
  ✗ 2 expensive evals (CONFIG-005)
  ✗ 1 unquoted source path (CONFIG-007)
  ✗ 1 dead code (non-existent path) (CONFIG-008)

Recommendations:
  1. Remove duplicate PATH entries
  2. Quote all variable expansions
  3. Replace $RANDOM with deterministic ID
  4. Lazy-load rbenv/pyenv (save ~300ms)
  5. Remove obsolete JAVA_6_HOME

Run: bashrs config purify ~/.bashrc --fix
```

### Step 2: Purify (Preview)

```bash
$ bashrs config purify ~/.bashrc --dry-run

Preview of changes to ~/.bashrc:
================================

--- ~/.bashrc (original)
+++ ~/.bashrc (purified)
@@ -1,19 +1,29 @@
-# My bashrc - accumulated over 5 years
+#!/bin/bash
+# Purified by bashrs v6.0.0
+# Original: ~/.bashrc

-export PATH="/usr/local/bin:$PATH"
-export PATH="/opt/homebrew/bin:$PATH"
-export PATH="/usr/local/bin:$PATH"
+# PATH configuration (deduplicated)
+export PATH="/usr/local/bin:${PATH}"
+export PATH="/opt/homebrew/bin:${PATH}"

-export EDITOR=vim
-export PROJECT_DIR=$HOME/my projects
+# Environment variables
+export EDITOR="vim"
+export PROJECT_DIR="${HOME}/my projects"

-alias ls='ls --color=auto'
+# Aliases (consolidated)
 alias ll='ls -lah'
-alias ls='ls -G'
+alias ls='ls -G'  # Note: overrides earlier ls alias

-eval "$(rbenv init -)"
-eval "$(pyenv init -)"
+# Version managers (lazy-loaded for performance)
+rbenv() { unset -f rbenv; eval "$(command rbenv init -)"; rbenv "$@"; }
+pyenv() { unset -f pyenv; eval "$(command pyenv init -)"; pyenv "$@"; }

-export SESSION=$RANDOM
-export BUILD="build-$(date +%s)"
+# Deterministic identifiers
+export SESSION="${USER}-${HOSTNAME}"
+export BUILD="build-local"

-source $HOME/.local/config.sh
+# Configuration loading
+if [ -f "${HOME}/.local/config.sh" ]; then
+    . "${HOME}/.local/config.sh"
+fi

-export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk
+# WARNING: Path does not exist - commented out
+# export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk

Apply changes? Use: bashrs config purify ~/.bashrc --fix
```

### Step 3: Apply Changes

```bash
$ bashrs config purify ~/.bashrc --fix

Creating backup: ~/.bashrc.bak.2024-10-22T14-30-45
Applying 11 fixes...
  ✓ Deduplicated 2 PATH entries
  ✓ Quoted 3 variables
  ✓ Consolidated 1 duplicate alias
  ✓ Replaced 2 non-deterministic constructs
  ✓ Lazy-loaded 2 version managers
  ✓ Quoted 1 source path
  ✓ Commented out 1 dead code entry

✓ Done! ~/.bashrc has been purified.

Backup: ~/.bashrc.bak.2024-10-22T14-30-45
Estimated speedup: 340ms → 40ms (88% faster)

To rollback: bashrs config rollback ~/.bashrc
```

### Final State (~/.bashrc - Purified)

```bash
#!/bin/bash
# Purified by bashrs v6.0.0
# Original: ~/.bashrc

# PATH configuration (deduplicated)
export PATH="/usr/local/bin:${PATH}"
export PATH="/opt/homebrew/bin:${PATH}"

# Environment variables
export EDITOR="vim"
export PROJECT_DIR="${HOME}/my projects"

# Aliases (consolidated)
alias ll='ls -lah'
alias ls='ls -G'  # Note: overrides earlier ls alias

# Version managers (lazy-loaded for performance)
rbenv() { unset -f rbenv; eval "$(command rbenv init -)"; rbenv "$@"; }
pyenv() { unset -f pyenv; eval "$(command pyenv init -)"; pyenv "$@"; }

# Deterministic identifiers
export SESSION="${USER}-${HOSTNAME}"
export BUILD="build-local"

# Configuration loading
if [ -f "${HOME}/.local/config.sh" ]; then
    . "${HOME}/.local/config.sh"
fi

# WARNING: Path does not exist - commented out
# export JAVA_6_HOME=/Library/Java/JavaVirtualMachines/1.6.0.jdk
```

---

## Implementation Roadmap

### Phase 1: Core Purification (v7.0)
- [ ] CONFIG-001: Deduplicate PATH entries
- [ ] CONFIG-002: Quote variable expansions
- [ ] CONFIG-003: Consolidate duplicate aliases
- [ ] CONFIG-004: Remove non-deterministic constructs
- [ ] CONFIG-007: Validate source paths
- [ ] Basic CLI: `analyze`, `lint`, `purify`

### Phase 2: Performance (v7.1)
- [ ] CONFIG-005: Lazy-load expensive operations
- [ ] Performance profiling
- [ ] Optimization suggestions

### Phase 3: Cross-Shell (v7.2)
- [ ] CONFIG-006: Shell-specific syntax detection
- [ ] Compatibility checking
- [ ] Multi-shell support (bash, zsh, posix)

### Phase 4: Advanced Features (v7.3)
- [ ] CONFIG-008: Dead code detection
- [ ] Modularization
- [ ] Explain command (documentation generation)

---

## Testing Strategy

### Unit Tests

Test each purification rule independently:

```rust
#[test]
fn test_config_001_deduplicate_path() {
    let input = r#"
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
    "#;

    let expected = r#"
export PATH="/usr/local/bin:${PATH}"
export PATH="/opt/homebrew/bin:${PATH}"
    "#;

    let result = purify_config(input);
    assert_eq!(result, expected);
}
```

### Integration Tests

Test with real-world config files:

```bash
# Test against actual .bashrc files
bashrs config purify tests/fixtures/configs/messy-bashrc.sh --dry-run
```

### Property-Based Tests

```rust
proptest! {
    #[test]
    fn prop_purified_configs_are_idempotent(
        config in generate_valid_config()
    ) {
        let purified1 = purify_config(&config);
        let purified2 = purify_config(&purified1);
        prop_assert_eq!(purified1, purified2);
    }
}
```

### Manual Testing

Test on diverse real configs:
- Sarah's 800-line .zshrc
- Mike's Ansible-deployed configs
- Emma's copied tutorial config
- Minimal POSIX .profile

---

## Success Metrics

### Quantitative
- **Performance improvement**: Average 60% reduction in shell startup time
- **Issue detection**: Find 8+ issues per 100 lines of config
- **Auto-fix rate**: 85% of issues fixed automatically
- **Adoption**: 1000+ GitHub stars, 10k+ downloads/month

### Qualitative
- **Developer confidence**: "Now I understand what my .bashrc does"
- **Maintainability**: "Config is organized and documented"
- **Safety**: "No more broken shell sessions after edits"

---

## Security Considerations

### Threat Model

1. **Malicious config injection**: User edits, attacker adds malicious code
2. **Supply chain attacks**: Configs that download/eval remote code
3. **Privilege escalation**: Configs that modify system paths unsafely
4. **Data exfiltration**: Configs that leak environment variables

### Mitigations

1. **Detect dangerous patterns**:
   ```bash
   eval "$(curl http://...)"  # BLOCK
   source <(wget -qO- ...)    # BLOCK
   rm -rf $VAR                # WARN (unquoted)
   ```

2. **Sandboxed execution**: Parse and analyze without executing

3. **User warnings**: Clear explanations of security risks

4. **Audit trail**: Log all purification changes

---

## Open Questions

1. **How to handle plugin managers?** (oh-my-zsh, prezto, bash-it)
   - Should we try to understand their structure?
   - Or treat them as opaque?

2. **Machine-specific vs portable configs?**
   - How to split them?
   - Use environment detection?

3. **Interactive prompt customization?**
   - These are often complex and shell-specific
   - Should we attempt to purify them?

4. **Handling comments?**
   - Preserve user comments?
   - Add generated documentation?

5. **Version control integration?**
   - Auto-commit before purification?
   - Generate diff reports?

---

## References

- [Bash Manual](https://www.gnu.org/software/bash/manual/)
- [Zsh Manual](https://zsh.sourceforge.io/Doc/)
- [POSIX Shell](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [ShellCheck Wiki](https://www.shellcheck.net/wiki/)

---

## Appendix A: Common Config Patterns

### Pattern: NVM (Node Version Manager)

**Common issue**:
```bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # Slow!
```

**Purified**:
```bash
# Lazy-load NVM (speeds up shell startup)
export NVM_DIR="${HOME}/.nvm"
nvm() {
    unset -f nvm node npm
    [ -s "${NVM_DIR}/nvm.sh" ] && . "${NVM_DIR}/nvm.sh"
    nvm "$@"
}
```

### Pattern: Homebrew

**Common issue**:
```bash
eval "$(/opt/homebrew/bin/brew shellenv)"  # Every shell launch!
```

**Purified**:
```bash
# Static Homebrew paths (no eval needed)
export HOMEBREW_PREFIX="/opt/homebrew"
export HOMEBREW_CELLAR="${HOMEBREW_PREFIX}/Cellar"
export HOMEBREW_REPOSITORY="${HOMEBREW_PREFIX}"
export PATH="${HOMEBREW_PREFIX}/bin:${HOMEBREW_PREFIX}/sbin:${PATH}"
export MANPATH="${HOMEBREW_PREFIX}/share/man:${MANPATH}"
export INFOPATH="${HOMEBREW_PREFIX}/share/info:${INFOPATH}"
```

### Pattern: Conda/Anaconda

**Common issue**:
```bash
# >>> conda initialize >>>
# Massive block of generated code (100+ lines)
eval "$(__conda_setup)"
# <<< conda initialize <<<
```

**Purified**:
```bash
# Conda (lazy-loaded)
__conda_setup() {
    # [Actual conda setup code, but only runs when 'conda' is invoked]
}

conda() {
    __conda_setup
    unset -f conda
    conda "$@"
}
```

---

## Appendix B: Error Codes

```
CONFIG-E001: Syntax error in config file
CONFIG-E002: Unable to create backup
CONFIG-E003: Target shell not supported
CONFIG-E004: File not readable
CONFIG-E005: File not writable
CONFIG-E006: Invalid purification rule
CONFIG-E007: Circular source dependency detected

CONFIG-W001: Duplicate PATH entry
CONFIG-W002: Unquoted variable expansion
CONFIG-W003: Non-idempotent operation
CONFIG-W004: Non-deterministic construct
CONFIG-W005: Expensive operation (performance)
CONFIG-W006: Shell-specific syntax in POSIX file
CONFIG-W007: Security risk (eval/source remote)
CONFIG-W008: Dead code (non-existent path)
CONFIG-W009: Obsolete pattern detected
CONFIG-W010: Missing error handling
```

---

**End of Specification**
