# Unix Runtime Improvements Specification

## Document Metadata

| Field | Value |
|-------|-------|
| Version | 1.0.0 |
| Status | Draft |
| Created | 2026-01-06 |
| Author | Claude Code |
| Stakeholders | duende, trueno-zram, pepita, bashrs |

---

## 1. Executive Summary

This specification defines Unix runtime improvements for bashrs to support the PAIML Sovereign AI Stack, with specific focus on Docker containerization, macOS compatibility, Bash/Zsh shell support, and daemon lifecycle management. Requirements are derived from three dependent projects: **duende** (daemon orchestration), **trueno-zram** (kernel-level memory compression), and **pepita** (distributed computing primitives).

### Toyota Way Principles Applied

> "The right process will produce the right results." — Taiichi Ohno

This specification follows Toyota Production System principles:
- **Jidoka** (自働化): Stop-the-line quality enforcement
- **Genchi Genbutsu** (現地現物): Go and see for yourself (derived from actual project analysis)
- **Kaizen** (改善): Continuous improvement through falsification testing
- **Poka-yoke** (ポカヨケ): Mistake-proofing through type safety

---

## 2. Stakeholder Requirements

### 2.1 Duende (Daemon Orchestration Framework)

**Project**: Cross-platform daemon lifecycle management for Sovereign AI Stack

#### Runtime Requirements

| Category | Requirement | Priority |
|----------|-------------|----------|
| Process Management | Fork/exec via `/bin/sh` | P0 |
| Signal Handling | SIGHUP, SIGTERM, SIGKILL, signal(0) | P0 |
| Memory Locking | `mlock()`/`mlockall()` for swap deadlock prevention | P0 |
| systemd Integration | Unit file generation and validation | P1 |
| launchd Integration | plist generation for macOS | P1 |
| Docker/OCI | Container runtime signal forwarding | P1 |
| Capability Detection | CAP_IPC_LOCK, RLIMIT_MEMLOCK | P0 |

#### Current Integration Points

```makefile
# From duende/Makefile (lines 138-163)
bashrs-lint:
    bashrs dockerfile lint docker/Dockerfile.*

bashrs-gate:
    # Enforces shell-free Docker images
    @test -z "$$(find docker -name '*.sh' 2>/dev/null)"
```

#### Shell-Free Philosophy

Duende enforces **zero shell scripts in production**:
> "Pure Rust test runner - no bash scripts (bashrs compliant)"

bashrs must validate that:
1. Dockerfiles contain no `/bin/sh` invocations in final image
2. No `.sh` files exist in `docker/` directories
3. Generated unit files are POSIX-compliant

### 2.2 trueno-zram (Kernel Memory Compression)

**Project**: GPU-accelerated userspace ZRAM replacement

#### Shell Script Requirements

| Script | Lines | Purpose | bashrs Needs |
|--------|-------|---------|--------------|
| `test-swap-deadlock.sh` | 254 | DT-007 swap deadlock detection | procfs parsing |
| `docker-test-harness.sh` | 690 | Test orchestration | Privileged Docker |
| `falsification-runner.sh` | 476 | 100-point falsification matrix | JSON reporting |

#### Kernel Operations Requiring Shell

```bash
# Module management
modprobe ublk_drv
lsmod | grep ublk_drv

# Swap management
mkswap /dev/ublkbN
swapon -p 150 /dev/ublkbN
swapoff /dev/ublkbN

# Device operations
blkdiscard /dev/ublkbN
stat -c "%a" /dev/ublk-control

# Filesystem operations
mkfs.ext4 -F /dev/ublkbN
mkfs.btrfs -f /dev/ublkbN
mount /dev/ublkbN /mnt/test
```

#### Critical Path: DT-007 Swap Deadlock Detection

```bash
# From test-swap-deadlock.sh - process state inspection
state=$(cat "/proc/$pid/stat" | awk '{print $3}')
if [ "$state" = "D" ]; then
    # state:D = uninterruptible sleep = deadlock risk
    echo "DEADLOCK DETECTED"
fi
```

