# bashrs-oracle

ML-powered error classification oracle for bashrs using aprender (GPU-accelerated).

## Features

- **GPU Acceleration**: Uses aprender with trueno SIMD backend for fast inference
- **Error Classification**: Classifies shell script errors into actionable categories
- **Fix Suggestions**: Provides ML-based fix suggestions for common errors

## Usage

```rust
use bashrs_oracle::{ErrorClassifier, ErrorCategory};

let classifier = ErrorClassifier::new()?;
let category = classifier.classify("unquoted variable expansion")?;
```

## Categories

- `Security` - Security vulnerabilities (injection, etc.)
- `Correctness` - Logic errors and bugs
- `Style` - Code style issues
- `Performance` - Performance problems
- `Portability` - Cross-shell compatibility issues

## License

MIT OR Apache-2.0
