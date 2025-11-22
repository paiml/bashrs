#![allow(clippy::unwrap_used)] // Benchmarks can use unwrap() for simplicity
//! Performance Benchmarks for Fix Safety Taxonomy
//!
//! FAST Validation - Throughput Component:
//! - Target: < 100ms for typical scripts (<500 LOC)
//! - Measure: Linting, fixing, and safety-level filtering
//! - Compare: SAFE vs SAFE-WITH-ASSUMPTIONS vs UNSAFE
//!
//! Using criterion for statistical benchmarking

use bashrs::linter::autofix::{apply_fixes, FixOptions};
use bashrs::linter::rules::{det001, idem001, lint_shell, sc2086};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

// ============================================================================
// Benchmark 1: Linting Performance
// ============================================================================

fn bench_lint_small_script(c: &mut Criterion) {
    let script = r#"#!/bin/bash
echo $VAR1
echo $VAR2
echo $VAR3
"#;

    c.bench_function("lint_small_script", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(script));
            black_box(result)
        })
    });
}

fn bench_lint_medium_script(c: &mut Criterion) {
    // Generate script with 50 unquoted variables
    let mut script = String::from("#!/bin/bash\n");
    for i in 0..50 {
        script.push_str(&format!("echo $VAR{}\n", i));
    }

    c.bench_function("lint_medium_script_50_vars", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(&script));
            black_box(result)
        })
    });
}

fn bench_lint_large_script(c: &mut Criterion) {
    // Generate script with 200 unquoted variables
    let mut script = String::from("#!/bin/bash\n");
    for i in 0..200 {
        script.push_str(&format!("echo $VAR{}\n", i));
    }

    c.bench_function("lint_large_script_200_vars", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(&script));
            black_box(result)
        })
    });
}

// ============================================================================
// Benchmark 2: Fix Application Performance (SAFE)
// ============================================================================

