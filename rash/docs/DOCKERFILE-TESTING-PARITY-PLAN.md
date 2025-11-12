# Dockerfile Testing Parity Implementation Plan

## Executive Summary

**Goal**: Achieve testing parity between Dockerfile and Makefile/script.sh transformations

**Current State Snapshot** (as of November 11, 2025):
- **Dockerfile**: 16 CLI tests, ~75% coverage, 0 property tests, 0 mutation tests
- **Makefile**: Comprehensive unit + property + CLI tests, ~88% coverage, mutation testing in progress
- **script.sh**: 6004+ tests, property + mutation + coverage >85%

**Target State**:
- **Dockerfile**: 50+ CLI tests, >85% coverage, 100+ property test cases, >90% mutation kill rate
- **Unified Quality**: Property testing, mutation testing, edge case coverage across all three

**Estimated Effort**: 280-320 hours (8-10 weeks at full-time development)
**Priority**: P2 (after script.sh completion, before v7.0 release)

---

## Current Testing Landscape

### Dockerfile (INCOMPLETE)

**File Locations**:
- CLI Tests: `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs` (529 lines, 16 tests)
- Property Tests: `/home/noah/src/bashrs/rash/tests/property_dockerfile_purify.rs` (454 lines, 10 proptest blocks + 4 edge case tests)
- Source: `/home/noah/src/bashrs/rash/src/linter/rules/docker*.rs` (5 rule files)

**Current Coverage Analysis**:
```
Dockerfile CLI Tests (16 tests):
├── Command existence (1 test)
├── Basic purification (2 tests)
├── DOCKER001 - Add USER (3 tests)
├── DOCKER002 - Pin base images (2 tests)
├── DOCKER003 - Add cleanup (2 tests)
├── DOCKER005 - --no-install-recommends (1 test)
├── DOCKER006 - ADD → COPY (2 tests)
└── CLI options (1 test)

Property Tests: 14 tests
├── Determinism (1 proptest block)
├── Idempotency (1 proptest block)
├── Preservation properties (4 blocks)
├── Edge case tests (4 tests)
└── Stress tests (3 tests)

Coverage Gap: ~75% (estimate)
- Missing: Error handling, recovery, edge cases
- Missing: Integration with other rules
- Missing: Performance characteristics
- Missing: CLI flag combinations
```

### Makefile (REFERENCE)

**File Locations**:
- CLI Tests: `/home/noah/src/bashrs/rash/tests/cli_make_*.rs` (multiple files)
- Unit Tests: `/home/noah/src/bashrs/rash/src/make_parser/tests.rs`
- Property Tests: `/home/noah/src/bashrs/rash/tests/make_formatting_property_tests.rs`
- Coverage: ~88%

**Makefile Test Pattern**:
```
Unit Tests (inline):
├── Lexer tests (20+)
├── Parser tests (50+)
├── Transformer tests (30+)
└── Emitter tests (25+)

CLI Tests:
├── Basic operations (10 tests)
├── Error handling (8 tests)
├── Flag combinations (12 tests)
└── Integration scenarios (15 tests)

Property Tests:
├── Determinism (1 block, 100 cases)
├── Idempotency (1 block, 100 cases)
├── Preservation properties (5 blocks, 500 cases)
└── Syntax validity (1 block, 100 cases)
```

### script.sh (GOLD STANDARD)

**Test Infrastructure**:
- 6004+ total tests
- Property-based testing: 100+ cases per property
- Mutation testing: >90% kill rate
- Coverage: >85% across all modules
- EXTREME TDD for all components
- assert_cmd for all CLI tests

**Key Patterns to Follow**:
1. RED → GREEN → REFACTOR cycle for each feature
2. Property tests covering 100+ cases
3. Mutation testing integrated into CI/CD
4. Edge case cataloging and testing
5. Integration tests alongside unit tests

---

## Phase 1: Test Infrastructure Enhancement (Weeks 1-2, 40 hours)

### Phase 1.1: Extend CLI Tests (20 hours)

**Objectives**:
- Expand from 16 to 35+ CLI tests
- Cover all Docker transformations (DOCKER001-DOCKER010)
- Add error handling paths
- Add flag combination tests

**EXTREME TDD Approach**:

```rust
// RED: Write failing tests first

#[test]
fn test_DOCKER_001_cli_user_directive_basic() {
    // ARRANGE: Create test Dockerfile without USER
    let dockerfile = "FROM debian:12\nCMD [\"bash\"]";
    let temp_file = write_temp_dockerfile(dockerfile);
    
    // ACT: Run purify command
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("USER"));
    
    // ASSERT: Verify user creation RUN command added
    // (This test should FAIL before implementation)
}
```

**New Tests to Add** (19 additional):

1. **DOCKER001 - User Directive Tests (5 new)**
   - test_DOCKER_001_user_in_layers (nested RUNs)
   - test_DOCKER_001_user_with_sudo (conditional execution)
   - test_DOCKER_001_user_permissions (special dirs)
   - test_DOCKER_001_user_combined_with_other_fixes (integration)
   - test_DOCKER_001_user_idempotency (running twice)

2. **DOCKER002 - Base Image Pinning (4 new)**
   - test_DOCKER_002_pin_multi_stage_builds
   - test_DOCKER_002_pin_sha256_digests (already pinned)
   - test_DOCKER_002_pin_registry_prefixes
   - test_DOCKER_002_pin_with_args (ARG in FROM)

3. **DOCKER003/004 - Cleanup (3 new)**
   - test_DOCKER_003_cleanup_combined_with_001
   - test_DOCKER_004_health_check_combinations
   - test_DOCKER_003_cleanup_already_present

4. **DOCKER005 - Package Manager Flags (3 new)**
   - test_DOCKER_005_yum_equivalent_flags
   - test_DOCKER_005_pip_upgrade_combinations
   - test_DOCKER_005_flag_order_preservation

5. **DOCKER006 - ADD to COPY (2 new)**
   - test_DOCKER_006_add_to_copy_with_wildcards
   - test_DOCKER_006_add_checksum_preserved

6. **CLI Flag Tests (4 new)**
   - test_DOCKER_CLI_010_dry_run_multiple_files
   - test_DOCKER_CLI_011_fix_with_backup_verification
   - test_DOCKER_CLI_012_quiet_flag
   - test_DOCKER_CLI_013_json_output

**Effort Breakdown**:
- Test design and setup: 8 hours
- Implementation per test: 0.5 hours × 19 = 9.5 hours
- Documentation and validation: 2.5 hours

**Success Criteria**:
- All 35 CLI tests written in RED phase
- assert_cmd pattern followed for all tests
- Test naming convention: `test_DOCKER_<NUMBER>_<feature>_<scenario>`
- All tests runnable (not necessarily passing)

### Phase 1.2: Property Test Generation (15 hours)

**Objectives**:
- Expand property tests from 10 to 20+ blocks
- Generate 100+ test cases per property
- Add edge case generators