### 2.3 pepita (Distributed Computing Primitives)

**Project**: Minimal kernel interfaces for Sovereign AI workloads

#### Runtime Requirements

| Component | Requirement | Shell Impact |
|-----------|-------------|--------------|
| Binary Execution | `std::process::Command` | None (pure Rust) |
| Task Scheduling | Multi-threaded work-stealing | None |
| KVM Virtualization | ioctls via nix crate | None |
| SIMD Detection | Runtime CPU feature detection | None |

**Key Finding**: pepita has **zero shell dependencies** by design:
- First-Principles Rust architecture
- 100% auditable code path
- No external executables required

#### Integration Opportunity

pepita's `pool` module could benefit from bashrs-generated init scripts:

```rust
// pepita/src/pool.rs - potential bashrs integration
pub struct TaskPool {
    scheduler: Scheduler,
    executor: Executor,
}

// Generated init script validation
// bashrs validate --pool-config pepita.toml
```

---

## 3. Open GitHub Issues

### 3.1 Parser Issues (P0 - Blocking)

| Issue | Title | Impact |
|-------|-------|--------|
| #93 | Parser fails on inline if/then/else/fi | Blocks script purification |
| #103 | Parser fails on common bash array syntax | Blocks array-heavy scripts |

### 3.2 False Positive Issues (P1 - Quality)

| Issue | Title | Rule | Root Cause |
|-------|-------|------|------------|
| #121 | MAKE008 triggers on .PHONY continuation | MAKE008 | Line continuation parsing |
| #120 | SC2247 triggers on Python in heredoc | SC2247 | Heredoc language detection |
| #119 | Multi-line .PHONY not recognized | MAKE004 | Multi-line parsing |
| #118 | False positive for quoted variables | MAKE003 | Quote context tracking |
| #117 | SC2032 false positive on standalone scripts | SC2032 | Script type detection |
| #116 | DET002 false positive for timing scripts | DET002 | Timestamp context |
| #102 | SC2128/SC2199 false positive on scalars | SC2128 | Variable type tracking |
| #101 | SC2024 false positive for sudo sh -c | SC2024 | Subshell detection |
| #100 | SC2024 warns on correct tee pattern | SC2024 | Pattern recognition |
| #99 | SC2154 false positive for case variables | SC2154 | Control flow analysis |
| #98 | SC2154 false positive for EUID builtin | SC2154 | Builtin recognition |
| #97 | SEC010 false positive after validation | SEC010 | Data flow analysis |
| #96 | False positives in quoted heredocs | Multiple | Heredoc parsing |
| #95 | SC2154/SC2140 for sourced variables | SC2154 | Source tracking |
| #94 | exec() generates shell exec | Transpiler | Semantic translation |

### 3.3 Enhancement Requests

| Issue | Title | Category |
|-------|-------|----------|
| #115 | ZRAM-backed command cache | Feature |

---

## 4. Technical Requirements

### 4.1 Docker Support

#### 4.1.1 Dockerfile Linting

```bash
# Required validation rules
bashrs dockerfile lint Dockerfile \
  --rule NO_SHELL_ENTRYPOINT \
  --rule MINIMIZE_LAYERS \
  --rule NO_ROOT_USER \
  --rule HEALTHCHECK_PRESENT
```

#### 4.1.2 Multi-stage Build Validation

```dockerfile
# Pattern to validate
FROM rust:1.82 AS builder
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/daemon /
# bashrs must verify: no /bin/sh in final image
```

#### 4.1.3 Privileged Container Testing

trueno-zram requires privileged Docker for ublk testing:

```bash
docker run --privileged \
  -v /lib/modules:/lib/modules:ro \
  -v /dev:/dev \
  --tmpfs /mnt/test:size=4G \
  trueno-zram-test
```

bashrs validation:
- Detect privileged mode usage
- Warn about device mounts
- Validate capability requirements

### 4.2 macOS Support

#### 4.2.1 launchd Integration (duende DP-004)

