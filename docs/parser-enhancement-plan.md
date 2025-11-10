# Bash Parser Enhancement Plan

**Status**: Planning
**Priority**: P0 - BLOCKS BASH PURIFICATION TO 100%
**Owner**: TBD
**Target**: Enable production-ready bash purification benchmarks

## Executive Summary

Discovered during bash purification benchmark development (Task 1 of 10-task roadmap): The bash parser has significant limitations that prevent processing realistic bash scripts. This blocks bash purification from reaching 100% production-ready.

**Critical Impact**: Cannot benchmark or purify real-world bash scripts containing common constructs like `date +FORMAT`, complex command substitution, and other standard bash syntax.

## Parser Limitations Discovered

### P0: Date Format Syntax (`date +FORMAT`)
**Error**: `LexerError(UnexpectedChar('+', line, col))`

**Examples that fail**:
```bash
date +%s                      # Unix timestamp
date +%Y%m%d%H%M%S            # Formatted date
date +'%Y-%m-%d %H:%M:%S'     # Quoted format
```

**Impact**: CRITICAL - date formatting is ubiquitous in production bash scripts (logging, timestamps, backups, monitoring)

**Occurrences in benchmark fixtures**: 10+ instances in medium.sh, 30+ in large.sh

### P0: Command Substitution Issues
**Error**: `InvalidSyntax("Expected command name")`

**Examples that fail**:
```bash
FILES=$(ls /tmp)              # Command substitution with external command
ID=$(generate_id)             # Command substitution calling function
TEMP="/tmp/file-$$"           # Process ID substitution
```

**Impact**: HIGH - command substitution is fundamental to bash scripting

**Occurrences**: 20+ instances across benchmark fixtures

### P1: Additional Bash Construct Gaps
**Symptoms**: Various parsing errors on valid bash syntax

**Examples**:
```bash
# Function-like variable expansions
echo ${VAR:-default}
echo ${VAR:?error}

# Complex redirections
command 2>&1 | tee log.txt

# Arithmetic expansion edge cases
result=$((x + y * z))
```

**Impact**: MEDIUM - limits parser coverage of advanced bash features

## Root Cause Analysis

### Lexer Limitations
The lexer (`rash/src/bash_parser/lexer.rs`) does not recognize:
1. `+` as part of command arguments (interprets as operator)
2. Complex format strings in command arguments
3. Nested substitutions and expansions

### Parser Limitations
The parser (`rash/src/bash_parser/parser.rs`) struggles with:
1. Context-dependent syntax (same char means different things)
2. Lookahead for disambiguating constructs
3. Proper handling of quoted vs unquoted contexts

## Enhancement Plan

### Phase 1: Date Format Support (1-2 weeks)

**Goal**: Support `date +FORMAT` and similar flag-based command syntax

**Approach - EXTREME TDD**:

#### Task 1.1: Lexer Enhancement for + in Arguments
```rust
// RED: Write failing test
#[test]
fn test_lexer_plus_in_command_args() {
    let input = "date +%s";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1].kind, TokenKind::Argument("+%s"));
}

// GREEN: Implement context-aware tokenization
// - Track if we're inside a command context
// - Allow + as part of argument token when not in arithmetic context
// - Handle quoted vs unquoted +

// REFACTOR: Ensure complexity <10, clean code
```

#### Task 1.2: Parser Integration
- Update AST to represent command arguments with flags
- Add tests for date command variants
- Property tests: ∀ valid date format, parser succeeds

#### Task 1.3: Purification Verification
- Verify purified scripts preserve date functionality
- Test determinism transformations on date commands
- Integration test: parse → purify → shellcheck

**Success Criteria**:
- ✅ All date +FORMAT variants parse successfully
- ✅ Property tests pass (100+ cases)
- ✅ Mutation score >90% on new code
- ✅ Zero regressions in existing 6004 tests

### Phase 2: Command Substitution Robustness (1-2 weeks)

**Goal**: Fix "Expected command name" errors in command substitution

**Approach - EXTREME TDD**:

#### Task 2.1: Analyze Command Substitution Edge Cases
```bash
# Test matrix
$(simple_command)          # Basic
$(command | pipe)          # With pipes
$(func_call arg)           # Function calls
$(nested $(inner))         # Nested (if needed)
```

#### Task 2.2: Parser State Machine Refinement
- Improve context tracking for `$(...)`
- Handle EOF properly inside substitution
- Validate command names in substitution context

#### Task 2.3: Integration and Testing
- Add comprehensive command substitution test suite
- Property tests for substitution invariants
- Verify purification preserves semantics

