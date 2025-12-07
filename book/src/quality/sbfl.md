# SBFL Fault Localization

**Spectrum-Based Fault Localization (SBFL)** helps identify which parts of your shell scripts are most likely to contain bugs based on test execution patterns.

## Supported Formulas

bashrs implements three proven SBFL formulas from academic research:

### Tarantula (Jones & Harrold, 2005)

```text
suspiciousness = (failed/total_failed) / ((failed/total_failed) + (passed/total_passed))
```

Best for: General-purpose fault localization with balanced precision.

### Ochiai (Abreu et al., 2007)

```text
suspiciousness = failed / sqrt(total_failed * (failed + passed))
```

Best for: Higher precision when you have many passing tests.

### DStar (Wong et al., 2014)

```text
suspiciousness = failed^2 / (passed + (total_failed - failed))
```

Best for: Maximizing the ranking of faulty statements. Uses `*=2` by default.

## Usage Example

```rust,ignore
use bashrs::quality::{CoverageData, FaultLocalizer, SbflFormula};

fn main() {
    // Create fault localizer with Ochiai formula
    let mut localizer = FaultLocalizer::new(SbflFormula::Ochiai);

    // Add coverage data from test runs
    // test_passed: did the test pass?
    // covered_lines: which lines were executed?
    localizer.add_coverage(CoverageData {
        test_name: "test_basic".to_string(),
        test_passed: true,
        covered_lines: vec![1, 2, 3, 5, 6],
    });

    localizer.add_coverage(CoverageData {
        test_name: "test_edge_case".to_string(),
        test_passed: false,  // This test failed
        covered_lines: vec![1, 2, 3, 4, 5, 6],  // Line 4 only in failing test
    });

    localizer.add_coverage(CoverageData {
        test_name: "test_another".to_string(),
        test_passed: true,
        covered_lines: vec![1, 2, 5, 6],
    });

    // Localize faults - get top 5 suspicious lines
    let rankings = localizer.localize_faults(5);

    for ranking in &rankings {
        println!(
            "Line {}: suspiciousness = {:.3}",
            ranking.line, ranking.suspiciousness
        );
    }
    // Output:
    // Line 4: suspiciousness = 1.000  <- Only in failing test!
    // Line 3: suspiciousness = 0.577
    // ...
}
```

## ASCII Report

Generate a visual fault localization report:

```rust,ignore
use bashrs::quality::{sbfl_report, SbflFormula};

fn main() {
    let rankings = localizer.localize_faults(10);
    let report = sbfl_report(&rankings, SbflFormula::Ochiai, 60);
    println!("{}", report);
}
```

Output:
```text
╔══════════════════════════════════════════════════════════════╗
║              SBFL Fault Localization (Ochiai)                ║
╠══════════════════════════════════════════════════════════════╣
║ Line 4   [████████████████████████████████████████] 1.000    ║
║ Line 3   [███████████████████████                 ] 0.577    ║
║ Line 2   [███████████████                         ] 0.408    ║
║ Line 1   [███████████████                         ] 0.408    ║
║ Line 5   [███████████████                         ] 0.408    ║
╚══════════════════════════════════════════════════════════════╝
```

## Integration with Linting

SBFL works best when combined with test coverage data:

1. Run your test suite with coverage enabled
2. Feed coverage data to the fault localizer
3. When tests fail, SBFL identifies likely fault locations
4. Focus debugging efforts on high-suspiciousness lines

## References

- Jones, J.A. & Harrold, M.J. (2005). "Empirical Evaluation of the Tarantula Automatic Fault-Localization Technique"
- Abreu, R., Zoeteweij, P., & van Gemund, A.J.C. (2007). "On the Accuracy of Spectrum-based Fault Localization"
- Wong, W.E., et al. (2014). "The DStar Method for Effective Software Fault Localization"