```xml
<!-- Generated plist must pass bashrs validation -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.paiml.duende</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/duende</string>
    <string>--config</string>
    <string>/etc/duende/config.toml</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
```

#### 4.2.2 mlock() on macOS

macOS requires entitlements for mlock:

```bash
# Entitlement check (bashrs should validate)
codesign -d --entitlements :- /path/to/daemon 2>&1 | \
  grep com.apple.security.cs.allow-mlock
```

#### 4.2.3 Homebrew Integration

```bash
# Formula installation script validation
bashrs lint Formula/duende.rb --shell-fragments
```

### 4.3 Bash/Zsh Shell Support

#### 4.3.1 Shebang Detection

| Shebang | Shell | Feature Set |
|---------|-------|-------------|
| `#!/bin/bash` | Bash | Full bash features |
| `#!/usr/bin/env bash` | Bash | Portable bash |
| `#!/bin/zsh` | Zsh | Zsh extensions |
| `#!/usr/bin/env zsh` | Zsh | Portable zsh |
| `#!/bin/sh` | POSIX | Strict POSIX only |
| `#!/bin/dash` | Dash | POSIX + minimal extensions |

#### 4.3.2 Bash Builtins Recognition

SC2154 must recognize bash builtins (Issue #98):

```bash
BASH_BUILTINS = [
  "EUID", "UID", "BASH_VERSION", "BASH_VERSINFO",
  "HOSTNAME", "HOSTTYPE", "OSTYPE", "MACHTYPE",
  "RANDOM", "SECONDS", "LINENO", "FUNCNAME",
  "BASH_SOURCE", "BASH_LINENO", "PIPESTATUS", "GROUPS",
  "PWD", "OLDPWD", "HOME", "PATH", "IFS",
  "REPLY", "COMP_WORDS", "COMP_CWORD", "COMP_LINE"
]
```

#### 4.3.3 Zsh-Specific Features

```zsh
# Zsh patterns bashrs should recognize
typeset -A assoc_array           # Associative array declaration
setopt NULL_GLOB                 # Glob options
print -P "%~"                    # Prompt expansion
autoload -Uz compinit            # Completion system
```

#### 4.3.4 Array Syntax (Issue #103)

```bash
# Patterns requiring parser support
local arr=()                     # Empty array
arr+=("item")                    # Array append
${arr[@]}                        # Array expansion
${#arr[@]}                       # Array length
```

### 4.4 Daemon Lifecycle Management

#### 4.4.1 systemd Unit Generation (duende DP-002)

```ini
# Generated unit file template
[Unit]
Description=PAIML Daemon Service
After=network.target

[Service]
Type=notify
ExecStart=/usr/bin/daemon --config /etc/daemon/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5
LimitMEMLOCK=infinity

[Install]
WantedBy=multi-user.target
```

bashrs validation rules:
- `SYSTEMD001`: Type must match daemon behavior
- `SYSTEMD002`: ExecStart must be absolute path
- `SYSTEMD003`: Restart policy appropriate for service type
- `SYSTEMD004`: Resource limits specified

#### 4.4.2 Signal Handling Validation

```bash
# Signal handler patterns to validate
trap 'cleanup' EXIT
trap 'reload_config' HUP
trap 'graceful_shutdown' TERM INT
trap '' PIPE                     # Ignore SIGPIPE
```

#### 4.4.3 PID File Management

```bash
# Patterns requiring validation
PIDFILE="/var/run/daemon.pid"
echo $$ > "$PIDFILE"             # Write PID
kill -0 "$(cat "$PIDFILE")"      # Check if running
rm -f "$PIDFILE"                 # Cleanup
```

---

## 5. Peer-Reviewed Citations

### 5.1 Toyota Production System

1. Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. ISBN: 978-0915299140
   - Foundation for Jidoka (autonomation) and just-in-time principles

2. Liker, J. K. (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill. ISBN: 978-0071392310
   - Principle 5: "Build a culture of stopping to fix problems, to get quality right the first time"

3. Shingo, S. (1986). *Zero Quality Control: Source Inspection and the Poka-Yoke System*. Productivity Press. ISBN: 978-0915299072
   - Mistake-proofing methodology applied to shell script validation

### 5.2 Shell Script Security

4. Wheeler, D. A. (2015). "Secure Programming HOWTO - Creating Secure Software." *Linux Documentation Project*.
   - Section 5.4: Shell script security considerations
   - URL: https://dwheeler.com/secure-programs/

5. OWASP Foundation. (2023). "OS Command Injection." *OWASP Testing Guide v4.2*.
   - Command injection prevention patterns
   - URL: https://owasp.org/www-community/attacks/Command_Injection

6. Viega, J., & McGraw, G. (2001). *Building Secure Software: How to Avoid Security Problems the Right Way*. Addison-Wesley. ISBN: 978-0201721522
   - Chapter 12: Input validation for shell commands

### 5.3 Software Testing & Falsification

7. Popper, K. (1959). *The Logic of Scientific Discovery*. Routledge. ISBN: 978-0415278447
   - Foundation for falsificationist testing methodology
   - "A theory which is not refutable by any conceivable event is non-scientific"

8. Hamlet, R. (1994). "Random testing." In *Encyclopedia of Software Engineering*. Wiley.
   - DOI: 10.1002/0471028959.sof268
   - Property-based testing foundations

9. Jia, Y., & Harman, M. (2011). "An Analysis and Survey of the Development of Mutation Testing." *IEEE Transactions on Software Engineering*, 37(5), 649-678.
   - DOI: 10.1109/TSE.2010.62
   - Mutation testing methodology for shell script validators

### 5.4 Container Security

10. Sultan, S., Ahmad, I., & Dimitriou, T. (2019). "Container Security: Issues, Challenges, and the Road Ahead." *IEEE Access*, 7, 52976-52996.
    - DOI: 10.1109/ACCESS.2019.2911732
    - Container isolation and privilege escalation risks

11. NIST. (2017). "Application Container Security Guide." *NIST Special Publication 800-190*.
    - DOI: 10.6028/NIST.SP.800-190
    - Container image security best practices

### 5.5 Unix Systems Programming

12. Stevens, W. R., & Rago, S. A. (2013). *Advanced Programming in the UNIX Environment* (3rd ed.). Addison-Wesley. ISBN: 978-0321637734
    - Chapters 9-10: Process relationships and signals
    - Chapter 14: Advanced I/O (async, memory-mapped)

13. Kerrisk, M. (2010). *The Linux Programming Interface*. No Starch Press. ISBN: 978-1593272203
    - Chapters 20-22: Signal handling
    - Chapter 37: Daemons

### 5.6 Memory Management

14. Gorman, M. (2004). *Understanding the Linux Virtual Memory Manager*. Prentice Hall. ISBN: 978-0131453487
    - Chapter 13: Memory locking (mlock/mlockall)
    - Swap deadlock scenarios

15. Love, R. (2010). *Linux Kernel Development* (3rd ed.). Addison-Wesley. ISBN: 978-0672329463
    - Chapter 15: Memory management
    - Chapter 4: Process scheduling

---

## 6. Popperian Falsification Checklist

> "The criterion of the scientific status of a theory is its falsifiability."
> — Karl Popper, *Conjectures and Refutations* (1963)

### Methodology

Each test case is designed to **falsify** a claim about bashrs behavior. A passing test **fails to falsify** the hypothesis, providing provisional confidence. A failing test **successfully falsifies** the hypothesis, requiring immediate remediation.

### 6.1 Parser Correctness (F001-F020)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F001 | Parser handles inline if/then/else/fi | `if cmd; then x; else y; fi` parses without error | PENDING |
| F002 | Parser handles empty array initialization | `local arr=()` parses without error | PENDING |
| F003 | Parser handles array append operator | `arr+=("item")` parses without error | PENDING |
| F004 | Parser handles stderr redirect shorthand | `cmd >&2` parses without error | PENDING |
| F005 | Parser handles combined redirect | `cmd &>/dev/null` parses without error | PENDING |
| F006 | Parser handles heredoc with quoted delimiter | `cat << 'EOF'` content not shell-parsed | PENDING |
| F007 | Parser handles line continuation in targets | `.PHONY: a \\<newline> b` parsed correctly | PENDING |
| F008 | Parser handles case statement variable assignment | Variables assigned in all branches recognized | PENDING |
| F009 | Parser handles nested command substitution | `$(cmd1 $(cmd2))` parsed correctly | PENDING |
| F010 | Parser handles process substitution | `diff <(cmd1) <(cmd2)` parsed correctly | PENDING |
| F011 | Parser handles brace expansion | `{a,b,c}` vs `${var:-default}` distinguished | PENDING |
| F012 | Parser handles arithmetic expansion | `$((x + y))` parsed correctly | PENDING |
| F013 | Parser handles parameter expansion modifiers | `${var:+set}` `${var:?error}` parsed | PENDING |
| F014 | Parser handles here-string | `cmd <<< "string"` parsed correctly | PENDING |
| F015 | Parser handles coprocess | `coproc cmd` parsed correctly | PENDING |
| F016 | Parser handles function with keyword | `function name { }` vs `name() { }` | PENDING |
| F017 | Parser handles select statement | `select x in a b c; do cmd; done` | PENDING |
| F018 | Parser handles extglob patterns | `@(a|b)` `+(x)` `!(y)` in case statements | PENDING |
| F019 | Parser handles associative arrays | `declare -A hash; hash[key]=val` | PENDING |
| F020 | Parser handles mapfile/readarray | `mapfile -t arr < file` | PENDING |

### 6.2 Linter Accuracy (F021-F040)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F021 | SC2154 recognizes bash builtins | `$EUID` does not trigger SC2154 | PENDING |
| F022 | SC2154 tracks sourced variables | Variables from `source file` recognized | PENDING |
| F023 | SC2154 handles case exhaustive assignment | All-branch assignment recognized | PENDING |
| F024 | SC2024 recognizes sudo sh -c pattern | `sudo sh -c 'cmd > file'` no warning | PENDING |
| F025 | SC2024 recognizes tee pattern | `cmd \| sudo tee file` no warning | PENDING |
| F026 | SC2031 distinguishes subshells | `$(cmd)` assignment not flagged | PENDING |
| F027 | SC2032 detects script type | Executable scripts not flagged | PENDING |
| F028 | SC2035 recognizes find -name | `find -name '*.txt'` not flagged | PENDING |
| F029 | SC2062 recognizes quoted patterns | Quoted grep patterns not flagged | PENDING |
| F030 | SC2125 distinguishes expansion types | `${var:-}` vs `{a,b}` | PENDING |
| F031 | SC2128 tracks variable types | Scalar vs array correctly identified | PENDING |
| F032 | SC2140 handles quote nesting | `'json' > "$path"` not flagged | PENDING |
| F033 | SC2247 respects heredoc boundaries | Python in heredoc not shell-parsed | PENDING |
| F034 | SC2317 understands short-circuit | `cmd \|\| exit; next` reachable | PENDING |
| F035 | DET002 recognizes timing patterns | `START=$(date)` `END=$(date)` allowed | PENDING |
| F036 | SEC010 recognizes validation | Path validated before use not flagged | PENDING |
| F037 | MAKE003 recognizes quoted context | `"path/$(VAR)/"` not flagged | PENDING |
| F038 | MAKE004 handles multi-line .PHONY | Line continuation targets recognized | PENDING |
| F039 | MAKE008 handles continuation lines | `.PHONY` continuation not recipe | PENDING |
| F040 | Linter handles shellcheck directives | `# shellcheck disable=SCxxxx` honored | PENDING |

### 6.3 Purification Correctness (F041-F060)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F041 | Purified output is deterministic | Same input produces byte-identical output | PENDING |
| F042 | Purified output is idempotent | `mkdir` becomes `mkdir -p` | PENDING |
| F043 | Purified output passes shellcheck | All output passes `shellcheck -s sh` | PENDING |
| F044 | Purified output removes $RANDOM | No `$RANDOM` in output | PENDING |
| F045 | Purified output removes $$ in data | No `$$` in filenames/data | PENDING |
| F046 | Purified output removes timestamps | No `date` in deterministic paths | PENDING |
| F047 | Purified output quotes variables | All `$var` become `"$var"` | PENDING |
| F048 | Purified output uses POSIX | No bash-specific constructs | PENDING |
| F049 | Purified output preserves semantics | Behavior identical to original | PENDING |
| F050 | Purified output handles edge cases | Empty strings, special chars | PENDING |
| F051 | Purified rm uses -f flag | `rm file` becomes `rm -f file` | PENDING |
| F052 | Purified ln uses -sf flags | `ln -s` becomes `ln -sf` | PENDING |
| F053 | Purified cp uses appropriate flags | `cp` idempotency ensured | PENDING |
| F054 | Purified touch is idempotent | Already idempotent, unchanged | PENDING |
| F055 | Purified output handles loops | For/while semantics preserved | PENDING |
| F056 | Purified output handles functions | Function definitions preserved | PENDING |
| F057 | Purified output handles traps | Signal handlers preserved | PENDING |
| F058 | Purified output handles redirects | I/O redirections preserved | PENDING |
| F059 | Purified output handles pipes | Pipeline semantics preserved | PENDING |
| F060 | Purified output handles subshells | Subshell semantics preserved | PENDING |

### 6.4 Docker Integration (F061-F075)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F061 | Detects shell entrypoints | `ENTRYPOINT ["/bin/sh"]` flagged | PENDING |
| F062 | Detects shell in CMD | `CMD ["sh", "-c", "..."]` flagged | PENDING |
| F063 | Validates multi-stage builds | Final stage shell-free verification | PENDING |
| F064 | Detects RUN shell usage | `RUN /bin/sh script.sh` flagged | PENDING |
| F065 | Validates HEALTHCHECK | Healthcheck command validated | PENDING |
| F066 | Handles build args | `ARG` and `ENV` correctly parsed | PENDING |
| F067 | Validates COPY/ADD | Source validation for scripts | PENDING |
| F068 | Detects privileged patterns | `--privileged` usage noted | PENDING |
| F069 | Validates USER directive | Non-root user encouraged | PENDING |
| F070 | Handles WORKDIR | Path validation | PENDING |
| F071 | Validates EXPOSE | Port specification validation | PENDING |
| F072 | Detects shell form vs exec form | `RUN cmd` vs `RUN ["cmd"]` | PENDING |
| F073 | Validates VOLUME | Volume mount path validation | PENDING |
| F074 | Handles LABEL | Metadata validation | PENDING |
| F075 | Validates STOPSIGNAL | Signal specification validation | PENDING |

### 6.5 macOS/launchd Integration (F076-F085)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F076 | Generates valid plist XML | Output passes `plutil -lint` | PENDING |
| F077 | Sets correct Label | Unique reverse-domain identifier | PENDING |
| F078 | Configures ProgramArguments | Array format correct | PENDING |
| F079 | Sets RunAtLoad correctly | Boolean value appropriate | PENDING |
| F080 | Handles KeepAlive | Dictionary or boolean | PENDING |
| F081 | Validates StandardOutPath | Path exists or creatable | PENDING |
| F082 | Validates StandardErrorPath | Path exists or creatable | PENDING |
| F083 | Handles EnvironmentVariables | Dictionary format correct | PENDING |
| F084 | Validates WorkingDirectory | Path validation | PENDING |
| F085 | Sets appropriate UserName | User existence validation | PENDING |

### 6.6 systemd Integration (F086-F095)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F086 | Generates valid unit file | `systemd-analyze verify` passes | PENDING |
| F087 | Sets correct Type | notify/simple/forking appropriate | PENDING |
| F088 | Validates ExecStart | Absolute path, executable | PENDING |
| F089 | Configures ExecReload | Signal or command correct | PENDING |
| F090 | Sets Restart policy | Appropriate for service type | PENDING |
| F091 | Configures RestartSec | Reasonable backoff value | PENDING |
| F092 | Sets LimitMEMLOCK | infinity for mlock services | PENDING |
| F093 | Validates After/Requires | Dependency ordering correct | PENDING |
| F094 | Configures WantedBy | Appropriate target | PENDING |
| F095 | Handles environment files | EnvironmentFile path valid | PENDING |

### 6.7 Signal & Process Management (F096-F100)

| ID | Hypothesis | Falsification Test | Status |
|----|------------|-------------------|--------|
| F096 | Validates trap handlers | `trap 'cmd' SIG` syntax correct | PENDING |
| F097 | Detects signal forwarding | Child process signal propagation | PENDING |
| F098 | Validates PID file patterns | Race-free PID file creation | PENDING |
| F099 | Detects zombie prevention | `wait` after background jobs | PENDING |
| F100 | Validates graceful shutdown | Cleanup before exit | PENDING |

---

## 7. Implementation Roadmap

### Phase 1: Parser Fixes (Q1 2026)

| Task | Issues | Priority |
|------|--------|----------|
| Inline if/then/else/fi | #93 | P0 |
| Array syntax support | #103 | P0 |
| Heredoc language detection | #120, #96 | P1 |
| Line continuation parsing | #121, #119 | P1 |

### Phase 2: Linter Improvements (Q1-Q2 2026)

| Task | Issues | Priority |
|------|--------|----------|
| Bash builtin recognition | #98 | P0 |
| Variable type tracking | #102 | P1 |
| Control flow analysis | #99, #93 | P1 |
| Quote context tracking | #118, #96 | P1 |
| Source file tracking | #95 | P2 |

### Phase 3: Platform Integration (Q2-Q3 2026)

| Task | Stakeholder | Priority |
|------|-------------|----------|
| systemd unit validation | duende | P1 |
| launchd plist validation | duende | P2 |
| Docker shell-free validation | duende, trueno-zram | P1 |
| mlock capability detection | duende, trueno-zram | P1 |

### Phase 4: Advanced Features (Q3-Q4 2026)

| Task | Stakeholder | Priority |
|------|-------------|----------|
| ZRAM command cache | trueno-zram | P2 |
| Procfs parsing validation | trueno-zram | P2 |
| Distributed task scripts | pepita | P3 |

---

## 8. Quality Gates

### 8.1 Release Criteria

- [ ] All 100 falsification tests pass (F001-F100)
- [ ] Zero regressions in existing 6000+ tests
- [ ] Mutation score >90% on new code
- [ ] Test coverage >95%
- [ ] All open P0 issues resolved
- [ ] Documentation updated
- [ ] CHANGELOG complete

### 8.2 Continuous Verification

```bash
# Pre-commit quality gate
make lint test coverage mutation

# CI/CD verification
cargo test --lib
cargo clippy --all-targets -- -D warnings
cargo llvm-cov --lcov --output-path lcov.info
cargo mutants --file src/parser/
```

---

## 9. Appendices

### A. Glossary

| Term | Definition |
|------|------------|
| Jidoka | Automation with human touch; stop-the-line on defects |
| Genchi Genbutsu | Go and see; understand through direct observation |
| Kaizen | Continuous improvement through small incremental changes |
| Poka-yoke | Mistake-proofing; design that prevents errors |
| Falsification | Popper's criterion: theories must be testable and refutable |
| POSIX | Portable Operating System Interface; IEEE 1003.1 |
| mlock | Memory lock; prevent page from being swapped |

### B. Related Documents

- `docs/BASH-INGESTION-ROADMAP.yaml` - Parser development roadmap
- `ROADMAP.yaml` - Project roadmap
- `CLAUDE.md` - Development guidelines
- `duende/docs/roadmaps/roadmap.yaml` - Daemon orchestration roadmap
- `trueno-zram/README.md` - ZRAM integration documentation

### C. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-06 | Claude Code | Initial specification |
