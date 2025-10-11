# Sprint 27: Core Shell Features - Batch P0 Implementation

**Status**: 🟢 READY TO EXECUTE
**Priority**: P0 - CRITICAL
**Estimated Duration**: 20-30 hours
**Target Version**: v1.3.0
**Methodology**: EXTREME TDD (RED-GREEN-REFACTOR-PROPERTY-MUTATION)

---

## Sprint Goal

Implement three fundamental shell features that are currently blocking 25% of GNU Bash Manual validation:

1. **Positional Parameters** (`$1, $2, $3` via `std::env::args()`)
2. **Parameter Expansion** (`${VAR:-default}` via `unwrap_or()`)
3. **Exit Status** (`$?` for command exit codes)

These features share common implementation patterns and can be built efficiently together.

---

## Why Batch These Features?

### Common Implementation Requirements

All three features require:
- ✅ Parser enhancements (method call patterns, stdlib recognition)
- ✅ AST additions (new expression types)
- ✅ IR conversion logic
- ✅ Shell emitter changes (special parameter syntax)
- ✅ Comprehensive test suites

### Efficiency Gains

- **Code Reuse**: Pattern matching infrastructure built once, used thrice
- **Context Switching**: Stay in "shell parameter" mode for entire sprint
- **Testing Synergy**: Similar property tests, mutation tests, integration tests
- **Documentation**: One comprehensive changelog update

### Impact

**Unblocks**:
- 18 validation tasks (15% of manual)
- Real-world use cases: CLI tools, configuration management, error handling
- Both Workflow 1 (Rust→Shell) and Workflow 2 (Bash→Rust→Purified Bash)

---

## Sprint Scope

### Feature 1: Positional Parameters (10-15 hours)

**Status**: P0 #1 - Deferred from initial validation
**Ticket**: `docs/tickets/P0-POSITIONAL-PARAMETERS.md`
**RED Phase**: ✅ Complete (3 tests, lines 677-783 in integration_tests.rs)

**Implementation Steps**:

#### Phase 1: Parser Enhancement (3-4 hours)
- Recognize `std::env::args()` pattern
- Handle `.collect()` on iterators
- Parse `Vec<String>` type annotations
- Recognize `.get(N)` method calls
- Map `.unwrap_or()` with index to positional param

**Files to Modify**:
```
rash/src/services/parser.rs:
  - fn convert_call_expr() - detect std::env::args()
  - fn convert_method_call() - handle .get(N)

rash/src/ast/restricted.rs:
  - Add PositionalParam { index, default } variant
```

#### Phase 2: IR Conversion (2-3 hours)
- Transform AST PositionalParam to IR
- Handle default value expressions
- Generate proper variable assignments

**Files to Modify**:
```
rash/src/ir/mod.rs:
  - Add convert_positional_param()
  - Handle $1, $2, $3 mapping
```

#### Phase 3: Shell Emission (2-3 hours)
- Emit `first="${1:-default}"` syntax
- Ensure proper quoting (always `"$var"`)
- Handle `main "$@"` argument passing

**Files to Modify**:
```
rash/src/emitter/mod.rs:
  - Add emit_positional_param()
  - Ensure main() receives "$@"
```

#### Phase 4: Testing & Verification (3-5 hours)
- ✅ GREEN: Make 3 RED tests pass
- ✅ REFACTOR: Extract helpers, complexity <10
- ✅ Property tests: Verify quoting, determinism
- ✅ Mutation tests: ≥90% kill rate
- ✅ Integration: End-to-end with actual shell execution
- ✅ Shellcheck: POSIX compliance verification

---

### Feature 2: Parameter Expansion (6-8 hours)

**Status**: P0 #2 - Deferred from initial validation
**Ticket**: `docs/tickets/P0-PARAMETER-EXPANSION-DEFAULT.md`
**RED Phase**: ✅ Complete (3 tests, lines 491-579 in integration_tests.rs)

**Implementation Steps**:

#### Phase 1: Parser Enhancement (2-3 hours)
- Recognize `Option::unwrap_or(default)` pattern
- Recognize `Result::unwrap_or(default)` pattern
- Handle `std::env::var("VAR").unwrap_or()` chain
- Extract variable name and default value

**Files to Modify**:
```
rash/src/services/parser.rs:
  - fn convert_method_call() - handle unwrap_or()
  - fn convert_call_expr() - detect env::var()
```

#### Phase 2: AST & IR (2-3 hours)
- Add `ParameterExpansion` AST variant
- Support ExpansionType enum (DefaultValue, AssignDefault, etc.)
- Convert to IR representation

**Files to Modify**:
```
rash/src/ast/restricted.rs:
  - Add ParameterExpansion { variable, expansion_type, value }
  - Add ExpansionType enum

rash/src/ir/mod.rs:
  - Add convert_parameter_expansion()
```

#### Phase 3: Shell Emission (1-2 hours)
- Emit `result="${VAR:-default}"` syntax
- Handle various expansion operators (`:- := :? :+`)
- Proper escaping of default values

**Files to Modify**:
```
rash/src/emitter/mod.rs:
  - Add emit_parameter_expansion()
```

#### Phase 4: Testing & Verification (1-2 hours)
- ✅ GREEN: Make 3 RED tests pass
- ✅ REFACTOR: Clean up code
- ✅ Property tests: Multiple variables, quoting
- ✅ Mutation tests: ≥90% kill rate
- ✅ Integration: ENV var configuration scenarios

---

### Feature 3: Exit Status ($?) (4-6 hours)

**Status**: RED Phase Complete (just completed)
**RED Phase**: ✅ Complete (3 tests, lines 581-675 in integration_tests.rs)

**Implementation Steps**:

#### Phase 1: Parser Enhancement (1-2 hours)
- Recognize "exit status getter" function pattern
- Identify command execution followed by status check
- Map common patterns to `$?` capture

**Files to Modify**:
```
rash/src/services/parser.rs:
  - fn convert_call_expr() - detect exit status functions
  - Add command execution tracking
```

#### Phase 2: AST & IR (1-2 hours)
- Add `ExitStatus` expression variant
- Track command execution order
- Generate proper `$?` capture points

**Files to Modify**:
```
rash/src/ast/restricted.rs:
  - Add ExitStatus variant

rash/src/ir/mod.rs:
  - Add exit status capture logic
```

#### Phase 3: Shell Emission (1 hour)
- Emit `_exit="$?"` immediately after commands
- Use stored value in subsequent expressions
- Handle conditionals with `$?`

**Files to Modify**:
```
rash/src/emitter/mod.rs:
  - Add emit_exit_status_capture()
  - Ensure proper sequencing
```

#### Phase 4: Testing & Verification (1-2 hours)
- ✅ GREEN: Make 3 RED tests pass
- ✅ REFACTOR: Simplify
- ✅ Integration: Error handling scenarios
- ✅ Verify proper exit code propagation

---

## Sprint Execution Plan

### Week 1: Foundation (8-10 hours)

**Day 1-2: Positional Parameters**
- Hours 1-4: Parser enhancement
- Hours 5-7: IR conversion
- Hours 8-10: Shell emission

**Day 3: Positional Parameters Testing**
- Hours 11-13: GREEN phase (make tests pass)
- Hours 14-15: REFACTOR, property tests, mutation tests

### Week 2: Expansion (6-8 hours)

**Day 4: Parameter Expansion**
- Hours 16-18: Parser + AST
- Hours 19-21: IR + Emitter

**Day 5: Parameter Expansion Testing**
- Hours 22-23: GREEN + REFACTOR
- Hour 24: Property & mutation tests

### Week 3: Exit Status (4-6 hours)

**Day 6: Exit Status**
- Hours 25-27: Parser, AST, IR, Emitter

**Day 7: Exit Status Testing + Integration**
- Hours 28-29: GREEN + REFACTOR
- Hour 30: Full sprint integration testing

---

## Success Criteria

