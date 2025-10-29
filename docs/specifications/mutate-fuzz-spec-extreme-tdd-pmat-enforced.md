# bashrs mutate & fuzz - Specification (EXTREME TDD + pmat Enforced)

## Executive Summary

**Goal**: Add mutation testing (`bashrs mutate`) and fuzzing (`bashrs fuzz`) capabilities to bashrs, using EXTREME TDD methodology with pmat quality enforcement.

**Priority**:
1. **Phase 1**: `bashrs mutate` (6 weeks, HIGH priority)
2. **Phase 2**: `bashrs fuzz` (12 weeks, MEDIUM priority)

**Methodology**: EXTREME TDD + pmat quality gates
- Every feature starts with RED test
- pmat enforces complexity <10, quality score ≥9.0
- Mutation testing validates our mutation tester (meta!)
- Property-based testing for correctness guarantees

**Success Criteria**:
- `bashrs mutate`: ≥90% kill rate on well-tested bash scripts
- `bashrs fuzz`: Find injection vulnerabilities shellcheck misses
- All modules: Complexity <10, pmat quality score ≥9.0, mutation score ≥90%

---

## Background: Lessons from pmat Comparison

### What We Learned (2025-10-29 Dogfooding)

**pmat Mutation Testing on Rust: FAILED**
- Generated 178 mutants for 2 Rust modules (301 lines)
- Kill rate: **0%** (34/34 mutants survived)
- Speed: 21-49s/mutant (claimed "20× faster", actually same speed as cargo-mutants)
- **Conclusion**: pmat unsuitable for Rust - too generic, doesn't understand Rust semantics

**cargo-mutants on Rust: SUCCESS** (expected)
- Generated 28 mutants for same 2 modules
- Expected kill rate: ≥90%
- Speed: ~30s/mutant
- **Conclusion**: Language-specific tools work better (understands Rust semantics)

### Key Insight for bashrs

**Language-specific mutation testing works**:
- cargo-mutants succeeds because it understands Rust semantics
- pmat fails because it's too generic
- **bashrs mutate** will succeed because it will understand bash semantics

**We must avoid pmat's mistakes**:
1. ❌ Don't use generic mutation operators
2. ✅ Use bash-specific mutation operators
3. ❌ Don't claim false performance benefits ("20× faster")
4. ✅ Measure and report realistic performance
5. ❌ Don't generate invalid/redundant mutants
6. ✅ Generate focused, meaningful mutants

---

## Feature 1: bashrs mutate (Mutation Testing for Bash)

### Overview

**What**: Mutation testing for bash scripts
- Generate mutants by modifying bash AST
- Run bash tests against each mutant
- Measure test effectiveness (kill rate)
- Report surviving mutants (test gaps)

**Why**:
- No good bash mutation testing tools exist
- pmat failed at Rust, would likely fail at bash too (generic)
- bashrs has bash parser → perfect fit

**Target**: ≥90% mutation kill rate for well-tested scripts

### Bash-Specific Mutation Operators

Based on real bash semantics (not generic):

#### 1. Conditional Operators (COR - Conditional Operator Replacement)

```bash
# Original
[ -f "$file" ]

# Mutants
[ -d "$file" ]  # File test → Directory test
[ -e "$file" ]  # File test → Exists test
[ -r "$file" ]  # File test → Readable test
[ -w "$file" ]  # File test → Writable test
[ -x "$file" ]  # File test → Executable test
```

**Why bash-specific**: Bash has unique file test operators (-f, -d, -e, -r, -w, -x) that don't exist in other languages.

#### 2. String Operators (SOR - String Operator Replacement)

```bash
# Original
[ "$a" = "$b" ]

# Mutants
[ "$a" != "$b" ]  # Equality → Inequality
[ "$a" \< "$b" ]  # Equality → Less than
[ "$a" \> "$b" ]  # Equality → Greater than
[ -z "$a" ]       # Equality → Empty test
[ -n "$a" ]       # Equality → Non-empty test
```

#### 3. Arithmetic Operators (AOR - Arithmetic Operator Replacement)

```bash
# Original
count=$((count + 1))

# Mutants
count=$((count - 1))  # Addition → Subtraction
count=$((count * 1))  # Addition → Multiplication
count=$((count / 1))  # Addition → Division
count=$((count % 1))  # Addition → Modulo
count=$((count))      # Remove operator
```

#### 4. Command Replacement (CRR - Command Replacement)

```bash
# Original
rm -rf "$dir"

# Mutants
echo "$dir"         # Neuter destructive command
true                # Replace with success
false               # Replace with failure
: # no-op           # Replace with no-op
# (remove command)  # Delete command entirely
```

**Why bash-specific**: Understanding bash's destructive commands (rm, mv, dd) vs safe commands (echo, cat, grep).

#### 5. Flag Mutation (FMR - Flag Mutation/Removal)

```bash
# Original
mkdir -p "$dir"

# Mutants
mkdir "$dir"        # Remove -p (idempotency flag)

# Original
rm -f "$file"

# Mutants
rm "$file"          # Remove -f (force flag)

# Original
ln -sf "$src" "$dst"

# Mutants
ln -s "$src" "$dst"  # Remove -f (overwrite flag)
ln "$src" "$dst"     # Remove -s (symbolic flag)
```

**Why bash-specific**: Bash idempotency flags (-p, -f) are critical for script robustness.

