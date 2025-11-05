# AST-Level Transformation

This chapter explores how bashrs uses Abstract Syntax Tree (AST) transformations to purify bash scripts, making them deterministic, idempotent, and POSIX-compliant.

## What is an Abstract Syntax Tree?

An **Abstract Syntax Tree (AST)** is a tree representation of source code that captures the hierarchical structure and semantics of a program while abstracting away syntactic details like whitespace and punctuation.

### Why ASTs Matter for Bash Purification

Traditional text-based transformations (like `sed` or regex replacements) are brittle and error-prone:

```bash
# ❌ Naive text replacement breaks on edge cases
sed 's/mkdir/mkdir -p/g' script.sh  # Breaks "my_mkdir_function"
```

AST-based transformations are:
- **Semantic**: Understand code structure, not just text patterns
- **Safe**: Only transform actual commands, not comments or strings
- **Precise**: Target specific constructs without false positives
- **Composable**: Multiple transformations can be applied systematically

## bashrs AST Structure

bashrs represents bash scripts using a type-safe Rust AST with three main layers:

### Layer 1: Statements (`BashStmt`)

Statements are top-level constructs:

```rust,ignore
pub enum BashStmt {
    /// Variable assignment: VAR=value
    Assignment {
        name: String,
        value: BashExpr,
        exported: bool,
        span: Span,
    },

    /// Command execution: echo "hello"
    Command {
        name: String,
        args: Vec<BashExpr>,
        span: Span,
    },

    /// Function definition
    Function {
        name: String,
        body: Vec<BashStmt>,
        span: Span,
    },

    /// If statement
    If {
        condition: BashExpr,
        then_block: Vec<BashStmt>,
        elif_blocks: Vec<(BashExpr, Vec<BashStmt>)>,
        else_block: Option<Vec<BashStmt>>,
        span: Span,
    },

    /// While/Until/For loops
    While { condition: BashExpr, body: Vec<BashStmt>, span: Span },
    Until { condition: BashExpr, body: Vec<BashStmt>, span: Span },
    For { variable: String, items: BashExpr, body: Vec<BashStmt>, span: Span },

    /// Case statement
    Case {
        word: BashExpr,
        arms: Vec<CaseArm>,
        span: Span,
    },

    /// Return statement
    Return { code: Option<BashExpr>, span: Span },

    /// Comment (preserved for documentation)
    Comment { text: String, span: Span },
}
```

### Layer 2: Expressions (`BashExpr`)

Expressions represent values and computations:

```rust,ignore
pub enum BashExpr {
    /// String literal: "hello"
    Literal(String),

    /// Variable reference: $VAR or ${VAR}
    Variable(String),

    /// Command substitution: $(cmd) or `cmd`
    CommandSubst(Box<BashStmt>),

    /// Arithmetic expansion: $((expr))
    Arithmetic(Box<ArithExpr>),

    /// Array/list: (item1 item2 item3)
    Array(Vec<BashExpr>),

    /// String concatenation
    Concat(Vec<BashExpr>),

    /// Test expression: [ expr ]
    Test(Box<TestExpr>),

    /// Glob pattern: *.txt
    Glob(String),

    /// Parameter expansion variants
    DefaultValue { variable: String, default: Box<BashExpr> },       // ${VAR:-default}
    AssignDefault { variable: String, default: Box<BashExpr> },     // ${VAR:=default}
    ErrorIfUnset { variable: String, message: Box<BashExpr> },      // ${VAR:?message}
    AlternativeValue { variable: String, alternative: Box<BashExpr> }, // ${VAR:+alt}
    StringLength { variable: String },                              // ${#VAR}
    RemoveSuffix { variable: String, pattern: Box<BashExpr> },      // ${VAR%pattern}
    RemovePrefix { variable: String, pattern: Box<BashExpr> },      // ${VAR#pattern}
    RemoveLongestSuffix { variable: String, pattern: Box<BashExpr> }, // ${VAR%%pattern}
    RemoveLongestPrefix { variable: String, pattern: Box<BashExpr> }, // ${VAR##pattern}
}
```

### Layer 3: Test and Arithmetic Expressions

Low-level constructs for conditionals and math:

```rust,ignore
pub enum TestExpr {
    // String comparisons
    StringEq(BashExpr, BashExpr),    // [ "$a" = "$b" ]
    StringNe(BashExpr, BashExpr),    // [ "$a" != "$b" ]

    // Integer comparisons
    IntEq(BashExpr, BashExpr),       // [ "$a" -eq "$b" ]
    IntLt(BashExpr, BashExpr),       // [ "$a" -lt "$b" ]
    // ... IntGt, IntLe, IntGe, IntNe

    // File tests
    FileExists(BashExpr),            // [ -e "$file" ]
    FileReadable(BashExpr),          // [ -r "$file" ]
    FileWritable(BashExpr),          // [ -w "$file" ]
    FileExecutable(BashExpr),        // [ -x "$file" ]
    FileDirectory(BashExpr),         // [ -d "$dir" ]

    // String tests
    StringEmpty(BashExpr),           // [ -z "$var" ]
    StringNonEmpty(BashExpr),        // [ -n "$var" ]

    // Logical operations
    And(Box<TestExpr>, Box<TestExpr>),
    Or(Box<TestExpr>, Box<TestExpr>),
    Not(Box<TestExpr>),
}

pub enum ArithExpr {
    Number(i64),
    Variable(String),
    Add(Box<ArithExpr>, Box<ArithExpr>),
    Sub(Box<ArithExpr>, Box<ArithExpr>),
    Mul(Box<ArithExpr>, Box<ArithExpr>),
    Div(Box<ArithExpr>, Box<ArithExpr>),
    Mod(Box<ArithExpr>, Box<ArithExpr>),
}
```

### Metadata and Source Tracking

Every AST node includes a `Span` for precise error reporting:

```rust,ignore
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}
```

Complete scripts are wrapped in `BashAst`:

```rust,ignore
pub struct BashAst {
    pub statements: Vec<BashStmt>,
    pub metadata: AstMetadata,
}

pub struct AstMetadata {
    pub source_file: Option<String>,
    pub line_count: usize,
    pub parse_time_ms: u64,
}
```

## How Purification Works via AST Transformations

bashrs purification is a three-stage pipeline:

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│ Parse Bash  │ ───▶ │  Transform   │ ───▶ │  Generate   │
│  to AST     │      │     AST      │      │ Purified Sh │
└─────────────┘      └──────────────┘      └─────────────┘
```

### Stage 1: Parse Bash to AST

```bash
# Input: Messy bash script
#!/bin/bash
SESSION_ID=$RANDOM
mkdir /app/releases
rm /app/current
```

Parses to:

```rust,ignore
BashAst {
    statements: vec![
        BashStmt::Assignment {
            name: "SESSION_ID",
            value: BashExpr::Variable("RANDOM"),
            exported: false,
            span: Span { start_line: 2, ... },
        },
        BashStmt::Command {
            name: "mkdir",
            args: vec![BashExpr::Literal("/app/releases")],
            span: Span { start_line: 3, ... },
        },
        BashStmt::Command {
            name: "rm",
            args: vec![BashExpr::Literal("/app/current")],
            span: Span { start_line: 4, ... },
        },
    ],
    metadata: AstMetadata { ... },
}
```

### Stage 2: Transform AST

Three categories of transformations:

#### 2.1: Determinism Transformations

Replace non-deterministic constructs:

```rust,ignore
// Before: SESSION_ID=$RANDOM
BashStmt::Assignment {
    name: "SESSION_ID",
    value: BashExpr::Variable("RANDOM"),
    ...
}

// After: SESSION_ID="fixed-session-id"
BashStmt::Assignment {
    name: "SESSION_ID",
    value: BashExpr::Literal("fixed-session-id"),
    ...
}
```

**Patterns transformed**:
- `$RANDOM` → fixed value or parameter
- `$(date +%s)` → fixed timestamp or parameter
- `$$` (process ID) → fixed identifier
- `$(hostname)` → parameter

#### 2.2: Idempotency Transformations

Make commands safe to re-run:

```rust,ignore
// Before: mkdir /app/releases
BashStmt::Command {
    name: "mkdir",
    args: vec![BashExpr::Literal("/app/releases")],
}

