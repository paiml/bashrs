# Rash API Reference

**Version**: 2.0.0 (Target)
**Last Updated**: 2024-10-18
**Language**: Rust

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Core Modules](#core-modules)
4. [Bash Parser API](#bash-parser-api)
5. [Bash Transpiler API](#bash-transpiler-api)
6. [Linter API](#linter-api)
7. [Makefile Parser API](#makefile-parser-api)
8. [Error Handling](#error-handling)
9. [Advanced Usage](#advanced-usage)
10. [Examples](#examples)

---

## Introduction

The Rash API provides programmatic access to bash parsing, purification, and linting functionality. This reference is for developers integrating Rash into their own tools.

### Use Cases

- **Build Tools**: Integrate bash purification into build pipelines
- **IDE Plugins**: Add real-time linting to editors
- **CI/CD Tools**: Automate script validation
- **Custom Tooling**: Build specialized bash analysis tools

---

## Getting Started

### Add Rash as a Dependency

```toml
# Cargo.toml
[dependencies]
rash = "2.0"
```

### Basic Usage

```rust
use rash::bash_parser;
use rash::bash_transpiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a bash script
    let source = r#"
        #!/bin/bash
        TEMP=/tmp/app-$$
        mkdir $TEMP
    "#;

    let ast = bash_parser::parse(source)?;

    // Transpile to purified bash
    let purified = bash_transpiler::transpile(&ast)?;

    println!("{}", purified);
    Ok(())
}
```

---

## Core Modules

### Module Structure

```
rash/
├── bash_parser/        # Parse bash scripts to AST
│   ├── mod.rs          # Main parser interface
│   ├── ast.rs          # AST definitions
│   ├── lexer.rs        # Tokenization
│   └── generators.rs   # Generate purified bash from AST
├── bash_transpiler/    # Transform bash AST
│   ├── mod.rs          # Transpiler interface
│   └── transforms.rs   # AST transformations
├── linter/             # Lint bash scripts
│   ├── mod.rs          # Linter interface
│   ├── rules/          # Linter rules
│   └── diagnostics.rs  # Diagnostic messages
├── make_parser/        # Parse Makefiles
│   ├── mod.rs          # Makefile parser
│   ├── ast.rs          # Makefile AST
│   └── purify.rs       # Makefile purification
└── errors.rs           # Error types
```

---

## Bash Parser API

### Module: `rash::bash_parser`

Parse bash scripts into an Abstract Syntax Tree (AST).

### Functions

#### `parse(source: &str) -> Result<BashAst, ParseError>`

Parse bash script source code into an AST.

**Parameters**:
- `source: &str` - Bash script source code

**Returns**:
- `Ok(BashAst)` - Parsed AST on success
- `Err(ParseError)` - Parse error with location information

**Example**:

```rust
use rash::bash_parser;

let source = r#"
#!/bin/bash
echo "Hello, World!"
"#;

match bash_parser::parse(source) {
    Ok(ast) => println!("Parsed successfully: {:?}", ast),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

---

#### `parse_with_options(source: &str, options: ParseOptions) -> Result<BashAst, ParseError>`

Parse with custom options.

**Parameters**:
- `source: &str` - Bash script source code
- `options: ParseOptions` - Parser configuration

**Returns**:
- `Ok(BashAst)` - Parsed AST on success
- `Err(ParseError)` - Parse error

**Example**:

```rust
use rash::bash_parser::{parse_with_options, ParseOptions};

let options = ParseOptions {
    strict_posix: true,
    allow_bashisms: false,
    preserve_comments: true,
};

let ast = parse_with_options(source, options)?;
```

---

### Types

#### `BashAst`

Represents a parsed bash script.

```rust
pub struct BashAst {
    pub shebang: Option<String>,
    pub items: Vec<BashItem>,
}

pub enum BashItem {
    Command(Command),
    Assignment(Assignment),
    Function(Function),
    If(IfStatement),
    For(ForLoop),
    While(WhileLoop),
    Case(CaseStatement),
    Comment(String),
}
```

**Example**:

```rust
use rash::bash_parser::{parse, BashItem};

let ast = parse(source)?;

for item in &ast.items {
    match item {
        BashItem::Command(cmd) => {
            println!("Command: {}", cmd.name);
        }
        BashItem::Assignment(assign) => {
            println!("Variable: {} = {}", assign.name, assign.value);
        }
        _ => {}
    }
}
```

---

#### `Command`

Represents a shell command.

```rust
pub struct Command {
    pub name: String,
    pub args: Vec<Argument>,
    pub redirects: Vec<Redirect>,
}

pub enum Argument {
    Literal(String),
    Variable(String),
    CommandSubstitution(Box<Command>),
    Quoted(Vec<Argument>),
}
```

**Example**:

```rust
use rash::bash_parser::{parse, BashItem, Argument};

let ast = parse(r#"echo "Hello, $USER!""#)?;

if let BashItem::Command(cmd) = &ast.items[0] {
    assert_eq!(cmd.name, "echo");

    // Check arguments
    for arg in &cmd.args {
        match arg {
            Argument::Variable(name) => println!("Variable: {}", name),
            Argument::Literal(s) => println!("Literal: {}", s),
            _ => {}
        }
    }
}
```

---

#### `ParseOptions`

Configuration for the parser.

```rust
pub struct ParseOptions {
    /// Enable strict POSIX compliance
    pub strict_posix: bool,

    /// Allow bash-specific features
    pub allow_bashisms: bool,

    /// Preserve comments in AST
    pub preserve_comments: bool,

    /// Fail on warnings
    pub strict: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            strict_posix: false,
            allow_bashisms: true,
            preserve_comments: false,
            strict: false,
        }
    }
}
```

---

### Generator Functions

#### `generate_purified(ast: &BashAst) -> Result<String, GenerateError>`

Generate purified POSIX shell script from AST.

**Parameters**:
- `ast: &BashAst` - Parsed AST

**Returns**:
- `Ok(String)` - Purified shell script
- `Err(GenerateError)` - Generation error

**Example**:

```rust
use rash::bash_parser::{parse, generate_purified};

let source = r#"
#!/bin/bash
TEMP=/tmp/app-$$
mkdir $TEMP
"#;

let ast = parse(source)?;
let purified = generate_purified(&ast)?;

println!("{}", purified);
// Output:
// #!/bin/sh
// TEMP="/tmp/app-${VERSION}"
// mkdir -p "${TEMP}"
```

---

## Bash Transpiler API

### Module: `rash::bash_transpiler`

Transform bash AST to apply purification rules.

### Functions

#### `transpile(ast: &BashAst) -> Result<String, TranspileError>`

Transform and generate purified bash script.

**Parameters**:
- `ast: &BashAst` - Parsed bash AST

**Returns**:
- `Ok(String)` - Purified script
- `Err(TranspileError)` - Transpilation error

**Example**:

```rust
use rash::{bash_parser, bash_transpiler};

let source = r#"
#!/bin/bash
SESSION_ID=$RANDOM
echo "Session: $SESSION_ID"
"#;

let ast = bash_parser::parse(source)?;
let purified = bash_transpiler::transpile(&ast)?;

// Purified output will replace $RANDOM with deterministic value
println!("{}", purified);
```

---

#### `transpile_with_config(ast: &BashAst, config: TranspileConfig) -> Result<String, TranspileError>`

Transpile with custom configuration.

**Parameters**:
- `ast: &BashAst` - Parsed bash AST
- `config: TranspileConfig` - Transpilation configuration

**Returns**:
- `Ok(String)` - Purified script
- `Err(TranspileError)` - Transpilation error

**Example**:

```rust
use rash::bash_transpiler::{transpile_with_config, TranspileConfig};

let config = TranspileConfig {
    determinism: DeterminismConfig {
        replace_random: true,
        replace_timestamps: true,
        replace_process_ids: true,
    },
    idempotency: IdempotencyConfig {
        force_mkdir_p: true,
        force_rm_f: true,
        force_ln_sf: true,
    },
    safety: SafetyConfig {
        quote_all_variables: true,
        add_error_handling: true,
    },
};

let purified = transpile_with_config(&ast, config)?;
```

---

### Types

#### `TranspileConfig`

Configuration for transpilation.

```rust
pub struct TranspileConfig {
    pub determinism: DeterminismConfig,
    pub idempotency: IdempotencyConfig,
    pub safety: SafetyConfig,
}

pub struct DeterminismConfig {
    /// Replace $RANDOM with predictable values
    pub replace_random: bool,

    /// Replace $(date) with fixed values
    pub replace_timestamps: bool,

    /// Replace $$ (process ID) with predictable values
    pub replace_process_ids: bool,
}

pub struct IdempotencyConfig {
    /// Force mkdir -p for all mkdir commands
    pub force_mkdir_p: bool,

    /// Force rm -f for all rm commands
    pub force_rm_f: bool,

    /// Force ln -sf for all ln -s commands
    pub force_ln_sf: bool,
}

pub struct SafetyConfig {
    /// Quote all variable expansions
    pub quote_all_variables: bool,

    /// Add || exit 1 error handling
    pub add_error_handling: bool,
}
```

---

## Linter API

### Module: `rash::linter`

Analyze bash scripts for issues.

### Functions

#### `lint_shell(source: &str) -> LintResult`

Lint a bash script for issues.

**Parameters**:
- `source: &str` - Bash script source code

**Returns**:
- `LintResult` - Linting results with diagnostics

**Example**:

```rust
use rash::linter;

let source = r#"
#!/bin/bash
eval "$user_input"
curl http://example.com | sh
"#;

let result = linter::lint_shell(source);

for diagnostic in result.diagnostics {
    println!("{}: {}", diagnostic.code, diagnostic.message);
    println!("  at line {}", diagnostic.span.start_line);
}

// Output:
// SEC001: Command injection risk via eval
//   at line 3
// SEC008: CRITICAL: Piping curl/wget to shell
//   at line 4
```

---

#### `lint_with_config(source: &str, config: LintConfig) -> LintResult`

Lint with custom configuration.

**Parameters**:
- `source: &str` - Bash script source code
- `config: LintConfig` - Linter configuration

**Returns**:
- `LintResult` - Linting results

**Example**:

```rust
use rash::linter::{lint_with_config, LintConfig, Severity};

let config = LintConfig {
    min_severity: Severity::Warning,
    enable_security_rules: true,
    enable_determinism_rules: true,
    enable_idempotency_rules: true,
    treat_warnings_as_errors: false,
};

let result = lint_with_config(source, config);
```

---

### Types

#### `LintResult`

Results of linting operation.

```rust
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl LintResult {
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Warning)
    }

    /// Count diagnostics by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == severity).count()
    }
}
```

---

#### `Diagnostic`

A single linting diagnostic (error/warning/info).

```rust
pub struct Diagnostic {
    pub code: String,          // e.g., "SEC001"
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub fix: Option<String>,   // Suggested fix
}

pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}
```

**Example**:

```rust
use rash::linter::{lint_shell, Severity};

let result = lint_shell(source);

// Filter errors only
let errors: Vec<_> = result.diagnostics
    .iter()
    .filter(|d| d.severity == Severity::Error)
    .collect();

println!("Found {} errors", errors.len());

for error in errors {
    println!("{} at {}:{}", error.code, error.span.start_line, error.span.start_column);
    if let Some(fix) = &error.fix {
        println!("  Suggested fix: {}", fix);
    }
}
```

---

### Linter Rules

Available linter rules (14 total):

**Determinism Rules (DET)**:
- `DET001`: Detect $RANDOM usage
- `DET002`: Detect timestamp generation
- `DET003`: Detect process ID usage

**Idempotency Rules (IDEM)**:
- `IDEM001`: Detect non-idempotent mkdir
- `IDEM002`: Detect non-idempotent rm
- `IDEM003`: Detect non-idempotent ln

**Security Rules (SEC)**:
- `SEC001`: Command injection via eval
- `SEC002`: Unquoted variables in commands
- `SEC003`: Unquoted find -exec {} pattern
- `SEC004`: wget/curl without TLS verification
- `SEC005`: Hardcoded secrets
- `SEC006`: Unsafe temporary file creation
- `SEC007`: Running commands as root without validation
- `SEC008`: curl | sh pattern

---

## Makefile Parser API

### Module: `rash::make_parser`

Parse and purify Makefiles.

### Functions

#### `parse_makefile(source: &str) -> Result<MakeAst, ParseError>`

Parse a Makefile into an AST.

**Parameters**:
- `source: &str` - Makefile source code

**Returns**:
- `Ok(MakeAst)` - Parsed Makefile AST
- `Err(ParseError)` - Parse error

**Example**:

```rust
use rash::make_parser;

let source = r#"
.PHONY: all clean

all: build

build:
	cargo build --release

clean:
	cargo clean
"#;

let ast = make_parser::parse_makefile(source)?;

for item in &ast.items {
    match item {
        MakeItem::Target(target) => {
            println!("Target: {}", target.name);
        }
        MakeItem::Variable(var) => {
            println!("Variable: {} = {}", var.name, var.value);
        }
        _ => {}
    }
}
```

---

#### `purify_makefile(ast: &MakeAst) -> Result<String, PurifyError>`

Purify a Makefile AST.

**Parameters**:
- `ast: &MakeAst` - Parsed Makefile AST

**Returns**:
- `Ok(String)` - Purified Makefile
- `Err(PurifyError)` - Purification error

**Example**:

```rust
use rash::make_parser::{parse_makefile, purify_makefile};

let source = r#"
TEMP = /tmp/build-$(shell date +%s)

build:
	mkdir $(TEMP)
"#;

let ast = parse_makefile(source)?;
let purified = purify_makefile(&ast)?;

// Purified output will make it deterministic and idempotent
println!("{}", purified);
```

---

### Types

#### `MakeAst`

Represents a parsed Makefile.

```rust
pub struct MakeAst {
    pub items: Vec<MakeItem>,
}

pub enum MakeItem {
    Target(Target),
    Variable(Variable),
    Include(String),
    Conditional(Conditional),
    Comment(String),
}

pub struct Target {
    pub name: String,
    pub dependencies: Vec<String>,
    pub recipes: Vec<String>,
    pub phony: bool,
}

pub struct Variable {
    pub name: String,
    pub value: String,
    pub assignment_type: AssignmentType,
}

pub enum AssignmentType {
    Simple,      // :=
    Recursive,   // =
    Conditional, // ?=
    Append,      // +=
}
```

---

## Error Handling

### Error Types

#### `ParseError`

Parsing errors with location information.

```rust
pub struct ParseError {
    pub message: String,
    pub location: Option<Location>,
}

pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "Parse error at line {}, column {}: {}",
                   loc.line, loc.column, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}
```

---

#### `TranspileError`

Transpilation errors.

```rust
pub enum TranspileError {
    ParseError(ParseError),
    UnsupportedConstruct(String),
    GenerationFailed(String),
}

impl std::fmt::Display for TranspileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::UnsupportedConstruct(s) => write!(f, "Unsupported construct: {}", s),
            Self::GenerationFailed(s) => write!(f, "Generation failed: {}", s),
        }
    }
}
```

---

## Advanced Usage

### Custom Linter Rules

Implement custom linting rules by extending the `LintRule` trait.

```rust
use rash::linter::{LintRule, LintResult, Diagnostic, Severity, Span};