**EXTREME TDD Approach**:

```rust
// RED: Define properties that MUST hold

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // Minimum for property tests
        max_shrink_iters: 100,
        .. ProptestConfig::default()
    })]
    
    #[test]
    fn prop_DOCKER_001_user_always_before_cmd(
        dockerfile in dockerfile_with_user_and_cmd()
    ) {
        // ASSERT: USER must come before CMD
        let purified = purify_dockerfile(&dockerfile);
        let user_pos = purified.find("USER").unwrap_or(usize::MAX);
        let cmd_pos = purified.find("CMD").unwrap_or(usize::MAX);
        prop_assert!(user_pos < cmd_pos);
    }
}
```

**New Property Blocks** (10 additional):

1. **Ordering Properties (3)**
   - prop_DOCKER_user_before_cmd
   - prop_DOCKER_cleanup_after_install
   - prop_DOCKER_healthcheck_after_expose

2. **Semantic Preservation (4)**
   - prop_DOCKER_image_digest_unchanged
   - prop_DOCKER_workdir_preserved
   - prop_DOCKER_labels_preserved
   - prop_DOCKER_env_vars_preserved

3. **Transformation Properties (3)**
   - prop_DOCKER_add_to_copy_only_for_local
   - prop_DOCKER_flags_only_when_needed
   - prop_DOCKER_multi_stage_independence

**Test Generators to Add** (5 new):

```rust
/// Generate Dockerfiles with multiple RUN commands
fn dockerfile_with_multiple_runs() -> impl Strategy<Value = String> { ... }

/// Generate Dockerfiles with multi-stage builds
fn dockerfile_multi_stage() -> impl Strategy<Value = String> { ... }

/// Generate Dockerfiles with complex FROM expressions
fn dockerfile_with_args() -> impl Strategy<Value = String> { ... }

/// Generate edge-case Dockerfiles (malformed but recognizable)
fn dockerfile_edge_cases() -> impl Strategy<Value = String> { ... }

/// Generate intentionally vulnerable Dockerfiles
fn dockerfile_vulnerable_patterns() -> impl Strategy<Value = String> { ... }
```

**Effort Breakdown**:
- Property definition: 5 hours
- Generator implementation: 6 hours
- Validation and refinement: 4 hours

**Success Criteria**:
- 20 proptest blocks written
- Each block generates 100+ test cases (configurable)
- All generators compile and produce valid Dockerfiles
- Properties all FAIL initially (RED phase)

### Phase 1.3: Edge Case Catalog (5 hours)

**Objectives**:
- Document all edge cases and boundary conditions
- Create test matrix for systematic coverage
- Link to EXTREME TDD requirements

**Edge Cases to Document**:

```markdown
## DOCKER001 (User Directive)
- [x] No FROM - recovery
- [x] FROM scratch - skip
- [ ] Existing USER - preserve
- [ ] USER in COPY --chown - interaction
- [ ] RUN with /root access - needs sudo
- [ ] Alpine (busybox useradd) - different syntax
- [ ] Multi-stage build - user in each stage?

## DOCKER002 (Pin Images)
- [x] Untagged image (FROM ubuntu)
- [x] :latest tag (FROM debian:latest)
- [ ] Digest-only (FROM ubuntu@sha256:...)
- [ ] Registry prefix (FROM gcr.io/ubuntu)
- [ ] ARG in FROM (FROM ${BASE_IMAGE})
- [ ] Multi-stage with different bases
- [ ] FROM alias (FROM base AS builder)

## DOCKER003 (Cleanup)
- [x] apt-get install → add cleanup
- [x] apk add → add cleanup
- [ ] Multi-line RUN with cleanup
- [ ] Already-clean RUN - no change
- [ ] yum/dnf/pacman - different syntax
- [ ] pip install - no cleanup needed
- [ ] Combined with other fixes

## DOCKER005 (Package Flags)
- [x] apt-get install missing flag
- [ ] Already has flag - no change
- [ ] apt install (shorthand) - needs flag
- [ ] Apt-get with conditional logic
- [ ] Non-apt managers (yum, apk)

## DOCKER006 (ADD to COPY)
- [x] Local files - convert
- [x] Remote URLs - preserve
- [ ] Glob patterns - handle correctly
- [ ] With --chown - preserve flag
- [ ] Tar extraction URLs - special case
```

**Effort Breakdown**:
- Research and cataloging: 2.5 hours
- Test matrix creation: 1.5 hours
- Documentation: 1 hour

**Success Criteria**:
- 40+ edge cases documented
- Each mapped to test requirement
- Created test matrix (rows = transformations, cols = conditions)

---

## Phase 2: Unit Test Expansion (Weeks 2-3, 60 hours)

### Phase 2.1: Docker Rule Unit Tests (40 hours)

**Objectives**:
- Expand inline unit tests in docker*.rs files
- Target >85% code coverage per rule
- Test all code paths (normal + error)

**Current Coverage Gap**:

```
docker001.rs: ~70% coverage
├── Identified gaps:
│   ├── Error path for invalid group names
│   ├── Special case: getent group check
│   ├── Multiple CMD/ENTRYPOINT handling
│   └── Comment preservation

docker002.rs: ~75% coverage
├── Identified gaps:
│   ├── Registry-prefixed images
│   ├── Digest+tag combinations
│   ├── Version mapping logic edge cases
│   └── Unknown image defaults

docker003.rs: ~65% coverage
├── Identified gaps:
│   ├── Multi-line RUN parsing
│   ├── Shell script in RUN
│   ├── Cleanup already present (no-op)
│   └── Different cleanup commands
```

**EXTREME TDD Pattern**:

```rust
// In docker001.rs

#[cfg(test)]
mod tests {
    use super::*;

    // RED: Write failing test
    #[test]
    fn test_docker001_user_group_name_validation() {
        let input = "FROM debian:12\nRUN useradd baduser-123!\nCMD [\"bash\"]";
        // Should fail due to invalid group name
        let result = purify_with_docker001(input);
        assert!(result.is_err() || result.unwrap().contains("Invalid user"));
    }
    
    // Test coverage matrix
    #[rstest]
    #[case("FROM ubuntu", true)]                    // Untagged
    #[case("FROM ubuntu:22.04", false)]             // Already pinned
    #[case("FROM ubuntu:latest", true)]             // Latest tag
    #[case("FROM gcr.io/ubuntu", true)]             // Registry prefix
    #[case("FROM ubuntu@sha256:abc", false)]        // Digest only
    fn test_docker002_pin_decision_matrix(
        #[case] from_line: &str,
        #[case] should_pin: bool
    ) {
        let dockerfile = format!("{}\nCMD [\"bash\"]", from_line);
        let result = purify_with_docker002(&dockerfile);
        let needs_pin = result.lines().any(|l| {
            l.contains("FROM") && !l.contains("${") && should_pin
        });
        assert_eq!(needs_pin, should_pin);
    }
}
```

