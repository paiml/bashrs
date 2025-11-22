#![allow(clippy::unwrap_used)] // Benchmarks can use unwrap() for simplicity
// Bash Purification Performance Benchmarks (Task 1 of 10: 70% → 100% Production)
// Target: <100ms per 1000 lines (scalability verification for production workloads)
//
// Fixtures:
// - small.sh: ~50 lines (basic purification opportunities)
// - medium.sh: ~500 lines (moderate complexity with functions)
// - large.sh: ~5741 lines (real-world production-scale script)
//
// Performance Baselines:
// - Small (<50 lines): Target <5ms (interactive CLI experience)
// - Medium (~500 lines): Target <50ms (responsive for CI/CD)
// - Large (~5000 lines): Target <500ms (<100ms/1000 lines)

use bashrs::bash_parser::BashParser;
use bashrs::bash_transpiler::purification::{PurificationOptions, Purifier};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// Benchmark fixtures
// Issue #4 RESOLVED: Parser now supports $RANDOM, $$, $(cmd), function keyword
const SMALL_BASH: &str = include_str!("fixtures/small.sh");
const MEDIUM_BASH: &str = include_str!("fixtures/medium.sh");
const LARGE_BASH: &str = include_str!("fixtures/large.sh");

/// Benchmark parsing only (bash → AST)
fn benchmark_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("bash_parse");

    group.bench_with_input(
        BenchmarkId::new("small", "50 lines"),
        &SMALL_BASH,
        |b, bash| {
            b.iter(|| {
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                parser.parse()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "500 lines"),
        &MEDIUM_BASH,
        |b, bash| {
            b.iter(|| {
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                parser.parse()
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("large", "5741 lines"),
        &LARGE_BASH,
        |b, bash| {
            b.iter(|| {
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                parser.parse()
            })
        },
    );

    group.finish();
}

/// Benchmark purification only (AST → purified AST, no parsing)
fn benchmark_purify(c: &mut Criterion) {
    let mut group = c.benchmark_group("bash_purify");

    // Pre-parse the bash scripts to isolate purification performance
    let small_ast = {
        let mut parser = BashParser::new(SMALL_BASH).expect("Failed to create parser");
        parser.parse().expect("Failed to parse small.sh")
    };

    let medium_ast = {
        let mut parser = BashParser::new(MEDIUM_BASH).expect("Failed to create parser");
        parser.parse().expect("Failed to parse medium.sh")
    };

    let large_ast = {
        let mut parser = BashParser::new(LARGE_BASH).expect("Failed to create parser");
        parser.parse().expect("Failed to parse large.sh")
    };

    group.bench_with_input(
        BenchmarkId::new("small", "50 lines"),
        &small_ast,
        |b, ast| {
            b.iter(|| {
                let mut purifier = Purifier::new(PurificationOptions::default());
                purifier.purify(black_box(ast))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "500 lines"),
        &medium_ast,
        |b, ast| {
            b.iter(|| {
                let mut purifier = Purifier::new(PurificationOptions::default());
                purifier.purify(black_box(ast))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("large", "5741 lines"),
        &large_ast,
        |b, ast| {
            b.iter(|| {
                let mut purifier = Purifier::new(PurificationOptions::default());
                purifier.purify(black_box(ast))
            })
        },
    );

    group.finish();
}

/// Benchmark end-to-end purification (parse + purify + codegen)
fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("bash_purify_end_to_end");

    // Performance targets based on production requirements:
    // - Small (<50 lines): <5ms (interactive CLI)
    // - Medium (~500 lines): <50ms (CI/CD pipeline)
    // - Large (~5000 lines): <500ms (<100ms per 1000 lines)

    group.bench_with_input(
        BenchmarkId::new("small", "50 lines"),
        &SMALL_BASH,
        |b, bash| {
            b.iter(|| {
                // Parse
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                let ast = parser.parse().expect("Parse failed");

                // Purify
                let mut purifier = Purifier::new(PurificationOptions::default());
                let purified_ast = purifier.purify(&ast).expect("Purify failed");

                // Return purified AST (codegen measured separately if needed)
                black_box(purified_ast)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "500 lines"),
        &MEDIUM_BASH,
        |b, bash| {
            b.iter(|| {
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                let ast = parser.parse().expect("Parse failed");

                let mut purifier = Purifier::new(PurificationOptions::default());
                let purified_ast = purifier.purify(&ast).expect("Purify failed");

                black_box(purified_ast)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("large", "5741 lines"),
        &LARGE_BASH,
        |b, bash| {
            b.iter(|| {
                let mut parser = BashParser::new(black_box(bash)).expect("Failed to create parser");
                let ast = parser.parse().expect("Parse failed");

                let mut purifier = Purifier::new(PurificationOptions::default());
                let purified_ast = purifier.purify(&ast).expect("Purify failed");

                black_box(purified_ast)
            })
        },
    );

    group.finish();
}

/// Benchmark throughput (lines per second)
fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("bash_purify_throughput");

    // Calculate lines per second for each fixture
    let small_lines = SMALL_BASH.lines().count();
    let medium_lines = MEDIUM_BASH.lines().count();
    let large_lines = LARGE_BASH.lines().count();

    group.bench_function(format!("small_{}_lines", small_lines).as_str(), |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(SMALL_BASH)).expect("Failed to create parser");
            let ast = parser.parse().expect("Parse failed");
            let mut purifier = Purifier::new(PurificationOptions::default());
            let purified = purifier.purify(&ast).expect("Purify failed");
            black_box(purified)
        })
    });

    group.bench_function(format!("medium_{}_lines", medium_lines).as_str(), |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(MEDIUM_BASH)).expect("Failed to create parser");
            let ast = parser.parse().expect("Parse failed");
            let mut purifier = Purifier::new(PurificationOptions::default());
            let purified = purifier.purify(&ast).expect("Purify failed");
            black_box(purified)
        })
    });

    group.bench_function(format!("large_{}_lines", large_lines).as_str(), |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(LARGE_BASH)).expect("Failed to create parser");
            let ast = parser.parse().expect("Parse failed");
            let mut purifier = Purifier::new(PurificationOptions::default());
            let purified = purifier.purify(&ast).expect("Purify failed");
            black_box(purified)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_parse,
    benchmark_purify,
    benchmark_end_to_end,
    benchmark_throughput
);
criterion_main!(benches);