fn bench_apply_safe_fixes_small(c: &mut Criterion) {
    let script = r#"#!/bin/bash
echo $VAR1
echo $VAR2
echo $VAR3
"#;

    let result = lint_shell(script);
    let options = FixOptions::default();

    c.bench_function("apply_safe_fixes_3_vars", |b| {
        b.iter(|| {
            let fixed = apply_fixes(black_box(script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });
}

fn bench_apply_safe_fixes_medium(c: &mut Criterion) {
    let mut script = String::from("#!/bin/bash\n");
    for i in 0..50 {
        script.push_str(&format!("echo $VAR{}\n", i));
    }

    let result = lint_shell(&script);
    let options = FixOptions::default();

    c.bench_function("apply_safe_fixes_50_vars", |b| {
        b.iter(|| {
            let fixed = apply_fixes(black_box(&script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });
}

// ============================================================================
// Benchmark 3: Safety Level Filtering Performance
// ============================================================================

fn bench_safe_filtering(c: &mut Criterion) {
    let script = r#"#!/bin/bash
echo $VAR
mkdir /tmp/dir
rm /tmp/file
SESSION_ID=$RANDOM
"#;

    let result = lint_shell(script);

    let mut group = c.benchmark_group("safety_filtering");

    group.bench_function("filter_safe_only", |b| {
        let options = FixOptions {
            create_backup: false,
            dry_run: false,
            backup_suffix: String::new(),
            apply_assumptions: false, // SAFE only
            output_path: None,
        };
        b.iter(|| {
            let fixed = apply_fixes(black_box(script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });

    group.bench_function("filter_safe_with_assumptions", |b| {
        let options = FixOptions {
            create_backup: false,
            dry_run: false,
            backup_suffix: String::new(),
            apply_assumptions: true, // SAFE + SAFE-WITH-ASSUMPTIONS
            output_path: None,
        };
        b.iter(|| {
            let fixed = apply_fixes(black_box(script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Individual Rule Performance
// ============================================================================

fn bench_sc2086_check(c: &mut Criterion) {
    let script = r#"#!/bin/bash
echo $VAR1
ls $VAR2
cat $VAR3
"#;

    c.bench_function("sc2086_check", |b| {
        b.iter(|| {
            let result = sc2086::check(black_box(script));
            black_box(result)
        })
    });
}

fn bench_idem001_check(c: &mut Criterion) {
    let script = r#"#!/bin/bash
mkdir /tmp/dir1
mkdir /tmp/dir2
mkdir /tmp/dir3
"#;

    c.bench_function("idem001_check", |b| {
        b.iter(|| {
            let result = idem001::check(black_box(script));
            black_box(result)
        })
    });
}

fn bench_det001_check(c: &mut Criterion) {
    let script = r#"#!/bin/bash
ID1=$RANDOM
ID2=$RANDOM
ID3=$RANDOM
"#;

    c.bench_function("det001_check", |b| {
        b.iter(|| {
            let result = det001::check(black_box(script));
            black_box(result)
        })
    });
}

// ============================================================================
// Benchmark 5: Throughput (scripts per second)
// ============================================================================

fn bench_throughput_small_scripts(c: &mut Criterion) {
    let script = r#"#!/bin/bash
echo $VAR
"#;

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(1));

    group.bench_function("scripts_per_second_small", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(script));
            let options = FixOptions::default();
            let fixed = apply_fixes(black_box(script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });

    group.finish();
}

fn bench_throughput_medium_scripts(c: &mut Criterion) {
    let mut script = String::from("#!/bin/bash\n");
    for i in 0..20 {
        script.push_str(&format!("echo $VAR{}\n", i));
    }

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(1));

    group.bench_function("scripts_per_second_medium", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(&script));
            let options = FixOptions::default();
            let fixed = apply_fixes(black_box(&script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 6: Real-world Scenario
// ============================================================================

fn bench_real_world_deploy_script(c: &mut Criterion) {
    let script = r#"#!/bin/bash
# Real-world deployment script

VERSION=$1
RELEASE_DIR=/app/releases/$VERSION
BUILD_ID=$RANDOM

mkdir $RELEASE_DIR
cp -r src/* $RELEASE_DIR/
rm /app/current
ln -s $RELEASE_DIR /app/current

echo "Deployed version $VERSION with build ID $BUILD_ID"
"#;

    c.bench_function("real_world_deploy_script", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(script));
            let options = FixOptions {
                create_backup: false,
                dry_run: false,
                backup_suffix: String::new(),
                apply_assumptions: true,
                output_path: None,
            };
            let fixed = apply_fixes(black_box(script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });
}

// ============================================================================
// Benchmark 7: Worst-case Performance
// ============================================================================

fn bench_worst_case_many_issues(c: &mut Criterion) {
    // Script with many different types of issues
    let mut script = String::from("#!/bin/bash\n");
    for i in 0..50 {
        script.push_str(&format!("echo $VAR{}\n", i)); // SC2086
        script.push_str(&format!("mkdir /tmp/dir{}\n", i)); // IDEM001
        script.push_str(&format!("rm /tmp/file{}\n", i)); // IDEM002
    }
    script.push_str("ID=$RANDOM\n"); // DET001

    c.bench_function("worst_case_150_issues", |b| {
        b.iter(|| {
            let result = lint_shell(black_box(&script));
            let options = FixOptions {
                create_backup: false,
                dry_run: false,
                backup_suffix: String::new(),
                apply_assumptions: true,
                output_path: None,
            };
            let fixed = apply_fixes(black_box(&script), black_box(&result), black_box(&options));
            black_box(fixed)
        })
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    linting,
    bench_lint_small_script,
    bench_lint_medium_script,
    bench_lint_large_script
);

criterion_group!(
    fixing,
    bench_apply_safe_fixes_small,
    bench_apply_safe_fixes_medium,
    bench_safe_filtering
);

criterion_group!(
    individual_rules,
    bench_sc2086_check,
    bench_idem001_check,
    bench_det001_check
);

criterion_group!(
    throughput,
    bench_throughput_small_scripts,
    bench_throughput_medium_scripts
);

criterion_group!(
    real_world,
    bench_real_world_deploy_script,
    bench_worst_case_many_issues
);

criterion_main!(linting, fixing, individual_rules, throughput, real_world);