**New Unit Tests Required** (per rule):

1. **docker001.rs additions** (10 new tests)
   - test_docker001_invalid_user_names
   - test_docker001_user_in_conditional
   - test_docker001_user_with_sudo_rules
   - test_docker001_user_home_directory
   - test_docker001_multi_stage_user_per_stage
   - test_docker001_user_with_volume_ownership
   - test_docker001_recovery_from_invalid_syntax
   - test_docker001_user_group_conflicts
   - test_docker001_skip_distroless_images
   - test_docker001_preserve_existing_root_runs

2. **docker002.rs additions** (12 new tests)
   - test_docker002_unknown_image_strategy
   - test_docker002_registry_credentials
   - test_docker002_sha256_digest_only
   - test_docker002_digest_plus_tag
   - test_docker002_version_mapping_cache
   - test_docker002_dev_vs_stable_channel
   - test_docker002_pre_release_handling
   - test_docker002_base_image_as_arg
   - test_docker002_custom_registries
   - test_docker002_mirror_redirection
   - test_docker002_pinning_idempotency
   - test_docker002_error_recovery

3. **docker003.rs additions** (8 new tests)
   - test_docker003_multiline_run_parsing
   - test_docker003_shell_script_in_run
   - test_docker003_cleanup_already_present
   - test_docker003_cleanup_partially_present
   - test_docker003_different_package_managers
   - test_docker003_run_with_conditions
   - test_docker003_cleanup_order
   - test_docker003_error_on_invalid_run

4. **docker004.rs (Health Checks)** (8 new tests)
   - test_docker004_healthcheck_only_once
   - test_docker004_healthcheck_replace_existing
   - test_docker004_healthcheck_interval_validation
   - test_docker004_healthcheck_timeout_validation
   - test_docker004_healthcheck_retries_validation
   - test_docker004_healthcheck_start_period
   - test_docker004_curl_vs_other_commands
   - test_docker004_shell_vs_exec_form

5. **docker005.rs additions** (8 new tests)
   - test_docker005_yum_equivalent
   - test_docker005_pip_no_cache
   - test_docker005_npm_production_flag
   - test_docker005_alpine_vs_debian
   - test_docker005_already_optimized
   - test_docker005_flag_order_preservation
   - test_docker005_complex_run_commands
   - test_docker005_security_flag_combination

6. **docker006.rs additions** (6 new tests)
   - test_docker006_glob_patterns
   - test_docker006_with_chown_flag
   - test_docker006_tar_archives
   - test_docker006_checksum_comments
   - test_docker006_whitespace_preservation
   - test_docker006_path_normalization

**Effort Breakdown**:
- Test design (coverage matrix): 12 hours
- Implementation: 24 hours (2 hours per rule)
- Validation and fixes: 4 hours

**Success Criteria**:
- All 52 new unit tests written (RED phase)
- Coverage tooling shows >85% per rule
- All code paths exercised
- Tests fail initially (RED)

### Phase 2.2: Integration Tests (20 hours)

**Objectives**:
- Test interactions between multiple Docker rules
- Test complete transformation pipeline
- Test CLI with combinations of flags

**Integration Scenarios**:

```rust
#[test]
fn test_DOCKER_integration_001_all_fixes_combined() {
    // Vulnerable Dockerfile with ALL issues:
    // - No USER
    // - Unpinned image
    // - No cleanup
    // - No --no-install-recommends
    // - Uses ADD for local files
    
    let dockerfile = r#"
FROM ubuntu
RUN apt-get update && apt-get install -y python3
ADD app.py /app/
CMD ["python3", "/app/app.py"]
    "#;
    
    let purified = purify_all(dockerfile);
    
    // ASSERT all fixes applied
    assert!(purified.contains("FROM ubuntu:22.04")); // DOCKER002
    assert!(purified.contains("--no-install-recommends")); // DOCKER005
    assert!(purified.contains("rm -rf /var/lib/apt/lists/*")); // DOCKER003
    assert!(purified.contains("USER")); // DOCKER001
    assert!(purified.contains("COPY app.py")); // DOCKER006
}

#[test]
fn test_DOCKER_integration_002_multi_stage_build() {
    // Multi-stage Dockerfile
    let dockerfile = r#"
FROM ubuntu AS builder
RUN apt-get update && apt-get install -y build-essential
COPY src /src
RUN cd /src && make

FROM ubuntu
COPY --from=builder /src/bin /app/bin
CMD ["/app/bin/main"]
    "#;
    
    let purified = purify_all(dockerfile);
    
    // Each stage should be independently fixed
    assert!(purified.contains("FROM ubuntu:22.04 AS builder"));
    assert!(purified.contains("FROM ubuntu:22.04")); // Second stage also pinned
}

#[test]
fn test_DOCKER_integration_003_idempotency_chain() {
    // Running purify 5 times should produce same result
    let mut current = dockerfile_input.to_string();
    
    for round in 1..=5 {
        let next = purify_all(&current);
        assert_eq!(current, next, "Round {} changed output", round);
        current = next;
    }
}
```

**Integration Test Categories**:

1. **Complete Pipeline Tests** (5 tests)
   - test_DOCKER_integration_001_all_fixes_combined
   - test_DOCKER_integration_002_multi_stage_build
   - test_DOCKER_integration_003_idempotency_chain
   - test_DOCKER_integration_004_no_double_application
   - test_DOCKER_integration_005_performance_large_file

2. **Cross-Rule Interaction** (6 tests)
   - test_DOCKER_interaction_001_002_pin_then_user
   - test_DOCKER_interaction_002_005_pin_with_flags
   - test_DOCKER_interaction_003_006_cleanup_with_copy
   - test_DOCKER_interaction_001_003_user_with_cleanup
   - test_DOCKER_interaction_004_005_health_check_ordering
   - test_DOCKER_interaction_all_priority_ordering

3. **Error Recovery** (3 tests)
   - test_DOCKER_integration_recovery_001_partial_fix
   - test_DOCKER_integration_recovery_002_invalid_input
   - test_DOCKER_integration_recovery_003_format_preservation

4. **Edge Case Combinations** (4 tests)
   - test_DOCKER_integration_edge_001_empty_dockerfile
   - test_DOCKER_integration_edge_002_only_comments
   - test_DOCKER_integration_edge_003_mixed_base_images
   - test_DOCKER_integration_edge_004_circular_dependencies

5. **Performance Integration** (2 tests)
   - test_DOCKER_integration_perf_001_large_dockerfile
   - test_DOCKER_integration_perf_002_many_stages

**Effort Breakdown**:
- Test scenario design: 8 hours
- Implementation: 10 hours (5 tests + integration framework)
- Validation: 2 hours

**Success Criteria**:
- 20 integration tests written
- All RED (failing initially)
- Test complete transformation pipeline

---

## Phase 3: Mutation Testing Implementation (Weeks 4-5, 80 hours)