struct CustomRule;

impl LintRule for CustomRule {
    fn code(&self) -> &str {
        "CUSTOM001"
    }

    fn check(&self, source: &str) -> LintResult {
        let mut result = LintResult::new();

        for (line_num, line) in source.lines().enumerate() {
            if line.contains("bad_pattern") {
                let diag = Diagnostic::new(
                    self.code(),
                    Severity::Warning,
                    "Found bad pattern",
                    Span::new(line_num + 1, 1, line_num + 1, line.len()),
                );
                result.add(diag);
            }
        }

        result
    }
}

// Use custom rule
let rule = CustomRule;
let result = rule.check(source);
```

---

### AST Transformation

Manually transform AST nodes.

```rust
use rash::bash_parser::{parse, BashAst, BashItem, Command, Argument};

fn transform_ast(ast: &mut BashAst) {
    for item in &mut ast.items {
        if let BashItem::Command(cmd) = item {
            // Quote all unquoted arguments
            for arg in &mut cmd.args {
                if let Argument::Variable(name) = arg {
                    *arg = Argument::Quoted(vec![Argument::Variable(name.clone())]);
                }
            }
        }
    }
}

let mut ast = parse(source)?;
transform_ast(&mut ast);
```

---

### Incremental Parsing

Parse scripts incrementally for large files.

```rust
use rash::bash_parser::{Parser, ParseOptions};

