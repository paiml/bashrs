# SPRINT-81: Makefile Linter Expansion - Advanced Rules

**Status**: 🟢 READY TO EXECUTE
**Priority**: P0 - CRITICAL (Phase 1 of v3.0 roadmap)
**Estimated Duration**: 2 weeks (60-80 hours)
**Sprint Type**: Feature Development + Quality Enhancement

---

## Context

**Current State** (v2.1.1):
- 5 Makefile linting rules (MAKE001-MAKE005)
- 100+ tests (40 linter + 60+ parser)
- Full parser infrastructure (lexer, AST, semantic analysis)
- CLI integration (bashrs make lint)
- Functional, ready for world-class enhancement

**Goal**: Expand from 5 rules to 20 rules (15 new rules) to achieve world-class Makefile linting.

**Part of**: v3.0 roadmap, Phase 1 (Makefile World-Class Enhancement)

---

## Objectives

### Primary Objectives
1. **Add 15 new Makefile linting rules** (MAKE006-MAKE020)
2. **Achieve 100% auto-fix coverage** on all new rules
3. **Comprehensive test coverage** (≥90% for new code)
4. **Zero regressions** (all 1,542 existing tests must pass)

### Quality Objectives
- Mutation testing: ≥90% kill rate on new rules
- Complexity: All functions <10
- Performance: No degradation (<200ms for typical Makefiles)
- Documentation: Each rule documented with examples

---

## New Rules (15)

### Category 1: Safety & Correctness (5 rules)

#### MAKE006: Missing target dependencies
**Description**: Detect when targets don't declare necessary dependencies
**Severity**: warning
**Example**:
```makefile
# BAD: app target doesn't list dependencies
app:
	gcc main.c -o app

# GOOD: Dependencies declared
app: main.c utils.c
	gcc main.c utils.c -o app
```
**Auto-fix**: Suggest common dependencies based on recipe commands

#### MAKE008: Tab vs spaces in recipes (CRITICAL)
**Description**: Detect spaces instead of tabs (fatal Make error)
**Severity**: error
**Example**:
```makefile
# BAD: Uses spaces (will fail)
build:
    gcc main.c

# GOOD: Uses tab
build:
	gcc main.c
```
**Auto-fix**: Replace leading spaces with tab

#### MAKE010: Missing error handling (|| exit 1)
**Description**: Detect commands without error handling in recipes
**Severity**: warning
**Example**:
```makefile
# BAD: No error handling
install:
	cp app /usr/bin/app

# GOOD: Error handling
install:
	cp app /usr/bin/app || exit 1
```
**Auto-fix**: Add || exit 1 to commands

#### MAKE015: Missing .DELETE_ON_ERROR
**Description**: Detect Makefiles without .DELETE_ON_ERROR special target
**Severity**: warning
**Example**:
```makefile
# BAD: Missing .DELETE_ON_ERROR
.PHONY: all
all: build

# GOOD: Has .DELETE_ON_ERROR
.DELETE_ON_ERROR:
.PHONY: all
all: build
```
**Auto-fix**: Add .DELETE_ON_ERROR: special target at top of Makefile

#### MAKE018: Parallel-unsafe targets (race conditions)
**Description**: Detect targets that modify shared state without synchronization
**Severity**: warning
**Example**:
```makefile
# BAD: Both targets write to same log
test1:
	echo "Test 1" >> results.log
test2:
	echo "Test 2" >> results.log

# GOOD: Use order-only prerequisites or .NOTPARALLEL
.NOTPARALLEL: test1 test2
test1:
	echo "Test 1" >> results.log
test2:
	echo "Test 2" >> results.log
```
**Auto-fix**: Suggest .NOTPARALLEL or order-only prerequisites

---

### Category 2: Best Practices (5 rules)