### Phase 3.1: Mutation Test Infrastructure (20 hours)

**Objectives**:
- Set up cargo-mutants for Docker rules
- Configure mutation test targets
- Establish baseline (should be ~0% initially)

**Setup Steps**:

```bash
# 1. Install mutation testing tool
cargo install cargo-mutants

# 2. Create mutants.toml configuration
[mutants]
timeout = 30
output-options = ["unambiguous", "json"]

[[exclude-functions]]
name = "logger"  # Don't mutate logging code

[[mutate]]
name = "docker001"
path = "src/linter/rules/docker001.rs"
min_kill_rate = 0.90  # 90% target

[[mutate]]
name = "docker002"
path = "src/linter/rules/docker002.rs"
min_kill_rate = 0.90

# ... etc for docker003-006

# 3. Run baseline mutation testing
cargo mutants --output results.json
```

**Effort Breakdown**:
- Configuration: 5 hours
- Infrastructure testing: 8 hours
- Baseline establishment: 7 hours

**Success Criteria**:
- cargo-mutants runs successfully
- mutants.toml configured for all rules
- Baseline mutation score recorded (should be ~0% with incomplete tests)

### Phase 3.2: Test Hardening (50 hours)

**Objective**: Improve mutation kill rate to >90% per rule

**EXTREME TDD Process**:

```
1. RUN: cargo mutants --file src/linter/rules/docker001.rs
2. ANALYZE: Which mutations survived?
   - Mutation survived: Changed condition from `==` to `!=`
   - Survived mutation: if (tag == "latest") → if (tag != "latest")
   - Indicates: Test not checking this branch
3. FIX: Add test for this condition
   #[test]
   fn test_docker002_latest_tag_is_pinned() {
       // Test the specific condition mutation found
   }
4. VERIFY: Re-run mutations, ensure kill rate increases
5. REPEAT: Until 90% kill rate achieved
```

**Rule-by-Rule Hardening**:

1. **docker001.rs** (10 hours)
   - Target: 90% kill rate
   - Estimated survivors: 5-8 mutations
   - Focus areas:
     - User creation commands (addgroup, adduser syntax)
     - Conditional logic (scratch images, existing USER)
     - String manipulation (user names, groups)

2. **docker002.rs** (15 hours)
   - Target: 90% kill rate
   - Estimated survivors: 8-12 mutations
   - Focus areas:
     - Version mapping logic
     - Tag detection (latest, untagged, digest)
     - Registry parsing
     - Edge cases (ARG in FROM)

3. **docker003.rs** (8 hours)
   - Target: 90% kill rate
   - Estimated survivors: 3-5 mutations
   - Focus areas:
     - Package manager detection
     - Cleanup command insertion
     - Multi-line RUN handling

4. **docker004.rs** (6 hours)
   - Target: 90% kill rate
   - Estimated survivors: 2-4 mutations
   - Focus areas:
     - Health check parameters
     - Syntax validation

5. **docker005.rs** (6 hours)
   - Target: 90% kill rate
   - Estimated survivors: 2-4 mutations
   - Focus areas:
     - Flag detection
     - Flag addition logic

6. **docker006.rs** (5 hours)
   - Target: 90% kill rate
   - Estimated survivors: 1-3 mutations
   - Focus areas:
     - ADD vs COPY distinction
     - URL detection

**Effort Breakdown**:
- Per-rule analysis and test writing: 40 hours
- Verification and iteration: 10 hours

**Success Criteria**:
- >90% kill rate for each rule
- All surviving mutations documented
- Mutation testing integrated into CI/CD

### Phase 3.3: Mutation Automation (10 hours)

**Objectives**:
- Integrate into Makefile targets
- Add CI/CD pipeline checks
- Document mutation testing workflow

**Implementation**:

```makefile
# In Makefile
.PHONY: mutate mutate-dockerfile mutate-ci

mutate-dockerfile: ## Run mutation tests on Dockerfile rules
	@echo "🧬 Running mutation tests on Dockerfile rules..."
	cargo mutants --file src/linter/rules/docker*.rs --output results.json
	@echo "✓ Mutation testing complete"
	@cargo mutants --analyze results.json

mutate-ci: ## Mutation testing for CI/CD (strict)
	cargo mutants --file src/linter/rules/docker*.rs \
		--minimum-kill-rate 0.90 \
		--output ci-results.json
	@if ! cargo mutants --analyze ci-results.json --fail-if-below 0.90; then \
		echo "❌ Kill rate below 90% threshold"; exit 1; \
	fi
```

**Effort Breakdown**:
- Makefile integration: 3 hours
- CI/CD configuration: 4 hours
- Documentation: 3 hours

**Success Criteria**:
- `make mutate-dockerfile` works
- CI/CD enforces 90% kill rate
- Mutation results logged and analyzed

---

## Phase 4: Coverage Analysis & Gap Filling (Weeks 5-6, 60 hours)

### Phase 4.1: Coverage Measurement (15 hours)

**Objectives**:
- Measure code coverage with llvm-cov
- Identify coverage gaps
- Create gap-filling test plan

**Process**:

```bash
# 1. Generate baseline coverage
cargo llvm-cov --no-report nextest --features default
cargo llvm-cov report --html --output-path target/coverage/dockerfile

# 2. Analyze per-file coverage
llvm-cov report --ignore-filename-regex="(test|bench)" \
    --output-format html \
    --output-path target/coverage/detailed

# 3. Parse coverage data
grep -A 5 "docker001.rs" target/coverage/detailed/index.html
grep -A 5 "docker002.rs" target/coverage/detailed/index.html
# ... etc
```

**Coverage Targets**:

```
docker001.rs: >85% (currently ~70%)
├── Lines: 85/100 (missing: error paths, edge cases)
├── Branches: 12/14 (missing: nested conditions)
└── Functions: 100%

docker002.rs: >85% (currently ~75%)
├── Lines: 85/100 (missing: registry parsing, edge cases)
├── Branches: 18/22 (missing: version logic branches)
└── Functions: 100%

docker003.rs: >85% (currently ~65%)
├── Lines: 85/100 (missing: multi-line handling)
├── Branches: 14/16 (missing: cleanup conditions)
└── Functions: 100%

docker004.rs: >85% (estimated ~80%)
docker005.rs: >85% (estimated ~80%)
docker006.rs: >85% (estimated ~82%)
```

**Effort Breakdown**:
- Tooling setup: 5 hours
- Analysis and gap identification: 7 hours
- Documentation: 3 hours

**Success Criteria**:
- Coverage report generated
- >85% target per module identified
- Gap analysis documented with test requirements

### Phase 4.2: Gap-Filling Tests (40 hours)

**Objective**: Write tests to cover identified gaps

**Example Gap-Filling Pattern**:

```rust
// DISCOVERED GAP: docker001.rs line 45, error path not covered
// Branch: if validate_user_name(&user_name).is_err()

#[test]
fn test_docker001_gap_fill_001_invalid_user_name_handling() {
    // This test fills gap in error path
    let input = "FROM debian:12\nCMD [\"bash\"]";
    let result = apply_docker001_with_invalid_user(input);
    assert!(result.is_err() || result.unwrap().contains("appuser"));
}

// DISCOVERED GAP: docker002.rs line 78, edge case not tested
// Branch: if image.contains("@") && image.contains(":")  

#[test]
fn test_docker002_gap_fill_002_digest_and_tag_combo() {
    let dockerfile = "FROM ubuntu:22.04@sha256:abc123\nCMD [\"bash\"]";
    let result = apply_docker002(dockerfile);
    // Should preserve both tag and digest
    assert!(result.contains("@sha256:"));
    assert!(result.contains(":22.04"));
}
```

**Gap-Filling Strategies**:

1. **Error Path Coverage** (8 tests)
   - Invalid inputs that should fail gracefully
   - Recovery from malformed instructions
   - Validation error messages

2. **Edge Case Coverage** (12 tests)
   - Boundary conditions
   - Empty/null values
   - Maximum lengths
   - Special characters

3. **Branch Coverage** (15 tests)
   - Untested conditional branches
   - Mutation survivors validation
   - Logic edge cases

4. **Integration Coverage** (5 tests)
   - Interactions between modules
   - State management
   - Cross-module dependencies

**Effort Breakdown**:
- Gap analysis per rule: 15 hours
- Test implementation: 20 hours
- Validation: 5 hours

**Success Criteria**:
- 40 gap-filling tests written
- Coverage increases to >85% per module
- All identified gaps tested

---

## Phase 5: Documentation & Specification (Weeks 6-7, 40 hours)

### Phase 5.1: Update unified-testing-quality-spec.md (15 hours)

**Objectives**:
- Document Dockerfile testing standards
- Establish quality gates
- Define CI/CD checks

**Changes to unified-testing-quality-spec.md**:

```markdown
## Dockerfile Testing Standards (NEW SECTION)

### Test Naming Convention
- Pattern: `test_DOCKER_<number>_<feature>_<scenario>`
- Example: `test_DOCKER_001_user_directive_basic`
- CLI tests must include `cli` in name: `test_DOCKER_cli_001_purify_command`

### Testing Matrix (per transformation)

| Transformation | Unit Tests | Property Tests | Integration | Mutation Target | Coverage Target |
|---|---|---|---|---|---|
| DOCKER001 | 15+ | 2 blocks | 3 | 90% | >85% |
| DOCKER002 | 17+ | 3 blocks | 4 | 90% | >85% |
| DOCKER003 | 11+ | 2 blocks | 3 | 90% | >85% |
| DOCKER004 | 8+ | 1 block | 2 | 90% | >85% |
| DOCKER005 | 9+ | 1 block | 2 | 90% | >85% |
| DOCKER006 | 8+ | 1 block | 2 | 90% | >85% |

### Quality Gates (MANDATORY)

**RED Phase** (Write failing tests first):
- [ ] All unit tests written and failing
- [ ] Property tests generated (100+ cases each)
- [ ] Integration tests defined
- [ ] Coverage map created

**GREEN Phase** (Implementation):
- [ ] All tests passing (100% pass rate)
- [ ] Coverage >85% per module
- [ ] No panics on any input
- [ ] Error paths covered

**REFACTOR Phase** (Polish):
- [ ] Code complexity <10
- [ ] Mutation kill rate >90%
- [ ] Performance baseline met
- [ ] Documentation complete

### CI/CD Integration

```yaml
dockerfile-quality-gates:
  - unit-tests: "cargo test --lib src/linter/rules/docker*.rs"
  - coverage: "cargo llvm-cov --fail-under-lines 85"
  - mutations: "cargo mutants --fail-if-below 90"
  - property: "cargo test --test property_dockerfile_purify -- --test-threads 1"
  - cli: "cargo test --test cli_dockerfile_purify"
  
  required-all-pass: true
  fail-fast: false
```

### Test Organization

```
tests/
├── cli_dockerfile_purify.rs (35+ CLI tests)
├── cli_dockerfile_integration.rs (20+ integration tests)
├── property_dockerfile_purify.rs (20+ property blocks)
└── unit_dockerfile_rules.rs (52+ unit tests in src/linter/rules/)
```

### Coverage Requirements

- **Critical Paths** (required): User detection, image pinning, cleanup
- **Error Paths** (required): Invalid input, malformed instructions
- **Integration** (required): Rule interactions, multi-stage builds
- **Performance** (target): <10ms per 1000 lines

### Property Testing Standards

Each property block must:
- [ ] Test 100+ cases minimum
- [ ] Cover at least 3 input categories
- [ ] Have clear failure messages
- [ ] Document assumptions
- [ ] Handle edge cases (empty input, max length, special chars)

### Mutation Testing Standards

Each rule must:
- [ ] Achieve >90% kill rate
- [ ] Document surviving mutations
- [ ] Test all conditional branches
- [ ] Validate error conditions
- [ ] Check boundary conditions
```

**Effort Breakdown**:
- Specification design: 7 hours
- Writing guidelines: 5 hours
- Review and refinement: 3 hours

**Success Criteria**:
- Document updated with Dockerfile section
- Quality gates defined clearly
- CI/CD integration specified
- Examples provided

### Phase 5.2: Update ROADMAP.yaml (15 hours)

**Objectives**:
- Document completion of Docker testing parity
- Define next priorities
- Establish timeline

**New ROADMAP.yaml Entry**:

```yaml
# docs/DOCKERFILE-TESTING-ROADMAP.yaml (NEW)

project: "Dockerfile Testing Parity"
version: "1.0"
status: "IN PROGRESS"
started: "2025-11-11"
target-completion: "2025-12-20"

phases:
  phase-1:
    name: "Test Infrastructure Enhancement"
    status: "PLANNED"
    duration: "2 weeks"
    effort: "40 hours"
    objectives:
      - Expand CLI tests from 16 to 35+
      - Add 10 property test blocks
      - Document 40+ edge cases
    
    tasks:
      task-1.1:
        name: "Extend CLI Tests"
        status: "PLANNED"
        effort: "20 hours"
        subtasks:
          - Add 19 CLI tests covering all transformations
          - Implement flag combination tests
          - Add error handling tests
          - document test naming convention
      
      task-1.2:
        name: "Property Test Generation"
        status: "PLANNED"
        effort: "15 hours"
        subtasks:
          - Implement 10 property blocks (100+ cases each)
          - Create 5 new Dockerfile generators
          - Validate property definitions
      
      task-1.3:
        name: "Edge Case Cataloging"
        status: "PLANNED"
        effort: "5 hours"
        subtasks:
          - Document 40+ edge cases
          - Create test matrix
          - Link to test requirements
  
  phase-2:
    name: "Unit Test Expansion"
    status: "PLANNED"
    duration: "2 weeks"
    effort: "60 hours"
    target-coverage: ">85%"
    
    tasks:
      task-2.1:
        name: "Docker Rule Unit Tests"
        status: "PLANNED"
        effort: "40 hours"
        per-rule-tests:
          docker001: "10 new tests"
          docker002: "12 new tests"
          docker003: "8 new tests"
          docker004: "8 new tests"
          docker005: "8 new tests"
          docker006: "6 new tests"
      
      task-2.2:
        name: "Integration Tests"
        status: "PLANNED"
        effort: "20 hours"
        categories:
          - "Complete pipeline tests (5)"
          - "Cross-rule interactions (6)"
          - "Error recovery (3)"
          - "Edge case combinations (4)"
          - "Performance integration (2)"
  
  phase-3:
    name: "Mutation Testing Implementation"
    status: "PLANNED"
    duration: "2 weeks"
    effort: "80 hours"
    target-kill-rate: ">90%"
    
    tasks:
      task-3.1:
        name: "Mutation Test Infrastructure"
        status: "PLANNED"
        effort: "20 hours"
      
      task-3.2:
        name: "Test Hardening"
        status: "PLANNED"
        effort: "50 hours"
        per-rule-hardening:
          docker001: "10 hours"
          docker002: "15 hours"
          docker003: "8 hours"
          docker004: "6 hours"
          docker005: "6 hours"
          docker006: "5 hours"
      
      task-3.3:
        name: "Mutation Automation"
        status: "PLANNED"
        effort: "10 hours"
  
  phase-4:
    name: "Coverage Analysis & Gap Filling"
    status: "PLANNED"
    duration: "2 weeks"
    effort: "60 hours"
    target-coverage: ">85%"
    
    tasks:
      task-4.1:
        name: "Coverage Measurement"
        status: "PLANNED"
        effort: "15 hours"
      
      task-4.2:
        name: "Gap-Filling Tests"
        status: "PLANNED"
        effort: "40 hours"
        categories:
          - "Error path coverage (8 tests)"
          - "Edge case coverage (12 tests)"
          - "Branch coverage (15 tests)"
          - "Integration coverage (5 tests)"
  
  phase-5:
    name: "Documentation & Specification"
    status: "PLANNED"
    duration: "1 week"
    effort: "40 hours"
    
    tasks:
      task-5.1:
        name: "Update unified-testing-quality-spec.md"
        status: "PLANNED"
        effort: "15 hours"
      
      task-5.2:
        name: "Update ROADMAP.yaml"
        status: "PLANNED"
        effort: "15 hours"
      
      task-5.3:
        name: "Create testing guide"
        status: "PLANNED"
        effort: "10 hours"

quality-gates:
  - unit-tests: "All 52+ unit tests passing"
  - property-tests: "All 20+ property blocks with 100+ cases"
  - integration-tests: "All 20+ integration tests passing"
  - coverage: ">85% per module (verified with llvm-cov)"
  - mutations: ">90% kill rate per rule (verified with cargo-mutants)"
  - cli: "All 35+ CLI tests passing with assert_cmd"
  - documentation: "unified-testing-quality-spec.md updated"

milestones:
  - "2025-11-18: Phase 1 complete (test infrastructure)"
  - "2025-11-25: Phase 2 complete (unit tests)"
  - "2025-12-02: Phase 3 complete (mutation testing)"
  - "2025-12-09: Phase 4 complete (coverage analysis)"
  - "2025-12-16: Phase 5 complete (documentation)"
  - "2025-12-20: RELEASE - Dockerfile testing parity achieved"

metrics:
  cli-tests: "16 → 35+ (2.2x increase)"
  unit-tests: "0 → 52+ (new category)"
  property-tests: "14 → 40+ (3x increase)"
  integration-tests: "0 → 20+ (new category)"
  coverage: "~75% → >85% (target)"
  mutations: "0% → >90% kill rate"
  total-tests: "30 → 147+"

dependencies:
  - "script.sh testing must be >85% complete first"
  - "Makefile testing patterns established"
  - "EXTREME TDD infrastructure in place"
  - "CI/CD pipeline ready for Docker tests"

next-phase:
  after: "Dockerfile testing parity achieved"
  name: "Rust → Shell Transpilation Testing (v3.0)"
  effort: "200+ hours"
  features:
    - "Rust → Shell type checking (new)"
    - "Stdlib mapping validation (new)"
    - "Integration testing with Rust code (new)"
```

**Effort Breakdown**:
- ROADMAP design: 8 hours
- Documentation: 5 hours
- Validation: 2 hours

**Success Criteria**:
- DOCKERFILE-TESTING-ROADMAP.yaml created
- All phases documented
- Timeline established
- Next priority defined

### Phase 5.3: Create Testing Guide (10 hours)

**Objectives**:
- Document how to write Dockerfile tests
- Provide examples and patterns
- Enable developer self-service

**docs/guides/DOCKERFILE-TESTING-GUIDE.md (NEW)**:

```markdown
# Dockerfile Testing Guide

## Quick Start

### Writing a Unit Test

```rust
#[test]
fn test_DOCKER_NNN_feature_scenario() {
    // ARRANGE: Set up test case
    let input = "FROM debian:12\n...";
    
    // ACT: Run transformation
    let result = transform(input);
    
    // ASSERT: Verify expected behavior
    assert!(result.contains("expected"), "Clear failure message");
}
```

### Writing a Property Test

```rust
proptest! {
    #[test]
    fn prop_DOCKER_NNN_property_description(
        input in dockerfile_generator()
    ) {
        let result = transform(&input);
        prop_assert!(some_property(&result), "Property must hold");
    }
}
```

### Writing a CLI Test

```rust
#[test]
fn test_DOCKER_cli_NNN_command_option() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, TEST_DOCKERFILE).unwrap();
    
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("expected"));
}
```

## Running Tests

```bash
# Run all Dockerfile tests
cargo test --test cli_dockerfile_purify
cargo test --test property_dockerfile_purify

# Run with coverage
cargo llvm-cov --test cli_dockerfile_purify

# Run mutation tests
cargo mutants --file src/linter/rules/docker*.rs
```

## Testing Checklist

- [ ] Test name follows convention
- [ ] RED phase: test fails initially
- [ ] GREEN phase: implementation makes test pass
- [ ] REFACTOR: code complexity <10
- [ ] Property tests: 100+ cases
- [ ] Coverage: >85%
- [ ] Mutations: >90% kill rate
```

**Effort Breakdown**:
- Guide design: 3 hours
- Examples and patterns: 4 hours
- Validation: 3 hours

**Success Criteria**:
- Testing guide created and published
- Examples compilable and runnable
- Clear patterns documented

---

## Phase 6: Release & Post-Release (Weeks 7-8, 40 hours)

### Phase 6.1: Final Verification (15 hours)

**Objectives**:
- Run full test suite
- Verify all quality gates
- Document final metrics

**Verification Checklist**:

```
TEST EXECUTION:
  [ ] cargo test --lib (all unit tests passing)
  [ ] cargo test --test cli_dockerfile_* (all CLI tests)
  [ ] cargo test --test property_dockerfile_* (all property tests)
  [ ] cargo test --test integration_dockerfile_* (all integration)
  
COVERAGE VERIFICATION:
  [ ] cargo llvm-cov report (>85% per module)
  [ ] Review coverage report
  [ ] Document any gaps
  
MUTATION VERIFICATION:
  [ ] cargo mutants --file src/linter/rules/docker*.rs
  [ ] Verify >90% kill rate per rule
  [ ] Document surviving mutations
  [ ] Justify or fix survivors
  
PERFORMANCE VERIFICATION:
  [ ] cargo bench --bench dockerfile_benchmarks
  [ ] Verify <10ms per 1000 lines
  [ ] No performance regression
  
DOCUMENTATION VERIFICATION:
  [ ] unified-testing-quality-spec.md updated
  [ ] DOCKERFILE-TESTING-ROADMAP.yaml complete
  [ ] DOCKERFILE-TESTING-GUIDE.md accessible
  [ ] CHANGELOG.md updated
  
INTEGRATION VERIFICATION:
  [ ] CI/CD passes all checks
  [ ] No breaking changes
  [ ] Backward compatibility verified
  [ ] All platforms tested (Linux, macOS, Windows)
```

**Effort Breakdown**:
- Verification execution: 8 hours
- Issue resolution: 5 hours
- Final documentation: 2 hours

**Success Criteria**:
- All quality gates passing
- Metrics documented
- Ready for release

### Phase 6.2: Release Preparation (15 hours)

**Objectives**:
- Update CHANGELOG
- Prepare release notes
- Create release tag

**Release Notes Template**:

```markdown
# v7.0.0 - Dockerfile Testing Parity

## Summary

Dockerfile testing now achieves testing parity with Makefile and script.sh 
transformation tools, providing enterprise-grade quality assurance.

## What's New

### Testing Infrastructure
- ✅ Expanded CLI tests: 16 → 35+ tests
- ✅ Property-based testing: 100+ cases per property
- ✅ Mutation testing: >90% kill rate per rule
- ✅ Coverage analysis: >85% per module
- ✅ Integration testing: Complete pipeline validation

### Quality Improvements
- ✅ DOCKER001: User directive (15 unit tests, 2 property blocks)
- ✅ DOCKER002: Image pinning (17 unit tests, 3 property blocks)
- ✅ DOCKER003: Package cleanup (11 unit tests, 2 property blocks)
- ✅ DOCKER004: Health checks (8 unit tests, 1 property block)
- ✅ DOCKER005: Package flags (9 unit tests, 1 property block)
- ✅ DOCKER006: ADD → COPY (8 unit tests, 1 property block)

### Documentation
- ✅ unified-testing-quality-spec.md updated with Dockerfile section
- ✅ DOCKERFILE-TESTING-ROADMAP.yaml created
- ✅ DOCKERFILE-TESTING-GUIDE.md for developers

## Test Coverage

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| CLI Tests | 16 | 35+ | 30+ |
| Unit Tests | 0 | 52+ | 50+ |
| Property Tests | 14 | 40+ | 30+ |
| Integration Tests | 0 | 20+ | 15+ |
| Code Coverage | ~75% | >85% | >85% |
| Mutation Kill Rate | N/A | >90% | >90% |

## Breaking Changes

None. This release maintains full backward compatibility with previous versions.

## Migration Guide

No migration required. All existing tools work unchanged.

## Testing

Run full Dockerfile test suite:
```bash
cargo test --test cli_dockerfile_purify
cargo test --test property_dockerfile_purify
cargo llvm-cov --test cli_dockerfile_*
cargo mutants --file src/linter/rules/docker*.rs
```

## Credits

Testing parity implementation following EXTREME TDD methodology:
- Property-based testing with 100+ cases per property
- Mutation testing with >90% kill rate target
- Comprehensive coverage analysis
- Complete integration testing

---

🤖 Generated with Claude Code
```

**Effort Breakdown**:
- CHANGELOG update: 5 hours
- Release notes: 6 hours
- Tag creation and verification: 4 hours

**Success Criteria**:
- CHANGELOG.md updated
- Release notes clear and complete
- Release tag created
- CI/CD passes final checks

### Phase 6.3: Post-Release Documentation (10 hours)

**Objectives**:
- Create retrospective
- Document lessons learned
- Plan next phase

**Retrospective Template**:

```markdown
# Dockerfile Testing Parity - Retrospective

## Goals Achieved

### Primary Goals
- [x] 35+ CLI tests (target: 30+)
- [x] 52+ unit tests (target: 50+)
- [x] 40+ property test blocks (target: 30+)
- [x] 20+ integration tests (target: 15+)
- [x] >85% code coverage (target: >85%)
- [x] >90% mutation kill rate (target: >90%)

### Metrics Achieved

| Metric | Target | Actual | Diff |
|--------|--------|--------|------|
| CLI Tests | 30+ | 35 | +5 |
| Unit Tests | 50+ | 52 | +2 |
| Property Blocks | 30+ | 40 | +10 |
| Coverage | >85% | 87% | +2% |
| Mutation Kill Rate | >90% | 92% | +2% |

## Challenges & Solutions

1. **Challenge**: Multi-stage builds require special handling
   **Solution**: Added dedicated test generators and property blocks
   
2. **Challenge**: Mutation testing exposed subtle logic errors
   **Solution**: Enhanced branch testing, improved mutation kill rate
   
3. **Challenge**: Coverage gaps in error paths
   **Solution**: Systematic error case enumeration and testing

## Lessons Learned

1. Property testing catches edge cases unit tests miss
2. Mutation testing validates test quality effectively
3. Early documentation prevents integration issues
4. Test infrastructure investment pays dividends

## Next Phase Readiness

Dockerfile testing parity achieved. Ready for:
- [ ] Integration with other transformation tools
- [ ] Rust → Shell transpilation testing (v3.0)
- [ ] Performance optimization
- [ ] Extended rule coverage (DOCKER007-DOCKER010)

## Recommendations

1. Maintain >90% mutation kill rate in CI/CD
2. Continue property-based testing for new rules
3. Use Dockerfile testing as pattern for future work
4. Consider automated test generation from failing mutations
```

**Effort Breakdown**:
- Retrospective writing: 5 hours
- Lessons learned documentation: 3 hours
- Next phase planning: 2 hours

**Success Criteria**:
- Retrospective completed
- Lessons documented
- Next phase clearly defined

---

## Consolidated Implementation Timeline

### Week 1-2: Test Infrastructure Enhancement (40 hours)
```
Week 1:
  Mon: Phase 1.1 planning + setup (4h)
  Tue-Thu: CLI test writing (12h)
  Fri: Property test setup (4h)

Week 2:
  Mon-Wed: Property tests implementation (12h)
  Thu-Fri: Edge case cataloging (4h + 4h)
```

