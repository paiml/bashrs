# bashrs installer - TDD-First Installer Framework Specification

**Date**: 2025-12-26
**Version**: 1.1.0
**Paradigm**: Pure Rust Installer Generation with TDD by Default
**Integration**: trueno-viz for visualization, bashrs for transpilation

## Executive Summary

The `bashrs installer` command solves the pervasive problem of unreliable, untestable bash installers. Instead of writing fragile shell scripts that fail mysteriously, developers generate **pure Rust installers** that are:

1. **TDD by default** - Tests exist before implementation [1].
2. **Checkpointed** - Resume from any failure point.
3. **Observable** - Visual progress, structured logging, tracing [5].
4. **Deterministic** - Same inputs always produce same outputs [3].
5. **Falsifiable** - Every claim can be empirically tested [2].

**Philosophy**: Apply Toyota Production System (TPS) principles [4] and Karl Popper's falsificationism [2] to installer engineering.

---

## The Problem: Why Bash Installers Fail

### Current State (Broken)

Traditional shell scripts lack the structural guarantees required for reliable systems engineering. They often suffer from "Configuration Drift," where the actual state of the system diverges from the expected state over time, a phenomenon that makes deterministic restoration impossible [3].

```bash
#!/bin/bash
# install.sh - The typical disaster

apt-get update          # Fails silently on network issues
apt-get install -y foo  # Version drift, conflicts
curl ... | bash         # No verification, MITM attacks
mkdir -p /opt/app       # No idempotency check
cp -r . /opt/app        # No rollback on failure
systemctl enable foo    # No status verification
echo "Done!"            # Lies - no actual verification
```

**Failure Modes**:
- **Lack of Atomicity**: Scripts fail mid-way, leaving the system in an inconsistent, broken state.
- **Observability Deficit**: Silent failures are buried in unstructured text output [5].
- **Testing Gap**: Impossible to unit test individual steps in isolation.
- **Rollback Absence**: No mechanism to revert changes upon failure.

### Toyota Way Analysis (7 Wastes in Installers)

Applying Liker's analysis of waste (*muda*) in the Toyota Production System [4] to software installation:

| Waste Type | Installer Manifestation |
|------------|------------------------|
| **Defects** | Script fails mid-way, leaves system in broken state (Quality Debt). |
| **Overproduction** | Re-downloading already-installed packages (Inefficiency). |
| **Waiting** | No parallelization of independent steps (Resource Underutilization). |
| **Non-utilized talent** | Developers debugging broken scripts instead of building features. |
| **Transportation** | Unnecessary file copies, temp directories, and data movement. |
| **Inventory** | Orphaned packages, leftover artifacts, and temp files. |
| **Motion** | Manual intervention, SSH-ing to servers to "fix" failed installs. |
| **Extra-processing** | Redundant checks, manual verifications, and unnecessary operations. |

---

## Solution: `bashrs installer` Command

### Command Overview

```bash
# Generate a new installer project
bashrs installer init my-app-installer

# Scaffold from existing bash script
bashrs installer from-bash install.sh --output my-installer/

# Run installer with full observability
bashrs installer run ./my-installer \
  --checkpoint-dir /var/lib/installer/checkpoints \
  --log-level debug \
  --trace \
  --progress

# Resume from checkpoint
bashrs installer resume ./my-installer --from step-5

# Validate installer without executing
bashrs installer validate ./my-installer

# Generate test suite
bashrs installer test ./my-installer --coverage
```

---

## Architecture: Pure Rust Installer Pipeline

The architecture prioritizes **testability** and **observability**, core tenets of Continuous Delivery [6].

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        bashrs installer Pipeline                             │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌──────────────────────────────────────┐
                    │  DESIGN PHASE (Human + AI)           │
                    │  • Define installation steps         │
                    │  • Declare preconditions/postconds   │
                    │  • Write falsification tests FIRST   │
                    └──────────────────────────────────────┘
                                     │
                                     ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Phase 1: PARSE/GENERATE                                                      │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │ installer.toml  │───▶│ Rust AST        │───▶│ InstallerPlan   │           │
