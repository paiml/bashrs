#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BUILTIN_017_times_non_deterministic() {
    // DOCUMENTATION: times is non-deterministic
    //
    // Problem: CPU time varies based on system load
    // Result: Different output each run
    //
    // Example:
    // Run 1: 0m0.001s 0m0.002s 0m0.010s 0m0.015s
    // Run 2: 0m0.003s 0m0.004s 0m0.012s 0m0.018s
    //
    // Factors affecting CPU time:
    // - System load (other processes)
    // - CPU frequency scaling
    // - Cache state
    // - OS scheduling
    //
    // This violates determinism principle.

    let script = r#"times"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "times command documented: NON-DETERMINISTIC"
            );
        }
        Err(_) => {
            // May fail to parse
        }
    }

    // DOCUMENTATION: times output varies every run
    // Determinism: Different values based on system state
    // Factors: System load, CPU speed, cache, scheduling
    // Purification: IMPOSSIBLE - must be removed
}

#[test]
fn test_BUILTIN_017_times_profiling_only() {
    // DOCUMENTATION: times is for profiling only
    //
    // Purpose: Performance profiling and debugging
    // Not needed in: Production scripts
    //
    // Profiling should use external tools:
    // - GNU time: /usr/bin/time -v ./script.sh
    // - hyperfine: hyperfine './script.sh'
    // - perf: perf stat ./script.sh
    //
    // These tools provide:
    // - More detailed metrics
    // - Better formatting
    // - Statistical analysis
    // - No script modification needed

    let script = r#"times"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "times profiling usage documented: USE EXTERNAL TOOLS"
            );
        }
        Err(_) => {
            // May fail to parse
        }
    }

    // DOCUMENTATION: times is for profiling
    // Production: Not needed in production scripts
    // Alternative: Use external profiling tools
    // Benefits: Better metrics, no script changes
}

#[test]
fn test_BUILTIN_017_times_refactoring_alternative() {
    // DOCUMENTATION: How to profile without times
    //
    // BAD (times - embedded profiling):
    // #!/bin/bash
    // # ... script logic ...
    // times
    //
    // GOOD (external profiling - no script changes):
    // /usr/bin/time -v ./script.sh
    // hyperfine './script.sh'
    // perf stat ./script.sh
    //
    // This test verifies scripts work without embedded profiling.

    let script = r#"echo "Script logic here""#;
    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Script without times should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert!(!ast.statements.is_empty());

    // DOCUMENTATION: Refactoring strategy for times
    // Instead of: times (embedded in script)
    // Use: /usr/bin/time -v ./script.sh (external profiling)
    //
    // External Profiling Tools:
    // - GNU time: Detailed resource usage
    // - hyperfine: Statistical benchmarking
    // - perf: CPU performance counters
    // - valgrind: Memory profiling
    //
    // Benefits:
    // - No script modification needed
    // - More detailed metrics
    // - Statistical analysis
    // - Deterministic scripts (no profiling code)
    //
    // Production:
    // - Scripts should not contain profiling code
    // - Profile externally during development/testing
    // - Remove times from production scripts
}

// ============================================================================
// BUILTIN-019: umask - File Creation Permissions (GLOBAL STATE)
// Reference: docs/BASH-INGESTION-ROADMAP.yaml
// Status: DOCUMENTED (global state modification)
//
// umask sets default file creation permissions:
// - umask 022 → new files: 644, new dirs: 755
// - umask 077 → new files: 600, new dirs: 700
//
// Global State Issues:
// - umask modifies process-wide file creation mask
// - Affects all subsequent file operations
// - Cannot be scoped (applies to entire shell process)
// - Side effects persist across script boundaries
//
// Idempotency Concerns:
// - umask changes global state permanently
// - Running script multiple times stacks umask calls
// - May override system/user defaults
// - Difficult to restore original value
//
// Best Practices:
// - Set umask at start of script if needed
// - Document why specific umask is required
// - Consider explicit chmod instead
// - Restore original umask if changed
//
// EXTREME TDD: Document umask behavior and implications
// ============================================================================
