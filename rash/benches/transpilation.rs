use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rash::{ir, services::parser, transpile, Config};
use std::time::Duration;

const SIMPLE_RUST: &str = r#"
fn main() {
    let x = 42;
    let greeting = "Hello, world!";
    echo(greeting);
}

fn echo(msg: &str) {}
"#;

const MEDIUM_RUST: &str = r#"
fn main() {
    let prefix = "/usr/local";
    let version = "1.0.0";
    let arch = "x86_64";
    
    if check_exists() {
        echo_installed();
        return;
    }
    
    mkdir_prefix();
    download_file();
    extract_archive();
    
    echo_complete();
}

fn check_exists() -> bool { true }
fn echo_installed() {}
fn mkdir_prefix() {}
fn download_file() {}
fn extract_archive() {}
fn echo_complete() {}
"#;

#[allow(dead_code)]
const COMPLEX_RUST: &str = r#"
fn main() {
    let config = "default";
    let system_info = "linux";
    
    validate_system();
    prepare_environment();
    
    install_component();
    install_component();
    install_component();
    
    configure_shell_integration();
    run_post_install_tests();
    
    finish_installation();
}

fn validate_system() {}
fn prepare_environment() {}
fn install_component() {}
fn configure_shell_integration() {}
fn run_post_install_tests() {}
fn finish_installation() {}
"#;

fn benchmark_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    group.bench_with_input(
        BenchmarkId::new("parse", "simple"),
        &SIMPLE_RUST,
        |b, source| b.iter(|| parser::parse(source).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("parse", "medium"),
        &MEDIUM_RUST,
        |b, source| b.iter(|| parser::parse(source).unwrap()),
    );

    group.finish();
}

fn benchmark_ir_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ir_generation");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let simple_ast = parser::parse(SIMPLE_RUST).unwrap();
    let medium_ast = parser::parse(MEDIUM_RUST).unwrap();

    group.bench_with_input(
        BenchmarkId::new("ast_to_ir", "simple"),
        &simple_ast,
        |b, ast| b.iter(|| ir::from_ast(ast).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("ast_to_ir", "medium"),
        &medium_ast,
        |b, ast| b.iter(|| ir::from_ast(ast).unwrap()),
    );

    group.finish();
}

fn benchmark_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let config = Config::default();
    let simple_ir = ir::from_ast(&parser::parse(SIMPLE_RUST).unwrap()).unwrap();

    group.bench_with_input(
        BenchmarkId::new("optimize", "simple"),
        &(&simple_ir, &config),
        |b, (ir, config)| b.iter(|| ir::optimize((*ir).clone(), config).unwrap()),
    );

    group.finish();
}

fn benchmark_emission(c: &mut Criterion) {
    let mut group = c.benchmark_group("emission");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let config = Config::default();
    let simple_ir = ir::optimize(
        ir::from_ast(&parser::parse(SIMPLE_RUST).unwrap()).unwrap(),
        &config,
    )
    .unwrap();

    group.bench_with_input(
        BenchmarkId::new("emit", "simple"),
        &(&simple_ir, &config),
        |b, (ir, config)| b.iter(|| rash::emitter::emit(ir, config).unwrap()),
    );

    group.finish();
}

fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(30);

    let config = Config::default();

    group.throughput(Throughput::Bytes(SIMPLE_RUST.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("transpile", "simple"),
        &(SIMPLE_RUST, &config),
        |b, (source, config)| b.iter(|| transpile(source, (*config).clone()).unwrap()),
    );

    group.finish();
}

#[allow(dead_code)]
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("ast_size", |b| {
        b.iter(|| {
            let ast = parser::parse(MEDIUM_RUST).unwrap();
            std::mem::size_of_val(&ast)
        })
    });

    group.bench_function("ir_size", |b| {
        b.iter(|| {
            let ast = parser::parse(MEDIUM_RUST).unwrap();
            let ir = ir::from_ast(&ast).unwrap();
            std::mem::size_of_val(&ir)
        })
    });

    group.finish();
}

#[allow(dead_code)]
fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    // Test with different input sizes
    for size in [10, 50, 100, 200].iter() {
        let large_source = generate_large_rust_source(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("large_input", size),
            &large_source,
            |b, source| {
                b.iter(|| {
                    let config = Config::default();
                    transpile(source, config).unwrap()
                })
            },
        );
    }

    group.finish();
}

#[allow(dead_code)]
fn generate_large_rust_source(num_functions: usize) -> String {
    let mut source = String::new();

    for i in 0..num_functions {
        source.push_str(&format!(
            r#"
fn function_{i}() {{
    let var1_{i} = {i};
    let var2_{i} = "string_{i}";
    let var3_{i} = var1_{i} + {i};
    helper_{i}(var2_{i});
}}

fn helper_{i}(msg: &str) {{
    let local = {i} * 2;
    echo(msg);
}}
"#
        ));
    }

    source.push_str(
        r#"
fn main() {
    let start = "begin";
    echo(start);
"#,
    );

    for i in 0..num_functions {
        source.push_str(&format!("    function_{i}();\n"));
    }

    source.push_str(
        r#"    echo("end");
}

fn echo(msg: &str) {}
"#,
    );

    source
}

criterion_group!(
    benches,
    benchmark_parsing,
    benchmark_ir_generation,
    benchmark_optimization,
    benchmark_emission,
    benchmark_end_to_end
);
criterion_main!(benches);