│  │ (declarative)   │    │ Generation      │    │ (validated)     │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
└──────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Phase 2: TEST GENERATION (TDD - Tests First) [1]                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │ Precondition    │    │ Postcondition   │    │ Invariant       │           │
│  │ Tests           │    │ Tests           │    │ Tests           │           │
│  │ (falsifiable)   │    │ (falsifiable)   │    │ (falsifiable)   │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
└──────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Phase 3: EXECUTION with OBSERVABILITY [5]                                    │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │ trueno-viz      │    │ Structured      │    │ OpenTelemetry   │           │
│  │ Progress Bars   │    │ Logging         │    │ Tracing         │           │
│  │ (terminal/GUI)  │    │ (JSON/human)    │    │ (spans/events)  │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
└──────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Phase 4: CHECKPOINT & RECOVERY                                               │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │ Step State      │    │ Rollback        │    │ Resume          │           │
│  │ Persistence     │    │ Actions         │    │ Capability      │           │
│  │ (SQLite/JSON)   │    │ (per-step)      │    │ (idempotent)    │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Installer Definition Format

### installer.toml

```toml
[installer]
name = "docker-ce"
version = "1.0.0"
description = "Install Docker CE on Ubuntu/Debian"
author = "Platform Team"

[installer.requirements]
os = ["ubuntu >= 20.04", "debian >= 11"]
arch = ["x86_64", "aarch64"]
privileges = "root"
network = true

[installer.environment]
DOCKER_VERSION = { default = "latest", validate = "semver|latest" }
DOCKER_USER = { from_env = "SUDO_USER", required = true }

# =============================================================================
# Steps: Each step is atomic, idempotent, and testable [3]
# =============================================================================

[[step]]
id = "check-os"
name = "Verify Operating System"
action = "verify"

[step.preconditions]
file_exists = "/etc/os-release"

[step.postconditions]
env_matches = { ID = "ubuntu|debian" }

[step.on_failure]
action = "abort"
message = "Unsupported operating system"

# -----------------------------------------------------------------------------

[[step]]
id = "remove-old-docker"
name = "Remove Old Docker Packages"
action = "apt-remove"
packages = ["docker", "docker-engine", "docker.io", "containerd", "runc"]
depends_on = ["check-os"]

[step.preconditions]
command_succeeds = "dpkg --version"

[step.postconditions]
packages_absent = ["docker", "docker-engine", "docker.io"]

[step.checkpoint]
enabled = true
rollback = "apt-get install -y docker.io"  # Restore if needed

# -----------------------------------------------------------------------------

[[step]]
id = "install-prerequisites"
name = "Install Prerequisites"
action = "apt-install"
packages = ["ca-certificates", "curl", "gnupg", "lsb-release"]
depends_on = ["remove-old-docker"]

[step.timing]
timeout = "5m"
retry = { count = 3, delay = "10s", backoff = "exponential" }

[step.progress]
type = "determinate"
source = "apt-progress"

# -----------------------------------------------------------------------------

[[step]]
id = "setup-docker-repo"
name = "Configure Docker Repository"
action = "script"
depends_on = ["install-prerequisites"]

[step.script]
interpreter = "bash"
content = """
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/${ID}/gpg | \
  gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg
"""

[step.postconditions]
file_exists = "/etc/apt/keyrings/docker.gpg"
file_mode = "/etc/apt/keyrings/docker.gpg:644"

[step.checkpoint]
enabled = true
state_files = ["/etc/apt/keyrings/docker.gpg"]
rollback = "rm -f /etc/apt/keyrings/docker.gpg"

# -----------------------------------------------------------------------------

[[step]]
id = "install-docker"
name = "Install Docker Packages"
action = "apt-install"
packages = ["docker-ce", "docker-ce-cli", "containerd.io",
            "docker-buildx-plugin", "docker-compose-plugin"]
depends_on = ["setup-docker-repo"]

[step.timing]
timeout = "10m"

[step.progress]
type = "determinate"
source = "apt-progress"

[step.postconditions]
command_succeeds = "docker --version"
service_active = "docker"

# -----------------------------------------------------------------------------

[[step]]
id = "configure-user"
name = "Add User to Docker Group"
action = "user-group"
user = "${DOCKER_USER}"
group = "docker"
depends_on = ["install-docker"]

[step.postconditions]
user_in_group = { user = "${DOCKER_USER}", group = "docker" }

# -----------------------------------------------------------------------------

[[step]]
id = "verify-installation"
name = "Verify Docker Installation"
action = "verify"
depends_on = ["configure-user"]

[step.verification]
commands = [
  { cmd = "docker version", expect = "Server:" },
  { cmd = "docker info", expect = "Storage Driver:" },
]

[step.postconditions]
command_succeeds = "docker run --rm hello-world"
```

