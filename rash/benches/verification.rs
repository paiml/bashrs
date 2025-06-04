use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rash::{ir, models::VerificationLevel, services::parser, verifier};
use std::time::Duration;

const SAFE_RUST: &str = r#"
fn main() {
    let message = "Hello, safe world!";
    let number = 42;
    echo(message);
    increment(number);
}

fn echo(msg: &str) {}
fn increment(n: u32) -> u32 { n + 1 }
"#;

const COMPLEX_SAFE_RUST: &str = r#"
fn main() {
    let prefix = "/usr/local";
    let version = "1.0.0";
    
    validate_prefix(prefix);
    create_directories(prefix);
    download_and_install(version, prefix);
    setup_permissions(prefix);
    
    echo("Installation complete");
}

fn validate_prefix(path: &str) -> bool {
    let writable = is_writable(path);
    let exists = path_exists(path);
    writable && exists
}

fn create_directories(base: &str) {
    mkdir(concat(base, "/bin"));
    mkdir(concat(base, "/lib"));
    mkdir(concat(base, "/share"));
}

fn download_and_install(version: &str, prefix: &str) {
    let url = build_url(version);
    let temp_file = "/tmp/download.tar.gz";
    
    download_verified(url, temp_file, get_checksum(version));
    extract_archive(temp_file, prefix);
    cleanup_temp(temp_file);
}

fn setup_permissions(prefix: &str) {
    chmod(concat(prefix, "/bin"), "755");
    chmod(prefix, "755");
}

// Helper functions
fn echo(msg: &str) {}
fn is_writable(path: &str) -> bool { true }
fn path_exists(path: &str) -> bool { true }
fn mkdir(path: &str) {}
fn concat(a: &str, b: &str) -> &str { a }
fn build_url(version: &str) -> &str { version }
fn get_checksum(version: &str) -> &str { version }
fn download_verified(url: &str, dest: &str, checksum: &str) {}
fn extract_archive(archive: &str, dest: &str) {}
fn cleanup_temp(path: &str) {}
fn chmod(path: &str, mode: &str) {}
"#;

const POTENTIALLY_UNSAFE_RUST: &str = r#"
fn main() {
    let user_input = get_user_input();
    let command = build_command(user_input);
    execute_shell(command);
    
    let url = get_download_url();
    download_file(url);
}

fn get_user_input() -> &'static str { "user data" }
fn build_command(input: &str) -> &str { input }
fn execute_shell(cmd: &str) {}
fn get_download_url() -> &'static str { "http://example.com/file" }
fn download_file(url: &str) {}
"#;

