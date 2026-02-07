# SPEC-COMPLY-2026-001: bashrs comply — Shell Artifact Compliance System

**Version**: 1.0.0
**Status**: Draft
**Author**: paiml engineering
**Date**: 2026-02-07
**Requires**: bashrs >= 7.1.0, pzsh >= 1.0.0 (optional peer)

---

## Abstract

This specification defines `bashrs comply`, a 3-layer compliance system for shell
artifacts across project and user scopes. It tracks, validates, and governs all
shell-related files: `*.sh`, `Makefile`, `Dockerfile`, `.bashrc`, `.zshrc`,
`.profile`, and pzsh-managed configurations. The system follows Toyota Production
System (TPS) quality principles and Popperian falsification methodology, with
peer-reviewed academic citations grounding each design decision.

---

## 1. Motivation

### 1.1 The Shell Artifact Governance Gap

Modern projects contain dozens of shell artifacts spread across two scopes:

| Scope | Examples | Current Governance |
|-------|----------|--------------------|
| **Project** | `*.sh`, `Makefile`, `Dockerfile`, `docker-compose.yml` | Ad-hoc linting |
| **User/System** | `~/.zshrc`, `~/.bashrc`, `~/.profile`, pzsh configs | None |

No tool today provides unified compliance tracking across both scopes. ShellCheck
lints individual files. `pmat comply` tracks Rust project health. But shell
artifacts—the glue of every deployment pipeline—have no compliance system.

### 1.2 Theoretical Foundation

**Popper's Falsificationism** (Popper, 1959): A compliance claim is scientific only
if it is falsifiable. Every assertion in `bashrs comply` must specify the test that
would refute it. "This project is POSIX-compliant" is meaningless without the
falsification test: `shellcheck -s sh` on every artifact.

> "In so far as a scientific statement speaks about reality, it must be falsifiable;
> and in so far as it is not falsifiable, it does not speak about reality."
> — Karl Popper, *The Logic of Scientific Discovery* (1959), §6.

**Toyota's Jidoka (自働化)** — Build quality in, don't inspect it in (Ohno, 1988).
Compliance is not a post-hoc audit; it is an integrated production constraint.
Non-compliant artifacts must stop the line.

> "Stop and fix problems when they first occur, even if it means stopping the
> production line."
> — Taiichi Ohno, *Toyota Production System: Beyond Large-Scale Production* (1988), Ch. 3.

### 1.3 Citations

| # | Citation | Relevance |
|---|----------|-----------|
| C1 | Popper, K. (1959). *The Logic of Scientific Discovery*. Routledge. | Falsification methodology for compliance claims |
| C2 | Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. | Jidoka (stop-the-line), Genchi Genbutsu (go and see) |
| C3 | Liker, J. (2004). *The Toyota Way: 14 Management Principles*. McGraw-Hill. | Principle 5 (build quality in), Principle 12 (go and see) |
| C4 | Deming, W.E. (1986). *Out of the Crisis*. MIT Press. | PDCA cycle, statistical process control for compliance |
| C5 | Wheeler, D. (2003). *Secure Programming for Linux and Unix HOWTO*. | POSIX shell security best practices |
| C6 | Bernstein, D.J. (1997). *qmail security guarantee*. | Falsifiable security claims methodology |
| C7 | Leveson, N. (2011). *Engineering a Safer World*. MIT Press. | System safety constraints as invariants |
| C8 | Lakatos, I. (1978). *The Methodology of Scientific Research Programmes*. Cambridge. | Progressive vs. degenerating compliance programs |

---

## 2. Architecture

### 2.1 Three-Layer Compliance Model

