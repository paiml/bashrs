use criterion::{criterion_group, criterion_main, Criterion};

/// Benchmark the core transpilation pipeline.
fn bench_transpile(c: &mut Criterion) {
    let input = r#"
        fn main() {
            let name = "World";
            echo("Hello, ${name}!");
        }
        fn echo(msg: &str) {}
    "#;

    c.bench_function("transpile_hello_world", |b| {
        b.iter(|| {
            let _ = bashrs::transpile(input, bashrs::Config::default());
        });
    });
}

/// Benchmark configuration parsing.
fn bench_config_parse(c: &mut Criterion) {
    c.bench_function("config_default", |b| {
        b.iter(|| {
            let _ = bashrs::Config::default();
        });
    });
}

criterion_group!(benches, bench_transpile, bench_config_parse);
criterion_main!(benches);
