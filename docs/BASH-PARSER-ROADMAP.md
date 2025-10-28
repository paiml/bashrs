# Bash Parser Improvement Roadmap

**Goal**: Expand bash parser to support all common bash constructs for formatter and purifier

**Current Status** (v6.15.0): Basic support (assignments, commands, simple control flow)

**Target** (v6.20.0): Comprehensive bash support for formatter (15/15 tests passing)

---

## Current Parser Capabilities (v6.15.0)

### ✅ Fully Supported
- **Assignments**: `VAR=value`, `VAR="string"`, `VAR=$OTHER`
- **Exports**: `export VAR=value`
- **Commands**: `echo hello`, `cd /path`, `ls -la`
- **Variable References**: `$VAR`, `${VAR}`, quoted variables
- **Comments**: `# comment` (basic, no positioning)
- **Shebangs**: `#!/bin/bash`, `#!/bin/sh`

### ⏳ Partial Support
- **Test Expressions**: Parser has infrastructure but doesn't handle all operators
- **Control Flow**: Basic if/while/for parsing but incomplete
- **Functions**: Declaration parsing only, not full bodies

### ❌ Not Yet Supported
- **Test Operators**: `-n`, `-z`, `-f`, `-d`, `-r`, `-w`, `-x`, `-e`
- **Case Statements**: Complete absence
- **Heredocs**: Not implemented
- **Process Substitution**: `<()`, `>()`
- **Array Operations**: `arr[0]`, `${arr[@]}`
- **Advanced Expansions**: Many parameter expansions missing

---

## Sprint Plan: v6.16.0 - v6.20.0

### Sprint 1: v6.16.0 - Test Expressions (Week 1)

**Goal**: Full support for `[ ]` and `[[ ]]` test expressions

**Tasks**:
1. **Lexer improvements**:
   - Recognize test operators as tokens
   - Handle `-n`, `-z`, `-f`, `-d`, `-r`, `-w`, `-x`, `-e`
   - Handle binary operators: `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`

2. **Parser improvements**:
   - Improve `parse_test_condition()` to handle unary operators
   - Support `[ -n "$VAR" ]` syntax
   - Support `[[ $VAR ]]` syntax
   - Handle logical operators: `&&`, `||`, `!`

3. **Formatter improvements**:
   - Format test expressions consistently
   - Preserve operator spacing
   - Quote variables in tests

**Tests Unblocked**:
- test_format_002 (basic formatting with if)
- test_format_003 (check mode)
- test_format_004 (check mode formatted)
- test_format_007 (if statements)

**Outcome**: 9/15 tests passing

---

### Sprint 2: v6.17.0 - Case Statements (Week 2)

**Goal**: Full support for case statement parsing and formatting

**Tasks**:
1. **Lexer improvements**:
   - Recognize `case`, `esac` keywords
   - Handle pattern matching: `*`, `?`, `[a-z]`
   - Support `;;`, `;;&`, `;&` terminators

2. **Parser improvements**:
   - Add `parse_case()` function
   - Parse case patterns and bodies
   - Handle multiple patterns per case

3. **Formatter improvements**:
   - Consistent case indentation
   - Pattern alignment
   - Body formatting

**Tests Unblocked**:
- test_format_011 (case statements)

**Outcome**: 10/15 tests passing

---

### Sprint 3: v6.18.0 - Comment Preservation (Week 3)

**Goal**: Preserve comments with correct positioning during formatting

**Tasks**:
1. **AST improvements**:
   - Add comment positioning metadata
   - Track inline vs standalone comments
   - Associate comments with statements

2. **Parser improvements**:
   - Attach comments to AST nodes
   - Preserve comment indentation context
   - Handle multi-line comments

3. **Formatter improvements**:
   - Output comments at correct positions
   - Preserve inline comment spacing
   - Handle comment blocks

**Tests Unblocked**:
- test_format_008 (preserve comments)

**Outcome**: 11/15 tests passing

---

### Sprint 4: v6.19.0 - Configuration Loading (Week 4)

**Goal**: Load and apply .bashrs-fmt.toml configuration

**Tasks**:
1. **Config file discovery**:
   - Search for `.bashrs-fmt.toml` in current dir
   - Search parent directories
   - Support explicit `--config` flag

2. **Config application**:
   - Apply indent_width setting
   - Apply use_tabs setting
   - Apply all formatter options

3. **Ignore directives**:
   - Recognize `# bashrs-fmt-ignore` comments
   - Skip formatting for ignored blocks
   - Handle ignore ranges

**Tests Unblocked**:
- test_format_009 (tabs config)
- test_format_010 (ignore directive)
- test_format_015 (indent width)

**Outcome**: 14/15 tests passing

---

### Sprint 5: v6.20.0 - Function Bodies & Polish (Week 5)

**Goal**: Complete function parsing and final formatter polish

**Tasks**:
1. **Function improvements**:
   - Parse complete function bodies
   - Handle nested functions
   - Format function definitions

2. **Final polish**:
   - Fix test_format_006 (normalize functions)
   - Fix test_format_014 (dry-run mode)
   - Comprehensive integration testing

3. **Documentation**:
   - Update all docs with complete capabilities
   - Add comprehensive examples
   - CI/CD integration guide

**Tests Unblocked**:
- test_format_006 (normalize functions)
- test_format_014 (dry-run)

**Outcome**: 15/15 tests passing ✅

---

## Implementation Strategy

### EXTREME TDD Process

For each parser improvement:

1. **RED**: Write failing test for bash construct
2. **GREEN**: Implement parser support
3. **REFACTOR**: Clean up implementation
4. **REPL**: Verify in interactive REPL
5. **PROPERTY**: Add property-based tests
6. **MUTATION**: Verify test quality (≥90% kill rate)

### Quality Gates

Every sprint must maintain:
- ✅ Zero regressions (all existing tests pass)
- ✅ Complexity <10 for all functions
- ✅ Test coverage >85%
- ✅ Mutation score >90%
- ✅ All formatted output passes shellcheck

---

## Success Metrics

### v6.16.0
- 9/15 formatter tests passing (60%)
- All test expression formats supported
- <100 LOC added to parser

### v6.17.0
- 10/15 formatter tests passing (67%)
- All case statement formats supported
- <150 LOC added to parser

### v6.18.0
- 11/15 formatter tests passing (73%)
- Comments preserved with positioning
- <100 LOC added to parser

### v6.19.0
- 14/15 formatter tests passing (93%)
- Config file loading works
- Ignore directives functional

### v6.20.0
- 15/15 formatter tests passing (100%) ✅
- Comprehensive bash support
- Full formatter feature parity

---

## Risk Mitigation

**Risk**: Parser changes break existing functionality
**Mitigation**: Comprehensive regression testing, incremental changes

**Risk**: Performance degradation
**Mitigation**: Benchmark after each sprint, maintain <100ms target

**Risk**: Scope creep
**Mitigation**: Strict sprint boundaries, focus on formatter needs only

---

## Post-v6.20.0 Goals

Once formatter is complete, parser improvements enable:

1. **Better bash purification** (determinism + idempotency)
2. **Advanced linting rules** (control flow analysis)
3. **Bash → Rust transpilation** (legacy migration)
4. **Static analysis tools** (security scanning)

The parser is **foundational infrastructure** that benefits multiple features.

---

**Status**: Roadmap defined, ready to execute
**Start Date**: After v6.15.0 release
**Target Completion**: 5 weeks
**Committed**: Full formatter support by v6.20.0
