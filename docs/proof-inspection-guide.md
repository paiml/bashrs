# Proof Inspection Guide

This guide demonstrates how to use rash's formal verification and proof inspection capabilities to generate detailed reports about the correctness of emitted shell code.

## Overview

The proof inspection system provides several types of intermediate artifacts:

1. **Annotated AST** - AST nodes with semantic annotations showing state transformations
2. **Execution Traces** - Step-by-step execution showing how states evolve
3. **Equivalence Analysis** - Detailed comparison proving semantic equivalence
4. **Emitter Justifications** - Explanations of why specific POSIX code was generated
5. **Verification Reports** - Comprehensive formal verification artifacts

## Using the Inspect Command

### Basic Usage

```bash
# Inspect a simple echo command
rash inspect echo-example

# Inspect a bootstrap script example
rash inspect bootstrap-example
```

### Output Formats

```bash
# Generate markdown report (default)
rash inspect echo-example --format markdown

# Generate JSON report for programmatic use
rash inspect bootstrap-example --format json

# Generate HTML report for viewing
rash inspect echo-example --format html
```

### Custom AST Input

You can provide your own AST as JSON:

```bash
# Simple environment variable
rash inspect '{"SetEnvironmentVariable": {"name": "VERSION", "value": "1.0.0"}}'

# Complex sequence
rash inspect '{
  "Sequence": {
    "commands": [
      {"SetEnvironmentVariable": {"name": "INSTALL_DIR", "value": "/usr/local/bin"}},
      {"ExecuteCommand": {"command_name": "mkdir", "args": ["-p", "/usr/local/bin"]}},
      {"ExecuteCommand": {"command_name": "echo", "args": ["Setup complete"]}}
    ]
  }
}'
```

### Save Reports

```bash
# Save to file
rash inspect bootstrap-example -o verification-report.md

# JSON for automated processing
rash inspect bootstrap-example --format json -o report.json
```

## Example Inspection Report

Here's what a complete inspection report looks like:

```bash
$ rash inspect bootstrap-example
```

```markdown
# Formal Verification Report

## Input AST
```
Sequence {
    commands: [
        SetEnvironmentVariable {
            name: "INSTALL_DIR",
            value: "/opt/rash",
        },
        ExecuteCommand {
            command_name: "mkdir",
            args: ["-p", "/opt/rash/bin"],
        },
        ChangeDirectory {
            path: "/opt/rash",
        },
        ExecuteCommand {
            command_name: "echo",
            args: ["Installation ready"],
        },
    ],
}
```

## Generated POSIX Code
```bash
INSTALL_DIR="/opt/rash"; mkdir -p /opt/rash/bin; cd /opt/rash; echo "Installation ready"
```

## Verification Result
✅ **SUCCESS** (confidence: 100.0%)

## Equivalence Analysis
- Environment variables: ✅
- Working directory: ✅
- Filesystem: ✅
- Standard output: ✅
- Standard error: ✅
- Exit code: ✅

## Emitter Justifications
### 1: Sequence
**Generated:** `INSTALL_DIR="/opt/rash"; mkdir -p /opt/rash/bin; cd /opt/rash; echo "Installation ready"`
**Reasoning:** Commands are joined with semicolons for sequential execution
**Considerations:**
- Semicolon separator ensures commands execute in order
- Each command is independently validated

### 2: SetEnvironmentVariable(INSTALL_DIR, /opt/rash)
**Generated:** `INSTALL_DIR="/opt/rash"`
**Reasoning:** Variable assignment uses POSIX-compliant syntax with quoted values
**Considerations:**
- Value is always quoted to handle spaces and special characters
- Variable name is validated to be POSIX-compliant

### 3: ExecuteCommand(mkdir, ["-p", "/opt/rash/bin"])
**Generated:** `mkdir -p "/opt/rash/bin"`
**Reasoning:** Command arguments are properly quoted to prevent shell injection
**Considerations:**
- Special characters are escaped within double quotes
- Empty arguments are preserved as empty quoted strings

### 4: ChangeDirectory(/opt/rash)
**Generated:** `cd "/opt/rash"`
**Reasoning:** Change directory uses cd command with quoted path
**Considerations:**
- Path is quoted to handle spaces and special characters

### 5: ExecuteCommand(echo, ["Installation ready"])
**Generated:** `echo "Installation ready"`
**Reasoning:** Command arguments are properly quoted to prevent shell injection
**Considerations:**
- Special characters are escaped within double quotes
- Empty arguments are preserved as empty quoted strings
```

## What the Inspector Provides

### 1. Annotated AST

The inspector annotates each AST node with:
- **Precondition state**: State before execution
- **Postcondition state**: State after execution  
- **Transformation description**: What changed and why
- **Child annotations**: For composite nodes like Sequence

### 2. Execution Traces

Step-by-step traces show:
- **Initial state**: Starting conditions
- **Each execution step**: Operation performed and state changes
- **Final state**: End result
- **Error information**: Any issues encountered

### 3. Equivalence Analysis

Detailed comparison of final states:
- **Environment variables**: Added, modified, or removed variables
- **Working directory**: Directory changes
- **Filesystem**: Created/modified files and directories
- **Output streams**: Standard output and error content
- **Exit codes**: Process exit status

### 4. Emitter Justifications

Explanations for code generation decisions:
- **AST node being processed**: Which node produced the code
- **Generated POSIX code**: Exact shell commands
- **Reasoning**: Why this code was chosen
- **Security considerations**: How safety is ensured

## JSON Output Structure

When using `--format json`, the report includes all data in a structured format:

```json
{
  "ast": { /* Original AST */ },
  "emitted_code": "INSTALL_DIR=\"/opt/rash\"; ...",
  "initial_state": { /* Starting state */ },
  "annotated_ast": { /* AST with annotations */ },
  "rash_trace": { /* Rash execution trace */ },
  "posix_trace": { /* POSIX execution trace */ },
  "equivalence_analysis": { /* Detailed comparison */ },
  "emitter_justifications": [ /* Code generation explanations */ ],
  "verification_result": { /* Success/failure with confidence */ }
}
```

## Integration with CI/CD

The inspector can be integrated into automated workflows:

```bash
# Generate JSON report and check for success
rash inspect "$AST_INPUT" --format json -o verification.json
if jq -e '.verification_result | has("Success")' verification.json; then
  echo "✅ Formal verification passed"
else
  echo "❌ Formal verification failed"
  exit 1
fi
```

## Advanced Features

### Custom Initial States

The inspector automatically sets up reasonable initial conditions, but you can customize the verification environment by modifying the AST or using different predefined examples.

### Property Verification

The underlying formal verification system checks:
- **Semantic equivalence**: AST and POSIX code have identical behavior
- **Command safety**: Only allowed commands are used  
- **Argument safety**: Proper quoting prevents injection
- **State consistency**: All state changes are preserved

### Proof Artifacts

The inspection system generates mathematically rigorous proofs that:
- Can be checked by external proof assistants
- Provide formal guarantees of correctness
- Support incremental verification of larger scripts
- Enable compositional reasoning about script behavior

## Limitations

The current inspector supports the "tiny AST subset" designed for bootstrap scripts:
- Simple commands (echo, mkdir, test, etc.)
- Environment variable assignments
- Directory changes
- Sequential execution

Future versions will expand support for more complex constructs while maintaining formal guarantees.