# Rich ASCII Reporting

bashrs provides visual feedback through rich ASCII reports following Toyota's **Mieruka** (visual management) principle.

## Progress Bars

Display progress with Unicode block characters:

```rust,ignore
use bashrs::quality::ReportBuilder;

fn main() {
    let report = ReportBuilder::new(60)
        .progress_bar(75.0, 100.0)
        .build();

    println!("{}", report);
    // Output: [████████████████████████████████████████░░░░░░░░░░░░░] 75%
}
```

## Sparklines

Compact trend visualization:

```rust,ignore
use bashrs::quality::sparkline;

fn main() {
    // Coverage trend over 10 runs
    let coverage_history = vec![82.0, 84.0, 83.0, 85.0, 87.0, 86.0, 88.0, 90.0, 89.0, 91.0];
    let spark = sparkline(&coverage_history);

    println!("Coverage trend: {}", spark);
    // Output: Coverage trend: ▂▃▂▄▅▄▆█▇█
}
```

Sparkline characters map values to 8 levels: `▁▂▃▄▅▆▇█`

## Histogram Bars

Visualize distributions:

```rust,ignore
use bashrs::quality::histogram_bar;

fn main() {
    let values = vec![
        ("SEC001", 45.0),
        ("SEC002", 30.0),
        ("DET001", 15.0),
        ("IDEM001", 10.0),
    ];

    let max_value = 45.0;
    for (name, value) in values {
        let bar = histogram_bar(value, max_value, 30);
        println!("{:8} {} ({:.0})", name, bar, value);
    }
    // Output:
    // SEC001   [██████████████████████████████] (45)
    // SEC002   [████████████████████          ] (30)
    // DET001   [██████████                    ] (15)
    // IDEM001  [███████                       ] (10)
}
```

## Grade Display

Letter grades with visual indicators:

```rust,ignore
use bashrs::quality::Grade;

fn main() {
    let grade = Grade::from_score(87.5);

    println!("Grade: {} ({})", grade, if grade.is_passing() { "PASS" } else { "FAIL" });
    // Output: Grade: B+ (PASS)

    // All grades
    for g in Grade::all() {
        let status = if g.is_passing() { "✓" } else { "✗" };
        println!("{} {}", status, g);
    }
}
```

## Rich Lint Report

Complete lint analysis with clustering:

```rust,ignore
use bashrs::quality::{RichLintReport, ErrorCluster, ShellErrorCategory};

fn main() {
    let mut report = RichLintReport::new("deploy.sh".to_string());

    // Add error clusters
    report.add_cluster(ErrorCluster {
        error_code: "SEC001".to_string(),
        count: 5,
        category: ShellErrorCategory::SecurityError,
        diagnostics: vec![
            "Line 10: Unquoted variable in command".to_string(),
            "Line 15: Unquoted variable in command".to_string(),
        ],
        fix_confidence: 0.92,
        auto_fixable: true,
    });

    report.add_cluster(ErrorCluster {
        error_code: "DET001".to_string(),
        count: 2,
        category: ShellErrorCategory::DeterminismError,
        diagnostics: vec![
            "Line 20: Use of $RANDOM".to_string(),
        ],
        fix_confidence: 0.85,
        auto_fixable: true,
    });

    // Render report
    println!("{}", report.render(80));
}
```

Output:
```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                         Lint Report: deploy.sh                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Total Issues: 7    Clusters: 2    Auto-fixable: 7 (100%)                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  ┌─ SEC001 (SecurityError) ─────────────────────────────────────────────────┐║
║  │ Count: 5    Confidence: 92%    [AUTO-FIX AVAILABLE]                      │║
║  │                                                                          │║
║  │ • Line 10: Unquoted variable in command                                  │║
║  │ • Line 15: Unquoted variable in command                                  │║
║  │ • ... and 3 more                                                         │║
║  └──────────────────────────────────────────────────────────────────────────┘║
║                                                                               ║
║  ┌─ DET001 (DeterminismError) ──────────────────────────────────────────────┐║
║  │ Count: 2    Confidence: 85%    [AUTO-FIX AVAILABLE]                      │║
║  │                                                                          │║
║  │ • Line 20: Use of $RANDOM                                                │║
║  │ • ... and 1 more                                                         │║
║  └──────────────────────────────────────────────────────────────────────────┘║
║                                                                               ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

## Gate Report

Quality gate status visualization:

```rust,ignore
use bashrs::quality::{gate_report, GateResult, GateSummary};

fn main() {
    let summary = GateSummary {
        total_gates: 5,
        passed: 4,
        failed: 1,
        results: vec![
            GateResult { name: "Clippy".into(), passed: true, message: None },
            GateResult { name: "Tests".into(), passed: true, message: None },
            GateResult { name: "Coverage".into(), passed: false, message: Some("78% < 85%".into()) },
            GateResult { name: "Complexity".into(), passed: true, message: None },
            GateResult { name: "Security".into(), passed: true, message: None },
        ],
    };

    println!("{}", gate_report(&summary, 60));
}
```

Output:
```text
╔════════════════════════════════════════════════════════════╗
║                   Quality Gate Summary                      ║
╠════════════════════════════════════════════════════════════╣
║  ✓ Clippy                                                   ║
║  ✓ Tests                                                    ║
║  ✗ Coverage                   78% < 85%                     ║
║  ✓ Complexity                                               ║
║  ✓ Security                                                 ║
╠════════════════════════════════════════════════════════════╣
║  Result: 4/5 gates passed                    [FAILED]       ║
╚════════════════════════════════════════════════════════════╝
```

## Box Drawing Characters

bashrs uses Unicode box-drawing for clean borders:

| Character | Name | Usage |
|-----------|------|-------|
| `╔` `╗` `╚` `╝` | Double corners | Report borders |
| `═` `║` | Double lines | Report borders |
| `╠` `╣` | Double tees | Section dividers |
| `┌` `┐` `└` `┘` | Single corners | Nested boxes |
| `─` `│` | Single lines | Nested boxes |
| `├` `┤` `┬` `┴` | Single tees | Table structure |

## Color Support

When terminal supports colors, reports use ANSI codes:

| Color | Meaning |
|-------|---------|
| Green | Passed, success |
| Red | Failed, error |
| Yellow | Warning, needs attention |
| Blue | Information, headers |
| Cyan | Highlights, emphasis |

Disable colors with `NO_COLOR=1` environment variable.

## Best Practices

1. **Keep reports scannable**: Use visual hierarchy
2. **Show trends**: Sparklines reveal patterns
3. **Cluster related issues**: Group by error code
4. **Highlight actionable items**: Mark auto-fixable issues
5. **Progressive disclosure**: Show summary first, details on demand