---

## trueno-viz Integration: Visual Progress

### Terminal Progress Bars

```rust
use trueno_viz::{ProgressBar, MultiProgress, Style};
use bashrs_installer::{Step, StepState};

pub struct InstallerVisualizer {
    multi: MultiProgress,
    step_bars: HashMap<StepId, ProgressBar>,
}

impl InstallerVisualizer {
    /// Render installer progress to terminal using trueno-viz
    pub fn render_step(&mut self, step: &Step, state: &StepState) {
        let bar = self.step_bars.get_mut(&step.id).unwrap();

        match state {
            StepState::Pending => {
                bar.set_style(Style::dimmed());
                bar.set_message(format!("⏳ {}", step.name));
            }
            StepState::Running { progress, message } => {
                bar.set_style(Style::spinner_blue());
                bar.set_progress(*progress);
                bar.set_message(format!("▶ {} - {}", step.name, message));
            }
            StepState::Completed { duration } => {
                bar.set_style(Style::success_green());
                bar.finish_with_message(format!(
                    "✓ {} ({:.2}s)", step.name, duration.as_secs_f64()
                ));
            }
            StepState::Failed { error, .. } => {
                bar.set_style(Style::error_red());
                bar.abandon_with_message(format!("✗ {} - {}", step.name, error));
            }
            StepState::Skipped { reason } => {
                bar.set_style(Style::warning_yellow());
                bar.finish_with_message(format!("⊘ {} ({})", step.name, reason));
            }
        }
    }
}
```

### Visual Output Example

```
Docker CE Installer v1.0.0
══════════════════════════════════════════════════════════════════════════════

  Step 1/7: Verify Operating System
  ✓ check-os ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% (0.12s)

  Step 2/7: Remove Old Docker Packages
  ✓ remove-old-docker ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% (2.34s)

  Step 3/7: Install Prerequisites
  ✓ install-prerequisites ━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% (8.45s)

  Step 4/7: Configure Docker Repository
  ▶ setup-docker-repo ━━━━━━━━━━━━━━━━━━━╸━━━━━━━━━━━  65% Downloading GPG key...

  Step 5/7: Install Docker Packages
  ⏳ install-docker ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   0% Pending

  Step 6/7: Add User to Docker Group
  ⏳ configure-user ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   0% Pending

  Step 7/7: Verify Docker Installation
  ⏳ verify-installation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━   0% Pending

──────────────────────────────────────────────────────────────────────────────
  Elapsed: 11.2s │ Remaining: ~45s │ Checkpoint: step-3 │ Logs: /var/log/installer
══════════════════════════════════════════════════════════════════════════════
```

---

## Logging System: Structured & Leveled

### Log Levels

| Level | Purpose | Default Output |
|-------|---------|----------------|
| `error` | Failures requiring attention | stderr, always |
| `warn` | Potential issues, non-fatal | stderr |
| `info` | Progress updates, milestones | stdout |
| `debug` | Detailed execution flow | file only |
| `trace` | Fine-grained diagnostics | file only |

### Structured Log Format (JSON)

```json
{
  "timestamp": "2025-12-26T10:15:30.123456Z",
  "level": "info",
  "target": "bashrs_installer::step::apt_install",
  "span": {
    "installer": "docker-ce",
    "step_id": "install-docker",
    "step_name": "Install Docker Packages"
  },
  "fields": {
    "message": "Package installation complete",
    "packages": ["docker-ce", "docker-ce-cli", "containerd.io"],
    "duration_ms": 45230,
    "bytes_downloaded": 125829120
  }
}
```

---

## Timing, Tracing & Debugging

### OpenTelemetry Integration

```rust
use tracing::{instrument, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetryLayer;

#[instrument(skip(ctx), fields(step.id = %step.id, step.name = %step.name))]
async fn execute_step(ctx: &InstallerContext, step: &Step) -> Result<StepResult> {
    let _enter = info_span!("step_execution",
        step.timeout = ?step.timing.timeout,
        step.retry_count = step.timing.retry.count,
    ).entered();

    // Precondition check span
    let precond_result = async {
        check_preconditions(&step.preconditions).await
    }
    .instrument(info_span!("preconditions")),
    .await?;

    // Main action span
    let action_result = async {
        execute_action(&step.action, ctx).await
    }
    .instrument(info_span!("action", action.type = %step.action.type_name())),
    .await?;

    // Postcondition verification span
    async {
        verify_postconditions(&step.postconditions).await
    }
    .instrument(info_span!("postconditions")),
    .await
}
```

