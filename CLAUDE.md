# CLAUDE.md - Rash Development Guidelines

## Project Context
Rash is a Rust-to-Shell transpiler targeting deterministic, idempotent bootstrap installers with formal correctness guarantees. The system must produce POSIX-compliant shell scripts that are verifiably safe against injection attacks.

## Development Principles

### 自働化 (Jidoka) - Build Quality In
- **Never ship incomplete code**: All transpiler outputs must include complete error handling paths
- **Verification-first development**: Every new AST transformation requires corresponding verification rules
- **Example**: When implementing control flow transpilation:
  ```rust
  // CORRECT: Complete error handling
  match stmt {
      If(cond, then_block, else_block) => {
          verify_condition_safety(&cond)?;
          emit_shell_if(cond, then_block, else_block)
      }
      _ => Err(TranspileError::UnsupportedStatement)
  }
  // NEVER: Partial implementations with TODO
  ```

### 現地現物 (Genchi Genbutsu) - Direct Observation
- **Test against real shells**: Don't rely on POSIX spec alone; test generated scripts on dash, ash, busybox sh
- **Profile actual bootstrap scenarios**: Measure script execution time/memory on Alpine containers, not just development machines
- **Debug at the shell level**: When transpilation fails, examine the actual generated shell code, not just the Rust AST

### 反省 (Hansei) - Fix Before Adding
- **Current broken functionality to prioritize**:
    1. Control flow statements generate non-idempotent shell code
    2. String escaping fails with unicode inputs
    3. Verification framework doesn't catch all injection vectors
- **Do not add**: Advanced type support, SMT verification, or new target dialects until core transpilation is bulletproof

### 改善 (Kaizen) - Continuous Improvement
- **Incremental verification**: Start with `--verify basic`, achieve 100% coverage, then advance to `strict`
- **Performance baselines**: Generated install.sh must execute in <100ms for minimal scripts
- **Code size targets**: Runtime overhead should not exceed 20 lines of shell boilerplate

## Critical Invariants
1. **POSIX compliance**: Every generated script must pass `shellcheck -s sh`
2. **Determinism**: Same Rust input must produce byte-identical shell output
3. **Safety**: No user input can escape proper quoting in generated scripts

## Verification with paiml-mcp-agent-toolkit
```bash
# Verify transpiler correctness
pmat verify --spec rash.spec --impl target/debug/bashrs

# Test generated scripts
pmat test --shell-matrix "sh,dash,ash" --input examples/*.rs
```