// After: mkdir -p /app/releases
BashStmt::Command {
    name: "mkdir",
    args: vec![
        BashExpr::Literal("-p"),
        BashExpr::Literal("/app/releases"),
    ],
}
```

**Patterns transformed**:
- `mkdir DIR` → `mkdir -p DIR`
- `rm FILE` → `rm -f FILE`
- `ln -s TARGET LINK` → `rm -f LINK && ln -s TARGET LINK`
- `cp SRC DST` → `cp -f SRC DST` (when overwrite intended)

#### 2.3: POSIX Compliance Transformations

Convert bash-isms to POSIX:

```rust,ignore
// Before: until CONDITION; do BODY; done
BashStmt::Until {
    condition: test_expr,
    body: statements,
}

// After: while ! CONDITION; do BODY; done
BashStmt::While {
    condition: BashExpr::Test(Box::new(
        TestExpr::Not(Box::new(test_expr))
    )),
    body: statements,
}
```

**Patterns transformed**:
- `until` → `while !`
- `[[ ]]` → `[ ]` (when possible)
- `${VAR^^}` → `$(echo "$VAR" | tr '[:lower:]' '[:upper:]')`
- `${VAR,,}` → `$(echo "$VAR" | tr '[:upper:]' '[:lower:]')`

### Stage 3: Generate Purified Shell

The transformed AST is converted back to shell code:

```bash
#!/bin/sh
# Purified by bashrs v6.31.0

SESSION_ID="fixed-session-id"
mkdir -p /app/releases
rm -f /app/current
```

## Example Transformations

### Example 1: Determinism - $RANDOM Removal

**Input bash**:
```bash
#!/bin/bash
TEMP_DIR="/tmp/build-$RANDOM"
mkdir "$TEMP_DIR"
```

**AST before transformation**:
```rust,ignore
vec![
    BashStmt::Assignment {
        name: "TEMP_DIR",
        value: BashExpr::Concat(vec![
            BashExpr::Literal("/tmp/build-"),
            BashExpr::Variable("RANDOM"),
        ]),
    },
    BashStmt::Command {
        name: "mkdir",
        args: vec![BashExpr::Variable("TEMP_DIR")],
    },
]
```

**Transformation logic**:
```rust,ignore
fn remove_random(expr: BashExpr) -> BashExpr {
    match expr {
        BashExpr::Variable(ref name) if name == "RANDOM" => {
            // Replace with deterministic value
            BashExpr::Literal("$(date +%Y%m%d-%H%M%S)")
        }
        BashExpr::Concat(exprs) => {
            BashExpr::Concat(
                exprs.into_iter().map(|e| remove_random(e)).collect()
            )
        }
        _ => expr,
    }
}
```

**AST after transformation**:
```rust,ignore
vec![
    BashStmt::Assignment {
        name: "TEMP_DIR",
        value: BashExpr::Concat(vec![
            BashExpr::Literal("/tmp/build-"),
            BashExpr::Literal("$(date +%Y%m%d-%H%M%S)"),
        ]),
    },
    BashStmt::Command {
        name: "mkdir",
        args: vec![
            BashExpr::Literal("-p"),  // Also made idempotent
            BashExpr::Variable("TEMP_DIR"),
        ],
    },
]
```

**Output purified shell**:
```bash
#!/bin/sh
TEMP_DIR="/tmp/build-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$TEMP_DIR"
```

### Example 2: Idempotency - Command Flag Addition

**Input bash**:
```bash
#!/bin/bash
rm /app/current
ln -s /app/releases/v1.0.0 /app/current
```

**AST before transformation**:
```rust,ignore
vec![
    BashStmt::Command {
        name: "rm",
        args: vec![BashExpr::Literal("/app/current")],
    },
    BashStmt::Command {
        name: "ln",
        args: vec![
            BashExpr::Literal("-s"),
            BashExpr::Literal("/app/releases/v1.0.0"),
            BashExpr::Literal("/app/current"),
        ],
    },
]
```

**Transformation logic**:
```rust,ignore
fn make_idempotent(stmt: BashStmt) -> BashStmt {
    match stmt {
        BashStmt::Command { name, mut args, span } => {
            match name.as_str() {
                "rm" => {
                    // Add -f flag if not present
                    if !args.iter().any(|arg| matches!(arg, BashExpr::Literal(s) if s.starts_with('-') && s.contains('f'))) {
                        args.insert(0, BashExpr::Literal("-f".to_string()));
                    }
                    BashStmt::Command { name, args, span }
                }
                "ln" => {
                    // For symlinks, ensure target is removed first
                    // This is handled at statement sequence level
                    BashStmt::Command { name, args, span }
                }
                "mkdir" => {
                    // Add -p flag if not present
                    if !args.iter().any(|arg| matches!(arg, BashExpr::Literal(s) if s == "-p")) {
                        args.insert(0, BashExpr::Literal("-p".to_string()));
                    }
                    BashStmt::Command { name, args, span }
                }
                _ => BashStmt::Command { name, args, span },
            }
        }
        _ => stmt,
    }
}
```

**AST after transformation**:
```rust,ignore
vec![
    BashStmt::Command {
        name: "rm",
        args: vec![
            BashExpr::Literal("-f"),  // Added for idempotency
            BashExpr::Literal("/app/current"),
        ],
    },
    BashStmt::Command {
        name: "ln",
        args: vec![
            BashExpr::Literal("-s"),
            BashExpr::Literal("/app/releases/v1.0.0"),
            BashExpr::Literal("/app/current"),
        ],
    },
]
```

**Output purified shell**:
```bash
#!/bin/sh
rm -f /app/current
ln -s /app/releases/v1.0.0 /app/current
```

### Example 3: POSIX Compliance - until → while !

**Input bash**:
```bash
#!/bin/bash
until [ -f /tmp/ready ]; do
    echo "Waiting..."
    sleep 1
