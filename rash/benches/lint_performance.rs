#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)] // Benchmarks can use unwrap() for simplicity
                               // REPL-016-001: Fast Linting Performance Benchmarks
                               //
                               // Criterion benchmarks for linting performance
                               //
                               // Targets:
                               // - 1,000 lines: <50ms (current: 91ms)
                               // - 10,000 lines: <150ms (current: 306ms)
                               //
                               // Run with: cargo bench --bench lint_performance

use bashrs::linter::lint_shell;
use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

/// Generate a bash script with the specified number of lines
fn generate_bash_script(lines: usize) -> String {
    let mut script = String::from("#!/bin/bash\n");
    script.push_str("# Generated test script for performance benchmarking\n\n");

    for i in 1..=lines {
        match i % 10 {
            0 => script.push_str(&format!("echo \"Line {}: Hello world\"\n", i)),
            1 => script.push_str("if [ -f /tmp/test ]; then echo \"found\"; fi\n"),
            2 => script.push_str("for x in 1 2 3; do echo $x; done\n"),
            3 => script.push_str(&format!("mkdir -p /tmp/dir{}\n", i)),
            4 => script.push_str(&format!("rm -f /tmp/file{}\n", i)),
            5 => script.push_str(&format!("VAR{}=\"value{}\"\n", i, i)),
            6 => script.push_str(&format!("# Comment line {}\n", i)),
            7 => script.push_str("echo $HOME\n"),
            8 => script.push_str("cd /tmp && ls\n"),
            9 => script.push_str(&format!("cat /dev/null > /tmp/out{}\n", i)),
            _ => unreachable!(),
        }
    }

    script
}

/// Benchmark: REPL-016-001-BENCH-001 - Lint 1,000-line script
fn bench_lint_1000_lines(c: &mut Criterion) {
    let script = generate_bash_script(1000);

    c.bench_function("lint_1000_lines", |b| {
        b.iter(|| lint_shell(black_box(&script)))
    });
}

/// Benchmark: REPL-016-001-BENCH-002 - Lint 10,000-line script
fn bench_lint_10000_lines(c: &mut Criterion) {
    let script = generate_bash_script(10_000);

    c.bench_function("lint_10000_lines", |b| {
        b.iter(|| lint_shell(black_box(&script)))
    });
}

/// Benchmark: REPL-016-001-BENCH-003 - Scaling across different sizes
fn bench_lint_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lint_scaling");

    for size in [100, 500, 1000, 5000, 10000].iter() {
        let script = generate_bash_script(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| lint_shell(black_box(&script)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_lint_1000_lines,
    bench_lint_10000_lines,
    bench_lint_scaling
);
criterion_main!(benches);