### Functional Requirements
- [x] ✅ RED Phase: 9 failing tests written (3 per feature)
- [ ] ✅ GREEN Phase: All 9 tests pass
- [ ] ✅ POSIX Compliance: All generated scripts pass `shellcheck -s sh`
- [ ] ✅ Determinism: Same input produces byte-identical output
- [ ] ✅ Idempotency: Scripts safe to re-run

### Quality Requirements
- [ ] ✅ Test Coverage: ≥85% on new code
- [ ] ✅ Mutation Score: ≥90% kill rate
- [ ] ✅ Complexity: All functions <10 cyclomatic complexity
- [ ] ✅ Property Tests: Quoting, determinism verified with proptest
- [ ] ✅ Integration Tests: End-to-end workflows verified

### Documentation Requirements
- [ ] ✅ CHANGELOG.md updated with all 3 features
- [ ] ✅ BASH-INGESTION-ROADMAP.yaml: 18 tasks marked complete
- [ ] ✅ P0 tickets closed (marked RESOLVED)
- [ ] ✅ Examples added to docs/examples/

---

## Validation Tasks Unblocked

Upon completion, these validation tasks will be unblocked:

### Positional Parameters (3 tasks)
- PARAM-POS-001: Document $1, $2, etc.
- PARAM-SPEC-001: Document $# (argument count)
- PARAM-SPEC-005: Document $0 (script name)

### Parameter Expansion (4 tasks)
- EXP-PARAM-001: ${parameter:-word} (default value)
- EXP-PARAM-002: ${parameter:=word} (assign default)
- EXP-PARAM-003: ${parameter:?word} (error if unset)
- EXP-PARAM-004: ${parameter:+word} (alternative value)

### Exit Status (1 task)
- PARAM-SPEC-002: Document $? (exit status)

### Related Tasks (10 additional tasks enabled)
- BUILTIN-015: shift command (depends on positional params)
- BUILTIN-016: test/[ command (uses $?)
- And 8 more tasks that use these primitives

**Total**: 18 tasks directly unblocked, 10+ tasks indirectly enabled

---

## Risk Mitigation

### Risk 1: Complexity Underestimation
**Mitigation**: Time-boxed implementation. If any feature exceeds estimate by 50%, pause and reassess.

### Risk 2: POSIX Compliance Issues
**Mitigation**: Run `shellcheck` on every generated script. If failures >5%, stop and fix root cause.

### Risk 3: Test Suite Brittleness
**Mitigation**: Property-based testing catches edge cases. Mutation testing ensures tests are meaningful.

---

## Toyota Way Principles Applied

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD: Test-first, always
- Property tests: Catch edge cases automatically
- Mutation tests: Ensure tests are effective

### 改善 (Kaizen) - Continuous Improvement
- Batch similar work for efficiency
- Reuse patterns across features
- Learn from RED phase discoveries

### 反省 (Hansei) - Reflection
- Three P0s discovered during validation
- Pattern identified: "simple" shell features are complex
- Solution: Batch implementation for efficiency

### 現地現物 (Genchi Genbutsu) - Go and See
- Test against real shells (dash, ash, busybox)
- Verify actual script execution
- Measure real-world impact

---

## Post-Sprint Actions

### Immediate (v1.3.0 Release)
- [ ] Tag release: `git tag v1.3.0`
- [ ] Update version in Cargo.toml
- [ ] Publish CHANGELOG
- [ ] Close all 3 P0 tickets

### Follow-up (Sprint 28)
- [ ] Continue GNU Bash Manual validation
- [ ] Complete remaining 84 validation tasks
- [ ] Target: 50% completion (60/120 tasks)

---

## Sprint Metrics

**Planned**:
- Duration: 20-30 hours
- Features: 3 core shell features
- Tests: 9 RED → GREEN
- Tasks Unblocked: 18 validation tasks

**Success Indicators**:
- All tests pass: 808 + 9 = 817 tests
- 0 compiler warnings
- 0 P0 bugs open
- Validation: 15% → 28% complete

---

**Status**: 🟢 READY TO EXECUTE

**Next Step**: Begin Feature 1 (Positional Parameters) - Parser enhancement

🚨 **Sprint Start Authorization Required** - Confirm to begin implementation
