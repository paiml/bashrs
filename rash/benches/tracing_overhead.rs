#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)] // Benchmarks can use unwrap() for simplicity
//! Tracing Overhead Benchmarks
//!
//! Week 1 Performance Checkpoint: Verify <10% overhead target
//! Target: OOPSLA2 2024 standard (<10% runtime overhead)
//!
//! Methodology:
//! - Baseline: Parse scripts WITHOUT tracer
//! - Traced: Parse scripts WITH tracer enabled
//! - Overhead = (Traced - Baseline) / Baseline * 100%
//!
//! Success Criteria: Overhead < 10% for all workloads

use bashrs::bash_parser::BashParser;
use bashrs::tracing::TraceManager;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Small script: 5 statements (~10 lines)
const SMALL_SCRIPT: &str = r#"
x=5
y=10
z=15
echo "Hello"
echo "World"
"#;

/// Medium script: 20 statements (~40 lines)
const MEDIUM_SCRIPT: &str = r#"
APP_NAME="myapp"
APP_VERSION="1.0.0"
DEPLOY_DIR="/opt/apps"
LOG_DIR="/var/log/apps"
COUNT=0

if [ -d "/opt/apps/myapp" ]; then
  echo "Directory exists"
  COUNT=1
fi

if [ -f "/opt/apps/myapp/app.sh" ]; then
  echo "File exists"
  COUNT=2
fi

if [ -n "$APP_NAME" ]; then
  echo "Variable is set"
fi

if [ -z "$EMPTY" ]; then
  echo "Variable is empty"
fi

echo "Installing $APP_NAME version $APP_VERSION"
echo "Deploy dir: $DEPLOY_DIR"
echo "Log dir: $LOG_DIR"
echo "Count: $COUNT"
"#;

/// Large script: 100+ statements (~300 lines)
fn generate_large_script() -> String {
    let mut script = String::from("#!/bin/bash\n");

    // Add 50 variable assignments
    for i in 0..50 {
        script.push_str(&format!("VAR_{i}=value_{i}\n"));
    }

    // Add 30 conditional statements
    for i in 0..30 {
        script.push_str(&format!(
            "if [ \"$VAR_{}\" = \"value_{}\" ]; then\n  echo \"Match {}\"\nfi\n",
            i % 50,
            i % 50,
            i
        ));
    }

    // Add 20 commands
    for i in 0..20 {
        script.push_str(&format!("echo \"Command {}\"\n", i));
    }

    script
}

/// Benchmark: Parse small script WITHOUT tracer (baseline)
fn bench_parse_small_baseline(c: &mut Criterion) {
    c.bench_function("parse_small_baseline", |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(SMALL_SCRIPT)).expect("Parser creation failed");
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark: Parse small script WITH tracer
fn bench_parse_small_traced(c: &mut Criterion) {
    c.bench_function("parse_small_traced", |b| {
        b.iter(|| {
            let tracer = TraceManager::new();
            let mut parser = BashParser::new(black_box(SMALL_SCRIPT))
                .expect("Parser creation failed")
                .with_tracer(tracer);
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark: Parse medium script WITHOUT tracer (baseline)
fn bench_parse_medium_baseline(c: &mut Criterion) {
    c.bench_function("parse_medium_baseline", |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(MEDIUM_SCRIPT)).expect("Parser creation failed");
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark: Parse medium script WITH tracer
fn bench_parse_medium_traced(c: &mut Criterion) {
    c.bench_function("parse_medium_traced", |b| {
        b.iter(|| {
            let tracer = TraceManager::new();
            let mut parser = BashParser::new(black_box(MEDIUM_SCRIPT))
                .expect("Parser creation failed")
                .with_tracer(tracer);
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark: Parse large script WITHOUT tracer (baseline)
fn bench_parse_large_baseline(c: &mut Criterion) {
    let large_script = generate_large_script();

    c.bench_function("parse_large_baseline", |b| {
        b.iter(|| {
            let mut parser =
                BashParser::new(black_box(&large_script)).expect("Parser creation failed");
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark: Parse large script WITH tracer
fn bench_parse_large_traced(c: &mut Criterion) {
    let large_script = generate_large_script();

    c.bench_function("parse_large_traced", |b| {
        b.iter(|| {
            let tracer = TraceManager::new();
            let mut parser = BashParser::new(black_box(&large_script))
                .expect("Parser creation failed")
                .with_tracer(tracer);
            let ast = parser.parse().expect("Parse failed");
            black_box(ast);
        });
    });
}

/// Benchmark group: Compare baseline vs traced across different script sizes
fn bench_overhead_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("overhead_comparison");

    // Small script
    group.bench_with_input(
        BenchmarkId::new("baseline", "small"),
        &SMALL_SCRIPT,
        |b, script| {
            b.iter(|| {
                let mut parser =
                    BashParser::new(black_box(script)).expect("Parser creation failed");
                let ast = parser.parse().expect("Parse failed");
                black_box(ast);
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("traced", "small"),
        &SMALL_SCRIPT,
        |b, script| {
            b.iter(|| {
                let tracer = TraceManager::new();
                let mut parser = BashParser::new(black_box(script))
                    .expect("Parser creation failed")
                    .with_tracer(tracer);
                let ast = parser.parse().expect("Parse failed");
                black_box(ast);
            });
        },
    );

    // Medium script
    group.bench_with_input(
        BenchmarkId::new("baseline", "medium"),
        &MEDIUM_SCRIPT,
        |b, script| {
            b.iter(|| {
                let mut parser =
                    BashParser::new(black_box(script)).expect("Parser creation failed");
                let ast = parser.parse().expect("Parse failed");
                black_box(ast);
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("traced", "medium"),
        &MEDIUM_SCRIPT,
        |b, script| {
            b.iter(|| {
                let tracer = TraceManager::new();
                let mut parser = BashParser::new(black_box(script))
                    .expect("Parser creation failed")
                    .with_tracer(tracer);
                let ast = parser.parse().expect("Parse failed");
                black_box(ast);
            });
        },
    );

    group.finish();
}

/// Benchmark: TraceManager memory overhead
fn bench_trace_manager_memory(c: &mut Criterion) {
    c.bench_function("trace_manager_creation", |b| {
        b.iter(|| {
            let tracer = TraceManager::new();
            black_box(tracer);
        });
    });

    c.bench_function("trace_manager_with_capacity_1024", |b| {
        b.iter(|| {
            let tracer = TraceManager::with_capacity(black_box(1024));
            black_box(tracer);
        });
    });
}

criterion_group!(
    benches,
    bench_parse_small_baseline,
    bench_parse_small_traced,
    bench_parse_medium_baseline,
    bench_parse_medium_traced,
    bench_parse_large_baseline,
    bench_parse_large_traced,
    bench_overhead_comparison,
    bench_trace_manager_memory,
);

criterion_main!(benches);
