# Control Flow Analysis

bashrs generates Control Flow Graphs (CFGs) and calculates complexity metrics to identify hard-to-maintain shell scripts.

## Complexity Metrics

Three complexity measures following software engineering best practices:

### Cyclomatic Complexity (McCabe, 1976)

Measures the number of linearly independent paths through code.

```text
V(G) = E - N + 2P

Where:
  E = number of edges
  N = number of nodes
  P = number of connected components (usually 1)
```

**Toyota Standard**: Maximum cyclomatic complexity of 10.

### Essential Complexity

Measures how much complexity remains after structured programming constructs are removed. High essential complexity indicates spaghetti code.

### Cognitive Complexity

Measures how difficult code is for humans to understand, penalizing:
- Deeply nested structures
- Breaking linear flow (break, continue, goto-like patterns)
- Recursion

## Usage

```rust,ignore
use bashrs::quality::{CfgBuilder, ComplexityMetrics};

fn main() {
    // Build CFG from shell script
    let script = r#"
        if [ -f "$1" ]; then
            while read -r line; do
                if [ -n "$line" ]; then
                    echo "$line"
                fi
            done < "$1"
        else
            echo "File not found"
            exit 1
        fi
    "#;

    let cfg = CfgBuilder::new().build_from_source(script);
    let metrics = ComplexityMetrics::from_cfg(&cfg);

    println!("Cyclomatic: {}", metrics.cyclomatic);
    println!("Essential: {}", metrics.essential);
    println!("Cognitive: {}", metrics.cognitive);
    println!("Max depth: {}", metrics.max_depth);
    println!("Decision points: {}", metrics.decision_points);
    println!("Loop count: {}", metrics.loop_count);

    // Check against Toyota threshold
    if metrics.exceeds_threshold() {
        println!("WARNING: Complexity {} exceeds threshold 10", metrics.cyclomatic);
    }

    // Get grade
    println!("Grade: {:?}", metrics.grade());
}
```

## Complexity Grades

| Grade | Cyclomatic | Risk Level |
|-------|------------|------------|
| Simple | 1-5 | Low risk, easy to test |
| Moderate | 6-10 | Acceptable, Toyota standard |
| Complex | 11-20 | High risk, needs attention |
| VeryComplex | 21-50 | Very high risk, should refactor |
| Untestable | 50+ | Must refactor immediately |

## CFG Nodes

The CFG represents code structure with these node types:

```rust,ignore
use bashrs::quality::CfgNode;

// Node types in the control flow graph
enum CfgNode {
    Entry,           // Function/script entry point
    Exit,            // Function/script exit point
    BasicBlock {     // Sequential statements
        statements: Vec<String>,
    },
    Conditional {    // if/elif/case branches
        condition: String,
    },
    LoopHeader {     // while/for/until loops
        condition: String,
    },
    FunctionEntry {  // Function definition
        name: String,
    },
    SubshellEntry,   // Subshell $(...) or (...)
}
```

## ASCII Visualization

Generate ASCII art representation of the CFG:

```rust,ignore
use bashrs::quality::{render_cfg_ascii, CfgBuilder, ComplexityMetrics};

fn main() {
    let script = r#"
        if [ "$1" = "start" ]; then
            start_service
        elif [ "$1" = "stop" ]; then
            stop_service
        else
            echo "Usage: $0 {start|stop}"
        fi
    "#;

    let cfg = CfgBuilder::new().build_from_source(script);
    let metrics = ComplexityMetrics::from_cfg(&cfg);
    let ascii = render_cfg_ascii(&cfg, &metrics, 60);

    println!("{}", ascii);
}
```

Output:
```text
╔══════════════════════════════════════════════════════════════╗
║                 Control Flow Graph Analysis                   ║
╠══════════════════════════════════════════════════════════════╣
║  Cyclomatic: 4   Essential: 2   Cognitive: 6                 ║
║  Grade: Simple   Max Depth: 2   Decisions: 3                 ║
╠══════════════════════════════════════════════════════════════╣
║                                                               ║
║    [ENTRY]                                                    ║
║       │                                                       ║
║       ▼                                                       ║
║    <$1 = "start"?>──────┐                                    ║
║       │ yes             │ no                                  ║
║       ▼                 ▼                                     ║
║  [start_service]   <$1 = "stop"?>────┐                       ║
║       │                 │ yes        │ no                     ║
║       │                 ▼            ▼                        ║
║       │           [stop_service] [echo Usage]                 ║
║       │                 │            │                        ║
║       └────────────────┴────────────┘                        ║
║                        │                                      ║
║                        ▼                                      ║
║                     [EXIT]                                    ║
║                                                               ║
╚══════════════════════════════════════════════════════════════╝
```

## Best Practices

### Keep Complexity Low

```bash
# BAD: High cyclomatic complexity (11)
process_args() {
    if [ "$1" = "a" ]; then
        if [ "$2" = "1" ]; then
            if [ "$3" = "x" ]; then
                # deeply nested...
            fi
        fi
    fi
}

# GOOD: Low complexity (3)
process_args() {
    case "$1:$2:$3" in
        a:1:x) handle_a1x ;;
        a:1:*) handle_a1 ;;
        a:*:*) handle_a ;;
        *)     handle_default ;;
    esac
}
```

### Extract Functions

```bash
# BAD: Everything in main
main() {
    # 50 lines of validation
    # 30 lines of processing
    # 20 lines of output
}

# GOOD: Separate concerns
validate_input() { ... }   # Complexity: 3
process_data() { ... }     # Complexity: 4
generate_output() { ... }  # Complexity: 2

main() {
    validate_input "$@"
    process_data
    generate_output
}
```

### Reduce Nesting

```bash
# BAD: Deep nesting
if [ -f "$file" ]; then
    if [ -r "$file" ]; then
        if [ -s "$file" ]; then
            process "$file"
        fi
    fi
fi

# GOOD: Early returns
[ -f "$file" ] || { echo "Not a file"; exit 1; }
[ -r "$file" ] || { echo "Not readable"; exit 1; }
[ -s "$file" ] || { echo "Empty file"; exit 1; }
process "$file"
```

## Integration with CI/CD

Add complexity checks to your pipeline:

```yaml
# .github/workflows/quality.yml
- name: Check complexity
  run: |
    cargo run -- complexity src/*.sh --max 10
    if [ $? -ne 0 ]; then
      echo "Complexity exceeds Toyota threshold (10)"
      exit 1
    fi
```

## References

- McCabe, T.J. (1976). "A Complexity Measure"
- Watson, A.H. & McCabe, T.J. (1996). "Structured Testing: A Testing Methodology Using the Cyclomatic Complexity Metric"
- Campbell, G.A. (2018). "Cognitive Complexity: A new way of measuring understandability"