#### 6. Exit Code Mutation (ECR - Exit Code Replacement)

```bash
# Original
command || return 1

# Mutants
command || return 0  # Failure → Success
command || return 2  # Different error code
command              # Remove error handling
```

#### 7. Variable Quoting (QMR - Quote Mutation/Removal)

```bash
# Original
rm "$file"

# Mutants
rm $file            # Remove quotes (injection risk!)

# Original
[ -f $file ]

# Mutants
[ -f "$file" ]      # Add quotes (fix injection)
```

**Why bash-specific**: Quoting is critical in bash for security and correctness.

#### 8. Pipeline Mutation (PMR - Pipeline Mutation)

```bash
# Original
cat file | grep pattern | wc -l

# Mutants
cat file | wc -l              # Remove middle stage
grep pattern file | wc -l     # Optimize/change structure
cat file | grep pattern       # Remove final stage
```

### Implementation Architecture

```
rash/src/mutation/
├── mod.rs                    # Public API
├── operators.rs              # Mutation operators
├── generator.rs              # Mutant generation from AST
├── executor.rs               # Test execution against mutants
├── report.rs                 # Mutation testing reports
└── cli.rs                    # CLI integration

rash/tests/mutation/
├── test_conditional_operators.rs
├── test_string_operators.rs
├── test_arithmetic_operators.rs
├── test_command_replacement.rs
├── test_flag_mutation.rs
├── test_exit_code_mutation.rs
├── test_quote_mutation.rs
└── test_pipeline_mutation.rs
```

### EXTREME TDD Workflow

**Every operator follows RED-GREEN-REFACTOR**:

#### Example: Conditional Operator Replacement (COR)

**Phase 1: RED (Write Failing Test)**

```rust
// rash/tests/mutation/test_conditional_operators.rs
#[test]
fn test_file_test_to_directory_test_mutation() {
    // ARRANGE
    let bash_code = r#"
#!/bin/bash
check_file() {
    if [ -f "$1" ]; then
        echo "is_file"
    fi
}
"#;

    let ast = parse_bash(bash_code).unwrap();

    // ACT
    let mutants = generate_mutants(&ast, &[MutationOperator::ConditionalReplacement]);

    // ASSERT
    assert!(mutants.len() > 0, "Should generate at least one mutant");

    // Find the -f → -d mutant
    let file_to_dir_mutant = mutants.iter()
        .find(|m| {
            matches!(m.operator, MutationOperator::ConditionalReplacement) &&
            m.original_code.contains("[ -f") &&
            m.mutated_code.contains("[ -d")
        });

    assert!(file_to_dir_mutant.is_some(), "Should generate [ -f ] → [ -d ] mutant");
}

// Run test: SHOULD FAIL (operator not implemented yet)
```

**Phase 2: GREEN (Implement Minimum to Pass)**

```rust
// rash/src/mutation/operators.rs
#[derive(Debug, Clone, PartialEq)]
pub enum MutationOperator {
    ConditionalReplacement,
    // ... more operators
}

// rash/src/mutation/generator.rs
pub fn generate_mutants(ast: &BashAst, operators: &[MutationOperator]) -> Vec<Mutant> {
    let mut mutants = Vec::new();

    for operator in operators {
        match operator {
            MutationOperator::ConditionalReplacement => {
                mutants.extend(generate_conditional_mutants(ast));
            }
            // ... more operators
        }
    }

    mutants
}

fn generate_conditional_mutants(ast: &BashAst) -> Vec<Mutant> {
    let mut mutants = Vec::new();

    // Walk AST looking for test expressions
    for node in ast.walk() {
        if let AstNode::Test(test_expr) = node {
            match test_expr.operator.as_str() {
                "-f" => {
                    // Generate [ -f ] → [ -d ] mutant
                    mutants.push(Mutant {
                        id: generate_mutant_id("COR"),
                        operator: MutationOperator::ConditionalReplacement,
                        location: node.location(),
                        original_code: "[ -f \"$1\" ]".to_string(),
                        mutated_code: "[ -d \"$1\" ]".to_string(),
                        description: "Replace file test (-f) with directory test (-d)".to_string(),
                    });
                }
                // ... more file test operators
                _ => {}
            }
        }
    }

    mutants
}

// Run test: SHOULD PASS
```

**Phase 3: REFACTOR (Clean Up)**

```rust
// Extract file test operator mapping
const FILE_TEST_MUTATIONS: &[(&str, &str, &str)] = &[
    ("-f", "-d", "file test → directory test"),
    ("-f", "-e", "file test → exists test"),
    ("-f", "-r", "file test → readable test"),
    ("-f", "-w", "file test → writable test"),
    ("-f", "-x", "file test → executable test"),
    ("-d", "-f", "directory test → file test"),
    // ... exhaustive mappings
];

fn generate_conditional_mutants(ast: &BashAst) -> Vec<Mutant> {
    let mut mutants = Vec::new();

    for node in ast.walk() {
        if let AstNode::Test(test_expr) = node {
            for (from_op, to_op, description) in FILE_TEST_MUTATIONS {
                if test_expr.operator == *from_op {
                    mutants.push(create_operator_mutant(
                        node,
                        from_op,
                        to_op,
                        description,
                    ));
                }
            }
        }
    }

    mutants
}

// Run test: STILL PASSES (refactored)
```

