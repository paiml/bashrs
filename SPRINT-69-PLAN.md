# Sprint 69 Plan: Makefile Purification CLI Integration

**Sprint ID**: SPRINT-69
**Type**: Feature Implementation (CLI Integration)
**Estimated Duration**: 4-6 hours
**Status**: PLANNING
**Date Created**: 2025-10-18
**Follows**: Sprint 68 (Code Generation Complete)

---

## Executive Summary

Sprint 68 delivered the complete end-to-end Makefile purification pipeline:
```
Input Makefile → Parse → AST → Analyze → Purify → Generate → Purified Makefile ✅
```

**However**, this functionality is only accessible programmatically through library functions. Sprint 69 will add CLI integration to make Makefile purification available as command-line tools, enabling users to:

1. Parse Makefiles and view their AST
2. Purify Makefiles with deterministic, idempotent transformations
3. Get detailed transformation reports
4. Integrate with CI/CD pipelines

---

## Sprint Goal

**Add Makefile purification commands to the `bashrs` CLI**, following EXTREME TDD methodology and `assert_cmd` testing patterns from CLAUDE.md.

---

## Success Criteria

### Functional Requirements
- [x] CLI subcommand structure defined
- [ ] `bashrs make parse <file>` - Parse Makefile to AST
- [ ] `bashrs make purify <file>` - Purify Makefile (analyze and report)
- [ ] `bashrs make purify --fix <file>` - Auto-apply transformations
- [ ] `bashrs make purify --fix -o <output> <input>` - Output to new file
- [ ] `bashrs make purify --report <file>` - Detailed transformation report
- [ ] Error handling for invalid Makefiles
- [ ] File I/O (read input, write output, create backups)

### Quality Requirements
- [ ] All CLI tests use `assert_cmd` (MANDATORY per CLAUDE.md)
- [ ] Test naming follows `test_<TASK_ID>_<feature>_<scenario>` convention
- [ ] ≥6 unit tests for CLI arg parsing
- [ ] ≥4 integration tests for end-to-end workflows
- [ ] 100% test pass rate
- [ ] Zero regressions (all 1,418 existing tests still pass)
- [ ] Code complexity <10
- [ ] Mutation testing ≥90% kill rate on new CLI code

### Documentation Requirements
- [ ] Sprint plan (this document)
- [ ] Handoff document with examples
- [ ] Quick reference card
- [ ] Updated README with CLI usage
- [ ] Session summary

---

## CLI Design

### Subcommand Structure

Add `Make` subcommand to `Commands` enum in `rash/src/cli/args.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Makefile parsing, purification, and transformation
    Make {
        #[command(subcommand)]
        command: MakeCommands,
    },
}

#[derive(Subcommand)]
pub enum MakeCommands {
    /// Parse Makefile to AST
    Parse {
        /// Input Makefile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: MakeOutputFormat,
    },

    /// Purify Makefile (determinism + idempotency)
    Purify {
        /// Input Makefile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to stdout or in-place with --fix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Apply fixes in-place (creates .bak backup)
        #[arg(long)]
        fix: bool,

        /// Show detailed transformation report
        #[arg(long)]
        report: bool,

        /// Report format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum MakeOutputFormat {
    /// Human-readable text
    Text,
    /// JSON AST
    Json,
    /// Debug format
    Debug,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ReportFormat {
    /// Human-readable report
    Human,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
}
```

### Usage Examples

```bash
# Parse Makefile and show AST
bashrs make parse Makefile

# Parse and output JSON AST
bashrs make parse Makefile --format json

# Purify Makefile (dry-run, show what would change)
bashrs make purify Makefile

# Purify with detailed report
bashrs make purify Makefile --report

# Apply purification in-place (creates Makefile.bak)
bashrs make purify --fix Makefile

# Purify and write to new file
bashrs make purify --fix -o Makefile.purified Makefile

# Purify with JSON report
bashrs make purify Makefile --report --format json
```

---

## EXTREME TDD Workflow

### Phase 1: RED (Write Failing Tests) - 1 hour

