# UX Quality Improvements Specification

**Document ID**: SPEC-UX-2025-001
**Version**: 2.0.0 (Restructured)
**Status**: ACTIVE
**Created**: 2025-12-15
**Methodology**: Toyota Production System (TPS) / Lean Manufacturing

---

## Quick Links

| Section | Description | Status |
|---------|-------------|--------|
| [Requirements](#1-falsifiable-requirements) | Parser, FP, quality, testing, docs | Inline |
| [Toyota Way](#2-toyota-way-principles) | TPS principles applied | Inline |
| [Research](#3-peer-reviewed-research) | Academic foundations | Inline |
| [Issue Registry](#4-issue-registry) | Consolidated issues | Inline |
| [Root Cause](#5-root-cause-analysis) | 5 Whys analysis | Inline |
| [QA Checklist](#6-qa-checklist) | 100-point Popperian | Inline |
| [Kanban](#7-implementation-kanban) | Task tracking | Inline |
| [Definition of Done](#8-definition-of-done) | DoD criteria | Inline |
| [Risk](#9-risk-management) | Risk assessment | Inline |
| [Appendices](#10-appendices) | Supporting materials | Inline |
| **[TUI + Probar](./11-tui-probar.md)** | **NEW: TUI testing spec** | **Separate file** |

---

## Executive Summary

**Total Open Items**: 91 (90 existing + 1 TUI)
- **Critical (P0)**: 0
- **High (P1)**: 9 (+1 TUI)
- **Medium (P2)**: 75
- **Low (P3-P4)**: 7

### Implementation Status

| Feature | CLI Command | Status |
|---------|-------------|--------|
| Shell Linting | `bashrs lint` | Done |
| Dockerfile Linting | `bashrs dockerfile lint` | Done |
| Coursera Profile | `bashrs lint --profile coursera` | Done (v6.43.0) |
| Dev Container Validation | `bashrs devcontainer validate` | Done (v6.43.0) |
| Docker Runtime Profiling | `bashrs dockerfile profile` | Done (v6.43.0) |
| **TUI Interface** | `bashrs tui` | **P1 - Planned** |
| **Probar Testing** | `cargo test --features tui` | **P1 - Planned** |

---

## 1. Falsifiable Requirements

Per Popper's criterion of demarcation, each requirement MUST be falsifiable.

### 1.1 Parser Requirements

| ID | Requirement | Test |
|----|-------------|------|
| REQ-PARSER-001 | Parse inline if/then/else/fi | `bashrs lint 'if true; then echo yes; fi'` |
| REQ-PARSER-002 | Handle short-circuit conditionals | `bashrs lint '[ -f /etc/passwd ] && echo exists'` |

### 1.2 False Positive Requirements

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-FP-001 | SC2031 not flag local vars in case | Implemented |
| REQ-FP-002 | SC2102 not flag valid ERE quantifiers | Implemented |
| REQ-FP-003 | SC2154 track `read` variables | **Fixed v6.43.0** |
| REQ-FP-004 | SC2201 not flag param expansions | Implemented |
| REQ-FP-005 | SC2104 not flag ] in array subscript | Implemented |
| REQ-FP-006 | SEC003 not flag {} in find -exec | **Fixed v6.43.0** |
| REQ-FP-007 | SEC011 recognize inline validation | Implemented |

### 1.3 Code Quality Requirements

| ID | Requirement | Metric |
|----|-------------|--------|
| REQ-QUAL-001 | Zero lifetime elision warnings | `grep -c "hidden lifetime"` = 0 |
| REQ-QUAL-002 | No unwrap() in production | `clippy -D unwrap_used` passes |
| REQ-QUAL-003 | 100% test pass rate | 7305+ tests, 0 failures |

### 1.4 Testing Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| REQ-TEST-001 | Mutation testing kill rate | >= 80% |
| REQ-TEST-002 | Property test cases | >= 100 per property |
| REQ-TEST-003 | Regression tests for fixed issues | >= 8 |
| REQ-TEST-004 | Line coverage | >= 95% |

### 1.5 TUI Requirements (NEW)

See **[11-tui-probar.md](./11-tui-probar.md)** for full specification.

| ID | Requirement | Target |
|----|-------------|--------|
| REQ-TUI-001 | GUI coverage via probar | >= 95% |
| REQ-TUI-002 | Frame capture assertions | All screens |
| REQ-TUI-003 | Deterministic replay | 100% reproducible |
| REQ-TUI-004 | Monte Carlo fuzzing | 1000+ iterations |
| REQ-TUI-005 | Visual regression | Golden snapshots |

---

## 2. Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Jidoka** | ML classifies, human approves |
| **Kaizen** | Learn from fix acceptance |
| **Mieruka** | Rich ASCII dashboards |
| **Genchi Genbutsu** | SBFL locates actual faults |
| **Poka-yoke** | Confidence scores prevent bad fixes |

---

## 3. Peer-Reviewed Research

| Topic | Reference |
|-------|-----------|
| Fault Localization | Jones & Harrold (2005) - Tarantula |
| Ochiai Formula | Abreu et al. (2007) |
| Cyclomatic Complexity | McCabe (1976) |
| Bug Localization | Kim et al. (2013) - Learning-to-rank |

---

## 4. Issue Registry

| Priority | Count | Examples |
|----------|-------|----------|
| P0 (Critical) | 0 | - |
| P1 (High) | 9 | TUI, parser edge cases |
| P2 (Medium) | 75 | Coursera, DevContainer, runtime |
| P3-P4 (Low) | 7 | Documentation, polish |

---

## 5. Root Cause Analysis

Using Toyota's "5 Whys" methodology:

| Issue | Root Cause | Fix |
|-------|------------|-----|
| SC2154 false positives | `read` not tracked in pipelines | Track assignments in all contexts |
| SEC003 false positives | {} not recognized as find placeholder | Pattern match find -exec {} |
| TUI edge cases | No comprehensive fuzzing | Probar Monte Carlo integration |

---

## 6. QA Checklist

100-point Popperian falsification checklist (abbreviated):

| Category | Points | Criteria |
|----------|--------|----------|
| Parser | 20 | All syntax constructs parsed |
| Linter | 20 | Zero false positives on test corpus |
| Purifier | 15 | Deterministic output |
| Tests | 20 | 95%+ coverage, 80%+ mutation |
| Docs | 10 | All examples compile |
| TUI | 15 | 95%+ GUI coverage via probar |

---

## 7. Implementation Kanban

### Backlog
- [ ] TUI core implementation (P1)
- [ ] Probar test suite (P1)
- [ ] Edge case registry (P2)

### In Progress
- [x] Coverage at 95%+ (Done)
- [x] False positive fixes (Done)

### Done
- [x] Coursera profile (v6.43.0)
- [x] DevContainer validation (v6.43.0)
- [x] Runtime profiling (v6.43.0)

---

## 8. Definition of Done

- [ ] All tests pass (7305+)
- [ ] Coverage >= 95%
- [ ] Mutation kill rate >= 80%
- [ ] Clippy clean
- [ ] Book examples compile
- [ ] TUI GUI coverage >= 95% (NEW)
- [ ] Probar playbook validated (NEW)

---

## 9. Risk Management

| Risk | Mitigation |
|------|------------|
| TUI complexity | Reuse existing REPL components |
| Probar integration | External crate, minimal coupling |
| Edge case explosion | Registry + triage process |

---

## 10. Appendices

### A. Related Specifications

- [bashrs-lint-spec.md](../bashrs-lint-spec.md)
- [Dockerfile-purification.md](../Dockerfile-purification.md)
- [MUTATION-TESTING.md](../MUTATION-TESTING.md)

### B. Testing Tools

| Tool | Purpose |
|------|---------|
| `jugar-probar` | TUI/GUI testing framework |
| `probador` | CLI for playbook validation |
| `cargo-mutants` | Mutation testing |
| `cargo-llvm-cov` | Coverage measurement |

### C. Commands Reference

```bash
# TUI (planned)
bashrs tui

# Testing
cargo test --lib -p bashrs --features tui
cargo test test_tui_gui_coverage
probador playbook tui-playbook.yaml --validate

# Coverage
make coverage
```

---

## Full Specification

For the complete 3300-line specification with all details, see:
[current-ux-quality-improvements.md](../current-ux-quality-improvements.md)

This index provides quick navigation to key sections. The original file contains:
- Detailed Coursera requirements (REQ-COURSERA-001 to 020)
- DevContainer requirements (REQ-DEVCONTAINER-001 to 011)
- Performance requirements (REQ-PERF-001 to 008)
- Runtime requirements (REQ-RUNTIME-001 to 012)
- Size requirements (REQ-SIZE-001 to 010)
