use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rash::{transpile, Config, services::parser, ir};
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
    
    let download_url = concat_strings(
        "https://releases.example.com/v",
        version,
        "/tool-",
        arch,
        ".tar.gz"
    );
    
    if check_exists(prefix) {
        echo("Already installed");
        return;
    }
    
    mkdir(prefix);
    download(download_url, "/tmp/tool.tar.gz");
    extract("/tmp/tool.tar.gz", prefix);
    
    echo("Installation complete");
}

fn concat_strings(a: &str, b: &str, c: &str, d: &str, e: &str) -> &str { a }
fn check_exists(path: &str) -> bool { true }
fn echo(msg: &str) {}
fn mkdir(path: &str) {}
fn download(url: &str, dest: &str) {}
fn extract(archive: &str, dest: &str) {}
"#;

const COMPLEX_RUST: &str = r#"
fn main() {
    let config = load_config();
    let system_info = detect_system();
    
    validate_system(system_info);
    prepare_environment(config);
    
    let components = vec![
        "core",
        "cli", 
        "runtime",
        "docs"
    ];
    
    for component in components {
        install_component(component, config, system_info);
    }
    
    configure_shell_integration();
    run_post_install_tests();
    
    echo("Installation successful!");
}

fn load_config() -> &'static str { "" }
fn detect_system() -> &'static str { "" }
fn validate_system(info: &str) {}
fn prepare_environment(config: &str) {}
fn install_component(name: &str, config: &str, system: &str) {}
fn configure_shell_integration() {}
fn run_post_install_tests() {}
fn echo(msg: &str) {}
"#;

fn benchmark_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    group.bench_with_input(
        BenchmarkId::new("parse", "simple"),
        &SIMPLE_RUST,
        |b, source| {
            b.iter(|| parser::parse(source).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("parse", "medium"),
        &MEDIUM_RUST,
        |b, source| {
            b.iter(|| parser::parse(source).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("parse", "complex"),
        &COMPLEX_RUST,
        |b, source| {
            b.iter(|| parser::parse(source).unwrap())
        },
    );
    
    group.finish();
}

fn benchmark_ir_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ir_generation");
    
    let simple_ast = parser::parse(SIMPLE_RUST).unwrap();
    let medium_ast = parser::parse(MEDIUM_RUST).unwrap();
    let complex_ast = parser::parse(COMPLEX_RUST).unwrap();
    
    group.bench_with_input(
        BenchmarkId::new("ast_to_ir", "simple"),
        &simple_ast,
        |b, ast| {
            b.iter(|| ir::from_ast(ast).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("ast_to_ir", "medium"),
        &medium_ast,
        |b, ast| {
            b.iter(|| ir::from_ast(ast).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("ast_to_ir", "complex"),
        &complex_ast,
        |b, ast| {
            b.iter(|| ir::from_ast(ast).unwrap())
        },
    );
    
    group.finish();
}

fn benchmark_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization");
    
    let config = Config::default();
    let simple_ir = ir::from_ast(&parser::parse(SIMPLE_RUST).unwrap()).unwrap();
    let medium_ir = ir::from_ast(&parser::parse(MEDIUM_RUST).unwrap()).unwrap();
    
    group.bench_with_input(
        BenchmarkId::new("optimize", "simple"),
        &(&simple_ir, &config),
        |b, (ir, config)| {
            b.iter(|| ir::optimize((*ir).clone(), config).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("optimize", "medium"),
        &(&medium_ir, &config),
        |b, (ir, config)| {
            b.iter(|| ir::optimize((*ir).clone(), config).unwrap())
        },
    );
    
    group.finish();
}

fn benchmark_emission(c: &mut Criterion) {
    let mut group = c.benchmark_group("emission");
    
    let config = Config::default();
    let simple_ir = ir::optimize(
        ir::from_ast(&parser::parse(SIMPLE_RUST).unwrap()).unwrap(),
        &config
    ).unwrap();
    let medium_ir = ir::optimize(
        ir::from_ast(&parser::parse(MEDIUM_RUST).unwrap()).unwrap(),
        &config
    ).unwrap();
    
    group.bench_with_input(
        BenchmarkId::new("emit", "simple"),
        &(&simple_ir, &config),
        |b, (ir, config)| {
            b.iter(|| rash::emitter::emit(ir, config).unwrap())
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("emit", "medium"),
        &(&medium_ir, &config),
        |b, (ir, config)| {
            b.iter(|| rash::emitter::emit(ir, config).unwrap())
        },
    );
    
    group.finish();
}

fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(10));
    
    let config = Config::default();
    
    group.throughput(Throughput::Bytes(SIMPLE_RUST.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("transpile", "simple"),
        &(SIMPLE_RUST, &config),
        |b, (source, config)| {
            b.iter(|| transpile(source, (*config).clone()).unwrap())
        },
    );
    
    group.throughput(Throughput::Bytes(MEDIUM_RUST.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("transpile", "medium"),
        &(MEDIUM_RUST, &config),
        |b, (source, config)| {
            b.iter(|| transpile(source, (*config).clone()).unwrap())
        },
    );
    
    group.throughput(Throughput::Bytes(COMPLEX_RUST.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("transpile", "complex"),
        &(COMPLEX_RUST, &config),
        |b, (source, config)| {
            b.iter(|| transpile(source, (*config).clone()).unwrap())
        },
    );
    
    group.finish();
}

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
"#,
            i = i
        ));
    }
    
    source.push_str(
        r#"
fn main() {
    let start = "begin";
    echo(start);
"#
    );
    
    for i in 0..num_functions {
        source.push_str(&format!("    function_{}();\n", i));
    }
    
    source.push_str(
        r#"    echo("end");
}

fn echo(msg: &str) {}
"#
    );
    
    source
}

criterion_group!(
    benches,
    benchmark_parsing,
    benchmark_ir_generation,
    benchmark_optimization,
    benchmark_emission,
    benchmark_end_to_end,
    benchmark_memory_usage,
    benchmark_scalability
);
criterion_main!(benches);