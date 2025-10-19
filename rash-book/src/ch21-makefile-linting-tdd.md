# Chapter 21: Makefile and Shell Linting

## Introduction

Rash provides comprehensive linting capabilities for both Makefiles and shell scripts. This chapter demonstrates the five Makefile linting rules (MAKE001-005) and how they improve build safety, determinism, and idempotency.

**Why Linting Matters:**
- Catch bugs before they cause build failures
- Enforce deterministic builds (same input = same output)
- Ensure idempotent operations (safe to re-run)
- Improve maintainability and safety

---

## MAKE001: Non-deterministic Wildcard

**Problem**: File globbing with `$(wildcard)` produces system-dependent ordering, causing non-deterministic builds.

**Detection**: Finds `$(wildcard ...)` without `$(sort ...)` wrapper.

### Example: Representing Linting Results

```rust
#[derive(Debug)]
struct LintDiagnostic {
    line_number: usize,
    message: String,
    severity: Severity,
    fix: Option<String>,
}

#[derive(Debug)]
enum Severity {
    Warning,
    Error,
}

fn main() {
    // Simulating detection of non-deterministic wildcard
    let makefile_line = "SOURCES = $(wildcard src/*.c)";

    let diagnostic = LintDiagnostic {
        line_number: 5,
        message: "Non-deterministic wildcard usage".to_string(),
        severity: Severity::Warning,
        fix: Some("$(sort $(wildcard src/*.c))".to_string()),
    };

    println!("Line {}: {:?}", diagnostic.line_number, diagnostic.severity);
    println!("  {}", diagnostic.message);

    if let Some(fix) = diagnostic.fix {
        println!("  Suggested fix: {}", fix);
    }
}
```

### Example: Checking for Wildcard Pattern

```rust
fn has_unsorted_wildcard(line: &str) -> bool {
    line.contains("$(wildcard") && !line.contains("$(sort")
}

fn main() {
    let unsafe_line = "SOURCES = $(wildcard src/*.c)";
    let safe_line = "SOURCES = $(sort $(wildcard src/*.c))";

    println!("Unsafe: {}", has_unsorted_wildcard(unsafe_line));
    // Output: Unsafe: true

    println!("Safe: {}", has_unsorted_wildcard(safe_line));
    // Output: Safe: false
}
```

---

## MAKE002: Non-idempotent mkdir

**Problem**: Using `mkdir` without `-p` flag fails on second run if directory exists.

**Detection**: Finds `mkdir` in recipe commands (lines starting with tab).

### Example: Detecting mkdir in Recipes

```rust
fn is_recipe_line(line: &str) -> bool {
    line.starts_with('\t')
}

fn contains_unsafe_mkdir(line: &str) -> bool {
    line.contains("mkdir") && !line.contains("mkdir -p")
}

fn main() {
    let recipe_line = "\tmkdir build";
    let safe_recipe = "\tmkdir -p build";

    if is_recipe_line(recipe_line) && contains_unsafe_mkdir(recipe_line) {
        println!("Found non-idempotent mkdir");
        println!("Fix: mkdir -p build");
    }

    if is_recipe_line(safe_recipe) && contains_unsafe_mkdir(safe_recipe) {
        println!("This won't print - line is safe");
    } else {
        println!("Safe recipe: uses mkdir -p");
    }
}
```

### Example: Applying mkdir Fix

```rust
fn fix_mkdir(line: &str) -> String {
    if line.contains("mkdir") && !line.contains("mkdir -p") {
        line.replace("mkdir ", "mkdir -p ")
    } else {
        line.to_string()
    }
}

fn main() {
    let original = "\tmkdir build";
    let fixed = fix_mkdir(original);

    println!("Original: {}", original.trim());
    println!("Fixed:    {}", fixed.trim());
    // Output: Fixed:    mkdir -p build
}
```

---

## MAKE003: Unsafe Variable Expansion

**Problem**: Unquoted variables in dangerous commands (rm, cp, mv) can cause word splitting and security issues.

**Detection**: Finds unquoted `$VAR` or `$(VAR)` in rm, cp, mv, chmod, chown commands.

### Example: Variable Quoting Checker

```rust
fn is_dangerous_command(line: &str) -> bool {
    let dangerous = ["rm ", "cp ", "mv ", "chmod ", "chown "];
    dangerous.iter().any(|cmd| line.contains(cmd))
}

fn has_unquoted_variable(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let before_quote = if i > 0 { chars[i - 1] != '"' } else { true };
            if before_quote {
                return true;
            }
        }
    }
    false
}

fn main() {
    let unsafe_line = "\trm -rf $BUILD_DIR/*";
    let safe_line = "\trm -rf \"$BUILD_DIR\"/*";

    if is_dangerous_command(unsafe_line) && has_unquoted_variable(unsafe_line) {
        println!("⚠ Unsafe variable expansion detected");
        println!("  Line: {}", unsafe_line.trim());
        println!("  Fix: rm -rf \"$BUILD_DIR\"/*");
    }

    if is_dangerous_command(safe_line) && has_unquoted_variable(safe_line) {
        println!("This won't print - variables are quoted");
    } else {
        println!("✓ Safe: variables properly quoted");
    }
}
```

