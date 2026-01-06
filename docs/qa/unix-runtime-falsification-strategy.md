# Unix Runtime Improvements: QA Falsification Strategy

## Document Metadata

| Field | Value |
|-------|-------|
| Target Spec | `docs/specifications/unix-runtime-improvements-docker-mac-bash-zsh-daemons.md` |
| Strategy Version | 1.0.0 |
| Date | 2026-01-06 |
| Status | Draft |
| QA Owner | Noah (AI Agent) |

---

## 1. Executive Summary

This strategy outlines the Quality Assurance (QA) approach for validating the "Unix Runtime Improvements" specification. It defines the methodology for implementing the 100-point Falsification Checklist (F001-F100) defined in the spec, ensuring strict adherence to the Toyota Way principles of *Jidoka* (automation) and *Genchi Genbutsu* (verification).

The goal is to prove or disprove the hypotheses in the spec through rigorous, automated testing, preventing regressions in parser correctness, linter accuracy, and platform integrations (Docker, macOS, systemd).

---

## 2. Test Architecture

To avoid ID collisions with existing falsification tests (which cover F001-F130 in `tests/falsification/RESULTS.md`), the new tests will be namespaced as **URI-Fxxx** (Unix Runtime Improvements) in tracking, though they map directly to F001-F100 in the spec.

### 2.1 Test Suites

We will introduce a new integration test suite `tests/falsification/unix_runtime_suite.rs` (or similar) managed by `cargo test`.

| Suite Component | Spec IDs | Implementation Strategy |
|-----------------|----------|-------------------------|
| **Parser Core** | F001-F020 | Rust Unit Tests (AST verification) |
| **Linter Logic** | F021-F040 | Rust Unit Tests (Diagnostic verification) |
| **Purification** | F041-F060 | Integration Tests (Input -> Output Golden Files) |
| **Docker Ops** | F061-F075 | Mocked Dockerfile Parsing + integration tests (if `docker` present) |
| **Platform Ops** | F076-F095 | Generation Verification (XML/INI parsing of output) |
| **Process Mgmt** | F096-F100 | Simulated Process Tests (using `std::process`) |

### 2.2 Testing Pyramid

1.  **L1 Unit Tests (70%)**: Parser and Linter logic. Fast, deterministic.
2.  **L2 Integration Tests (20%)**: Transpiler output verification (Purification), Unit file generation.
3.  **L3 System Tests (10%)**: Docker build simulation, mock systemd verification.

---

## 3. Implementation Strategy

### 3.1 Phase 1: Core Parsing & Linting (F001-F040)

**Goal**: Validate the bashrs parser's ability to handle complex Unix/Bash patterns.

*   **Mechanism**: Use the existing `probar` or `falsification` harness structure.
*   **Action**: Create `tests/falsification/uri_parser_tests.rs`.
*   **Verification**:
    *   Input: Complex bash snippet (e.g., inline `if/then/else`).
    *   Assert: AST is generated successfully (no errors).
    *   Assert: No false positive diagnostics (for linter tests).

### 3.2 Phase 2: Purification & determinism (F041-F060)

**Goal**: Ensure `bashrs purify` produces safe, idempotent, POSIX-compliant code.

*   **Mechanism**: Golden file testing.
*   **Action**: Create `tests/fixtures/uri/purify/`.
*   **Verification**:
    *   Input: `script.sh` (with bashisms).
    *   Expected: `script.sh.purified` (POSIX, quoted, safe).
    *   Property: `purify(purify(x)) == purify(x)` (Idempotency).

### 3.3 Phase 3: Infrastructure as Code (F061-F095)

**Goal**: Validate Dockerfile, launchd plist, and systemd unit file handling.

*   **Mechanism**: Output generation and structural validation.
*   **Action**:
    *   **Docker**: Feed invalid Dockerfiles (with shell entrypoints) -> Assert lint failure.
    *   **macOS**: Generate plist -> Parse with `plist` crate -> Assert keys exist.
    *   **systemd**: Generate unit file -> Parse INI -> Assert `ExecStart` is absolute.

### 3.4 Phase 4: Runtime Behavior (F096-F100)

**Goal**: Verify signal handling and process management logic (simulated).

*   **Mechanism**: `std::process::Command` tests.
*   **Action**: Spawn child processes that trap signals, send signals, verify exit codes.

---

## 4. Execution Plan

### 4.1 Prerequisites

*   Rust Toolchain (Stable)
*   `cargo-nextest` (recommended for reporting)
*   Optional: `docker` CLI (for L3 tests, can be mocked)
*   Optional: `plutil` (macOS only, mocked on Linux)

### 4.2 Automation

Tests will be integrated into the standard `cargo test` flow:

```bash
# Run all Unix Runtime Improvement tests
cargo test --test unix_runtime_suite

# Run specific category
cargo test --test unix_runtime_suite parser_
```

### 4.3 Falsification Reporting

We will maintain a `tests/falsification/URI_RESULTS.md` (parallel to `RESULTS.md`) to track the status of F001-F100.

| Status | Definition | Action |
|--------|------------|--------|
| **PASS** | Hypothesis confirmed (feature works/bug absent) | Lock behavior with regression test |
| **FAIL** | Hypothesis falsified (bug found) | Create GitHub Issue, Mark as blocker |
| **SKIP** | Test environment not available (e.g. macOS on Linux) | Use mocks or CI specific runners |

---

## 5. Verification Matrix (Sample)

| ID | Description | Test Type | File / Harness |
|----|-------------|-----------|----------------|
| F001 | Inline if/then/else | Unit | `uri_parser_tests.rs` |
| F061 | Docker Shell Entrypoint | Unit | `uri_docker_tests.rs` |
| F076 | Valid plist XML | Integration | `uri_platform_tests.rs` |
| F096 | Trap Handlers | System | `uri_process_tests.rs` |

---

## 6. Success Criteria

The QA Strategy is considered successfully implemented when:
1.  All 100 test cases are codified in Rust.
2.  `cargo test` executes them reliably in < 30 seconds.
3.  Any failure in the spec's hypotheses is reported as a test failure.
4.  Documentation (`URI_RESULTS.md`) reflects the live state of the codebase.