**Phase 4: Property Testing**

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_file_test_mutations_valid(
            file_path in "[a-z/]{1,50}\\.txt"
        ) {
            let bash_code = format!(r#"
                if [ -f "{}" ]; then
                    echo "exists"
                fi
            "#, file_path);

            let ast = parse_bash(&bash_code).unwrap();
            let mutants = generate_mutants(&ast, &[MutationOperator::ConditionalReplacement]);

            // All mutants should parse successfully
            for mutant in &mutants {
                let mutated_ast = parse_bash(&mutant.mutated_code);
                prop_assert!(mutated_ast.is_ok(),
                    "Mutant should produce valid bash: {}", mutant.mutated_code);
            }

            // Should generate at least one mutant
            prop_assert!(mutants.len() > 0, "Should generate mutants for file tests");
        }

        #[test]
        fn prop_mutants_are_different(
            test_operator in "-[fdedrwx]"
        ) {
            let bash_code = format!(r#"
                if [ {} "$file" ]; then
                    echo "test"
                fi
            "#, test_operator);

            let ast = parse_bash(&bash_code).unwrap();
            let mutants = generate_mutants(&ast, &[MutationOperator::ConditionalReplacement]);

            // All mutants should be different from original
            for mutant in &mutants {
                prop_assert_ne!(
                    mutant.mutated_code,
                    mutant.original_code,
                    "Mutant should differ from original"
                );
            }
        }
    }
}
```

**Phase 5: Mutation Testing (Meta!)**

```bash
# Test our mutation testing implementation with mutation testing!
$ cargo mutants --file rash/src/mutation/operators.rs

# Expected: ≥90% kill rate (our tests should catch mutants in our mutant generator)
```

**Phase 6: pmat Quality Gates**

```bash
# Complexity analysis
$ pmat analyze complexity rash/src/mutation/operators.rs --max 10
✅ Max cyclomatic complexity: 7 (target: <10)

# Quality score
$ pmat quality-score rash/src/mutation/operators.rs --min 9.0
✅ Quality score: 9.3/10

# Full quality gate
$ pmat analyze quality rash/src/mutation/ --min-score 9.0 --max-complexity 10
✅ All modules pass quality gates
```

### Test Execution Engine

```rust
// rash/src/mutation/executor.rs
pub struct MutationTestExecutor {
    sandbox: Sandbox,
    timeout: Duration,
}

impl MutationTestExecutor {
    pub fn execute_mutant(&self, mutant: &Mutant, tests: &[BashTest]) -> MutantResult {
        // 1. Generate mutated script
        let mutated_script = emit_bash(&mutant.mutated_ast);

        // 2. Write to temp file
        let temp_file = self.sandbox.write_temp(&mutated_script);

        // 3. Run all tests with timeout
        let start = Instant::now();
        let mut killed_by = None;

        for test in tests {
            match self.run_test_with_timeout(&temp_file, test, self.timeout) {
                TestResult::Pass => continue, // Test passed, mutant survived this test
                TestResult::Fail(reason) => {
                    // Test failed, mutant is KILLED!
                    killed_by = Some((test.name.clone(), reason));
                    break;
                }
                TestResult::Timeout => {
                    // Test timed out, mutant is KILLED!
                    killed_by = Some((test.name.clone(), "timeout".to_string()));
                    break;
                }
            }
        }

        let duration = start.elapsed();

        match killed_by {
            Some((test_name, reason)) => MutantResult::Killed {
                mutant_id: mutant.id.clone(),
                killed_by: test_name,
                reason,
                duration,
            },
            None => MutantResult::Survived {
                mutant_id: mutant.id.clone(),
                mutant: mutant.clone(),
                duration,
            },
        }
    }

    fn run_test_with_timeout(
        &self,
        script: &Path,
        test: &BashTest,
        timeout: Duration,
    ) -> TestResult {
        use std::process::Command;
        use std::time::Instant;

        let start = Instant::now();

        // Run test function from script
        let output = Command::new("bash")
            .arg("-c")
            .arg(format!("source {} && {}", script.display(), test.name))
            .timeout(timeout)
            .output();

        match output {
            Ok(output) if output.status.success() => TestResult::Pass,
            Ok(output) => TestResult::Fail(
                String::from_utf8_lossy(&output.stderr).to_string()
            ),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => TestResult::Timeout,
            Err(e) => TestResult::Fail(e.to_string()),
        }
    }
}
```

### CLI Usage

```bash
# Basic usage
$ bashrs mutate script.sh

🧬 Mutation Testing: script.sh
📝 Generating mutants...
✅ Generated 45 mutants

🧪 Running tests on mutants (45 mutants × ~30s = 22.5 minutes)...

[1/45] COR_12ae: [ -f ] → [ -d ] ... ✅ KILLED by test_check_file_exists (2.3s)
[2/45] COR_f072: [ -z ] → [ -n ] ... ✅ KILLED by test_empty_string (2.1s)
[3/45] AOR_9a4d: + → - ... ❌ SURVIVED (2.4s)
[4/45] CRR_fcbc: rm -f → echo ... ✅ KILLED by test_cleanup (2.2s)
...
[45/45] FMR_b5be: mkdir -p → mkdir ... ✅ KILLED by test_idempotent_create (2.3s)

📊 Mutation Testing Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Mutants:    45
Killed:           41 (91.1%)  ✅
Survived:         4  (8.9%)   ⚠️
Timeout:          0  (0%)
Error:            0  (0%)