---

## MAKE004: Missing .PHONY Declaration

**Problem**: Targets with common names (clean, test, install) conflict with files of the same name.

**Detection**: Checks if common non-file targets are marked as `.PHONY`.

### Example: Tracking PHONY Targets

```rust
use std::collections::HashSet;

fn main() {
    let common_phony_targets = vec![
        "all", "clean", "test", "install", "uninstall",
        "build", "run", "help", "lint", "format",
    ];

    let makefile = r#"
.PHONY: clean test

build:
	gcc main.c -o app

clean:
	rm -f *.o

test:
	./app --test
"#;

    let mut declared_phony: HashSet<String> = HashSet::new();
    let mut found_targets: HashSet<String> = HashSet::new();

    for line in makefile.lines() {
        if line.starts_with(".PHONY:") {
            let targets = line.replace(".PHONY:", "").trim().to_string();
            for target in targets.split_whitespace() {
                declared_phony.insert(target.to_string());
            }
        } else if line.contains(':') && !line.starts_with('\t') && !line.starts_with('.') {
            if let Some(target) = line.split(':').next() {
                let target = target.trim().to_string();
                if common_phony_targets.contains(&target.as_str()) {
                    found_targets.insert(target);
                }
            }
        }
    }

    for target in &found_targets {
        if !declared_phony.contains(target) {
            println!("⚠ Target '{}' should be marked as .PHONY", target);
        }
    }

    println!("\nDeclared as .PHONY: {:?}", declared_phony);
    println!("Missing .PHONY: {:?}",
             found_targets.difference(&declared_phony).collect::<Vec<_>>());
}
```

---

## MAKE005: Recursive Variable Assignment

**Problem**: Using `=` with `$(shell ...)` causes command to re-execute every time variable is referenced.

**Detection**: Finds `=` (not `:=`, `+=`, `?=`, `!=`) followed by `$(shell ...)`.

### Example: Assignment Type Detector

```rust
#[derive(Debug, PartialEq)]
enum AssignmentType {
    Recursive,      // =
    Immediate,      // :=
    Conditional,    // ?=
    Append,         // +=
    Shell,          // !=
}

fn detect_assignment_type(line: &str) -> Option<AssignmentType> {
    if line.contains(":=") {
        Some(AssignmentType::Immediate)
    } else if line.contains("!=") {
        Some(AssignmentType::Shell)
    } else if line.contains("+=") {
        Some(AssignmentType::Append)
    } else if line.contains("?=") {
        Some(AssignmentType::Conditional)
    } else if line.contains('=') {
        Some(AssignmentType::Recursive)
    } else {
        None
    }
}

fn main() {
    let problematic = "VERSION = $(shell git describe)";
    let correct = "VERSION := $(shell git describe)";

    let prob_type = detect_assignment_type(problematic);
    let correct_type = detect_assignment_type(correct);

    println!("Problematic line uses: {:?}", prob_type);
    println!("Correct line uses: {:?}", correct_type);

    if prob_type == Some(AssignmentType::Recursive) && problematic.contains("$(shell") {
        println!("\n⚠ Recursive assignment with $(shell) detected!");
        println!("  This will re-execute the command every time VERSION is used");
        println!("  Fix: Change = to :=");
    }
}
```

### Example: Shell Command Impact

```rust
fn main() {
    println!("=== Recursive Assignment (=) ===");
    println!("VERSION = $(shell git describe)");
    println!("  - Command runs EVERY time VERSION is referenced");
    println!("  - Non-deterministic if repo state changes");
    println!("  - Performance impact");

    println!("\n=== Immediate Assignment (:=) ===");
    println!("VERSION := $(shell git describe)");
    println!("  - Command runs ONCE when Makefile is parsed");
    println!("  - Result cached for all uses");
    println!("  - Deterministic and fast");

    let recursive_cost = 5; // arbitrary cost units
    let references = 10;
    let immediate_cost = 5;

    println!("\n=== Performance Comparison ===");
    println!("Recursive (=):  {} * {} refs = {} units", recursive_cost, references, recursive_cost * references);
    println!("Immediate (:=): {} * 1 exec   = {} units", immediate_cost, immediate_cost);
    println!("Savings: {}x faster", (recursive_cost * references) / immediate_cost);
}
```

---

## Complete Linting Workflow

### Example: Full Makefile Analyzer

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct LintResults {
    total_issues: usize,
    issues_by_rule: HashMap<String, usize>,
}