**Success Criteria**:
- ✅ All command substitution patterns parse
- ✅ Handles nested and piped commands
- ✅ Property tests pass
- ✅ Mutation score >90%
- ✅ Zero regressions

### Phase 3: Comprehensive Bash Coverage (2-3 weeks)

**Goal**: Expand parser to handle advanced bash constructs

**Tasks**:
1. Parameter expansion variants (`${VAR:-default}`, `${VAR:?}`, etc.)
2. Complex redirection patterns
3. Arithmetic expansion edge cases
4. Process substitution `<(command)` (if needed)

**Approach**: Same EXTREME TDD cycle for each construct

**Success Criteria**:
- ✅ Parse 95%+ of common bash constructs
- ✅ Comprehensive test coverage
- ✅ Property tests for all new features
- ✅ Mutation score >90%
- ✅ Zero regressions

## Validation Strategy

### Test Coverage Targets
- Unit tests: >85% coverage on new parser code
- Integration tests: End-to-end parse → purify → shellcheck
- Property tests: 100+ cases per new feature
- Mutation tests: >90% kill rate

### Real-World Validation
Test parser against:
1. Benchmark fixtures (small.sh, medium.sh, large.sh)
2. Production bash scripts from bashrs examples
3. Common patterns from GitHub bash projects
4. Shell script corpus (if available)

### Performance Baseline
- Parsing: <100µs for 100-line scripts (maintain current ~11µs baseline)
- Memory: <10MB for large scripts
- Zero performance regressions

## Rollout Plan

### Stage 1: Date Format (Week 1-2)
- Implement Phase 1
- Run all quality gates
- Document known limitations

### Stage 2: Command Substitution (Week 3-4)
- Implement Phase 2
- Integrate with Phase 1
- Comprehensive testing

### Stage 3: Advanced Constructs (Week 5-7)
- Implement Phase 3
- Final integration
- Production validation

### Stage 4: Bash Purification Benchmarks (Week 8)
- Resume Task 1 of purification roadmap
- Run benchmarks with realistic fixtures
- Verify <100ms/1000 lines target
- Complete purification to 100%

## Risk Mitigation

### Risk 1: Breaking Changes to Existing Parser
**Mitigation**:
- Run full test suite (6004 tests) after each change
- Property tests to verify invariants
- Git branching strategy for safe experimentation

### Risk 2: Performance Regression
**Mitigation**:
- Benchmark after each phase
- Profile hotspots
- Optimize if needed (target: <10% slowdown max)

### Risk 3: Scope Creep
**Mitigation**:
- Focus on P0 blockers first (date, command substitution)
- Defer P1 features if schedule slips
- Document remaining limitations clearly

## Dependencies

**Blocks**:
- Bash purification benchmarking (Task 1 of 10)
- Bash purification test expansion (Task 2)
- Bash purification to 100% production-ready

**Depends On**:
- Current parser infrastructure (rash/src/bash_parser/)
- Existing test framework
- Purification engine (ready)

## Success Metrics

### Completion Criteria
- [ ] ✅ Date +FORMAT fully supported
- [ ] ✅ Command substitution robust
- [ ] ✅ 95%+ bash construct coverage
- [ ] ✅ All quality gates pass
- [ ] ✅ Zero regressions
- [ ] ✅ Benchmark fixtures parse successfully
- [ ] ✅ Documentation updated

### Quality Gates (EXTREME TDD)
- [ ] ✅ >85% test coverage
- [ ] ✅ Property tests passing (100+ cases per feature)
- [ ] ✅ Mutation score >90%
- [ ] ✅ Complexity <10 for all functions
- [ ] ✅ Clippy clean
- [ ] ✅ All 6004+ existing tests passing

## Estimated Effort

- **Phase 1 (Date Format)**: 1-2 weeks
- **Phase 2 (Command Substitution)**: 1-2 weeks
- **Phase 3 (Advanced Constructs)**: 2-3 weeks
- **Total**: 4-7 weeks (calendar time, single developer)

## Next Steps

1. **Immediate**: Document known parser limitations in bashrs docs
2. **Week 1**: Begin Phase 1 (Date Format) with EXTREME TDD
3. **Weekly**: Review progress, adjust timeline if needed
4. **Week 8**: Resume bash purification roadmap with working parser

## References

- Bash Manual: https://www.gnu.org/software/bash/manual/
- Current parser: `rash/src/bash_parser/`
- Benchmark fixtures: `rash/benches/fixtures/`
- Purification roadmap: GitHub issue or docs/ROADMAP.yaml
- Related: BASH-INGESTION-ROADMAP.yaml (Workflow 2 validation)

---

**Created**: 2025-01-10
**Last Updated**: 2025-01-10
**Status**: Awaiting approval to begin Phase 1