Mutation Score:   91.1% ✅ PASS (target: ≥90%)

⏱  Total Time:     22.3 minutes
⚡ Avg per mutant: 29.7s

⚠️  Surviving Mutants (Test Gaps)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. AOR_9a4d (Line 23)
   Mutation: count=$((count + 1)) → count=$((count - 1))
   Operator: Arithmetic Operator Replacement
   Suggestion: Add test verifying counter increment logic

   test_counter_increment() {
       count=0
       increment_counter
       [ "$count" -eq 1 ] || return 1
   }

2. SOR_ba36 (Line 45)
   Mutation: [ "$status" = "success" ] → [ "$status" != "success" ]
   Operator: String Operator Replacement
   Suggestion: Add test verifying success status handling

   test_success_status() {
       status="success"
       process_status "$status"
       # Verify expected behavior
   }

3. FMR_dd2e (Line 67)
   Mutation: ln -sf → ln -s
   Operator: Flag Mutation
   Suggestion: Add test verifying symlink overwrite behavior

   test_symlink_overwrites_existing() {
       ln -s old_target link
       create_symlink new_target link
       [ "$(readlink link)" = "new_target" ] || return 1
   }

4. QMR_483b (Line 89)
   Mutation: rm "$file" → rm $file
   Operator: Quote Mutation
   Suggestion: Add test with filename containing spaces

   test_handles_spaces_in_filename() {
       file="test file.txt"
       touch "$file"
       cleanup_file "$file"
       [ ! -f "$file" ] || return 1
   }

💡 Recommendations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Add 4 tests to cover surviving mutants
2. Re-run mutation testing after adding tests
3. Target: 100% mutation coverage (0 surviving mutants)

📄 Detailed Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Full report saved to: mutation_report.json
HTML report: mutation_report.html
```

---

## Feature 2: bashrs fuzz (Fuzzing for Bash)

### Overview

**What**: Fuzzing for bash scripts with property verification
- Generate random/smart inputs
- Detect crashes, hangs, injection vulnerabilities
- Verify properties (idempotency, determinism)
- Security testing (command injection, path traversal)

**Why**:
- Bash scripts often have injection vulnerabilities
- No good bash fuzzer exists (AFL/libFuzzer don't work well)
- Property verification ensures robustness

**Target**: Find injection vulnerabilities shellcheck misses

### Fuzzing Strategies

#### 1. Mutation-Based Fuzzing

```rust
// Start with valid input, mutate it
pub struct MutationFuzzer {
    corpus: Vec<FuzzInput>,
}

impl MutationFuzzer {
    pub fn mutate(&self, input: &FuzzInput) -> FuzzInput {
        let mut mutated = input.clone();

        match rand::random::<u8>() % 10 {
            0 => self.bit_flip(&mut mutated),      // Flip random bits
            1 => self.byte_insert(&mut mutated),   // Insert random bytes
            2 => self.byte_delete(&mut mutated),   // Delete bytes
            3 => self.splice(&mut mutated),        // Splice with corpus
            4 => self.inject_special(&mut mutated),// Inject special chars
            5 => self.path_traversal(&mut mutated),// Inject ../
            6 => self.command_injection(&mut mutated), // Inject ;
            7 => self.null_bytes(&mut mutated),    // Inject \x00
            8 => self.format_strings(&mut mutated),// Inject %s %n
            9 => self.unicode(&mut mutated),       // Inject unicode
            _ => unreachable!(),
        }

        mutated
    }
}
```

#### 2. Grammar-Based Fuzzing

```rust
// Use bash grammar to generate valid inputs
pub struct GrammarFuzzer {
    grammar: BashGrammar,
}

impl GrammarFuzzer {
    pub fn generate(&self) -> FuzzInput {
        // Generate syntactically valid bash input
        let args = self.grammar.generate_args();
        let env = self.grammar.generate_env();
        let stdin = self.grammar.generate_stdin();

        FuzzInput { args, env, stdin }
    }
}
```

#### 3. Property-Based Fuzzing

```rust
// Generate inputs that test specific properties
pub enum BashProperty {
    Idempotent,       // f(f(x)) = f(x)
    Deterministic,    // f(x) = f(x) always
    NoInjection,      // No command injection
    SafeFailure,      // Failures don't corrupt state
    NoPathTraversal,  // Can't escape directory
}

pub fn verify_property(
    script: &str,
    property: BashProperty,
    inputs: Vec<FuzzInput>,
) -> PropertyVerificationResult {
    match property {
        BashProperty::Idempotent => {
            for input in inputs {
                let result1 = run_script(script, &input);
                let result2 = run_script(script, &input);

                if result1 != result2 {
                    return PropertyVerificationResult::Violated {
                        property,
                        input,
                        reason: "Multiple runs produced different results".into(),
                    };
                }
            }
            PropertyVerificationResult::Verified
        }

        BashProperty::NoInjection => {
            let injection_payloads = [
                "; rm -rf /",
                "$(whoami)",
                "`cat /etc/passwd`",
                "$((1+1)); ls",
                "| cat /etc/passwd",
                "&& cat /etc/passwd",
                "|| cat /etc/passwd",
            ];

            for payload in injection_payloads {
                let input = FuzzInput::with_arg(payload);
                let result = run_script_in_sandbox(script, &input);

                if result.executed_unintended_command() {
                    return PropertyVerificationResult::Violated {
                        property,
                        input,
                        reason: format!("Command injection: {}", payload),
                    };
                }
            }
            PropertyVerificationResult::Verified
        }

        // ... more properties
    }
}
```

### Sandbox Execution

```rust
// rash/src/fuzz/sandbox.rs
pub struct BashSandbox {
    container: Option<Container>,
    limits: ResourceLimits,
}

