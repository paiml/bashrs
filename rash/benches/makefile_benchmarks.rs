#![allow(clippy::unwrap_used)] // Benchmarks can use unwrap() for simplicity
// Sprint 84 - Day 1: Makefile Performance Benchmarks
// Performance targets: <10ms (small), <50ms (medium), <100ms (large)

use bashrs::make_parser::{parse_makefile, purify_makefile};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// Benchmark fixtures
const SMALL_MAKEFILE: &str = include_str!("fixtures/small.mk");
const MEDIUM_MAKEFILE: &str = include_str!("fixtures/medium.mk");
const LARGE_MAKEFILE: &str = include_str!("fixtures/large.mk");

/// Benchmark parsing only (without purification)
fn benchmark_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_with_input(
        BenchmarkId::new("small", "46 lines"),
        &SMALL_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(black_box(makefile))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "174 lines"),
        &MEDIUM_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(black_box(makefile))),
    );

    group.bench_with_input(
        BenchmarkId::new("large", "2021 lines"),
        &LARGE_MAKEFILE,
        |b, makefile| b.iter(|| parse_makefile(black_box(makefile))),
    );

    group.finish();
}

/// Benchmark purification only (without parsing)
fn benchmark_purify(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify");

    // Pre-parse the Makefiles
    let small_ast = parse_makefile(SMALL_MAKEFILE).unwrap();
    let medium_ast = parse_makefile(MEDIUM_MAKEFILE).unwrap();
    let large_ast = parse_makefile(LARGE_MAKEFILE).unwrap();

    group.bench_with_input(
        BenchmarkId::new("small", "46 lines"),
        &small_ast,
        |b, ast| b.iter(|| purify_makefile(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "174 lines"),
        &medium_ast,
        |b, ast| b.iter(|| purify_makefile(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("large", "2021 lines"),
        &large_ast,
        |b, ast| b.iter(|| purify_makefile(black_box(ast))),
    );

    group.finish();
}

/// Benchmark end-to-end workflow (parse + purify)
fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // Set performance targets
    // Target: <10ms (small), <50ms (medium), <100ms (large)

    group.bench_with_input(
        BenchmarkId::new("small", "46 lines"),
        &SMALL_MAKEFILE,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(black_box(makefile)).unwrap();
                purify_makefile(&ast)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "174 lines"),
        &MEDIUM_MAKEFILE,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(black_box(makefile)).unwrap();
                purify_makefile(&ast)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("large", "2021 lines"),
        &LARGE_MAKEFILE,
        |b, makefile| {
            b.iter(|| {
                let ast = parse_makefile(black_box(makefile)).unwrap();
                purify_makefile(&ast)
            })
        },
    );

    group.finish();
}

/// Benchmark individual purification analysis functions
fn benchmark_purify_analyses(c: &mut Criterion) {
    let mut group = c.benchmark_group("purify_analyses");

    // Use medium-sized Makefile for analysis benchmarks
    let medium_ast = parse_makefile(MEDIUM_MAKEFILE).unwrap();

    // Note: This benchmarks the full purification, which runs all 5 analyses
    // Individual analysis functions are not public, so we benchmark the complete purify
    group.bench_function("all_analyses", |b| {
        b.iter(|| purify_makefile(black_box(&medium_ast)))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_parse,
    benchmark_purify,
    benchmark_end_to_end,
    benchmark_purify_analyses,
);
criterion_main!(benches);
