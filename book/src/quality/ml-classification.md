# ML Error Classification

bashrs uses machine learning to classify shell script errors and suggest fixes with confidence scores.

## Error Categories

Errors are classified into 15 categories:

| Category | Description | Example |
|----------|-------------|---------|
| `QuotingError` | Missing or incorrect quotes | `echo $var` vs `echo "$var"` |
| `CommandNotFound` | Invalid command | `gti status` (typo) |
| `SyntaxError` | Parse errors | Missing `fi`, `done` |
| `PermissionDenied` | Access issues | Running without execute bit |
| `PathError` | File/directory not found | `cd /nonexistent` |
| `VariableError` | Unset or misused variables | `$UNDEFINED_VAR` |
| `RedirectionError` | I/O redirection issues | `> /root/file` without perms |
| `PipelineError` | Pipe failures | `cmd1 | cmd2` when cmd1 fails |
| `SubshellError` | Subshell issues | `$(invalid command)` |
| `ArithmeticError` | Math errors | Division by zero |
| `SignalError` | Signal handling | Unhandled SIGTERM |
| `ResourceError` | Resource exhaustion | Out of memory, file descriptors |
| `NetworkError` | Network failures | Connection refused |
| `SecurityError` | Security violations | Injection vulnerabilities |
| `Unknown` | Unclassified errors | New/rare error types |

## Feature Extraction

The classifier extracts 73 features from each error:

```rust,ignore
use bashrs::quality::{FeatureVector, ShellErrorCategory};

fn main() {
    // Extract features from an error message
    let features = FeatureVector::from_error(
        "line 10: unbound variable: MYVAR",
        10,  // line number
        "echo $MYVAR",  // source line
    );

    // Features include:
    // - Lexical: word count, special char count, quote balance
    // - Structural: nesting depth, pipe count, redirect count
    // - Semantic: command type, variable usage patterns
    // - Contextual: line position, surrounding code patterns
    println!("Extracted {} features", features.len());
}
```

## k-NN Classifier

Classification uses k-Nearest Neighbors with rule-based fallback:

```rust,ignore
use bashrs::quality::{KnnClassifier, FeatureVector, ShellErrorCategory};

fn main() {
    // Create classifier with k=5
    let mut classifier = KnnClassifier::new(5);

    // Train with labeled examples
    classifier.train(
        FeatureVector::from_error("unbound variable", 1, "echo $X"),
        ShellErrorCategory::VariableError,
    );
    classifier.train(
        FeatureVector::from_error("command not found", 1, "gti status"),
        ShellErrorCategory::CommandNotFound,
    );

    // Classify new error
    let features = FeatureVector::from_error(
        "bash: UNDEFINED: unbound variable",
        5,
        "echo $UNDEFINED",
    );

    let result = classifier.classify(&features);
    println!("Category: {:?}", result.category);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    // Output:
    // Category: VariableError
    // Confidence: 85.0%
}
```

## Fix Patterns

The pattern library contains 15 bootstrap patterns:

```rust,ignore
use bashrs::quality::{FixPattern, bootstrap_patterns, ShellErrorCategory};

fn main() {
    // Get all bootstrap patterns
    let patterns = bootstrap_patterns();

    // Find patterns for a specific category
    let quoting_fixes: Vec<_> = patterns
        .iter()
        .filter(|p| p.category == ShellErrorCategory::QuotingError)
        .collect();

    for pattern in quoting_fixes {
        println!("Pattern: {}", pattern.name);
        println!("  Before: {}", pattern.before_pattern);
        println!("  After:  {}", pattern.after_pattern);
        println!("  Success rate: {:.1}%", pattern.success_rate() * 100.0);
    }
}
```

### Bootstrap Patterns

| Pattern | Category | Fix |
|---------|----------|-----|
| Add double quotes | QuotingError | `$var` -> `"$var"` |
| Add -p flag | IdempotencyError | `mkdir` -> `mkdir -p` |
| Add -f flag | IdempotencyError | `rm` -> `rm -f` |
| Use shellcheck directive | SecurityError | Add `# shellcheck disable=...` |
| Replace $RANDOM | DeterminismError | Use fixed seed |
| Add error handling | PipelineError | `set -o pipefail` |
| Quote command substitution | QuotingError | `$(cmd)` -> `"$(cmd)"` |

## Oracle System

The Oracle combines classification, patterns, and drift detection:

```rust,ignore
use bashrs::quality::Oracle;

fn main() {
    // Create oracle with default settings
    let mut oracle = Oracle::new();

    // Classify and get fix suggestion
    let result = oracle.analyze_error(
        "unbound variable: CONFIG_PATH",
        15,
        "source $CONFIG_PATH",
    );

    println!("Category: {:?}", result.category);
    println!("Confidence: {:.1}%", result.confidence * 100.0);

    if let Some(fix) = result.suggested_fix {
        println!("Suggested fix: {}", fix.description);
        println!("Apply: {} -> {}", fix.before, fix.after);
    }

    // Record feedback for learning
    oracle.record_feedback(result.id, true);  // User accepted fix
}
```

## Drift Detection

Monitor fix acceptance rates to detect model degradation:

```rust,ignore
use bashrs::quality::{DriftDetector, DriftStatus};

fn main() {
    let mut detector = DriftDetector::new(0.7, 100);  // 70% threshold, 100 sample window

    // Simulate fix acceptances
    for i in 0..50 {
        detector.record(i % 3 != 0);  // 66% acceptance
    }

    match detector.status() {
        DriftStatus::Stable => println!("Model performing well"),
        DriftStatus::Warning(rate) => println!("Warning: acceptance at {:.1}%", rate * 100.0),
        DriftStatus::Degraded(rate) => println!("ALERT: Model degraded to {:.1}%", rate * 100.0),
    }
}
```

## Best Practices

1. **Start with rule-based**: The classifier falls back to rules when confidence is low
2. **Collect feedback**: Record whether users accept fixes to improve the model
3. **Monitor drift**: Watch for declining acceptance rates
4. **Review low-confidence**: Manual review for confidence < 70%
5. **Retrain periodically**: Update patterns based on accumulated feedback