let mut parser = Parser::new(ParseOptions::default());

// Parse in chunks
parser.feed_line("#!/bin/bash")?;
parser.feed_line("echo 'Hello'")?;
parser.feed_line("exit 0")?;

let ast = parser.finish()?;
```

---

## Examples

### Example 1: Basic Purification Pipeline

```rust
use rash::{bash_parser, bash_transpiler, linter};

fn purify_script(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Lint first to identify issues
    let lint_result = linter::lint_shell(source);

    if lint_result.has_errors() {
        eprintln!("Linting found {} errors", lint_result.count_by_severity(linter::Severity::Error));
        for diag in &lint_result.diagnostics {
            eprintln!("  {}: {}", diag.code, diag.message);
        }
    }

    // 2. Parse to AST
    let ast = bash_parser::parse(source)?;

    // 3. Transpile to purified bash
    let purified = bash_transpiler::transpile(&ast)?;

    Ok(purified)
}

fn main() {
    let source = r#"
    #!/bin/bash
    TEMP=/tmp/app-$$
    mkdir $TEMP
    "#;

    match purify_script(source) {
        Ok(purified) => println!("{}", purified),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

### Example 2: Custom Validation Tool

```rust
use rash::{bash_parser, linter};

fn validate_deployment_script(source: &str) -> Result<(), String> {
    // Parse
    let ast = bash_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Lint
    let lint_result = linter::lint_shell(source);

    // Enforce: no eval, no curl|sh, no hardcoded secrets
    for diag in &lint_result.diagnostics {
        match diag.code.as_str() {
            "SEC001" | "SEC005" | "SEC008" => {
                return Err(format!("CRITICAL: {} at line {}",
                                   diag.message, diag.span.start_line));
            }
            _ => {}
        }
    }

    // Enforce: must be idempotent
    let non_idempotent = lint_result.diagnostics
        .iter()
        .filter(|d| d.code.starts_with("IDEM"))
        .count();

    if non_idempotent > 0 {
        return Err(format!("Script has {} non-idempotent operations", non_idempotent));
    }

    Ok(())
}

fn main() {
    match validate_deployment_script(source) {
        Ok(_) => println!("✓ Script passes deployment validation"),
        Err(e) => eprintln!("✗ Validation failed: {}", e),
    }
}
```

---

### Example 3: Batch Processing

```rust
use std::fs;
use std::path::Path;
use rash::{bash_parser, bash_transpiler};

fn purify_directory(dir: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sh") {
            let source = fs::read_to_string(&path)?;

            match bash_parser::parse(&source) {
                Ok(ast) => {
                    match bash_transpiler::transpile(&ast) {
                        Ok(purified) => {
                            let output_path = output_dir.join(path.file_name().unwrap());
                            fs::write(output_path, purified)?;
                            println!("✓ Purified {}", path.display());
                        }
                        Err(e) => {
                            eprintln!("✗ Failed to transpile {}: {}", path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let input_dir = Path::new("scripts");
    let output_dir = Path::new("purified");

    purify_directory(input_dir, output_dir).expect("Batch purification failed");
}
```

---

### Example 4: Integration with Testing Framework

```rust
use rash::{bash_parser, bash_transpiler, linter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purification_makes_script_deterministic() {
        let source = r#"
        #!/bin/bash
        SESSION_ID=$RANDOM
        "#;

        let ast = bash_parser::parse(source).unwrap();
        let purified = bash_transpiler::transpile(&ast).unwrap();

        // Verify $RANDOM was removed
        assert!(!purified.contains("$RANDOM"));
        assert!(purified.contains("SESSION_ID"));
    }

    #[test]
    fn test_linter_detects_security_issues() {
        let source = r#"
        #!/bin/bash
        eval "$user_input"
        "#;

        let result = linter::lint_shell(source);

        // Should detect SEC001 (eval injection)
        let has_sec001 = result.diagnostics
            .iter()
            .any(|d| d.code == "SEC001");

        assert!(has_sec001, "Linter should detect eval injection");
    }

    #[test]
    fn test_purification_makes_mkdir_idempotent() {
        let source = r#"
        #!/bin/bash
        mkdir /tmp/test
        "#;

        let ast = bash_parser::parse(source).unwrap();
        let purified = bash_transpiler::transpile(&ast).unwrap();

        // Verify mkdir became mkdir -p
        assert!(purified.contains("mkdir -p"));
    }
}
```

---

## Performance Considerations

### Parsing Performance

- **Small scripts (<100 lines)**: ~10ms
- **Medium scripts (100-500 lines)**: ~30ms
- **Large scripts (500-1000 lines)**: ~80ms
- **Very large scripts (1000+ lines)**: ~150ms

### Memory Usage

- **Small scripts**: ~1MB
- **Medium scripts**: ~3MB
- **Large scripts**: ~8MB
- **Very large scripts**: ~15MB

### Optimization Tips

1. **Reuse Parser**: Create parser once for multiple files
2. **Batch Processing**: Process files in parallel
3. **Incremental Parsing**: For very large files, use incremental parsing
4. **Caching**: Cache parsed ASTs for frequently-used scripts

---

## Best Practices

1. **Always lint before purifying** to understand issues
2. **Test purified scripts** before deploying to production
3. **Use strict mode** for critical scripts
4. **Handle errors gracefully** in production code
5. **Validate POSIX compliance** with shellcheck after purification
6. **Version control** both original and purified scripts

---

## API Stability

- **Stable APIs** (v2.0+): Core parsing, transpilation, linting
- **Experimental APIs**: Custom transformations, incremental parsing
- **Deprecated APIs**: None (v2.0 is first stable release)

---

## Support

- **Documentation**: https://docs.rash.sh/api
- **GitHub Issues**: https://github.com/yourusername/bashrs/issues
- **API Examples**: https://github.com/yourusername/bashrs/tree/main/examples

---

**Last Updated**: 2024-10-18
**Version**: 2.0.0 (Target)
**License**: MIT