done
```

**AST before transformation**:
```rust,ignore
BashStmt::Until {
    condition: BashExpr::Test(Box::new(
        TestExpr::FileExists(BashExpr::Literal("/tmp/ready"))
    )),
    body: vec![
        BashStmt::Command {
            name: "echo",
            args: vec![BashExpr::Literal("Waiting...")],
        },
        BashStmt::Command {
            name: "sleep",
            args: vec![BashExpr::Literal("1")],
        },
    ],
    span: Span { ... },
}
```

**Transformation logic**:
```rust,ignore
fn posixify(stmt: BashStmt) -> BashStmt {
    match stmt {
        BashStmt::Until { condition, body, span } => {
            // until COND is equivalent to while ! COND
            BashStmt::While {
                condition: BashExpr::Test(Box::new(
                    TestExpr::Not(Box::new(match condition {
                        BashExpr::Test(test) => *test,
                        _ => TestExpr::StringNonEmpty(condition),
                    }))
                )),
                body,
                span,
            }
        }
        _ => stmt,
    }
}
```

**AST after transformation**:
```rust,ignore
BashStmt::While {
    condition: BashExpr::Test(Box::new(
        TestExpr::Not(Box::new(
            TestExpr::FileExists(BashExpr::Literal("/tmp/ready"))
        ))
    )),
    body: vec![
        BashStmt::Command {
            name: "echo",
            args: vec![BashExpr::Literal("Waiting...")],
        },
        BashStmt::Command {
            name: "sleep",
            args: vec![BashExpr::Literal("1")],
        },
    ],
    span: Span { ... },
}
```

**Output purified shell**:
```bash
#!/bin/sh
while ! [ -f /tmp/ready ]; do
    echo "Waiting..."
    sleep 1
done
```

## Writing Custom AST Transformations

You can extend bashrs with custom transformations using the visitor pattern:

### Step 1: Define Your Transformation

```rust,ignore
use bashrs::bash_parser::ast::{BashStmt, BashExpr, BashAst};

/// Custom transformation: Convert all echo commands to printf
fn echo_to_printf(ast: BashAst) -> BashAst {
    BashAst {
        statements: ast.statements.into_iter()
            .map(transform_stmt)
            .collect(),
        metadata: ast.metadata,
    }
}

