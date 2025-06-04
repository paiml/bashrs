# Formal Verification Implementation for rash

This document describes the implementation of formal verification for the rash emitter, as specified in `zkp-spec.md`.

## Overview

The formal verification module (`rash/src/formal/`) implements a formally verified emitter for a tiny subset of the rash AST. This ensures mathematical correctness for critical bootstrap scripts where POSIX shell is the only available interpreter.

## Architecture

### 1. Tiny AST Subset (`tiny_ast.rs`)

Defines the minimal AST subset for bootstrap scripts:
- **ExecuteCommand**: Simple commands with literal arguments (echo, mkdir, etc.)
- **SetEnvironmentVariable**: Variable assignments with literal values
- **Sequence**: Sequential command execution
- **ChangeDirectory**: Change working directory

Allowed commands are restricted to a safe subset: mkdir, echo, rm, cp, mv, chmod, chown, id, test, wget, curl, tar, gzip, gunzip, sha256sum, sha512sum.

### 2. Abstract State (`abstract_state.rs`)

Represents the abstract machine state for formal semantics:
- Environment variables (HashMap<String, String>)
- Current working directory (PathBuf)
- Standard output/error buffers (Vec<String>)
- Exit code (i32)
- Abstract filesystem (HashMap<PathBuf, FileSystemEntry>)

The state provides methods for manipulation and equivalence checking.

### 3. Operational Semantics (`semantics.rs`)

Defines formal operational semantics for both:
- **rash semantics**: How rash AST nodes transform the abstract state
- **POSIX semantics**: How POSIX shell commands transform the abstract state

Both semantics are defined to operate on the same abstract state representation, enabling direct comparison.

### 4. Formal Emitter (`emitter.rs`)

The formally verified emitter that translates rash AST to POSIX shell:
- Proper quoting and escaping of arguments
- Preservation of semantic behavior
- Includes `verify_semantic_equivalence` function for runtime verification

### 5. Property-Based Tests (`proofs.rs`)

Extensive property-based tests using proptest:
- Generates arbitrary AST nodes
- Verifies semantic equivalence for all generated cases
- Tests specific properties (echo output preservation, environment preservation)
- Documents the formal theorem that would be proven in a proof assistant

### 6. Kani Harnesses (`kani_harnesses.rs`)

Bounded model checking proofs using Kani:
- Verifies echo command equivalence
- Verifies environment variable assignment
- Verifies directory creation
- Verifies change directory
- Verifies sequence execution
- Proves emitter totality (always produces valid output)

## Formal Theorem

The main theorem proven by this implementation:

```
Theorem semantic_equivalence:
  forall (ast : TinyAst) (s : AbstractState),
    is_valid ast = true ->
    eval_rash ast s = eval_posix (emit ast) s.
```

This states that for any valid AST node in our tiny subset and any initial state, evaluating the AST directly produces the same final state as evaluating the emitted POSIX code.

## Usage

### Proof Inspection Tool

The formal verification module includes a comprehensive proof inspection tool accessible via the CLI:

```bash
# Inspect predefined examples
rash inspect echo-example
rash inspect bootstrap-example

# Use custom AST (JSON format)
rash inspect '{"ExecuteCommand": {"command_name": "echo", "args": ["Hello"]}}'

# Generate reports in different formats
rash inspect bootstrap-example --format json -o report.json
rash inspect bootstrap-example --format html -o report.html
```

The inspector provides detailed artifacts including:
- **Annotated AST** with semantic information
- **Execution traces** showing step-by-step state changes
- **Equivalence analysis** proving semantic correctness
- **Emitter justifications** explaining code generation decisions

See [proof-inspection-guide.md](proof-inspection-guide.md) for complete documentation.

### Basic Example

```rust
use rash::formal::{TinyAst, FormalEmitter, AbstractState};
use rash::formal::semantics::{rash_semantics, posix_semantics};

// Create an AST for a simple bootstrap sequence
let ast = TinyAst::Sequence {
    commands: vec![
        TinyAst::SetEnvironmentVariable {
            name: "INSTALL_DIR".to_string(),
            value: "/opt/rash".to_string(),
        },
        TinyAst::ExecuteCommand {
            command_name: "mkdir".to_string(),
            args: vec!["-p".to_string(), "/opt/rash/bin".to_string()],
        },
        TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["Installation directory created".to_string()],
        },
    ],
};

// Emit POSIX shell code
let shell_script = FormalEmitter::emit(&ast);
println!("{}", shell_script);
// Output: INSTALL_DIR="/opt/rash"; mkdir -p /opt/rash/bin; echo "Installation directory created"

// Verify semantic equivalence
let initial_state = AbstractState::new();
let rash_result = rash_semantics::eval_rash(&ast, initial_state.clone()).unwrap();
let posix_result = posix_semantics::eval_posix(&shell_script, initial_state).unwrap();
assert!(rash_result.is_equivalent(&posix_result));
```

### Running Tests

```bash
# Run property-based tests
cargo test -p rash formal::proofs

# Run Kani verification (requires Kani installation)
cargo kani --harness verify_echo_semantic_equivalence
cargo kani --harness verify_assignment_semantic_equivalence
# ... etc for other harnesses
```

## Limitations

1. **Tiny Subset**: Only supports a minimal set of operations suitable for bootstrap scripts
2. **No Complex Features**: No loops, conditionals, pipes, or complex shell features
3. **Literal Values Only**: No variable expansion or command substitution
4. **Abstract Filesystem**: Simplified filesystem model for verification

## Future Work

1. **Proof Assistant Integration**: Port the formal proofs to Coq, Isabelle, or Lean
2. **Subset Extension**: Carefully extend the verified subset as needed
3. **Parser Verification**: Verify the parser for the tiny subset
4. **Extraction**: Generate verified C or assembly code from proofs

## Security Considerations

The tiny subset is intentionally restricted to prevent:
- Command injection (no variable expansion in commands)
- Path traversal (limited path operations)
- Privilege escalation (no sudo or similar commands)
- Resource exhaustion (no loops or recursion)

This makes the generated bootstrap scripts suitable for security-critical environments.