#### MAKE007: Silent recipe errors (missing @ or -)
**Description**: Detect recipes that should use @ or - prefix
**Severity**: info
**Example**:
```makefile
# BAD: echo command shown in output
hello:
	echo "Hello World"

# GOOD: Silent with @
hello:
	@echo "Hello World"
```
**Auto-fix**: Add @ prefix for echo/printf commands

#### MAKE009: Hardcoded paths (non-portable)
**Description**: Detect hardcoded absolute paths like /usr/local
**Severity**: warning
**Example**:
```makefile
# BAD: Hardcoded path
install:
	cp app /usr/local/bin

# GOOD: Use variable
PREFIX ?= /usr/local
install:
	cp app $(PREFIX)/bin
```
**Auto-fix**: Suggest $(PREFIX) variable

#### MAKE012: Recursive make considered harmful
**Description**: Detect $(MAKE) -C pattern (Miller's paper)
**Severity**: info
**Example**:
```makefile
# BAD: Recursive make (hides dependencies)
subdirs:
	$(MAKE) -C src
	$(MAKE) -C tests

# GOOD: Include submakefiles
include src/Makefile
include tests/Makefile
```
**Auto-fix**: Suggest include alternative

#### MAKE013: Missing .SUFFIXES management
**Description**: Detect builtin suffix rules pollution
**Severity**: info
**Example**:
```makefile
# BAD: Builtin suffix rules enabled
.PHONY: all
all: main.o

# GOOD: Disable builtin suffix rules
.SUFFIXES:
.PHONY: all
all: main.o
```
**Auto-fix**: Add .SUFFIXES: to disable builtins

#### MAKE017: Missing .ONESHELL for multi-line recipes
**Description**: Detect multi-line recipes without .ONESHELL
**Severity**: info
**Example**:
```makefile
# BAD: Each line runs in separate shell
deploy:
	cd /app
	git pull
	systemctl restart app

# GOOD: Use .ONESHELL or && chaining
.ONESHELL:
deploy:
	cd /app
	git pull
	systemctl restart app
```
**Auto-fix**: Suggest .ONESHELL: or && chaining

---

### Category 3: Performance & Optimization (5 rules)

#### MAKE011: Dangerous pattern rules (%)
**Description**: Detect pattern rules without proper guards
**Severity**: warning
**Example**:
```makefile
# BAD: Overly broad pattern rule
%.o: %.c
	gcc -c $< -o $@

# GOOD: Scoped pattern rule
src/%.o: src/%.c
	gcc -c $< -o $@
```
**Auto-fix**: Suggest scoped pattern rules

#### MAKE014: Inefficient shell invocation
**Description**: Detect multiple shell invocations per recipe
**Severity**: info
**Example**:
```makefile
# BAD: 3 shell invocations
build:
	mkdir -p build
	gcc main.c -o build/app
	strip build/app

# GOOD: Single shell invocation
build:
	mkdir -p build && gcc main.c -o build/app && strip build/app
```
**Auto-fix**: Combine commands with &&

#### MAKE016: Unquoted variable in prerequisites
**Description**: Detect $(VAR) in prerequisites without quotes
**Severity**: warning
**Example**:
```makefile
# BAD: Unquoted variable
SOURCES = main.c utils.c
app: $(SOURCES)
	gcc $(SOURCES) -o app

# GOOD: Quoted (if needed)
# Note: Prerequisites don't need quotes in Make, but catch edge cases
```
**Auto-fix**: Add quotes if variable contains spaces

#### MAKE019: Environment variable pollution
**Description**: Detect export without careful control
**Severity**: warning
**Example**:
```makefile
# BAD: Exports all variables
export

# GOOD: Selective export
export PATH CC CFLAGS
```
**Auto-fix**: Suggest selective export or .UNEXPORT_ALL_VARIABLES

#### MAKE020: Missing include guard for -include
**Description**: Detect include without error handling
**Severity**: info
**Example**:
```makefile
# BAD: include fails if file missing
include config.mk

# GOOD: -include ignores missing files
-include config.mk
```
**Auto-fix**: Change include to -include for optional files

---

## Implementation Plan

### Week 1: Rules 1-8 (MAKE006-MAKE013)

#### Day 1-2: Safety & Correctness (MAKE006, MAKE008, MAKE010)
**Tasks**:
1. RED: Write failing tests for MAKE006 (missing dependencies)
2. GREEN: Implement MAKE006 detection + auto-fix
3. REFACTOR: Clean up, ensure complexity <10
4. RED: Write failing tests for MAKE008 (tab vs spaces)
5. GREEN: Implement MAKE008 detection + auto-fix
6. REFACTOR: Extract helper for whitespace detection
7. RED: Write failing tests for MAKE010 (error handling)
8. GREEN: Implement MAKE010 detection + auto-fix
9. REFACTOR: Extract helper for command analysis

**Deliverables**:
- 3 new rules implemented
- 24 unit tests (8 per rule)
- 3 property tests (1 per rule)
- Total: 27 tests

**Quality Gates**:
- All tests passing
- Clippy clean
- Complexity <10

#### Day 3-4: Safety & Correctness (MAKE015, MAKE018)
**Tasks**:
1. RED: Write failing tests for MAKE015 (.DELETE_ON_ERROR)
2. GREEN: Implement MAKE015 detection + auto-fix
3. REFACTOR: Extract helper for special target detection
4. RED: Write failing tests for MAKE018 (parallel safety)
5. GREEN: Implement MAKE018 detection + auto-fix
6. REFACTOR: Clean up, reduce complexity

**Deliverables**:
- 2 new rules implemented
- 16 unit tests (8 per rule)
- 2 property tests (1 per rule)
- Total: 18 tests

**Quality Gates**:
- All tests passing (45 new + 1,542 existing = 1,587 total)
- Clippy clean
- Complexity <10

#### Day 5: Best Practices (MAKE007, MAKE009, MAKE012)
**Tasks**:
1. RED: Write failing tests for MAKE007 (silent errors)
2. GREEN: Implement MAKE007 detection + auto-fix
3. REFACTOR: Extract helper for recipe command prefixes
4. RED: Write failing tests for MAKE009 (hardcoded paths)
5. GREEN: Implement MAKE009 detection + auto-fix
6. REFACTOR: Extract helper for path analysis
7. RED: Write failing tests for MAKE012 (recursive make)
8. GREEN: Implement MAKE012 detection + auto-fix
9. REFACTOR: Clean up

**Deliverables**:
- 3 new rules implemented
- 24 unit tests (8 per rule)
- 3 property tests (1 per rule)
- Total: 27 tests

**Quality Gates**:
- All tests passing (72 new + 1,542 existing = 1,614 total)
- Clippy clean
- Complexity <10

---

### Week 2: Rules 9-15 + Validation (MAKE013-MAKE020)

#### Day 6-7: Best Practices + Performance (MAKE013, MAKE017, MAKE011, MAKE014)
**Tasks**:
1. RED: Write failing tests for MAKE013 (.SUFFIXES)
2. GREEN: Implement MAKE013 detection + auto-fix
3. REFACTOR: Extract helper for special target management
4. RED: Write failing tests for MAKE017 (.ONESHELL)
5. GREEN: Implement MAKE017 detection + auto-fix
6. REFACTOR: Clean up multi-line recipe analysis
7. RED: Write failing tests for MAKE011 (pattern rules)
8. GREEN: Implement MAKE011 detection + auto-fix
9. REFACTOR: Extract helper for pattern rule analysis
10. RED: Write failing tests for MAKE014 (shell invocation)
11. GREEN: Implement MAKE014 detection + auto-fix
12. REFACTOR: Extract helper for shell command optimization

**Deliverables**:
- 4 new rules implemented
- 32 unit tests (8 per rule)
- 4 property tests (1 per rule)
- Total: 36 tests

**Quality Gates**:
- All tests passing (108 new + 1,542 existing = 1,650 total)
- Clippy clean
- Complexity <10

#### Day 8-9: Performance + Integration (MAKE016, MAKE019, MAKE020)
**Tasks**:
1. RED: Write failing tests for MAKE016 (unquoted vars)
2. GREEN: Implement MAKE016 detection + auto-fix
3. REFACTOR: Reuse variable expansion helpers
4. RED: Write failing tests for MAKE019 (env pollution)
5. GREEN: Implement MAKE019 detection + auto-fix
6. REFACTOR: Clean up
7. RED: Write failing tests for MAKE020 (include guards)
8. GREEN: Implement MAKE020 detection + auto-fix
9. REFACTOR: Extract helper for include directive analysis
10. **Integration testing**: Test all 20 rules together
11. **CLI integration**: Ensure bashrs make lint works with all rules

**Deliverables**:
- 3 new rules implemented
- 24 unit tests (8 per rule)
- 3 property tests (1 per rule)
- 5 integration tests (all rules working together)
- Total: 32 tests

**Quality Gates**:
- All tests passing (140 new + 1,542 existing = 1,682 total)
- Clippy clean
- Complexity <10
- CLI integration verified

#### Day 10: Validation, Documentation, and Completion
**Tasks**:
1. **Mutation testing**: Run cargo-mutants on all new rules
   - Target: ≥90% kill rate
   - Add targeted tests for survivors
2. **Performance benchmarking**: Validate no regression
   - Benchmark on small/medium/large Makefiles
   - Ensure <200ms for typical Makefiles
3. **Real-world validation**: Test on production Makefiles
   - Linux kernel Makefile (partial)
   - GNU coreutils Makefiles
   - Project's own Makefiles
4. **Documentation**:
   - Update docs/linting/MAKEFILE-RULES-COMPLETE.md
   - Create SPRINT-81-COMPLETE.md
   - Update CHANGELOG.md
5. **Final verification**:
   - All 1,682 tests passing (140 new + 1,542 existing)
   - Clippy clean (zero warnings)
   - Mutation kill rate ≥90%
   - Performance targets met

**Deliverables**:
- Mutation testing report (≥90% kill rate)
- Performance benchmarks (all targets met)
- Real-world validation report
- Complete documentation
- SPRINT-81-COMPLETE.md

**Quality Gates**:
- All 1,682 tests passing (100%)
- Mutation kill rate ≥90%
- Clippy clean (zero warnings)
- Performance <200ms for typical Makefiles
- Zero regressions

---

## Success Criteria

### Functional Requirements ✅
- [ ] All 15 new rules implemented (MAKE006-MAKE020)
- [ ] 100% auto-fix coverage (all rules provide fixes)
- [ ] 120 unit tests passing (8 per rule)
- [ ] 15 property tests passing (1 per rule, 100+ cases each)
- [ ] 5 integration tests passing
- [ ] CLI integration working (bashrs make lint)

### Quality Requirements ✅
- [ ] Zero regressions (all 1,542 existing tests pass)
- [ ] Total tests: 1,682 (140 new + 1,542 existing)
- [ ] Test coverage: ≥90% on new code
- [ ] Mutation kill rate: ≥90% on new rules
- [ ] Clippy clean (zero warnings)
- [ ] Complexity: All functions <10

### Performance Requirements ✅
- [ ] No performance degradation
- [ ] <10ms for small Makefiles (<100 lines)
- [ ] <50ms for medium Makefiles (100-500 lines)
- [ ] <200ms for large Makefiles (500-2000 lines)

### Documentation Requirements ✅
- [ ] All rules documented with examples
- [ ] docs/linting/MAKEFILE-RULES-COMPLETE.md updated
- [ ] SPRINT-81-COMPLETE.md created
- [ ] CHANGELOG.md updated

---

## Risks and Mitigation

### Risk 1: Complex Rule Implementation
**Probability**: Medium
**Impact**: High (could block sprint)

**Mitigation**:
- Start with simpler rules (MAKE007, MAKE008, MAKE020)
- Build helpers incrementally
- Use property testing to validate edge cases
- Defer complex rules to end of sprint if needed

### Risk 2: Performance Regression
**Probability**: Low
**Impact**: Medium

**Mitigation**:
- Benchmark after each rule implementation
- Use efficient algorithms (avoid O(n²) scans)
- Profile if performance degrades
- Optimize hot paths

### Risk 3: False Positives/Negatives
**Probability**: Medium
**Impact**: Medium

**Mitigation**:
- Comprehensive test coverage (8 tests per rule)
- Property-based testing
- Validate on real-world Makefiles
- User feedback loop

### Risk 4: Scope Creep
**Probability**: Medium
**Impact**: Medium

**Mitigation**:
- Strict adherence to 15 rules only
- Defer nice-to-have features to Sprint 82
- Focus on defined deliverables
- Weekly progress reviews

---

## Dependencies

### Internal Dependencies
- Existing Makefile parser (rash/src/make_parser/)
- Existing linter infrastructure (rash/src/linter/)
- Existing test infrastructure
- CLI integration (rash/src/cli/)

### External Dependencies
- cargo-llvm-cov (coverage)
- cargo-mutants (mutation testing)
- criterion (benchmarking)
- proptest (property testing)
- assert_cmd (CLI testing)

### Blocking Dependencies
- None (all prerequisites met)

---

## Deliverables Summary

### Code
- **15 new rule files**: rash/src/linter/rules/make006.rs through make020.rs
- **Tests**: 140 new tests (120 unit + 15 property + 5 integration)
- **Documentation**: Updated MAKEFILE-RULES-COMPLETE.md
- **Total lines**: ~2,500-3,000 new lines

### Documentation
- SPRINT-81-COMPLETE.md (comprehensive completion report)
- docs/linting/MAKEFILE-RULES-COMPLETE.md (updated)
- CHANGELOG.md (updated)
- Individual rule documentation (15 rules)

### Quality Reports
- Mutation testing report (≥90% kill rate)
- Performance benchmarks (statistical analysis)
- Real-world validation report

---

## Methodology

### EXTREME TDD
1. **RED**: Write failing test first
2. **GREEN**: Implement minimal code to pass test
3. **REFACTOR**: Clean up, extract helpers, reduce complexity
4. **DOCUMENT**: Update docs and examples

### Property-Based Testing
- 1 property test per rule
- 100+ generated cases per property
- Validates invariants across input space

### Mutation Testing
- Run cargo-mutants on all new rules
- Target: ≥90% kill rate
- Add targeted tests for survivors

### Performance Benchmarking
- Use criterion for statistical analysis
- Benchmark after each rule
- Ensure no regression

---

## Communication Plan

### Daily Updates
- Progress report at end of each day
- Tests added and passing
- Blockers identified

### Weekly Review (Friday)
- Week 1: 8 rules complete (MAKE006-MAKE013)
- Week 2: 15 rules complete (MAKE006-MAKE020)

### Decision Gates
- End of Week 1: Review progress, adjust if needed
- Day 10: Final quality review before completion

---

## Next Steps After Sprint 81

Upon successful completion:
1. **SPRINT-82**: Makefile Parser Enhancement (advanced features)
2. **SPRINT-83**: Makefile Purification Enhancement (GNU Make best practices)
3. **SPRINT-84**: Makefile Performance & Quality Validation

**Estimated Completion**: 2 weeks (10 working days)
**Ready to Start**: ✅ YES (all prerequisites met)

---

**Sprint Created**: 2025-10-19
**Part of**: v3.0 roadmap, Phase 1 (Makefile World-Class Enhancement)
**Methodology**: EXTREME TDD + FAST (Fuzz, AST, Safety, Throughput)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
