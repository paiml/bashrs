# Unified Specification: WASM + Safe Shell + Rash Shell

**File**: `wasm-shell-safe-bash-rash-shell-spec.yaml`
**Version**: 1.0.0
**Date**: 2025-10-23
**Lines**: 1,085
**Format**: YAML (ROADMAP.yaml style)
**Methodology**: EXTREME TDD

## Executive Summary

This specification unifies three major initiatives:
1. **WASM Browser Runtime** (v7.0) - 2-3 weeks
2. **Safe Shell Interpreter** (v8.0) - 4-6 weeks  
3. **Interactive REPL** (v8.5) - 3-4 weeks

**Total Timeline**: 9-13 weeks across 3 phases

## Quick Stats

- **Total Tasks**: 27 (8 WASM + 11 Shell + 8 REPL)
- **WASM Tests**: 40 browser canary tests (B01-B40)
- **Shell Builtins**: 20 commands in safe Rust
- **Quality Target**: NASA-level, SQLite-inspired

## Three Phases

### Phase 1: WASM Browser Runtime (v7.0)
**Timeline**: 2-3 weeks | **Status**: Phase 0 Complete

Deploy bashrs linter in browsers for WOS + interactive.paiml.com

**Tasks**:
- WASM-001: Playwright setup
- WASM-002 to WASM-005: 40 canary tests (B01-B40)
- WASM-006: Virtual filesystem
- WASM-007 to WASM-008: WOS + interactive.paiml.com integration

**Deliverables**:
- 40 Playwright browser tests
- Cross-browser compatibility (Chrome, Firefox, Safari)
- Streaming I/O >10 MB/s
- Production deployment

### Phase 2: Safe Shell Interpreter (v8.0)
**Timeline**: 4-6 weeks | **Status**: Design Phase

Execute bash scripts directly in Rust (no /bin/sh subprocess)

**Tasks**:
- SHELL-001: Architecture design
- SHELL-002 to SHELL-006: 20 builtin commands
- SHELL-007: Variable scope management
- SHELL-008: Function call stack
- SHELL-009: Pipeline execution
- SHELL-010: Redirection
- SHELL-011: Integration

**Deliverables**:
- Memory-safe bash interpreter
- 20 builtins in safe Rust
- Execute 90% of simple scripts
- 100% memory safe (no unsafe blocks)

### Phase 3: Interactive REPL (v8.5)
**Timeline**: 3-4 weeks | **Status**: Design Phase (after Phase 2)

Build interactive shell as bash replacement

**Tasks**:
- REPL-001: Rustyline integration
- REPL-002: Syntax highlighting
- REPL-003: Tab completion
- REPL-004: Command history
- REPL-005: Prompt customization
- REPL-006: Keybindings
- REPL-007: Multi-line editing
- REPL-008: Job control

**Deliverables**:
- Daily-use interactive shell
- `chsh -s /usr/bin/rash`
- Modern UX (syntax highlighting, completion)

## Testing Strategy (EXTREME TDD)

**RED → GREEN → REFACTOR**

Every feature follows:
1. **RED**: Write failing test first
2. **GREEN**: Make test pass (minimal code)
3. **REFACTOR**: Clean up while tests stay green

**Quality Gates**:
- Unit tests: >85% coverage
- Property tests: 100+ cases per feature
- Mutation tests: ≥90% kill rate
- Browser tests: 40 canary tests
- Zero clippy warnings

## Success Metrics

### WASM (v7.0)
- ✅ Loads in <5s
- ✅ Analyzes 1KB in <100ms
- ✅ Streams >10 MB/s
- ✅ Works in 3 browsers

### Shell (v8.0)
- ✅ 20 builtins implemented
- ✅ Executes 1000 cmd/sec
- ✅ 100% memory safe
- ✅ Zero injection vulnerabilities

### REPL (v8.5)
- ✅ Startup <100ms
- ✅ Tab completion <10ms
- ✅ Works as login shell

## Next Actions (This Week)

1. **Review specification** (30 min)
2. **Approve Phase 1 start** (decision point)
3. **Set up Playwright** (30 min)
   ```bash
   cd rash/examples/wasm
   npm init -y
   npm install playwright @playwright/test
   npx playwright install chromium firefox webkit
   ```
4. **Implement B01 test** (RED phase - 2 hours)

## File Structure

```yaml
wasm-shell-safe-bash-rash-shell-spec.yaml:
  - project: Vision and priorities
  - current_state: v6.2.0 baseline
  - quality_standards: NASA-level requirements
  - phase_1_wasm: 8 tasks, 40 browser tests
  - phase_2_safe_shell: 11 tasks, 20 builtins
  - phase_3_repl: 8 tasks, interactive shell
  - testing_strategy: EXTREME TDD
  - success_metrics: Performance targets
  - risks: Technical and organizational
  - dependencies: External and internal
  - timeline: 9-13 weeks
  - next_actions: This week's tasks
  - references: Related docs
```

## Key References

- `ROADMAP.yaml` - Project roadmap
- `CLAUDE.md` - Development guidelines
- `docs/SAFE-SHELL-VISION.md` - Safe shell architecture
- `rash/examples/wasm/TESTING-SPEC.md` - WASM testing
- `rash/examples/wasm/PHASE0-RESULTS.md` - Feasibility

## Decision Points

**Awaiting approval**:
- [ ] Start WASM Phase 1 (v7.0)?
- [ ] Parallel development (WASM + Shell)?
- [ ] Resource allocation?
- [ ] Timeline (9-13 weeks) acceptable?

---

**Status**: ✅ Specification Complete | 📋 Awaiting Approval | 🚀 Ready to Start