pub struct ResourceLimits {
    timeout: Duration,
    memory: usize,      // bytes
    disk: usize,        // bytes
    processes: usize,   // max processes
    network: bool,      // allow network?
}

impl BashSandbox {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            container: Container::new_if_available("alpine:latest"),
            limits,
        }
    }

    pub fn execute(&self, script: &str, input: &FuzzInput) -> FuzzResult {
        if let Some(container) = &self.container {
            self.execute_in_container(container, script, input)
        } else {
            self.execute_in_process(script, input)
        }
    }

    fn execute_in_container(
        &self,
        container: &Container,
        script: &str,
        input: &FuzzInput,
    ) -> FuzzResult {
        // 1. Copy script to container
        container.write_file("/tmp/script.sh", script);

        // 2. Set up resource limits
        container.set_limits(&self.limits);

        // 3. Execute with input
        let result = container.exec(
            &["/bin/sh", "/tmp/script.sh"],
            &input.args,
            &input.env,
            &input.stdin,
        );

        // 4. Analyze result
        self.analyze_result(result)
    }

    fn analyze_result(&self, result: ExecResult) -> FuzzResult {
        match result {
            Ok(output) => FuzzResult::Success(output),
            Err(ExecError::Timeout) => FuzzResult::Hang,
            Err(ExecError::Crash(signal)) => FuzzResult::Crash(signal),
            Err(ExecError::MemoryLimit) => FuzzResult::ResourceExhaustion("memory"),
            Err(ExecError::DiskLimit) => FuzzResult::ResourceExhaustion("disk"),
            Err(ExecError::ProcessLimit) => FuzzResult::ResourceExhaustion("processes"),
        }
    }
}
```

### CLI Usage

```bash
# Basic fuzzing
$ bashrs fuzz script.sh --iterations 10000

🔬 Fuzzing: script.sh
📊 Strategy: Smart mutation + grammar-based
⏱  Timeout: 5s per execution
🔒 Sandbox: Docker container (alpine:latest)

[100/10000] Iterations (1% complete)...
  ✅ Normal inputs: 95 passed
  ⚠️  Hang detected: Input "" (empty string)

[500/10000] Iterations (5% complete)...
  ✅ Normal inputs: 487 passed
  🚨 Command injection found!
     Input: file.txt; rm -rf /tmp
     Line: 45
     Code: rm $file

[1000/10000] Iterations (10% complete)...
  ✅ Normal inputs: 982 passed
  🚨 Path traversal found!
     Input: ../../etc/passwd
     Line: 67
     Code: cat "$input"

...

[10000/10000] Iterations (100% complete)

📊 Fuzzing Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Iterations: 10,000
Successful:       9,982 (99.8%)
Hangs:            3    (0.03%)
Crashes:          0    (0%)
Vulnerabilities:  15   (0.15%)

Execution Speed:  125 execs/sec
Total Time:       1m 20s

🚨 Security Vulnerabilities Found
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Command Injection (HIGH)
   Line: 45
   Code: rm $file
   Input: "file.txt; rm -rf /tmp"
   Impact: Allows arbitrary command execution
   Fix: Quote variable: rm "$file"

