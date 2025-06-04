# RASH Rigid Verification Specification

Version: 1.0.0  
Date: 2025-01-04  
Criticality: DO-178C Level A

## 1. Introduction

This specification defines the formal verification requirements for RASH (Rust Abstract Shell), a safety-critical transpiler from a restricted subset of Rust to POSIX-compliant shell scripts. The verification approach ensures mathematical correctness of the transpilation process suitable for DO-178C Level A certification.

## 2. Formal Language Definitions

### 2.1 Source Language: Rust₀ (Restricted Rust Subset)

```bnf
⟨program⟩ ::= ⟨item⟩*

⟨item⟩ ::= ⟨function⟩ | ⟨const-decl⟩ | ⟨use-decl⟩

⟨function⟩ ::= 'fn' ⟨ident⟩ '(' ⟨params⟩? ')' ⟨ret-type⟩? ⟨block⟩

⟨params⟩ ::= ⟨param⟩ (',' ⟨param⟩)*

⟨param⟩ ::= ⟨ident⟩ ':' ⟨type⟩

⟨type⟩ ::= 'i32' | 'u32' | 'bool' | '&str' | 'String' 
         | '[' ⟨type⟩ ';' ⟨literal⟩ ']'  // Fixed-size arrays only
         | 'Result<' ⟨type⟩ ',' ⟨type⟩ '>'

⟨stmt⟩ ::= ⟨let-stmt⟩ | ⟨expr-stmt⟩ | ⟨return-stmt⟩ | ⟨if-stmt⟩ | ⟨loop-stmt⟩

⟨expr⟩ ::= ⟨literal⟩ | ⟨ident⟩ | ⟨binary-op⟩ | ⟨unary-op⟩ 
         | ⟨call-expr⟩ | ⟨index-expr⟩ | ⟨block⟩
```

**Semantic Restrictions:**
- No heap allocation (`Box`, `Vec`, `HashMap` prohibited)
- No unsafe blocks
- No traits or generics (monomorphization required pre-transpilation)
- No closures or function pointers
- No panic-inducing operations
- Bounded recursion depth (static analysis required)

### 2.2 Target Language: POSIX_sh (POSIX Shell Subset)

```bnf
⟨script⟩ ::= ⟨shebang⟩? ⟨command⟩*

⟨command⟩ ::= ⟨simple-cmd⟩ | ⟨compound-cmd⟩ | ⟨function-def⟩

⟨simple-cmd⟩ ::= ⟨word⟩+ ⟨io-redirect⟩*

⟨compound-cmd⟩ ::= ⟨if-cmd⟩ | ⟨while-cmd⟩ | ⟨for-cmd⟩ | ⟨case-cmd⟩

⟨function-def⟩ ::= ⟨name⟩ '()' ⟨compound-cmd⟩

⟨var-ref⟩ ::= '$' ⟨name⟩ | '${' ⟨name⟩ '}'
```

**Safety Constraints:**
- All variable expansions must be quoted: `"${var}"`
- No command substitution in unsafe contexts
- No `eval` or dynamic code execution
- Validated against injection attacks

## 3. Formal Semantics

### 3.1 Rust₀ Operational Semantics

Define evaluation context E and values v:

```
E ::= [] | E + e | v + E | if E then e₁ else e₂ | ...
v ::= n | true | false | "string" | λx.e
```

Small-step operational semantics:
```
(E-App)     (λx.e) v → e[x ↦ v]
(E-IfTrue)  if true then e₁ else e₂ → e₁
(E-IfFalse) if false then e₁ else e₂ → e₂
(E-Plus)    n₁ + n₂ → n₃ where n₃ = n₁ + n₂
```

### 3.2 POSIX_sh Denotational Semantics

Shell state Σ = Variables × FileSystem × ProcessState

```
⟦_⟧ : Command → Σ → Σ × ExitCode

⟦c₁ ; c₂⟧(σ) = let (σ', _) = ⟦c₁⟧(σ) in ⟦c₂⟧(σ')
⟦x=v⟧(σ) = (σ[x ↦ v], 0)
⟦if c then t else e⟧(σ) = 
    let (σ', exit) = ⟦c⟧(σ) in
    if exit = 0 then ⟦t⟧(σ') else ⟦e⟧(σ')
```

## 4. Transpilation Correctness Theorem

### 4.1 Semantic Preservation

For transpilation function T : Rust₀ → POSIX_sh, we require:

**Theorem (Correctness):** ∀p ∈ Rust₀, ∀σ ∈ InitialStates:
```
⟦p⟧_rust(σ) = (v, σ') ⟹ ⟦T(p)⟧_sh(embed(σ)) = (embed(v), embed(σ'))
```

Where `embed` maps Rust values/states to shell representations.

### 4.2 Safety Properties

**Property 1 (Injection Safety):** The transpiled code contains no unquoted variable expansions:
```
∀s ∈ T(p), ∀var_expansion ∈ s : is_quoted(var_expansion)
```

**Property 2 (Termination Preservation):** If Rust₀ program terminates, shell script terminates:
```
terminates(p) ⟹ terminates(T(p))
```

**Property 3 (Error Preservation):** Error states map correctly:
```
p ↝ Error(e) ⟺ T(p) ↝ ExitCode(map_error(e))
```

## 5. Verification Architecture

### 5.1 Component Verification Strategy

```rust
// Core transpilation pipeline with verification points
pub struct VerifiedTranspiler {
    parser: VerifiedParser,      // Theorem: Parses iff ∈ Rust₀
    ir_gen: VerifiedIRGen,        // Theorem: IR ≅ AST semantics  
    optimizer: VerifiedOptimizer, // Theorem: Preserves semantics
    emitter: VerifiedEmitter,     // Theorem: Shell ≅ IR semantics
}
```

### 5.2 Verification Tool Selection

**Primary Tool: Kani Model Checker**
- Chosen for: Rust-native support, bounded model checking, CBMC backend
- Usage: Verify panic-freedom, bounds checking, semantic preservation

```rust
#[kani::proof]
fn verify_parser_soundness() {
    let input: &str = kani::any();
    kani::assume(input.len() < 1000); // Bound input
    
    match parser::parse(input) {
        Ok(ast) => {
            // Property: Valid AST implies valid Rust₀
            kani::assert!(validate_rust0_ast(&ast));
        }
        Err(_) => {
            // Property: Parse error implies ∉ Rust₀
            kani::assert!(!is_valid_rust0(input));
        }
    }
}
```

**Secondary Tool: Creusot**
- For: Complex semantic preservation proofs
- Generates Why3 proof obligations for SMT solving

```rust
#[creusot::ensures(|result| 
    matches!(result, Ok(ir)) ==> ir.semantics() == ast.semantics()
)]
fn ast_to_ir(ast: &VerifiedAst) -> Result<IR, CompileError> {
    // Implementation with ghost code for proof
}
```

### 5.3 Verification Levels

1. **Level 1: Type Safety & Memory Safety**
    - Tool: rustc + miri
    - Properties: No undefined behavior, no data races

2. **Level 2: Functional Correctness**
    - Tool: Kani
    - Properties: Parsing correctness, IR generation correctness

3. **Level 3: Semantic Preservation**
    - Tool: Creusot + Why3/Z3
    - Properties: End-to-end correctness theorem

4. **Level 4: Security Properties**
    - Tool: Custom Kani harnesses
    - Properties: Injection prevention, resource bounds

## 6. Correctness-by-Construction Patterns

### 6.1 Parser Correctness

```rust
// Use parser combinators with proven correctness
type Parser<T> = Box<dyn Fn(&str) -> IResult<&str, T>>;

// Theorem: Combinators preserve correctness
// If p1 correct and p2 correct, then sequence(p1, p2) correct
fn sequence<A, B>(p1: Parser<A>, p2: Parser<B>) -> Parser<(A, B)> {
    Box::new(move |input| {
        let (input, a) = p1(input)?;
        let (input, b) = p2(input)?;
        Ok((input, (a, b)))
    })
}
```

### 6.2 IR Properties

```rust
#[derive(Debug, Clone)]
pub enum IR {
    Assign { 
        var: VarId, 
        value: Value,
        #[creusot::ghost] semantics: Semantics 
    },
    Sequence {
        stmts: Vec<IR>,
        #[creusot::ghost] invariant: SequenceInvariant
    },
    // ...
}

// Invariant: Variables are initialized before use
type SequenceInvariant = |stmts: &[IR]| -> bool {
    let mut initialized = HashSet::new();
    for stmt in stmts {
        match stmt {
            IR::Assign { var, .. } => { initialized.insert(var); }
            IR::Read { var } => {
                if !initialized.contains(var) { return false; }
            }
            // ...
        }
    }
    true
};
```

### 6.3 Emitter Safety