**RED-001: Parse command**
```rust
#[test]
fn test_CLI_MAKE_001_parse_basic_makefile() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    fn rash_cmd() -> Command {
        Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
    }

    let makefile = "tests/fixtures/simple.mk";
    std::fs::write(makefile, "target:\n\techo hello").unwrap();

    rash_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target"));

    let _ = std::fs::remove_file(makefile);
}
```

**RED-002: Purify command (dry-run)**
```rust
#[test]
fn test_CLI_MAKE_002_purify_dry_run() {
    let makefile = "tests/fixtures/wildcard.mk";
    std::fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    rash_cmd()
        .arg("make")
        .arg("purify")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("$(sort $(wildcard"));

    // Verify original file unchanged
    let content = std::fs::read_to_string(makefile).unwrap();
    assert_eq!(content, "FILES := $(wildcard *.c)");

    let _ = std::fs::remove_file(makefile);
}
```

**RED-003: Purify with --fix (in-place)**
```rust
#[test]
fn test_CLI_MAKE_003_purify_fix_inplace() {
    let makefile = "tests/fixtures/wildcard_fix.mk";
    std::fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    rash_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg(makefile)
        .assert()
        .success();

    // Verify file changed
    let content = std::fs::read_to_string(makefile).unwrap();
    assert!(content.contains("$(sort $(wildcard"));

    // Verify backup created
    let backup = format!("{}.bak", makefile);
    assert!(std::path::Path::new(&backup).exists());

    let _ = std::fs::remove_file(makefile);
    let _ = std::fs::remove_file(&backup);
}
```

**RED-004: Purify with -o (output to new file)**
```rust
#[test]
fn test_CLI_MAKE_004_purify_output_file() {
    let input = "tests/fixtures/wildcard_input.mk";
    let output = "tests/fixtures/wildcard_output.mk";
    std::fs::write(input, "FILES := $(wildcard *.c)").unwrap();

    rash_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(output)
        .arg(input)
        .assert()
        .success();

    // Verify output file created
    let content = std::fs::read_to_string(output).unwrap();
    assert!(content.contains("$(sort $(wildcard"));

    // Verify input file unchanged
    let input_content = std::fs::read_to_string(input).unwrap();
    assert_eq!(input_content, "FILES := $(wildcard *.c)");

    let _ = std::fs::remove_file(input);
    let _ = std::fs::remove_file(output);
}
```

**RED-005: Purify with --report**
```rust
#[test]
fn test_CLI_MAKE_005_purify_report() {
    let makefile = "tests/fixtures/wildcard_report.mk";
    std::fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    rash_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied: 1"))
        .stdout(predicate::str::contains("wildcard"));

    let _ = std::fs::remove_file(makefile);
}
```

**RED-006: Error handling (invalid Makefile)**
```rust
#[test]
fn test_CLI_MAKE_006_parse_invalid_makefile() {
    let makefile = "tests/fixtures/invalid.mk";
    std::fs::write(makefile, "target: : :\n").unwrap();

    rash_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));

    let _ = std::fs::remove_file(makefile);
}
```

**Verification**: Run `cargo test --test cli_tests` - all 6 tests should FAIL ❌

---

### Phase 2: GREEN (Implement Functionality) - 2 hours

**GREEN-001: Update `rash/src/cli/args.rs`**
- Add `Make` subcommand to `Commands` enum
- Add `MakeCommands` enum with `Parse` and `Purify` variants
- Add `MakeOutputFormat` and `ReportFormat` enums

**GREEN-002: Implement CLI handlers in `rash/src/cli/commands.rs`**
- Add `handle_make_command()` function
- Implement `make_parse_command()`
- Implement `make_purify_command()`
- Add file I/O helpers
- Add backup file creation

**GREEN-003: Wire up in `execute_command()`**
```rust
match cli.command {
    // ... existing commands ...
    Commands::Make { command } => {
        handle_make_command(command, cli.verbose)
    }
}
```