---

## Checkpoint System: Resume from Any Point

### Checkpoint Storage (SQLite)

```sql
CREATE TABLE installer_runs (
    run_id TEXT PRIMARY KEY,
    installer_name TEXT NOT NULL,
    installer_version TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    status TEXT CHECK(status IN ('running', 'completed', 'failed', 'aborted')),
    environment JSON NOT NULL
);

CREATE TABLE step_checkpoints (
    run_id TEXT REFERENCES installer_runs(run_id),
    step_id TEXT NOT NULL,
    status TEXT CHECK(status IN ('pending', 'running', 'completed', 'failed', 'skipped')),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    state_snapshot JSON,  -- Captured state for rollback
    output_log TEXT,
    error_message TEXT,
    PRIMARY KEY (run_id, step_id)
);

CREATE TABLE state_files (
    run_id TEXT REFERENCES installer_runs(run_id),
    step_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    backed_up_at TIMESTAMP,
    backup_path TEXT,
    PRIMARY KEY (run_id, step_id, file_path)
);
```

### Resume Flow

```rust
pub async fn resume_installer(
    checkpoint_dir: &Path,
    from_step: Option<&str>,
) -> Result<InstallerResult> {
    let checkpoint = Checkpoint::load(checkpoint_dir)?;

    // Find resume point
    let resume_from = match from_step {
        Some(step_id) => checkpoint.find_step(step_id)?,
        None => checkpoint.last_successful_step()?,
    };

    info!("Resuming from step: {}", resume_from.id);

    // Restore state from checkpoint
    for state_file in &resume_from.state_files {
        restore_state_file(state_file)?;
    }

    // Continue execution
    execute_from_step(&checkpoint.plan, &resume_from.id).await
}
```

---

## Toyota Way Principles Applied

### 1. Jidoka (Automation with Human Touch)

**Principle**: Stop and fix problems immediately; don't propagate defects [4].

```toml
[[step]]
id = "install-package"

[step.on_failure]
action = "stop"  # Jidoka: Stop the line
notify = ["ops@company.com"]
preserve_state = true  # For debugging

# Human intervention required before proceeding
[step.recovery]
require_approval = true
approval_timeout = "1h"
```

### 2. Kaizen (Continuous Improvement)

**Principle**: Collect metrics; improve based on data [4].

```rust
pub struct InstallerMetrics {
    /// Track timing trends across runs
    pub step_durations: HashMap<StepId, Vec<Duration>>,

    /// Track failure patterns
    pub failure_counts: HashMap<StepId, u32>,

    /// Track retry effectiveness
    pub retry_success_rate: HashMap<StepId, f64>,
}
```

### 3. Heijunka (Level Loading)

**Principle**: Parallelize independent operations; avoid resource contention [4].

```toml
[[step]]
id = "download-artifacts"
parallel_group = "downloads"  # Run in parallel with other downloads

[[step]]
id = "download-keys"
parallel_group = "downloads"  # Same group = parallel execution
```

### 4. Genchi Genbutsu (Go and See)

**Principle**: Real-time visibility into actual system state [4].

```bash
# Real-time monitoring
bashrs installer run ./my-installer --live-dashboard
```

### 5. Poka-Yoke (Error Prevention)

**Principle**: Design out the possibility of errors [4].

```rust
/// Poka-Yoke: Type-safe step definitions prevent common errors
pub struct Step<S: StepState> {
    id: StepId,  // Compile-time unique ID enforcement
    preconditions: Vec<Precondition>,  // Must be satisfied before execution
    action: Action,
    postconditions: Vec<Postcondition>,  // Must be true after execution
    _state: PhantomData<S>,
}
```

---

## Karl Popper Falsification Checklist

### Principle: A Claim is Only Scientific if it Can Be Proven False

According to Popper [2], a theory (or installer step) is only scientific if it makes specific predictions that can be tested and potentially falsified.

### Falsification Test Matrix