fn transform_stmt(stmt: BashStmt) -> BashStmt {
    match stmt {
        BashStmt::Command { name, args, span } if name == "echo" => {
            // Convert echo "text" to printf "%s\n" "text"
            let mut new_args = vec![BashExpr::Literal("%s\\n".to_string())];
            new_args.extend(args);

            BashStmt::Command {
                name: "printf".to_string(),
                args: new_args,
                span,
            }
        }
        // Recursively transform nested statements
        BashStmt::If { condition, then_block, elif_blocks, else_block, span } => {
            BashStmt::If {
                condition,
                then_block: then_block.into_iter().map(transform_stmt).collect(),
                elif_blocks: elif_blocks.into_iter()
                    .map(|(cond, block)| (cond, block.into_iter().map(transform_stmt).collect()))
                    .collect(),
                else_block: else_block.map(|block|
                    block.into_iter().map(transform_stmt).collect()
                ),
                span,
            }
        }
        BashStmt::Function { name, body, span } => {
            BashStmt::Function {
                name,
                body: body.into_iter().map(transform_stmt).collect(),
                span,
            }
        }
        // ... handle other statement types
        _ => stmt,
    }
}
```

### Step 2: Test Your Transformation

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;
    use bashrs::bash_parser::Parser;

    #[test]
    fn test_echo_to_printf_simple() {
        let input = r#"
#!/bin/bash
echo "hello world"
"#;

        let parser = Parser::new();
        let ast = parser.parse(input).expect("Parse failed");
        let transformed = echo_to_printf(ast);

        // Verify transformation
        assert_eq!(transformed.statements.len(), 1);
        match &transformed.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "printf");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_echo_to_printf_in_function() {
        let input = r#"
#!/bin/bash
greet() {
    echo "Hello, $1"
}
"#;

        let parser = Parser::new();
        let ast = parser.parse(input).expect("Parse failed");
        let transformed = echo_to_printf(ast);

        // Verify nested transformation
        match &transformed.statements[0] {
            BashStmt::Function { name, body, .. } => {
                assert_eq!(name, "greet");
                match &body[0] {
                    BashStmt::Command { name, .. } => {
                        assert_eq!(name, "printf");
                    }
                    _ => panic!("Expected Command in function body"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }
}
```

### Step 3: Integrate with bashrs Pipeline

```rust,ignore
use bashrs::bash_parser::Parser;
use bashrs::bash_transpiler::codegen::BashCodegen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse input
    let input = std::fs::read_to_string("input.sh")?;
    let parser = Parser::new();
    let ast = parser.parse(&input)?;

    // Apply custom transformation
    let transformed = echo_to_printf(ast);

    // Generate output
    let codegen = BashCodegen::new();
    let output = codegen.generate(&transformed)?;

    println!("{}", output);
    Ok(())
}
```

## Testing Transformations

bashrs uses EXTREME TDD methodology for transformation testing:

### Unit Tests

Test individual transformation rules:

```rust,ignore
#[test]
fn test_random_variable_removal() {
    let expr = BashExpr::Variable("RANDOM".to_string());
    let transformed = remove_random(expr);

    match transformed {
        BashExpr::Literal(s) => {
            assert!(!s.contains("RANDOM"));
        }
        _ => panic!("Expected Literal after transformation"),
    }
}
```

### Integration Tests

Test complete transformation pipeline:

```rust,ignore
#[test]
fn test_full_purification_pipeline() {
    let input = r#"
#!/bin/bash
SESSION_ID=$RANDOM
mkdir /tmp/session-$SESSION_ID
rm /tmp/current
ln -s /tmp/session-$SESSION_ID /tmp/current
"#;

    let ast = parse(input).unwrap();
    let purified = purify(ast).unwrap();
    let output = generate(purified).unwrap();

    // Verify determinism
    assert!(!output.contains("$RANDOM"));

    // Verify idempotency
    assert!(output.contains("mkdir -p"));
    assert!(output.contains("rm -f"));

    // Verify POSIX compliance
    let shellcheck = std::process::Command::new("shellcheck")
        .arg("-s").arg("sh")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();

    shellcheck.stdin.unwrap().write_all(output.as_bytes()).unwrap();
    let result = shellcheck.wait_with_output().unwrap();
    assert!(result.status.success(), "Shellcheck failed: {}",
        String::from_utf8_lossy(&result.stderr));
}
```

### Property Tests