### Week 3-4: Unit Test Expansion (60 hours)
```
Week 3:
  Mon-Wed: docker001-002 unit tests (20h)
  Thu-Fri: docker003-004 unit tests (10h)

Week 4:
  Mon-Wed: docker005-006 + integration tests (20h)
  Thu-Fri: Validation and fixes (10h)
```

### Week 5-6: Mutation Testing (80 hours)
```
Week 5:
  Mon-Tue: Infrastructure setup (10h)
  Wed-Fri: docker001-002 hardening (20h)

Week 6:
  Mon-Wed: docker003-005 hardening (16h)
  Thu: docker006 hardening + automation (8h)
  Fri: Verification (4h)
```

### Week 7: Coverage Analysis (60 hours)
```
Week 7:
  Mon-Tue: Coverage measurement (8h)
  Wed-Fri: Gap-filling tests (36h)
  Remaining: Analysis and iteration (16h)
```

### Week 8: Documentation (40 hours)
```
Week 8:
  Mon-Tue: Spec updates (10h)
  Wed: ROADMAP update (8h)
  Thu: Testing guide (6h)
  Fri: Release prep + verification (16h)
```

---

## Success Metrics & Quality Gates

### Phase Completion Criteria

**Phase 1 (Test Infrastructure)**:
- [ ] 35+ CLI tests written (RED phase)
- [ ] 20+ property blocks written (RED phase)
- [ ] 40+ edge cases documented
- [ ] All tests fail initially (expected in RED)

**Phase 2 (Unit Tests)**:
- [ ] 52+ new unit tests GREEN (passing)
- [ ] 20 integration tests GREEN (passing)
- [ ] 100% pass rate on all tests
- [ ] Code complexity <10

**Phase 3 (Mutation Tests)**:
- [ ] cargo-mutants configured
- [ ] >90% kill rate per rule
- [ ] All surviving mutations documented
- [ ] Mutation tests in CI/CD

**Phase 4 (Coverage)**:
- [ ] >85% coverage per module (verified)
- [ ] All identified gaps tested
- [ ] 40 gap-filling tests GREEN
- [ ] Coverage report generated

**Phase 5 (Documentation)**:
- [ ] unified-testing-quality-spec.md updated
- [ ] DOCKERFILE-TESTING-ROADMAP.yaml created
- [ ] DOCKERFILE-TESTING-GUIDE.md published
- [ ] All examples compilable

**Phase 6 (Release)**:
- [ ] All quality gates passing
- [ ] Final metrics documented
- [ ] Release notes prepared
- [ ] Retrospective completed

---

## Resource Requirements

### Team Composition
- **1 Primary Developer** (280-320 hours)
  - Phases 1-2: Full focus on testing infrastructure
  - Phases 3-4: Mutation testing + gap analysis
  - Phase 5-6: Documentation + release

- **Optional: Code Review** (40 hours)
  - Technical review of tests (20 hours)
  - Mutation analysis (10 hours)
  - Documentation review (10 hours)

### Infrastructure
- **CI/CD Pipeline**:
  - llvm-cov for coverage analysis (already installed)
  - cargo-mutants for mutation testing (to install)
  - GitHub Actions for test automation

- **Developer Tools**:
  - Rust 1.70+ (already available)
  - cargo-llvm-cov (already installed)
  - proptest 1.0+ (already in Cargo.toml)

### Knowledge Base
- EXTREME TDD principles (documented in CLAUDE.md)
- Makefile testing patterns (already implemented)
- script.sh testing standards (6004+ tests reference)
- Property testing best practices (proptest docs)

---

## Risk Assessment

### High-Risk Areas

1. **Mutation Test Configuration**
   - Risk: Difficult to achieve >90% kill rate
   - Mitigation: Start with 80%, gradually increase; document survivors
   - Impact: Could add 20-40 hours if complex

2. **Multi-Stage Build Handling**
   - Risk: Interactions between stages not fully tested
   - Mitigation: Dedicated test generators + integration tests
   - Impact: Could require 10-20 additional tests

3. **Performance Regression**
   - Risk: New tests slow down CI/CD pipeline
   - Mitigation: Property tests run in parallel; benchmark performance
   - Impact: Acceptable if <10% CI/CD time increase

### Mitigation Strategies

1. **Regular Checkpoints**
   - Week 2 checkpoint: Phase 1 complete assessment
   - Week 4 checkpoint: Unit test quality review
   - Week 6 checkpoint: Mutation kill rate evaluation

2. **Fallback Options**
   - If mutation testing too difficult: Accept >85% kill rate
   - If coverage gaps significant: Extend phase 4 by 1 week
   - If time constraint: Defer Phase 5.2 (ROADMAP) to post-release

3. **Communication Plan**
   - Weekly status updates
   - Phase completion sign-offs
   - Risk escalation if >10% off schedule

---

## Next Priority After Dockerfile Parity

### Phase 7: Rust → Shell Transpilation Testing (v3.0)

**Estimated Effort**: 200-250 hours (6-8 weeks)

**Scope**:
- Comprehensive testing of Rust → Shell conversion
- Stdlib mapping validation
- Type system verification
- Integration testing with real Rust code

**Deliverables**:
- 100+ property test blocks (for type checking)
- 75+ unit tests (for stdlib coverage)
- 50+ integration tests (for end-to-end workflows)
- >90% mutation kill rate
- >85% code coverage

**Prerequisites**:
- Dockerfile testing parity complete
- Infrastructure improvements complete
- Team experience with property testing established

---

## References & Related Documents

- `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs` - Current CLI tests (16 tests)
- `/home/noah/src/bashrs/rash/tests/property_dockerfile_purify.rs` - Current property tests (14 tests)
- `/home/noah/src/bashrs/rash/Makefile` - Build automation reference
- `/home/noah/src/bashrs/rash/docs/BASH-INGESTION-ROADMAP.yaml` - Roadmap pattern
- `/home/noah/src/bashrs/rash/src/linter/rules/docker*.rs` - Implementation files
- `/home/noah/src/bashrs/CLAUDE.md` - EXTREME TDD guidelines

---

## Conclusion

Achieving Dockerfile testing parity represents a significant investment in quality 
assurance and developer confidence. By following EXTREME TDD principles and leveraging 
proven patterns from script.sh testing, this implementation plan provides a structured 
approach to comprehensive test coverage.

The phased approach allows for:
- **Early validation** (Phases 1-2)
- **Quality verification** (Phase 3-4)
- **Sustainable maintenance** (Phase 5-6)
- **Continuous improvement** (via CI/CD integration)

Upon completion, Dockerfile transformations will meet enterprise-grade testing standards 
with >85% coverage and >90% mutation kill rate, enabling confident deployment in 
production environments.

**Target Release Date**: December 20, 2025 (v7.0.0)
**Status**: Ready to implement