fn benchmark_verification_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_levels");
    group.measurement_time(Duration::from_secs(5));

    let safe_ir = ir::from_ast(&parser::parse(SAFE_RUST).unwrap()).unwrap();
    let complex_ir = ir::from_ast(&parser::parse(COMPLEX_SAFE_RUST).unwrap()).unwrap();
    let unsafe_ir = ir::from_ast(&parser::parse(POTENTIALLY_UNSAFE_RUST).unwrap()).unwrap();

    let levels = [
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ];

    for level in levels.iter() {
        group.bench_with_input(
            BenchmarkId::new("safe_code", format!("{:?}", level)),
            &(&safe_ir, level),
            |b, (ir, level)| {
                b.iter(|| {
                    let _ = verifier::verify(ir, **level);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("complex_code", format!("{:?}", level)),
            &(&complex_ir, level),
            |b, (ir, level)| {
                b.iter(|| {
                    let _ = verifier::verify(ir, **level);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unsafe_code", format!("{:?}", level)),
            &(&unsafe_ir, level),
            |b, (ir, level)| {
                b.iter(|| {
                    let _ = verifier::verify(ir, **level);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_individual_verifications(c: &mut Criterion) {
    let mut group = c.benchmark_group("individual_verifications");

    let safe_ir = ir::from_ast(&parser::parse(SAFE_RUST).unwrap()).unwrap();
    let complex_ir = ir::from_ast(&parser::parse(COMPLEX_SAFE_RUST).unwrap()).unwrap();
    let unsafe_ir = ir::from_ast(&parser::parse(POTENTIALLY_UNSAFE_RUST).unwrap()).unwrap();

    group.bench_with_input(
        BenchmarkId::new("command_injection", "safe"),
        &safe_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_no_command_injection(ir).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("command_injection", "complex"),
        &complex_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_no_command_injection(ir).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("command_injection", "potentially_unsafe"),
        &unsafe_ir,
        |b, ir| {
            b.iter(|| {
                let _ = rash::verifier::properties::verify_no_command_injection(ir);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("determinism", "safe"),
        &safe_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_deterministic(ir).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("determinism", "complex"),
        &complex_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_deterministic(ir).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("idempotency", "safe"),
        &safe_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_idempotency(ir).unwrap()),
    );

    group.bench_with_input(
        BenchmarkId::new("resource_safety", "complex"),
        &complex_ir,
        |b, ir| b.iter(|| rash::verifier::properties::verify_resource_safety(ir).unwrap()),
    );

    group.finish();
}

fn benchmark_verification_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_scalability");
    group.measurement_time(Duration::from_secs(10));

    // Generate increasingly complex IR structures
    for complexity in [10, 25, 50, 100].iter() {
        let complex_source = generate_complex_rust_for_verification(*complexity);
        let ir = ir::from_ast(&parser::parse(&complex_source).unwrap()).unwrap();

        group.bench_with_input(
            BenchmarkId::new("strict_verification", complexity),
            &ir,
            |b, ir| b.iter(|| verifier::verify(ir, VerificationLevel::Strict).unwrap()),
        );

        group.bench_with_input(
            BenchmarkId::new("paranoid_verification", complexity),
            &ir,
            |b, ir| b.iter(|| verifier::verify(ir, VerificationLevel::Paranoid).unwrap()),
        );
    }

    group.finish();
}

fn benchmark_verification_with_errors(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_errors");

    // Test how verification performs when it finds errors
    let error_cases = [
        ("injection_attempt", generate_injection_attempt()),
        ("non_deterministic", generate_non_deterministic()),
        ("resource_intensive", generate_resource_intensive()),
    ];

    for (name, source) in error_cases.iter() {
        let ir = ir::from_ast(&parser::parse(source).unwrap()).unwrap();

        group.bench_with_input(BenchmarkId::new("error_detection", name), &ir, |b, ir| {
            b.iter(|| {
                let _ = verifier::verify(ir, VerificationLevel::Strict);
            })
        });
    }

    group.finish();
}

fn benchmark_effect_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("effect_analysis");

    let sources = [
        ("pure_functions", SAFE_RUST),
        ("file_operations", COMPLEX_SAFE_RUST),
        ("network_operations", POTENTIALLY_UNSAFE_RUST),
    ];

    for (name, source) in sources.iter() {
        let ir = ir::from_ast(&parser::parse(source).unwrap()).unwrap();

        group.bench_with_input(BenchmarkId::new("analyze_effects", name), &ir, |b, ir| {
            b.iter(|| {
                let effects = ir.effects();
                (
                    effects.is_pure(),
                    effects.has_filesystem_effects(),
                    effects.has_network_effects(),
                    effects.has_system_effects(),
                )
            })
        });
    }

    group.finish();
}

fn generate_complex_rust_for_verification(complexity: usize) -> String {
    let mut source = String::new();

    for i in 0..complexity {
        source.push_str(&format!(
            r#"
fn process_step_{i}() {{
    let input_{i} = "data_{i}";
    let processed_{i} = transform_{i}(input_{i});
    validate_{i}(processed_{i});
    store_{i}(processed_{i});
}}

fn transform_{i}(data: &str) -> &str {{ data }}
fn validate_{i}(data: &str) -> bool {{ true }}
fn store_{i}(data: &str) {{}}
"#,
            i = i
        ));
    }

    source.push_str("fn main() {\n");
    for i in 0..complexity {
        source.push_str(&format!("    process_step_{}();\n", i));
    }
    source.push_str("}\n");

    source
}

fn generate_injection_attempt() -> String {
    r#"
fn main() {
    let user_input = "; rm -rf /";
    let command = concat("echo ", user_input);
    execute(command);
}

fn concat(a: &str, b: &str) -> &str { a }
fn execute(cmd: &str) {}
"#
    .to_string()
}

fn generate_non_deterministic() -> String {
    r#"
fn main() {
    let timestamp = current_time();
    let random_value = generate_random();
    echo(timestamp);
    echo(random_value);
}

fn current_time() -> &'static str { "time" }
fn generate_random() -> &'static str { "random" }
fn echo(msg: &str) {}
"#
    .to_string()
}

fn generate_resource_intensive() -> String {
    let mut source = String::new();

    source.push_str("fn main() {\n");

    // Generate many network operations
    for i in 0..20 {
        source.push_str(&format!("    download_file_{}();\n", i));
    }

    // Generate many file operations
    for i in 0..60 {
        source.push_str(&format!("    process_file_{}();\n", i));
    }

    source.push_str("}\n");

    for i in 0..20 {
        source.push_str(&format!(
            "fn download_file_{}() {{ curl(\"http://example.com/file{}\"); }}\n",
            i, i
        ));
    }

    for i in 0..60 {
        source.push_str(&format!(
            "fn process_file_{}() {{ cp(\"/src/file{}\", \"/dst/file{}\"); }}\n",
            i, i, i
        ));
    }

    source.push_str("fn curl(url: &str) {}\n");
    source.push_str("fn cp(src: &str, dst: &str) {}\n");

    source
}

criterion_group!(
    benches,
    benchmark_verification_levels,
    benchmark_individual_verifications,
    benchmark_verification_scalability,
    benchmark_verification_with_errors,
    benchmark_effect_analysis
);
criterion_main!(benches);