Modeled after pmat comply's governance layers, adapted for shell artifacts:

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: GOVERNANCE (監査 Kansa)                        │
│  Signed audit artifacts, sovereign compliance trail      │
│  bashrs comply audit                                     │
├─────────────────────────────────────────────────────────┤
│  Layer 2: REVIEW (現地現物 Genchi Genbutsu)              │
│  Evidence-based review with reproducibility checks       │
│  bashrs comply review                                    │
├─────────────────────────────────────────────────────────┤
│  Layer 1: CHECK (自働化 Jidoka)                          │
│  Automated compliance verification, stop-the-line        │
│  bashrs comply check                                     │
└─────────────────────────────────────────────────────────┘
```

**Design rationale** (C3, Principle 5): Quality layers are cumulative. Layer 1
runs on every commit (automated). Layer 2 runs on every PR (human + machine).
Layer 3 runs on every release (governance artifact).

### 2.2 Artifact Scopes

```
┌──────────────────────────────────────────┐
│             PROJECT SCOPE                │
│  *.sh, Makefile, Dockerfile,             │
│  docker-compose.yml, .github/workflows/* │
│  scripts/*, hooks/*                      │
├──────────────────────────────────────────┤
│             USER SCOPE                   │
│  ~/.zshrc, ~/.bashrc, ~/.profile,        │
│  ~/.bash_profile, ~/.zprofile,           │
│  ~/.config/pzsh/*, ~/.bashrsrc           │
├──────────────────────────────────────────┤
│             SYSTEM SCOPE                 │
│  /etc/profile, /etc/bash.bashrc,         │
│  /etc/zsh/zshrc, /etc/environment        │
│  (read-only audit, no modification)      │
└──────────────────────────────────────────┘
```

### 2.3 pzsh Integration

bashrs comply is a peer to pzsh, not a dependency. When pzsh is installed:

| Feature | Without pzsh | With pzsh |
|---------|-------------|-----------|
| `~/.zshrc` analysis | bashrs config analyze | bashrs + pzsh performance profile |
| Startup budget | Not checked | Enforced (<10ms, pzsh invariant) |
| Plugin audit | Skip | pzsh plugin compliance check |
| Config compilation | Skip | pzsh compile verification |
| Slow pattern detection | bashrs lint only | bashrs lint + pzsh lint (unified) |

**Discovery protocol**:
```
1. Check PATH for `pzsh` binary
2. If found: pzsh --version → extract version
3. If >= 1.0.0: enable pzsh integration features
4. If not found: degrade gracefully, skip pzsh-specific checks
```

**Rationale** (C3, Principle 11 — Respect your partners): pzsh manages shell
startup performance. bashrs manages shell safety. Neither subsumes the other.
Comply bridges them.

---

## 3. CLI Specification

### 3.1 Command Tree

```
bashrs comply
├── init        Initialize .bashrs/comply.toml manifest
├── check       Layer 1: Automated compliance verification
├── review      Layer 2: Evidence-based review checklist
├── audit       Layer 3: Governance artifact generation
├── report      Generate compliance report
├── track       Add/remove artifacts from tracking
├── status      Show current compliance status (alias: check)
├── diff        Show compliance changes since last check
├── enforce     Install git hooks for compliance enforcement
└── migrate     Migrate to latest bashrs compliance standards
```

### 3.2 `bashrs comply init`

Initialize compliance tracking for a project.

```bash
bashrs comply init [OPTIONS]

Options:
  --scope <SCOPE>       Scopes to track [default: project]
                        [possible values: project, user, system, all]
  --pzsh                Enable pzsh integration (auto-detected)
  --strict              Strict mode (all rules enforced)
  -f, --format <FMT>    Output format [default: text]
                        [possible values: text, json, markdown]
```

**Output**: Creates `.bashrs/comply.toml`:

```toml
[comply]
version = "1.0.0"
bashrs_version = "7.1.0"
created = "2026-02-07T10:00:00Z"

[scopes]
project = true
user = false
system = false

[project]
# Auto-discovered artifacts
artifacts = [
    "Makefile",
    "Dockerfile",
    "scripts/*.sh",
    ".github/workflows/*.yml",
]

[user]
# Tracked user configs (opt-in)
artifacts = [
    "~/.zshrc",
    "~/.bashrc",
]

[rules]
# Compliance rules (all enabled by default)
posix = true              # COMPLY-001: POSIX compliance
determinism = true        # COMPLY-002: No non-deterministic patterns
idempotency = true        # COMPLY-003: Safe to re-run
security = true           # COMPLY-004: No injection vectors
quoting = true            # COMPLY-005: All variables quoted
shellcheck = true         # COMPLY-006: Passes shellcheck -s sh
makefile_safety = true    # COMPLY-007: Makefile security rules
dockerfile_best = true    # COMPLY-008: Dockerfile best practices
config_hygiene = true     # COMPLY-009: Config file hygiene
pzsh_budget = "auto"      # COMPLY-010: pzsh startup budget (auto-detect)

[thresholds]
min_score = 80            # Minimum compliance score (0-100)
max_violations = 0        # Maximum allowed violations (strict)
shellcheck_severity = "warning"  # Minimum shellcheck severity

[integration]
pzsh = "auto"             # auto | enabled | disabled
pmat = "auto"             # auto | enabled | disabled
```

### 3.3 `bashrs comply check`

**Layer 1: Jidoka (自働化)** — Automated stop-the-line verification.

```bash
bashrs comply check [OPTIONS]

Options:
  -p, --path <PATH>         Project path [default: .]
  --scope <SCOPE>           Scope to check [default: project]
  --strict                  Exit with error if non-compliant
  --failures-only           Show only failures
  -f, --format <FMT>        Output format [default: text]
  -o, --output <FILE>       Write output to file
```

**Compliance Rules (COMPLY-001 through COMPLY-010)**:

| Rule | Name | Falsification Test | Citation |
|------|------|--------------------|----------|
| COMPLY-001 | POSIX Compliance | `shellcheck -s sh <file>` returns 0 | C5, C6 |
| COMPLY-002 | Determinism | No `$RANDOM`, `$$`, `date +%s`, `mktemp` without seed | C1 §6 |
| COMPLY-003 | Idempotency | All `mkdir` → `mkdir -p`, `rm` → `rm -f`, `ln` → `ln -sf` | C2 Ch.3 |
| COMPLY-004 | Security | SEC001-SEC008 pass (no eval injection, no curl\|bash) | C5, C7 |
| COMPLY-005 | Variable Quoting | All `$VAR` → `"${VAR}"` in non-arithmetic contexts | C5 §4.3 |
| COMPLY-006 | ShellCheck Clean | `shellcheck --severity=warning` returns 0 | C5 |
| COMPLY-007 | Makefile Safety | No shell injection in recipes, proper quoting | C5 |
| COMPLY-008 | Dockerfile Best Practices | docker007-012 rules pass | C7 |
| COMPLY-009 | Config Hygiene | No PATH duplicates, proper sourcing order | C3 P.5 |
| COMPLY-010 | pzsh Budget | Shell startup < 10ms (when pzsh available) | pzsh invariant |

**Falsification methodology** (C1): Each rule is expressed as a falsifiable
hypothesis. The check attempts to **falsify** compliance. If the falsification
attempt fails (no violations found), the artifact is provisionally compliant.
A single counterexample refutes the claim.

**Output example**:

```
bashrs comply check
═══════════════════════════════════════════════════════════
  COMPLIANCE CHECK — Layer 1 (Jidoka)
═══════════════════════════════════════════════════════════

Scope: project (14 artifacts tracked)
bashrs: 7.1.0 | pzsh: 1.2.0 (integrated)

 Artifact                    Score  Status
─────────────────────────────────────────────────
 Makefile                    100    ✅ COMPLIANT
 Dockerfile                   95    ✅ COMPLIANT
 scripts/deploy.sh            90    ✅ COMPLIANT
 scripts/setup.sh             60    ❌ NON-COMPLIANT
   COMPLY-002: $RANDOM on line 14
   COMPLY-003: mkdir without -p on line 22
   COMPLY-005: unquoted $DIR on line 31
 .github/workflows/ci.yml    100    ✅ COMPLIANT

─────────────────────────────────────────────────
 Overall: 92/100 (13/14 compliant)
 Grade: A
 Falsification attempts: 140 (14 artifacts × 10 rules)
 Falsifications succeeded: 3 (scripts/setup.sh)
═══════════════════════════════════════════════════════════
```

### 3.4 `bashrs comply review`

**Layer 2: Genchi Genbutsu (現地現物)** — Go and see. Evidence-based review
with reproducibility requirements.

```bash
bashrs comply review [OPTIONS]

Options:
  -p, --path <PATH>         Project path [default: .]
  -f, --format <FMT>        Output format [default: markdown]
  -o, --output <FILE>       Write output to file
  --scope <SCOPE>           Scope to review [default: project]
```

**Review checklist** (generated per-artifact):

```markdown
## Review: scripts/deploy.sh

### Hypothesis
> This script is deterministic, idempotent, and POSIX-compliant.

### Falsification Attempts
| # | Test | Result | Evidence |
|---|------|--------|----------|
| 1 | shellcheck -s sh scripts/deploy.sh | PASS | Exit code 0, 0 warnings |
| 2 | grep -n '$RANDOM\|$$\|date +%s' | PASS | No matches |
| 3 | grep -n 'mkdir [^-]' (missing -p) | PASS | No matches |
| 4 | bashrs lint scripts/deploy.sh | PASS | 0 violations |
| 5 | Idempotency: run twice, diff output | PASS | Identical output |

### Reproducibility
```
$ shellcheck -s sh scripts/deploy.sh; echo $?
0
$ bashrs lint scripts/deploy.sh --format json | jq '.violations | length'
0
```

### Verdict
- [x] Hypothesis not falsified after 5 attempts
- [x] All evidence reproducible
- [x] Reviewer: <pending human review>
```

**Rationale** (C2, C3 Principle 12): "Go and see for yourself to thoroughly
understand the situation." Layer 2 requires a human reviewer to verify machine
evidence. The checklist provides reproducible commands so reviewers can confirm
findings independently.

### 3.5 `bashrs comply audit`

**Layer 3: Kansa (監査)** — Governance. Signed, immutable compliance artifact.

```bash
bashrs comply audit [OPTIONS]

Options:
  -p, --path <PATH>         Project path [default: .]
  -f, --format <FMT>        Output format [default: json]
  -o, --output <FILE>       Write output to file
  --scope <SCOPE>           Scope to audit [default: all]
```

**Requires**: Clean git state (no uncommitted changes).

**Output** (JSON audit artifact):

```json
{
  "schema": "bashrs-comply-audit-v1",
  "timestamp": "2026-02-07T10:30:00Z",
  "git_sha": "d8d88240ab...",
  "git_clean": true,
  "bashrs_version": "7.1.0",
  "pzsh_version": "1.2.0",
  "scopes": {
    "project": {
      "artifacts": 14,
      "compliant": 14,
      "score": 98,
      "grade": "A+"
    },
    "user": {
      "artifacts": 2,
      "compliant": 2,
      "score": 95,
      "grade": "A+"
    }
  },
  "rules": {
    "COMPLY-001": { "tested": 16, "passed": 16, "falsified": 0 },
    "COMPLY-002": { "tested": 16, "passed": 16, "falsified": 0 },
    "COMPLY-003": { "tested": 16, "passed": 15, "falsified": 1 },
    "...": "..."
  },
  "falsification_summary": {
    "total_attempts": 160,
    "successful_falsifications": 1,
    "unfalsified_claims": 159,
    "methodology": "Popperian (C1)"
  },
  "pzsh_integration": {
    "startup_ms": 0.003,
    "budget_ms": 10,
    "within_budget": true
  },
  "signature": {
    "method": "git-commit-sha",
    "value": "d8d88240ab..."
  }
}
```

**Rationale** (C1 §10, C4): The audit artifact is a snapshot of falsification
results at a specific git commit. It provides:
1. **Reproducibility**: Any claim can be re-tested at the recorded SHA
2. **Immutability**: Tied to git commit, cannot be retroactively changed
3. **Completeness**: Every rule tested against every artifact
4. **Sovereignty**: The project owns its compliance evidence

### 3.6 `bashrs comply track`

Manage tracked artifacts.

```bash
bashrs comply track [OPTIONS] <ACTION> [PATHS...]

Actions:
  add       Add artifacts to tracking
  remove    Remove artifacts from tracking
  list      List tracked artifacts
  discover  Auto-discover artifacts in project

Options:
  --scope <SCOPE>   Scope [default: project]
  --recursive       Discover recursively
```

**Examples**:

```bash
# Auto-discover all shell artifacts
bashrs comply track discover --recursive

# Add user configs to tracking
bashrs comply track add --scope user ~/.zshrc ~/.bashrc

# List all tracked artifacts
bashrs comply track list --scope all

# Add pzsh config
bashrs comply track add --scope user ~/.config/pzsh/config.toml
```

### 3.7 `bashrs comply enforce`

Install git hooks for pre-commit compliance enforcement.

```bash
bashrs comply enforce [OPTIONS]

Options:
  --tier <TIER>     Enforcement tier [default: 1]
                    1 = fast (COMPLY-001,005,006 only, <5s)
                    2 = standard (all rules, <30s)
                    3 = strict (all rules + pzsh budget, <60s)
  --uninstall       Remove enforcement hooks
```

**Hook behavior**: On pre-commit, runs `bashrs comply check --strict` on staged
shell artifacts. Blocks commit if non-compliant. This is Jidoka: stop the line
when a defect is detected (C2, Ch. 3).

### 3.8 `bashrs comply report`

Generate a compliance report (human, JSON, or markdown).

```bash
bashrs comply report [OPTIONS]

Options:
  -p, --path <PATH>         Project path [default: .]
  -f, --format <FMT>        Output format [default: markdown]
  -o, --output <FILE>       Write output to file
  --include-history          Include compliance history over time
  --scope <SCOPE>            Scope [default: all]
```

### 3.9 `bashrs comply diff`

Show compliance changes since last recorded check.

```bash
bashrs comply diff [OPTIONS]

Options:
  --since <SHA>     Compare against specific commit
  --since-last      Compare against last comply check
```

### 3.10 `bashrs comply migrate`

Migrate compliance config to latest bashrs standards.

```bash
bashrs comply migrate [OPTIONS]

Options:
  --dry-run         Show changes without applying
  --from <VERSION>  Source version [default: auto-detect]
```

---

## 4. Artifact Discovery

### 4.1 Project Scope Discovery

```
Glob patterns (searched in project root):
  *.sh
  scripts/**/*.sh
  bin/**/*.sh
  hooks/**/*.sh
  .github/workflows/*.yml
  .github/workflows/*.yaml
  .husky/*
  Makefile
  makefile
  GNUmakefile
  *.mk
  Dockerfile
  Dockerfile.*
  docker-compose.yml
  docker-compose.yaml
  .dockerignore
  .devcontainer/devcontainer.json
  .bashrsignore
```

### 4.2 User Scope Discovery

```
Known paths (platform-aware):
  ~/.zshrc
  ~/.bashrc
  ~/.bash_profile
  ~/.profile
  ~/.zprofile
  ~/.zshenv
  ~/.zlogout
  ~/.bash_logout
  ~/.config/pzsh/config.toml      (pzsh config)
  ~/.config/pzsh/plugins.toml     (pzsh plugins)
  ~/.config/bashrs/comply.toml    (bashrs user config)
  $XDG_CONFIG_HOME/pzsh/*         (XDG-compliant pzsh)
```

### 4.3 System Scope Discovery (read-only)

```
Known paths (audit only, never modified):
  /etc/profile
  /etc/bash.bashrc
  /etc/zsh/zshrc
  /etc/zsh/zshenv
  /etc/environment
  /etc/shells
```

**System scope constraint** (C7): bashrs comply NEVER modifies system files.
System scope is audit-only. Any remediation must be performed manually by an
administrator. This is a safety constraint, not a convenience trade-off.

---

## 5. Scoring Model

### 5.1 Per-Artifact Score

Each artifact is scored 0-100:

```
score = Σ(rule_weight × rule_pass) / Σ(rule_weight) × 100
```

| Rule | Weight | Rationale |
|------|--------|-----------|
| COMPLY-001 (POSIX) | 20 | Portability is foundational (C5) |
| COMPLY-002 (Determinism) | 15 | Reproducibility requirement (C1, C4) |
| COMPLY-003 (Idempotency) | 15 | Safe re-run requirement (C2) |
| COMPLY-004 (Security) | 20 | Non-negotiable safety (C7) |
| COMPLY-005 (Quoting) | 10 | Injection prevention (C5) |
| COMPLY-006 (ShellCheck) | 10 | Industry standard validation |
| COMPLY-007 (Makefile) | 5 | Format-specific (Makefile only) |
| COMPLY-008 (Dockerfile) | 5 | Format-specific (Dockerfile only) |
| COMPLY-009 (Config) | 5 | Scope-specific (user configs only) |
| COMPLY-010 (pzsh) | 5 | Optional (only when pzsh present) |

Format-specific rules (007-010) only apply to matching artifacts. Weights are
renormalized per artifact.

### 5.2 Project Score

```
project_score = Σ(artifact_score) / artifact_count
```

### 5.3 Grade Scale

| Grade | Score Range | Interpretation |
|-------|-------------|----------------|
| A+ | 95-100 | Exemplary compliance |
| A | 85-94 | Strong compliance |
| B | 70-84 | Adequate, needs improvement |
| C | 50-69 | Below standard, remediation required |
| F | 0-49 | Non-compliant, stop the line |

### 5.4 Gateway Barrier (Popperian)

Per Popper's demarcation criterion (C1, §4): a compliance claim below 60% is
**unfalsifiable** (too many violations to meaningfully test). Below the gateway,
the score reflects only the count of passing rules, not a quality assessment.

---

## 6. Falsification Protocol

### 6.1 Methodology

Every compliance rule is a **hypothesis** (C1):

> H: "Artifact X satisfies rule COMPLY-NNN."

The check attempts to **falsify** H by finding a counterexample. If no
counterexample is found after exhaustive testing, H is **provisionally accepted**
(not proven true — Popper's asymmetry).

### 6.2 Falsification Tests

| Rule | Hypothesis | Falsification Test |
|------|-----------|-------------------|
| COMPLY-001 | "X is POSIX-compliant" | Run `shellcheck -s sh X`. Any warning falsifies. |
| COMPLY-002 | "X is deterministic" | Search for `$RANDOM`, `$$`, `date`, `mktemp` without seed. Any match falsifies. |
| COMPLY-003 | "X is idempotent" | Search for `mkdir` without `-p`, `rm` without `-f`, `ln` without `-sf`. Any match falsifies. |
| COMPLY-004 | "X is secure" | Run bashrs SEC001-SEC008. Any violation falsifies. |
| COMPLY-005 | "X quotes all variables" | Run bashrs SC2086 equivalent. Any unquoted expansion falsifies. |
| COMPLY-006 | "X passes shellcheck" | Run `shellcheck --severity=warning X`. Any finding falsifies. |
| COMPLY-007 | "Makefile Y is safe" | Run bashrs make lint Y. Any violation falsifies. |
| COMPLY-008 | "Dockerfile Z follows best practices" | Run bashrs dockerfile lint Z. Any violation falsifies. |
| COMPLY-009 | "Config C is hygienic" | Run bashrs config lint C. Any violation falsifies. |
| COMPLY-010 | "Shell startup is within budget" | Run `pzsh bench`. p99 > 10ms falsifies. |

### 6.3 Progressive Falsification (Lakatos)

Following Lakatos (C8), the comply system distinguishes between:

- **Progressive compliance**: New rules added, existing rules strengthened,
  falsification coverage increases over time. This indicates a healthy project.
- **Degenerating compliance**: Rules weakened, exceptions added, violations
  suppressed. This indicates compliance theater.

The `bashrs comply report --include-history` command tracks this trajectory.

---

## 7. pzsh Peer Protocol

### 7.1 Discovery

```rust
fn discover_pzsh() -> Option<PzshInfo> {
    // 1. Check PATH
    let path = which("pzsh")?;
    // 2. Get version
    let version = exec("pzsh --version")?;
    // 3. Check compatibility
    if version >= "1.0.0" { Some(PzshInfo { path, version }) }
    else { None }
}
```

### 7.2 Integration Points

| bashrs comply | pzsh | Data Flow |
|--------------|------|-----------|
| `check --scope user` | `pzsh lint` | bashrs invokes pzsh lint on zshrc |
| `check COMPLY-010` | `pzsh bench` | bashrs reads pzsh benchmark result |
| `track discover` | `pzsh status` | bashrs discovers pzsh-managed configs |
| `audit` | `pzsh profile` | bashrs includes pzsh profile in audit |

### 7.3 Graceful Degradation

When pzsh is not installed:
- COMPLY-010 is skipped (not counted in score)
- pzsh-specific config paths are still tracked if files exist
- No error, just an info message: "pzsh not found, skipping COMPLY-010"

---

## 8. Storage

### 8.1 Project State

```
.bashrs/
├── comply.toml           # Configuration (checked into git)
├── comply-state.json     # Last check result (checked into git)
└── audits/               # Audit artifacts (checked into git)
    ├── 2026-02-07.json
    └── 2026-02-14.json
```

### 8.2 User State

```
~/.config/bashrs/
├── comply-user.toml      # User scope config
└── comply-user-state.json # Last user check result
```

---

## 9. Falsification Checklist (Popper Tests)

These tests attempt to **disprove** that the specification is correct. Each test
must be automated.

| ID | Falsification Attempt | Expected Result |
|----|----------------------|-----------------|
| F-001 | Run comply check on empty project | Score 0, no crash |
| F-002 | Run comply check on project with no shell files | Score 100 (vacuously true) |
| F-003 | Run comply check with $RANDOM in script | COMPLY-002 fails |
| F-004 | Run comply check with `mkdir /foo` (no -p) | COMPLY-003 fails |
| F-005 | Run comply check with `eval "$USER_INPUT"` | COMPLY-004 fails |
| F-006 | Run comply check with unquoted `$VAR` | COMPLY-005 fails |
| F-007 | Run comply check when pzsh not installed | COMPLY-010 skipped, no error |
| F-008 | Run comply check when pzsh startup > 10ms | COMPLY-010 fails |
| F-009 | Run comply audit with dirty git state | Error: requires clean state |
| F-010 | Run comply audit, verify JSON schema | Valid schema |
| F-011 | Run comply track add on nonexistent file | Error with path |
| F-012 | Run comply check --scope system | Read-only audit, no modifications |
| F-013 | Run comply init twice | Idempotent (no duplicate config) |
| F-014 | Run comply enforce, commit non-compliant file | Commit blocked |
| F-015 | Run comply check on Makefile with shell injection | COMPLY-007 fails |
| F-016 | Run comply check on Dockerfile without USER | COMPLY-008 fails |
| F-017 | Run comply check on ~/.zshrc with PATH dupes | COMPLY-009 fails |
| F-018 | Run comply diff with no prior check | Graceful error message |
| F-019 | Run comply migrate --dry-run | No files modified |
| F-020 | Run comply report --format json | Valid JSON output |

---

## 10. Implementation Phases

### Phase 1: Foundation (v7.1.0)

- [ ] `bashrs comply init` — Create .bashrs/comply.toml
- [ ] `bashrs comply check` — Layer 1 (COMPLY-001 through COMPLY-006)
- [ ] `bashrs comply track` — Artifact discovery and management
- [ ] `bashrs comply status` — Alias for check
- [ ] Falsification tests F-001 through F-006

### Phase 2: Full Rules (v7.2.0)

- [ ] COMPLY-007 through COMPLY-009 (Makefile, Dockerfile, Config)
- [ ] `bashrs comply enforce` — Git hooks
- [ ] `bashrs comply diff` — Compliance delta
- [ ] `bashrs comply report` — Markdown/JSON reports
- [ ] Falsification tests F-007 through F-017
- [ ] pzsh peer discovery (without COMPLY-010)

### Phase 3: Governance (v7.3.0)

- [ ] `bashrs comply review` — Layer 2 (Genchi Genbutsu)
- [ ] `bashrs comply audit` — Layer 3 (signed artifacts)
- [ ] `bashrs comply migrate` — Version migration
- [ ] COMPLY-010 (pzsh integration)
- [ ] Falsification tests F-018 through F-020
- [ ] Progressive/degenerating trajectory analysis (Lakatos)

---

## 11. Relationship to Existing Commands

| Existing Command | Comply Equivalent | Relationship |
|-----------------|-------------------|--------------|
| `bashrs lint` | COMPLY-004, 005, 006 | Comply invokes lint internally |
| `bashrs purify` | Remediation for COMPLY-002, 003 | `comply --fix` calls purify |
| `bashrs gate` | COMPLY check tier 1 | Gate is subset of comply |
| `bashrs audit` | Single-file audit | Comply audits all artifacts |
| `bashrs config lint` | COMPLY-009 | Comply invokes config lint |
| `bashrs make lint` | COMPLY-007 | Comply invokes make lint |
| `bashrs dockerfile lint` | COMPLY-008 | Comply invokes dockerfile lint |
| `pmat comply` | Peer (Rust project) | bashrs comply = shell artifacts |

**Principle**: bashrs comply is an orchestrator. It does not reimplement linting,
purification, or analysis. It invokes existing bashrs commands and aggregates
results into a compliance assessment.

---

## 12. Non-Goals

1. **Replace pmat comply** — pmat handles Rust code; bashrs handles shell artifacts
2. **Modify system files** — System scope is read-only audit
3. **Replace shellcheck** — ShellCheck is invoked as a dependency, not replaced
4. **Enforce pzsh installation** — pzsh is optional; comply degrades gracefully
5. **Configuration management** — comply tracks compliance, not configuration state

---

## References

1. Popper, K. (1959). *The Logic of Scientific Discovery*. Routledge.
2. Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press.
3. Liker, J. (2004). *The Toyota Way: 14 Management Principles*. McGraw-Hill.
4. Deming, W.E. (1986). *Out of the Crisis*. MIT Press.
5. Wheeler, D. (2003). *Secure Programming for Linux and Unix HOWTO*.
6. Bernstein, D.J. (1997). *qmail security guarantee*.
7. Leveson, N. (2011). *Engineering a Safer World*. MIT Press.
8. Lakatos, I. (1978). *The Methodology of Scientific Research Programmes*. Cambridge University Press.