```rust
// Proven-safe shell string escaping
fn escape_shell_string(s: &str) -> String {
    // Theorem: ∀s. no_injection(escape_shell_string(s))
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

#[kani::proof]
fn verify_escape_safety() {
    let input: String = kani::any();
    kani::assume(input.len() < 100);
    
    let escaped = escape_shell_string(&input);
    // Property: No command injection possible
    kani::assert!(!contains_unescaped_metachar(&escaped));
}
```

## 7. Verification Execution Plan

### 7.1 Continuous Verification Pipeline

```yaml
verification-stages:
  pre-commit:
    - cargo clippy -- -D warnings
    - cargo miri test
    
  ci-fast: # < 5 minutes
    - kani --function verify_parser_* --unwind 10
    - cargo test --features verification
    
  ci-thorough: # < 1 hour  
    - kani --all-harnesses --unwind 20
    - creusot --why3-cmd 'why3 prove -P z3,cvc4'
    
  nightly: # Unbounded
    - kani --all-harnesses --unwind 50 --solver kissat
    - mutation testing with mutagen
    - fuzzing with cargo-fuzz
```

### 7.2 Verification Metrics

Required coverage for DO-178C Level A:
- Statement Coverage: 100%
- Branch Coverage: 100%
- MC/DC Coverage: 100%
- Proof Obligation Discharge: 100%

## 8. Formal Test Oracle

### 8.1 Differential Testing Setup

```rust
pub trait TestOracle {
    fn execute_rust(&self, input: &str) -> Result<Value, Error>;
    fn execute_shell(&self, script: &str) -> Result<Value, Error>;
    
    fn verify_equivalence(&self, rust_src: &str) -> Result<(), OracleError> {
        let shell_script = transpile(rust_src)?;
        
        let rust_result = self.execute_rust(rust_src)?;
        let shell_result = self.execute_shell(&shell_script)?;
        
        assert_semantic_equivalence(rust_result, shell_result)?;
        Ok(())
    }
}
```

### 8.2 Property-Based Testing

```rust
#[proptest]
fn transpilation_preserves_semantics(
    #[strategy(arbitrary_rust0_program())] program: Rust0Program
) {
    let oracle = DifferentialOracle::new();
    prop_assert!(oracle.verify_equivalence(&program.to_string()).is_ok());
}
```

## 9. Certification Evidence

### 9.1 DO-178C Compliance Matrix

| Objective | Evidence | Tool | Status |
|-----------|----------|------|--------|
| High-Level Requirements | This specification | - | Complete |
| Low-Level Requirements | Component specs | - | In Progress |
| Source Code Verification | Kani proofs | Kani | In Progress |
| Executable Object Code | Shell validation | shellcheck | Pending |
| Requirements Coverage | Traceability matrix | Custom | Pending |

### 9.2 Verification Condition Database

All verification conditions stored in machine-readable format:

```toml
[[verification_conditions]]
id = "VC_PARSER_001"
description = "Parser accepts only valid Rust₀"
property = "∀input. parse(input) = Ok(ast) ⟹ valid_rust0(ast)"
proof_method = "Kani bounded model checking"
status = "Proven with bound=1000"
```

## 10. Limitations and Assumptions

### 10.1 Assumptions
- POSIX shell implementation is compliant with IEEE Std 1003.1-2017
- Host system has sufficient resources for verification tools
- Input Rust code is trusted (not adversarial)

### 10.2 Out of Scope
- Verification of shell interpreter implementation
- Timing/performance guarantees
- Concurrency (no parallel execution)
- Non-POSIX shell extensions

## Appendix A: Shell Safety Patterns

```rust
// Verified safe patterns for shell generation
mod safe_patterns {
    pub fn safe_variable_expansion(var: &str) -> String {
        format!("\"${{{}}}\"", var)
    }
    
    pub fn safe_array_access(arr: &str, idx: usize) -> String {
        format!("\"${{{}[{}]}}\"", arr, idx)
    }
    
    pub fn safe_command_substitution(cmd: &str) -> String {
        format!("\"$(set -eu; {})\"", cmd)
    }
}
```

## Appendix B: Verification Harness Template

```rust
#[cfg(kani)]
mod verification {
    use super::*;
    
    #[kani::proof]
    #[kani::unwind(20)]
    fn verify_component_property() {
        // 1. Generate arbitrary valid input
        let input: ValidInput = kani::any();
        kani::assume(input.satisfies_precondition());
        
        // 2. Execute component
        let result = component_under_test(input);
        
        // 3. Verify postcondition
        kani::assert!(result.satisfies_postcondition());
        
        // 4. Verify invariants maintained
        kani::assert!(global_invariants_hold());
    }
}
```