**GREEN-004: Implement parse command**
```rust
fn make_parse_command(input: &Path, format: MakeOutputFormat) -> Result<()> {
    use crate::make_parser::parser::parse_makefile;

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)?;

    match format {
        MakeOutputFormat::Text => {
            println!("{:#?}", ast);
        }
        MakeOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&ast)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{}", json);
        }
        MakeOutputFormat::Debug => {
            println!("{:?}", ast);
        }
    }

    Ok(())
}
```

**GREEN-005: Implement purify command**
```rust
fn make_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    report: bool,
    format: ReportFormat,
) -> Result<()> {
    use crate::make_parser::{
        parser::parse_makefile,
        purify::purify_makefile,
        generators::generate_purified_makefile,
    };

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let ast = parse_makefile(&source)?;
    let purify_result = purify_makefile(&ast);

    if report {
        // Print transformation report
        print_purify_report(&purify_result, format);
    }

    if fix {
        let purified = generate_purified_makefile(&purify_result.ast);

        if let Some(output_path) = output {
            // Write to specified output file
            fs::write(output_path, purified).map_err(Error::Io)?;
            info!("Purified Makefile written to {}", output_path.display());
        } else {
            // In-place: create backup and overwrite
            let backup_path = input.with_extension("mk.bak");
            fs::copy(input, &backup_path).map_err(Error::Io)?;
            fs::write(input, purified).map_err(Error::Io)?;
            info!("Purified Makefile written to {}", input.display());
            info!("Backup created at {}", backup_path.display());
        }
    } else {
        // Dry-run: print purified output to stdout
        let purified = generate_purified_makefile(&purify_result.ast);
        println!("{}", purified);
    }

    Ok(())
}

fn print_purify_report(result: &PurifyResult, format: ReportFormat) {
    match format {
        ReportFormat::Human => {
            println!("Makefile Purification Report");
            println!("============================");
            println!("Transformations Applied: {}", result.transformations_applied);
            for (i, transform) in result.transformations.iter().enumerate() {
                println!("{}: {} (line {})", i + 1, transform.kind, transform.location);
            }
        }
        ReportFormat::Json => {
            let json = serde_json::to_string_pretty(result).unwrap();
            println!("{}", json);
        }
        ReportFormat::Markdown => {
            println!("# Makefile Purification Report\n");
            println!("**Transformations**: {}\n", result.transformations_applied);
            for (i, transform) in result.transformations.iter().enumerate() {
                println!("{}. **{}** (line {})", i + 1, transform.kind, transform.location);
            }
        }
    }
}
```

**Verification**: Run `cargo test --test cli_tests` - all 6 tests should PASS ✅

---

### Phase 3: REFACTOR (Clean Up) - 30 minutes

**REFACTOR-001: Extract helpers**
- Extract `create_backup_file()` helper
- Extract `write_purified_output()` helper
- Extract `validate_makefile_path()` helper

**REFACTOR-002: Code cleanup**
- Ensure all functions have complexity <10
- Add comprehensive documentation
- Extract constants (e.g., `.bak` suffix)

**REFACTOR-003: Error handling**
- Use `thiserror` for custom errors
- Provide helpful error messages
- Test all error paths

**Verification**:
- Run `cargo clippy` - zero warnings
- Run `cargo test` - all tests pass
- Check complexity with `pmat analyze complexity`

---

### Phase 4: Property Testing - 1 hour