Test transformation invariants:

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_purification_is_deterministic(ast in bash_ast_strategy()) {
        let purified1 = purify(ast.clone()).unwrap();
        let purified2 = purify(ast.clone()).unwrap();

        // Same input must produce identical output
        assert_eq!(purified1, purified2);
    }

    #[test]
    fn prop_purification_preserves_semantics(ast in bash_ast_strategy()) {
        let original_semantics = evaluate(ast.clone());
        let purified = purify(ast).unwrap();
        let purified_semantics = evaluate(purified);

        // Purification must not change behavior
        assert_eq!(original_semantics, purified_semantics);
    }
}
```

## Best Practices

### 1. Preserve Semantics

**Always** verify that transformations preserve the original script's behavior:

```rust,ignore
// ❌ BAD: Changes behavior
fn bad_transform(cmd: &str) -> &str {
    match cmd {
        "rm" => "echo",  // Changes behavior!
        _ => cmd,
    }
}

// ✅ GOOD: Preserves behavior, adds safety
fn good_transform(cmd: &str, args: Vec<String>) -> (String, Vec<String>) {
    match cmd {
        "rm" => {
            let mut new_args = args;
            if !new_args.contains(&"-f".to_string()) {
                new_args.insert(0, "-f".to_string());
            }
            ("rm".to_string(), new_args)
        }
        _ => (cmd.to_string(), args),
    }
}
```

### 2. Handle Edge Cases

Consider all possible AST node variations:

```rust,ignore
fn transform_expr(expr: BashExpr) -> BashExpr {
    match expr {
        // Handle all variants
        BashExpr::Literal(s) => BashExpr::Literal(s),
        BashExpr::Variable(v) => transform_variable(v),
        BashExpr::CommandSubst(cmd) => BashExpr::CommandSubst(
            Box::new(transform_stmt(*cmd))
        ),
        BashExpr::Arithmetic(arith) => BashExpr::Arithmetic(
            Box::new(transform_arith(*arith))
        ),
        BashExpr::Array(items) => BashExpr::Array(
            items.into_iter().map(transform_expr).collect()
        ),
        BashExpr::Concat(exprs) => BashExpr::Concat(
            exprs.into_iter().map(transform_expr).collect()
        ),
        // ... handle ALL variants, not just common ones
        _ => expr,
    }
}
```

### 3. Use Span Information for Error Reporting

```rust,ignore
fn validate_transformation(
    stmt: &BashStmt,
    span: Span,
) -> Result<(), TransformError> {
    match stmt {
        BashStmt::Command { name, args, .. } if name == "eval" => {
            Err(TransformError::UnsafeCommand {
                command: name.clone(),
                line: span.start_line,
                col: span.start_col,
                message: "eval cannot be safely transformed".to_string(),
            })
        }
        _ => Ok(()),
    }
}
```

### 4. Compose Transformations

Apply multiple transformations in order:

```rust,ignore
fn purify_ast(ast: BashAst) -> Result<BashAst, PurifyError> {
    ast
        .transform(remove_nondeterminism)?   // Step 1: Determinism
        .transform(make_idempotent)?         // Step 2: Idempotency
        .transform(posixify)?                // Step 3: POSIX compliance
        .transform(quote_variables)?         // Step 4: Safety
}
```

### 5. Test with Real Scripts

Validate against actual bash scripts from production:

```rust,ignore
#[test]
fn test_real_world_deployment_script() {
    let input = std::fs::read_to_string("tests/fixtures/deploy.sh")
        .expect("Failed to read test fixture");

    let purified = purify_bash(&input).expect("Purification failed");

    // Verify output is valid
    assert!(shellcheck_passes(&purified));

    // Verify original behavior is preserved
    assert_eq!(
        execute_in_docker("bash", &input),
        execute_in_docker("sh", &purified),
    );
}
```

## Summary

AST-based transformations are the foundation of bashrs purification:

1. **Parse** bash to type-safe AST
2. **Transform** AST to enforce determinism, idempotency, and POSIX compliance
3. **Generate** purified shell code
4. **Verify** with shellcheck and tests

This approach provides:
- **Safety**: No false positives from regex transformations
- **Precision**: Semantic understanding of code
- **Composability**: Multiple transformations can be layered
- **Testability**: Unit tests, integration tests, and property tests

For more details on testing transformations, see the [Property Testing](./property-testing.md) and [Mutation Testing](./mutation-testing.md) chapters.