impl LintResults {
    fn new() -> Self {
        Self {
            total_issues: 0,
            issues_by_rule: HashMap::new(),
        }
    }

    fn add_issue(&mut self, rule: &str) {
        self.total_issues += 1;
        *self.issues_by_rule.entry(rule.to_string()).or_insert(0) += 1;
    }
}

fn lint_makefile(content: &str) -> LintResults {
    let mut results = LintResults::new();

    for (line_num, line) in content.lines().enumerate() {
        // MAKE001: Non-deterministic wildcard
        if line.contains("$(wildcard") && !line.contains("$(sort") {
            results.add_issue("MAKE001");
        }

        // MAKE002: Non-idempotent mkdir
        if line.starts_with('\t') && line.contains("mkdir") && !line.contains("mkdir -p") {
            results.add_issue("MAKE002");
        }

        // MAKE003: Unsafe variable expansion
        let dangerous = ["rm ", "cp ", "mv "];
        if dangerous.iter().any(|cmd| line.contains(cmd)) && line.contains('$') {
            if !line.contains('"') {
                results.add_issue("MAKE003");
            }
        }

        // MAKE005: Recursive assignment with shell
        if line.contains("= $(shell") && !line.contains(":=") {
            results.add_issue("MAKE005");
        }
    }

    results
}

fn main() {
    let makefile = r#"
VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)

build:
	mkdir build
	gcc $(SOURCES) -o app

clean:
	rm -rf $BUILD_DIR
"#;

    let results = lint_makefile(makefile);

    println!("📊 Linting Results");
    println!("Total issues: {}", results.total_issues);
    println!("\nIssues by rule:");
    for (rule, count) in &results.issues_by_rule {
        println!("  {}: {}", rule, count);
    }
}
```

---

## Quality Enforcement Integration

### Example: CI/CD Quality Gate Simulator

```rust
fn main() {
    let makefile_has_issues = true;
    let max_allowed_issues = 0;
    let actual_issues = 4;

    println!("=== CI/CD Quality Gate ===");
    println!("Running Makefile linter...");

    if actual_issues > max_allowed_issues {
        println!("\n❌ Quality gate FAILED");
        println!("  Found {} issues (max allowed: {})", actual_issues, max_allowed_issues);
        println!("\n  Please fix the following:");
        println!("  - 1x MAKE001: Non-deterministic wildcard");
        println!("  - 1x MAKE002: Non-idempotent mkdir");
        println!("  - 1x MAKE003: Unsafe variable expansion");
        println!("  - 1x MAKE005: Recursive assignment with shell");
        println!("\n  Run: bashrs make lint Makefile --fix");

        // In real CI, this would: std::process::exit(1);
        println!("\n  [Simulated] Build would fail here");
    } else {
        println!("\n✅ Quality gate PASSED");
        println!("  No issues found");
    }
}
```

### Example: Pre-commit Hook Logic

```rust
use std::process::Command;

fn simulate_precommit_check() -> Result<(), String> {
    println!("🔒 Running pre-commit checks...");

    // Simulate running linter
    let issues_found = 2;

    if issues_found > 0 {
        return Err(format!("Found {} Makefile issues", issues_found));
    }

    Ok(())
}

fn main() {
    match simulate_precommit_check() {
        Ok(()) => {
            println!("✅ Pre-commit checks passed");
            println!("Proceeding with commit...");
        }
        Err(e) => {
            println!("❌ Pre-commit checks failed: {}", e);
            println!("Fix issues before committing:");
            println!("  bashrs make lint Makefile --fix");
            println!("  git add Makefile");
            println!("  git commit");
        }
    }
}
```

---

## Summary

This chapter demonstrated all five Makefile linting rules through practical Rust examples:

1. **MAKE001**: Deterministic wildcard (wrap in `$(sort ...)`)
2. **MAKE002**: Idempotent mkdir (use `mkdir -p`)
3. **MAKE003**: Safe variable expansion (quote variables in dangerous commands)
4. **MAKE004**: .PHONY declarations (mark non-file targets)
5. **MAKE005**: Immediate assignment (use `:=` with `$(shell ...)`)

### Key Takeaways

- **Determinism**: Same input → Same output (MAKE001, MAKE005)
- **Idempotency**: Safe to re-run (MAKE002)
- **Safety**: Prevent injection and word-splitting (MAKE003)
- **Correctness**: Proper target declarations (MAKE004)

### Integration Patterns

All examples showed how to:
- Detect issues programmatically
- Apply automatic fixes
- Integrate into CI/CD pipelines
- Enforce quality gates
- Use pre-commit hooks

### Next Steps

1. Run `bashrs make lint Makefile` on your projects
2. Add linting to your CI/CD pipeline
3. Install pre-commit hooks for local enforcement
4. Contribute new linting rules to the project

---

**Quality Guarantee**: All examples in this chapter compile and run successfully, demonstrating the linting concepts through working Rust code.