**PROPERTY-001: Round-trip property**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_CLI_MAKE_007_purify_roundtrip(
        wildcard_pattern in "[a-z/*.]{3,20}"
    ) {
        // Create Makefile with wildcard
        let makefile = format!("FILES := $(wildcard {})", wildcard_pattern);
        let path = "tests/fixtures/prop_test.mk";
        std::fs::write(path, &makefile).unwrap();

        // Purify to output file
        let output = "tests/fixtures/prop_test_purified.mk";
        rash_cmd()
            .arg("make")
            .arg("purify")
            .arg("--fix")
            .arg("-o")
            .arg(output)
            .arg(path)
            .assert()
            .success();

        // Verify purified version contains $(sort)
        let purified_content = std::fs::read_to_string(output).unwrap();
        prop_assert!(purified_content.contains("$(sort $(wildcard"));

        // Cleanup
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(output);
    }
}
```

**PROPERTY-002: Idempotency property**
```rust
proptest! {
    #[test]
    fn prop_CLI_MAKE_008_purify_idempotent(
        target_name in "[a-z]{3,10}"
    ) {
        // Create simple Makefile
        let makefile = format!(".PHONY: {}\n{}:\n\techo test", target_name, target_name);
        let path = "tests/fixtures/idempotent.mk";
        std::fs::write(path, &makefile).unwrap();

        // First purification
        let output1 = "tests/fixtures/idempotent_1.mk";
        rash_cmd()
            .arg("make")
            .arg("purify")
            .arg("--fix")
            .arg("-o")
            .arg(output1)
            .arg(path)
            .assert()
            .success();

        // Second purification
        let output2 = "tests/fixtures/idempotent_2.mk";
        rash_cmd()
            .arg("make")
            .arg("purify")
            .arg("--fix")
            .arg("-o")
            .arg(output2)
            .arg(output1)
            .assert()
            .success();

        // Verify byte-identical
        let content1 = std::fs::read_to_string(output1).unwrap();
        let content2 = std::fs::read_to_string(output2).unwrap();
        prop_assert_eq!(&content1, &content2);

        // Cleanup
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(output1);
        let _ = std::fs::remove_file(output2);
    }
}
```

**Verification**: Run `cargo test` - 100+ property test cases pass

---

### Phase 5: Integration Testing - 1 hour

**INTEGRATION-001: End-to-end purification workflow**
```rust
#[test]
fn test_CLI_MAKE_009_integration_full_workflow() {
    // ARRANGE: Create complex Makefile with multiple issues
    let input = "tests/fixtures/integration_complex.mk";
    std::fs::write(input, r#"
# Build configuration
CC := gcc
CFLAGS := -O2 -Wall

# Non-deterministic wildcard
SOURCES := $(wildcard src/*.c)
OBJECTS := $(wildcard src/*.o)

# Non-deterministic timestamp
RELEASE := $(shell date +%s)

.PHONY: build clean

build: $(OBJECTS)
	$(CC) $(CFLAGS) -o myapp $(OBJECTS)

clean:
	rm -f $(OBJECTS) myapp
"#).unwrap();

    // ACT: Parse, purify, verify

    // Step 1: Parse should succeed
    rash_cmd()
        .arg("make")
        .arg("parse")
        .arg(input)
        .assert()
        .success();

    // Step 2: Purify with report
    let output = "tests/fixtures/integration_purified.mk";
    rash_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("--report")
        .arg("-o")
        .arg(output)
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied:"));

    // ASSERT: Verify purified output
    let purified_content = std::fs::read_to_string(output).unwrap();

    // Wildcard wrapped with $(sort)
    assert!(purified_content.contains("$(sort $(wildcard src/*.c))"));
    assert!(purified_content.contains("$(sort $(wildcard src/*.o))"));

    // Timestamp removed or replaced
    assert!(!purified_content.contains("$(shell date"));

    // Structure preserved
    assert!(purified_content.contains(".PHONY: build clean"));
    assert!(purified_content.contains("build:"));
    assert!(purified_content.contains("clean:"));

    // Re-parse purified Makefile
    rash_cmd()
        .arg("make")
        .arg("parse")
        .arg(output)
        .assert()
        .success();

    // Cleanup
    let _ = std::fs::remove_file(input);
    let _ = std::fs::remove_file(output);
}
```

**Verification**: Integration test passes, validates entire workflow

---

## Timeline

| Phase | Duration | Tasks | Status |
|-------|----------|-------|--------|
| **Phase 1: RED** | 1 hour | Write 6 failing CLI tests | PENDING |
| **Phase 2: GREEN** | 2 hours | Implement CLI commands | PENDING |
| **Phase 3: REFACTOR** | 30 min | Clean up, extract helpers | PENDING |
| **Phase 4: PROPERTY** | 1 hour | Add property tests | PENDING |
| **Phase 5: INTEGRATION** | 1 hour | End-to-end testing | PENDING |
| **Documentation** | 30 min | Handoff, QRC, README | PENDING |
| **TOTAL** | **6 hours** | **All phases** | **READY** |

---

## Files to Create/Modify

### New Files
1. `rash/tests/cli_make_tests.rs` - CLI integration tests
2. `SPRINT-69-HANDOFF.md` - Sprint completion handoff
3. `SPRINT-69-QRC.md` - Quick reference card

### Modified Files
1. `rash/src/cli/args.rs` - Add `Make` subcommand
2. `rash/src/cli/commands.rs` - Implement `make_parse_command()`, `make_purify_command()`
3. `rash/Cargo.toml` - Ensure `assert_cmd` and `predicates` in dev-dependencies (already present)
4. `README.md` - Add CLI usage examples
5. `ROADMAP.yaml` - Update with Sprint 69 status

---

## Quality Gates

Before marking Sprint 69 as complete, ALL must be ✅:

- [ ] **RED Phase**: 6 failing tests written and verified
- [ ] **GREEN Phase**: All 6 tests pass
- [ ] **REFACTOR Phase**: Complexity <10, clippy warnings = 0
- [ ] **Property Tests**: 2 property tests with 100+ cases each
- [ ] **Integration Test**: 1 end-to-end workflow test passes
- [ ] **All Tests Pass**: 1,418 + 9 = 1,427 tests (100% pass rate)
- [ ] **Zero Regressions**: All existing tests still pass
- [ ] **CLI Testing**: All tests use `assert_cmd` (MANDATORY)
- [ ] **Test Naming**: All tests follow `test_<TASK_ID>_<feature>_<scenario>`
- [ ] **Documentation**: Handoff, QRC, session summary complete
- [ ] **Commits**: All work committed with proper attribution

---

## Dependencies

### Prerequisite Sprints
- ✅ Sprint 67 Phase 2 (Property tests + idempotency)
- ✅ Sprint 68 (Code generation complete)

### Required Modules
- ✅ `make_parser::parser` (parse_makefile)
- ✅ `make_parser::purify` (purify_makefile)
- ✅ `make_parser::generators` (generate_purified_makefile)
- ✅ `clap` (CLI argument parsing)
- ✅ `assert_cmd` (CLI testing - already in dev-dependencies)
- ✅ `predicates` (CLI test assertions - already in dev-dependencies)

---

## Risks and Mitigations

### Risk 1: Binary name confusion
**Issue**: Project uses `bashrs` binary but documentation mentions `rash`
**Mitigation**: Use `Command::cargo_bin("bashrs")` in tests, clarify in docs

### Risk 2: File I/O errors
**Issue**: Missing files, permission errors, disk full
**Mitigation**: Comprehensive error handling, helpful error messages

### Risk 3: Backup file conflicts
**Issue**: `.bak` file already exists
**Mitigation**: Use timestamped backups or warn user

---

## Next Steps After Sprint 69

1. **Sprint 70**: Add `bashrs make lint` for Makefile linting
2. **Sprint 71**: Add `bashrs make check` for Makefile validation
3. **Sprint 72**: Add `bashrs make transpile` for Makefile → Rust conversion
4. **Sprint 73**: Performance optimization for large Makefiles
5. **Sprint 74**: CI/CD integration examples

---

## Appendix: Test File Structure

```
rash/tests/
├── cli_make_tests.rs         # CLI integration tests
├── fixtures/
│   ├── simple.mk
│   ├── wildcard.mk
│   ├── wildcard_fix.mk
│   ├── wildcard_input.mk
│   ├── wildcard_output.mk
│   ├── wildcard_report.mk
│   ├── invalid.mk
│   ├── integration_complex.mk
│   ├── integration_purified.mk
│   └── prop_test.mk
```

---

**Sprint Status**: READY TO EXECUTE
**Methodology**: EXTREME TDD (RED-GREEN-REFACTOR-PROPERTY-INTEGRATION)
**Quality Target**: 100% test pass rate, zero regressions, ≥90% mutation kill rate
**Estimated Completion**: 6 hours from start
