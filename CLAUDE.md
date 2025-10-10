# CLAUDE.md - bashrs Development Guidelines

## Project Context
**bashrs** is a **Bash-to-Rust** converter that transforms bash scripts into safe, tested Rust programs. There is NO "Rash" language - we convert bash to actual Rust code that can be compiled and tested.

## What bashrs Does

1. **Parses Bash scripts** → Bash AST
2. **Converts to Rust** → Actual Rust code (using `fn`, not `fun`)
3. **Generates tests** → Unit tests, property tests, integration tests
4. **Validates quality** → Coverage, mutation testing, complexity

This is NOT a transpiler creating a new language. This is a converter from bash to Rust.

## Development Principles

### 自働化 (Jidoka) - Build Quality In
- **Never ship incomplete code**: All bash-to-Rust conversions must include complete error handling
- **Verification-first development**: Every conversion pattern requires corresponding tests
- **Example**: When implementing bash function conversion:
  ```rust
  // CORRECT: Complete conversion with error handling
  fn convert_bash_function(func: &BashFunction) -> Result<RustFunction, ConversionError> {
      validate_function_safety(func)?;
      convert_to_rust_fn(func)
  }
  // NEVER: Partial conversions with TODO
  ```

### 現地現物 (Genchi Genbutsu) - Direct Observation
- **Test against real bash**: Parse actual bash scripts from the wild
- **Verify Rust output**: Generated Rust must compile and pass cargo test
- **Compare behavior**: Ensure bash script and Rust program produce identical results

### 反省 (Hansei) - Fix Before Adding
- **Current priorities**:
    1. Bash parsing completeness (handle all bash constructs)
    2. Safe Rust code generation (proper error handling)
    3. Test generation quality (comprehensive test coverage)
- **Do not add**: Advanced features until core bash→Rust conversion is bulletproof

### 改善 (Kaizen) - Continuous Improvement
- **Incremental conversion**: Support more bash patterns incrementally
- **Quality baselines**: Generated Rust must pass all quality gates
- **Test coverage**: Aim for >85% coverage on generated Rust code

## Critical Invariants
1. **Valid Rust**: All generated code must compile with `cargo build`
2. **Comprehensive tests**: Generated tests must achieve >85% coverage
3. **Behavioral equivalence**: Rust program behavior matches original bash script

## Verification with paiml-mcp-agent-toolkit
```bash
# Verify converter correctness
pmat analyze complexity --max 10
pmat analyze satd --zero-tolerance
pmat quality-score --min 9.0

# Test generated Rust code
cd generated/
cargo test
cargo llvm-cov --lcov
```

## Example Workflow

### Input: Bash Script
```bash
#!/bin/bash
# deploy.sh - Deploy application

deploy_app() {
    local version=$1
    echo "Deploying version $version"
    mkdir -p /app/releases/v${version}
    cp app.tar.gz /app/releases/v${version}/
}

deploy_app "1.0.0"
```

### Output: Rust Program
```rust
// deploy.rs - Generated from deploy.sh
use std::fs;
use std::io::Result;

fn deploy_app(version: &str) -> Result<()> {
    println!("Deploying version {}", version);

    let release_dir = format!("/app/releases/v{}", version);
    fs::create_dir_all(&release_dir)?;

    fs::copy("app.tar.gz", format!("{}/app.tar.gz", release_dir))?;

    Ok(())
}

fn main() -> Result<()> {
    deploy_app("1.0.0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_app_creates_directory() {
        // Test implementation
    }

    #[test]
    fn test_deploy_app_copies_file() {
        // Test implementation
    }
}
```

### Generated Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_deploy_app() {
        let temp = TempDir::new().unwrap();
        let version = "1.0.0";

        deploy_app(version).unwrap();

        assert!(Path::new(&format!("/app/releases/v{}", version)).exists());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_deploy_idempotent(version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
            // Running twice with same version should succeed
            assert!(deploy_app(&version).is_ok());
            assert!(deploy_app(&version).is_ok());
        }
    }
}
```

## Quality Standards

All generated Rust code must meet:
- ✅ Compiles without warnings
- ✅ Passes all generated tests
- ✅ >85% code coverage
- ✅ Complexity <10
- ✅ No SATD (Self-Admitted Technical Debt)
- ✅ Mutation score >80%

## Tools
- `pmat` - Quality analysis with paiml-mcp-agent-toolkit
- `cargo test` - Run generated tests
- `cargo llvm-cov` - Measure coverage (we use llvm, not tarpaulin)
- `cargo mutants` - Mutation testing
