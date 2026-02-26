use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_error_classification(c: &mut Criterion) {
    let sample_error = "bash: line 42: syntax error near unexpected token `)'";

    c.bench_function("classify_error", |b| {
        b.iter(|| {
            let _result = black_box(sample_error.len());
        });
    });
}

criterion_group!(benches, bench_error_classification);
criterion_main!(benches);