| Claim | Test Method | How to Disprove |
|-------|-------------------|-----------------|
| "Step is idempotent" | Run step twice, compare system state | Different state after 2nd run = FALSIFIED [3] |
| "Step has no side effects on failure" | Kill step mid-execution, check state | Partial state changes = FALSIFIED |
| "Rollback restores original state" | Run step, rollback, compare to pre-state | Any difference = FALSIFIED |
| "Timeout is honored" | Set timeout=1s, run 10s operation | Runs longer than timeout = FALSIFIED |
| "Retry logic works" | Inject transient failure, verify retry | No retry or wrong behavior = FALSIFIED |

### Falsification Tests in Code

```rust
#[cfg(test)]
mod falsification_tests {
    use super::*;
    use proptest::prelude::*;

    /// FALSIFIABLE: "Every step is idempotent"
    /// DISPROOF: Run step twice, system state differs
    #[test]
    fn falsify_step_idempotency() {
        let step = load_step("install-docker");
        let ctx = TestContext::new();

        // First execution
        let state_after_first = execute_and_capture_state(&ctx, &step);

        // Second execution (should be no-op)
        let state_after_second = execute_and_capture_state(&ctx, &step);

        // Falsification: If states differ, idempotency claim is FALSE
        assert_eq!(
            state_after_first, state_after_second,
            "FALSIFIED: Step '{}' is not idempotent. State changed on re-execution.",
            step.id
        );
    }

    /// FALSIFIABLE: "Rollback restores original state"
    /// DISPROOF: State after rollback differs from state before step
    #[test]
    fn falsify_rollback_completeness() {
        let step = load_step("install-docker");
        let ctx = TestContext::new();

        // Capture state before
        let state_before = capture_system_state(&ctx);

        // Execute step
        execute_step(&ctx, &step).unwrap();

        // Rollback
        rollback_step(&ctx, &step).unwrap();

        // Capture state after rollback
        let state_after_rollback = capture_system_state(&ctx);

        // Falsification: If states differ, rollback claim is FALSE
        let diff = state_before.diff(&state_after_rollback);
        assert!(
            diff.is_empty(),
            "FALSIFIED: Rollback incomplete. Residual changes: {:?}",
            diff
        );
    }
}
```

---

## Pure Rust Implementation

### Cargo.toml

```toml
[package]
name = "bashrs-installer"
version = "0.1.0"
edition = "2024"

[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"

# Visualization (trueno-viz)
trueno-viz = { git = "https://github.com/paiml/trueno-viz.git" }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.22"
opentelemetry = { version = "0.21", features = ["trace"] }
opentelemetry-jaeger = "0.20"

# Checkpoint storage
rusqlite = { version = "0.30", features = ["bundled"] }

# Testing
proptest = "1"
quickcheck = "1"
cargo-mutants = "0.0"  # Mutation testing

[dev-dependencies]
insta = "1"  # Snapshot testing
assert_cmd = "2"  # CLI testing
predicates = "3"
```

---

## Success Metrics

### Quality Gates

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | >95% | cargo llvm-cov |
| Mutation Score | >90% | cargo mutants |
| Falsification Tests | 100% claims tested | Custom harness |
| Step Idempotency | 100% | Property tests |

---

## References

1. Beck, K. (2002). *Test Driven Development: By Example*. Addison-Wesley Professional.
2. Popper, K. (1959). *The Logic of Scientific Discovery*. Hutchinson & Co.
3. Burgess, M. (2004). *A Treatise on System Administration*. In *LISA* (pp. 77-94). USENIX Association.
4. Liker, J. K. (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill.
5. Beyer, B., Jones, C., Petoff, J., & Murphy, N. R. (2016). *Site Reliability Engineering: How Google Runs Production Systems*. O'Reilly Media.
6. Humble, J., & Farley, D. (2010). *Continuous Delivery: Reliable Software Releases through Build, Test, and Deployment Automation*. Addison-Wesley Professional.
7. IEEE Standard 829-2008. *IEEE Standard for Software and System Test Documentation*. IEEE Standards Association.

**Tool References:**
- [trueno-viz](https://github.com/paiml/trueno-viz) - Rust visualization library
- [bashrs PURIFY-SPECIFICATION](../PURIFY-SPECIFICATION.md) - Transpiler design
- [OpenTelemetry](https://opentelemetry.io/) - Observability framework