2. Path Traversal (MEDIUM)
   Line: 67
   Code: cat "$input"
   Input: "../../etc/passwd"
   Impact: Can read files outside intended directory
   Fix: Validate path before use:

   validate_path() {
       case "$1" in
           *..*)  return 1 ;;  # Reject path traversal
           /*)    return 1 ;;  # Reject absolute paths
       esac
       return 0
   }

3. Unquoted Variable Expansion (MEDIUM)
   Line: 89
   Code: for f in $FILES; do
   Input: FILES="a b c"
   Impact: Word splitting on spaces (may be intended or bug)
   Fix: Quote if single item expected: for f in "$FILES"; do

...

⚠️  Hangs Detected
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Empty Input Hang
   Input: ""
   Location: Line 123 (read loop)
   Cause: Infinite loop waiting for input
   Fix: Add timeout or validate input first

💡 Recommendations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Fix 15 security vulnerabilities (2 high, 13 medium)
2. Add input validation for all user-controlled data
3. Quote all variable expansions
4. Add timeouts to prevent hangs
5. Re-run fuzzing after fixes

📄 Detailed Reports
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

JSON report: fuzz_report.json
HTML report: fuzz_report.html
Minimized inputs: fuzz_crashes/
```

### Property Verification

```bash
# Verify specific properties
$ bashrs fuzz script.sh --property idempotent --iterations 1000

🔬 Property Verification: Idempotency
📊 Testing script.sh with 1000 random inputs

[1000/1000] Iterations complete

✅ Idempotency: VERIFIED
   All 1000 inputs produced identical results on repeated execution

$ bashrs fuzz script.sh --property no-injection --iterations 1000

🔬 Property Verification: No Command Injection
📊 Testing script.sh with known injection payloads

❌ Injection Vulnerability: FOUND
   Payload: "; rm -rf /"
   Line: 45
   Code: rm $file

🚨 FAIL: Script vulnerable to command injection
```

---

## EXTREME TDD Methodology

### Test-First Development (Mandatory)

**Every feature follows this cycle**:

1. **RED**: Write failing test
2. **GREEN**: Implement minimum code to pass
3. **REFACTOR**: Clean up implementation
4. **PROPERTY**: Add property-based tests
5. **MUTATION**: Run mutation testing
6. **PMAT**: Verify quality gates

### Quality Gates (Enforced by CI)

**Before any code is merged**:

```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates (EXTREME TDD + pmat)

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      # Phase 1: Unit Tests (RED-GREEN)
      - name: Run unit tests
        run: cargo test --lib
        # Must pass: 100% tests passing

      # Phase 2: Property Tests
      - name: Run property tests
        run: cargo test --lib -- --include-ignored property
        # Must pass: 100% property tests passing

      # Phase 3: Mutation Testing
      - name: Run mutation tests
        run: cargo mutants --timeout 60
        # Must pass: ≥90% kill rate

      # Phase 4: pmat Quality Gates
      - name: Complexity analysis
        run: pmat analyze complexity --max 10
        # Must pass: All functions cyclomatic complexity <10

      - name: Quality score
        run: pmat quality-score --min 9.0
        # Must pass: Overall quality score ≥9.0

      - name: Security scan
        run: pmat analyze security
        # Must pass: No security issues

      # Phase 5: Integration Tests
      - name: Run integration tests
        run: cargo test --test '*'
        # Must pass: 100% integration tests passing
```

### Coverage Requirements

**Minimum coverage for each module**:

- **Unit tests**: ≥85% line coverage
- **Property tests**: ≥3 properties per major function
- **Mutation score**: ≥90% kill rate
- **Integration tests**: All CLI workflows covered

### Example: Complete TDD Cycle for Command Replacement

```rust
// PHASE 1: RED (Failing Test)
#[test]
fn test_rm_command_to_echo_mutation() {
    let bash_code = r#"
cleanup() {
    rm -rf "$dir"
}
"#;
    let ast = parse_bash(bash_code).unwrap();
    let mutants = generate_mutants(&ast, &[MutationOperator::CommandReplacement]);

    // Should generate rm → echo mutant
    assert!(mutants.iter().any(|m|
        m.original_code.contains("rm -rf") &&
        m.mutated_code.contains("echo")
    ));
}
// cargo test → FAILS (not implemented yet)

// PHASE 2: GREEN (Minimal Implementation)
fn generate_command_replacement_mutants(ast: &BashAst) -> Vec<Mutant> {
    let mut mutants = Vec::new();

    for node in ast.walk() {
        if let AstNode::Command(cmd) = node {
            if cmd.name == "rm" {
                mutants.push(Mutant {
                    original_code: format!("rm {}", cmd.args.join(" ")),
                    mutated_code: format!("echo {}", cmd.args.join(" ")),
                    // ...
                });
            }
        }
    }

    mutants
}
// cargo test → PASSES

// PHASE 3: REFACTOR (Clean Up)
const DESTRUCTIVE_COMMANDS: &[(&str, &str)] = &[
    ("rm", "echo"),
    ("mv", "echo"),
    ("dd", "echo"),
    ("truncate", "echo"),
];

fn generate_command_replacement_mutants(ast: &BashAst) -> Vec<Mutant> {
    let mut mutants = Vec::new();

    for node in ast.walk() {
        if let AstNode::Command(cmd) = node {
            for (destructive, safe) in DESTRUCTIVE_COMMANDS {
                if cmd.name == *destructive {
                    mutants.push(create_command_mutant(node, destructive, safe));
                }
            }
        }
    }

    mutants
}
// cargo test → STILL PASSES

// PHASE 4: PROPERTY TESTING
proptest! {
    #[test]
    fn prop_destructive_commands_get_neutered(
        cmd_name in "(rm|mv|dd|truncate)",
        args in prop::collection::vec("[a-z/]{1,20}", 1..5)
    ) {
        let bash_code = format!("{} {}", cmd_name, args.join(" "));
        let ast = parse_bash(&bash_code).unwrap();
        let mutants = generate_mutants(&ast, &[MutationOperator::CommandReplacement]);

        // Should generate at least one mutant
        prop_assert!(mutants.len() > 0);

        // All mutants should replace destructive command with safe one
        for mutant in &mutants {
            prop_assert!(
                !mutant.mutated_code.contains(&cmd_name),
                "Mutant should not contain destructive command"
            );
        }
    }
}
// cargo test → PASSES

// PHASE 5: MUTATION TESTING
// $ cargo mutants --file rash/src/mutation/operators.rs
// Expected: ≥90% kill rate

// PHASE 6: PMAT QUALITY GATES
// $ pmat analyze complexity rash/src/mutation/operators.rs --max 10
// ✅ Max cyclomatic: 7
// $ pmat quality-score rash/src/mutation/operators.rs --min 9.0
// ✅ Quality score: 9.4/10
```

---

## Implementation Phases

### Phase 1: bashrs mutate (6 weeks)

**Week 1: Core Infrastructure + COR**
- ✅ RED: Test for conditional operator mutations
- ✅ GREEN: Implement COR (Conditional Operator Replacement)
- ✅ REFACTOR: Clean up, extract helpers
- ✅ PROPERTY: Add property tests
- ✅ PMAT: Verify complexity <10, quality ≥9.0

**Week 2: String & Arithmetic Operators**
- ✅ RED: Tests for SOR and AOR
- ✅ GREEN: Implement SOR (String Operator Replacement)
- ✅ GREEN: Implement AOR (Arithmetic Operator Replacement)
- ✅ REFACTOR: Extract common patterns
- ✅ PROPERTY: Property tests for both
- ✅ PMAT: Quality gates

**Week 3: Command & Flag Mutations**
- ✅ RED: Tests for CRR and FMR
- ✅ GREEN: Implement CRR (Command Replacement)
- ✅ GREEN: Implement FMR (Flag Mutation)
- ✅ REFACTOR: Unify mutation generation
- ✅ PROPERTY: Property tests
- ✅ PMAT: Quality gates

**Week 4: Test Execution Engine**
- ✅ RED: Test executor tests
- ✅ GREEN: Implement sandbox execution
- ✅ GREEN: Implement timeout handling
- ✅ GREEN: Implement result collection
- ✅ REFACTOR: Optimize parallel execution
- ✅ PROPERTY: Test with various inputs
- ✅ PMAT: Quality gates

**Week 5: Reporting & CLI**
- ✅ RED: Report generation tests
- ✅ GREEN: Implement JSON/HTML reports
- ✅ GREEN: Implement CLI interface
- ✅ GREEN: Implement progress display
- ✅ REFACTOR: Polish UX
- ✅ INTEGRATION: End-to-end CLI tests
- ✅ PMAT: Quality gates

**Week 6: Integration & Validation**
- ✅ Run on real bash scripts (dogfooding)
- ✅ Validate ≥90% kill rate
- ✅ Performance benchmarks
- ✅ Documentation
- ✅ Final pmat quality verification

### Phase 2: bashrs fuzz (12 weeks)

**Weeks 1-2: Input Generators**
- ✅ RED: Generator tests
- ✅ GREEN: Mutation-based fuzzer
- ✅ GREEN: Grammar-based fuzzer
- ✅ GREEN: Injection payload generator
- ✅ REFACTOR: Unified generator interface
- ✅ PROPERTY: Validate generated inputs
- ✅ PMAT: Quality gates

**Weeks 3-4: Sandbox Execution**
- ✅ RED: Sandbox tests
- ✅ GREEN: Container-based sandbox
- ✅ GREEN: Process-based sandbox (fallback)
- ✅ GREEN: Resource limits enforcement
- ✅ REFACTOR: Abstract sandbox interface
- ✅ PROPERTY: Test with malicious inputs
- ✅ PMAT: Quality gates

**Weeks 5-6: Property Verification**
- ✅ RED: Property verifier tests
- ✅ GREEN: Idempotency verification
- ✅ GREEN: Determinism verification
- ✅ GREEN: Injection detection
- ✅ GREEN: Path traversal detection
- ✅ REFACTOR: Property framework
- ✅ PROPERTY: Meta-verification
- ✅ PMAT: Quality gates

**Weeks 7-8: Coverage & Corpus Management**
- ✅ RED: Coverage tracker tests
- ✅ GREEN: Execution path tracking
- ✅ GREEN: Corpus management (interesting inputs)
- ✅ GREEN: Input minimization
- ✅ GREEN: Crash reproduction
- ✅ REFACTOR: Optimize storage
- ✅ PROPERTY: Verify corpus quality
- ✅ PMAT: Quality gates

**Weeks 9-10: Security Analysis**
- ✅ RED: Security detector tests
- ✅ GREEN: Command injection detector
- ✅ GREEN: Path traversal detector
- ✅ GREEN: Quote escaping validator
- ✅ GREEN: Environment pollution detector
- ✅ REFACTOR: Security framework
- ✅ PROPERTY: Test with known CVEs
- ✅ PMAT: Quality gates

**Weeks 11-12: Integration & Validation**
- ✅ CLI integration
- ✅ Run on real bash scripts
- ✅ Validate vulnerability detection
- ✅ Performance benchmarks
- ✅ Documentation
- ✅ Final pmat quality verification

---

## Success Criteria

### bashrs mutate

**Functional**:
- ✅ Generates ≥90% meaningful mutants (no duplicates/invalids)
- ✅ Achieves ≥90% kill rate on well-tested scripts
- ✅ Completes mutation testing in reasonable time (≤60s per mutant)
- ✅ Provides actionable feedback (shows surviving mutants with suggestions)

**Quality** (pmat enforced):
- ✅ All modules: Cyclomatic complexity <10
- ✅ All modules: Quality score ≥9.0
- ✅ All modules: Mutation score ≥90%
- ✅ Zero security issues
- ✅ Zero dead code

**Performance**:
- ✅ Parallel execution (use all CPU cores)
- ✅ Smart test filtering (stop after first kill)
- ✅ Incremental testing (cache results)

### bashrs fuzz

**Functional**:
- ✅ Finds injection vulnerabilities shellcheck misses
- ✅ Verifies idempotency/determinism properties
- ✅ Runs safely in sandbox (no host damage)
- ✅ Generates actionable security reports

**Quality** (pmat enforced):
- ✅ All modules: Cyclomatic complexity <10
- ✅ All modules: Quality score ≥9.0
- ✅ All modules: Mutation score ≥90%
- ✅ Zero security issues in fuzzer itself
- ✅ Zero dead code

**Performance**:
- ✅ ≥100 execs/sec (vs AFL's millions/sec, but bash is interpreted)
- ✅ Smart input generation (not pure random)
- ✅ Corpus management (save interesting inputs)

---

## Risks & Mitigations

### Risk 1: Slow Execution (Bash is Interpreted)

**Risk**: Mutation testing/fuzzing might be too slow for practical use

**Mitigations**:
1. Parallel execution (use all cores)
2. Smart test filtering (stop after first kill)
3. Incremental testing (cache results)
4. User expectations (be honest about speed, like cargo-mutants)

**Example**: cargo-mutants takes ~30s per mutant, we'll match that

### Risk 2: Sandbox Escapes

**Risk**: Malicious bash code might escape sandbox and damage host

**Mitigations**:
1. Use Docker containers (strong isolation)
2. Resource limits (CPU, memory, disk, processes)
3. Network isolation (no external access)
4. Read-only mounts (can't modify host)
5. Timeout enforcement (kill runaway scripts)

**Example**: Run all fuzzing in disposable Alpine containers

### Risk 3: False Positives (Over-reporting)

**Risk**: Might report too many "vulnerabilities" that aren't real

**Mitigations**:
1. Context-aware detection (understand bash semantics)
2. Severity ratings (HIGH/MEDIUM/LOW)
3. Configurable thresholds (let users tune)
4. Clear explanations (why it's a problem)

**Example**: Quote mutations only reported if variable could contain user input

### Risk 4: Complexity Creep

**Risk**: Code might become too complex (violate pmat gates)

**Mitigations**:
1. EXTREME TDD (refactor constantly)
2. pmat gates in CI (fail if complexity ≥10)
3. Regular refactoring sprints
4. Extract helper functions aggressively

**Example**: If any function hits complexity 8, immediate refactor

---

## Comparison to Existing Tools

### bashrs mutate vs pmat

| Feature | pmat | bashrs mutate |
|---------|------|---------------|
| **Language Support** | Generic (Rust, Python, JS, Bash) | **Bash-specific** |
| **Rust Kill Rate** | 0% (FAILED) | N/A |
| **Bash Kill Rate** | Unknown (likely poor) | **≥90% (expected)** |
| **Semantics** | Generic operators | **Bash-specific operators** |
| **Speed Claim** | "20× faster" (FALSE) | **Honest: ~30s/mutant** |
| **Mutant Quality** | Redundant/invalid | **Focused, meaningful** |
| **Integration** | Standalone | **Native bashrs** |
| **Maturity** | Research tool | **Production-ready** |

**Key Advantage**: Language-specific understanding (like cargo-mutants for Rust)

### bashrs fuzz vs AFL

| Feature | AFL | bashrs fuzz |
|---------|-----|-------------|
| **Target** | Compiled binaries | **Bash scripts** |
| **Speed** | Millions execs/sec | **~100 execs/sec** |
| **Coverage** | Binary instrumentation | **Heuristic-based** |
| **Properties** | Crash detection | **Property verification** |
| **Security** | Generic crashes | **Bash-specific vulns** |
| **Sandbox** | Minimal | **Docker containers** |

**Key Advantage**: Bash-specific vulnerability detection (injection, traversal)

---

## Documentation Requirements

### User Documentation

1. **README**: High-level overview, installation, quick start
2. **Tutorial**: Step-by-step mutation testing & fuzzing
3. **Reference**: Complete operator reference, CLI flags
4. **Cookbook**: Common patterns, best practices
5. **FAQ**: Common questions, troubleshooting

### Developer Documentation

1. **Architecture**: System design, module structure
2. **EXTREME TDD Guide**: How to add new operators
3. **pmat Integration**: How quality gates work
4. **Testing Strategy**: Unit, property, mutation, integration
5. **Performance**: Benchmarks, optimization tips

### Example Documentation

```markdown
# bashrs mutate - Tutorial

## Installation

```bash
cargo install bashrs
```

## Quick Start

1. Write bash script with tests:

```bash
#!/bin/bash

add() {
    echo $(($1 + $2))
}

test_add() {
    result=$(add 2 3)
    [ "$result" -eq 5 ] || return 1
}
```

2. Run mutation testing:

```bash
$ bashrs mutate script.sh
```

3. Review results and add tests for surviving mutants

4. Re-run until 100% coverage

## Advanced Usage

See [MUTATION_TESTING.md](./MUTATION_TESTING.md) for:
- Custom operators
- Performance tuning
- Integration with CI/CD
- Interpreting results
```

---

## Conclusion

**bashrs mutate** and **bashrs fuzz** will provide:

1. **Unique value**: First bash-specific mutation tester & fuzzer
2. **Quality**: EXTREME TDD + pmat enforcement guarantees robustness
3. **Practical**: Realistic performance expectations (no false "20× faster" claims)
4. **Dogfooded**: We'll use it on bashrs itself

**Next Steps**:
1. Review and approve this spec
2. Start Phase 1: bashrs mutate Week 1
3. Follow EXTREME TDD religiously
4. Ship high-quality, well-tested features

**Timeline**:
- bashrs mutate: 6 weeks (production-ready)
- bashrs fuzz: 12 weeks (production-ready)
- Total: 18 weeks to both features

**Quality Guarantee**:
- Every module: Complexity <10
- Every module: Quality score ≥9.0
- Every module: Mutation score ≥90%
- Zero compromises on quality
