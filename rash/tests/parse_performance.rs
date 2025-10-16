//! Performance benchmarks for Makefile parser
//!
//! Following SQLite principles: Detect performance regressions early
//!
//! These benchmarks establish baseline performance and detect any regressions.

use bashrs::make_parser::parse_makefile;
use std::time::{Duration, Instant};

// ============================================================================
// Performance Baselines (from testing-sqlite-style.md spec)
// ============================================================================

const PARSE_SIMPLE_THRESHOLD: Duration = Duration::from_millis(1);
const PARSE_COMPLEX_THRESHOLD: Duration = Duration::from_millis(10);

// ============================================================================
// Benchmark Helpers
// ============================================================================

/// Run a benchmark and verify it meets the threshold
fn benchmark<F>(name: &str, threshold: Duration, f: F)
where
    F: Fn(),
{
    // Warm up
    for _ in 0..10 {
        f();
    }

    // Measure
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        f();
    }

    let elapsed = start.elapsed();
    let avg_duration = elapsed / iterations;

    println!("{}: avg {:?} (threshold {:?})", name, avg_duration, threshold);

    assert!(
        avg_duration <= threshold,
        "{} too slow: {:?} > {:?}",
        name,
        avg_duration,
        threshold
    );
}

// ============================================================================
// Simple Makefile Benchmarks
// ============================================================================

#[test]
fn bench_parse_simple_makefile() {
    let makefile = "target:\n\techo hello";

    benchmark(
        "parse_simple_makefile",
        PARSE_SIMPLE_THRESHOLD,
        || {
            let _ = parse_makefile(makefile);
        },
    );
}

#[test]
fn bench_parse_variable() {
    let makefile = "VAR = value";

    benchmark("parse_variable", PARSE_SIMPLE_THRESHOLD, || {
        let _ = parse_makefile(makefile);
    });
}

#[test]
fn bench_parse_comment() {
    let makefile = "# This is a comment";

    benchmark("parse_comment", PARSE_SIMPLE_THRESHOLD, || {
        let _ = parse_makefile(makefile);
    });
}

// ============================================================================
// Complex Makefile Benchmarks
// ============================================================================

#[test]
fn bench_parse_typical_rust_makefile() {
    let makefile = r#"
CARGO = cargo

build:
	$(CARGO) build --release

test:
	@$(CARGO) test

clean:
	$(CARGO) clean

.PHONY: build test clean
"#;

    benchmark(
        "parse_typical_rust_makefile",
        PARSE_COMPLEX_THRESHOLD,
        || {
            let _ = parse_makefile(makefile);
        },
    );
}

#[test]
fn bench_parse_many_variables() {
    // Generate 100 variables
    let mut makefile = String::new();
    for i in 0..100 {
        makefile.push_str(&format!("VAR_{} = value_{}\n", i, i));
    }

    benchmark(
        "parse_many_variables",
        PARSE_COMPLEX_THRESHOLD,
        || {
            let _ = parse_makefile(&makefile);
        },
    );
}

#[test]
fn bench_parse_many_targets() {
    // Generate 100 targets
    let mut makefile = String::new();
    for i in 0..100 {
        makefile.push_str(&format!("target_{}:\n\techo {}\n\n", i, i));
    }

    benchmark("parse_many_targets", PARSE_COMPLEX_THRESHOLD, || {
        let _ = parse_makefile(&makefile);
    });
}

// ============================================================================
// Preprocessing Benchmarks
// ============================================================================

#[test]
fn bench_parse_line_continuations() {
    let makefile = r#"
FILES = src/main.rs \
        src/lib.rs \
        src/parser.rs \
        src/ast.rs \
        src/transpiler.rs

build: $(FILES)
	cargo build
"#;

    benchmark(
        "parse_line_continuations",
        PARSE_COMPLEX_THRESHOLD,
        || {
            let _ = parse_makefile(makefile);
        },
    );
}

// ============================================================================
// Real-World Example Benchmarks
// ============================================================================

#[test]
fn bench_parse_gnu_make_example() {
    let makefile = r#"
edit : main.o kbd.o command.o display.o
	cc -o edit main.o kbd.o command.o display.o

main.o : main.c defs.h
	cc -c main.c

kbd.o : kbd.c defs.h command.h
	cc -c kbd.c

command.o : command.c defs.h command.h
	cc -c command.c

display.o : display.c defs.h buffer.h
	cc -c display.c

clean :
	rm edit main.o kbd.o command.o display.o
"#;

    benchmark(
        "parse_gnu_make_example",
        PARSE_COMPLEX_THRESHOLD,
        || {
            let _ = parse_makefile(makefile);
        },
    );
}

// ============================================================================
// Performance Report Generation
// ============================================================================

#[test]
#[ignore] // Run explicitly with --ignored
fn generate_performance_report() {
    println!("\n=== Bashrs Parser Performance Report ===\n");

    let test_cases = vec![
        ("Simple target", "target:\n\techo hello", PARSE_SIMPLE_THRESHOLD),
        ("Variable", "VAR = value", PARSE_SIMPLE_THRESHOLD),
        ("Comment", "# Comment", PARSE_SIMPLE_THRESHOLD),
        (
            "Typical Rust Makefile",
            r#"
build:
	cargo build

test:
	cargo test
"#,
            PARSE_COMPLEX_THRESHOLD,
        ),
    ];

    for (name, makefile, threshold) in test_cases {
        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = parse_makefile(makefile);
        }

        let elapsed = start.elapsed();
        let avg = elapsed / iterations;

        let status = if avg <= threshold { "✅ PASS" } else { "❌ FAIL" };

        println!(
            "{:<30} {:>10?} (threshold {:>10?}) {}",
            name, avg, threshold, status
        );
    }

    println!("\n=== End Report ===\n");
}
