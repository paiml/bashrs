use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rash::validation::ValidationLevel;
use rash::{transpile, Config};
use std::time::Instant;

fn generate_test_script(lines: usize) -> String {
    let mut script = String::from("fn main() {\n");

    for i in 0..lines {
        script.push_str(&format!("    let var{} = \"value{}\";\n", i, i));
        script.push_str(&format!(
            "    println!(\"Variable {} = {{}}\", var{});\n",
            i, i
        ));

        if i % 10 == 0 {
            script.push_str(&format!("    if var{} == \"value{}\" {{\n", i, i));
            script.push_str("        println!(\"Match!\");\n");
            script.push_str("    }\n");
        }
    }

    script.push_str("}\n");
    script
}

fn bench_validation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_overhead");

    for size in [100, 500, 1000].iter() {
        let script = generate_test_script(*size);

        group.bench_with_input(BenchmarkId::new("none", size), size, |b, _| {
            let mut config = Config::default();
            config.validation_level = Some(ValidationLevel::None);

            b.iter(|| {
                let _ = transpile(black_box(&script), config.clone());
            });
        });

        group.bench_with_input(BenchmarkId::new("minimal", size), size, |b, _| {
            let mut config = Config::default();
            config.validation_level = Some(ValidationLevel::Minimal);

            b.iter(|| {
                let _ = transpile(black_box(&script), config.clone());
            });
        });

        group.bench_with_input(BenchmarkId::new("strict", size), size, |b, _| {
            let mut config = Config::default();
            config.validation_level = Some(ValidationLevel::Strict);

            b.iter(|| {
                let _ = transpile(black_box(&script), config.clone());
            });
        });
    }

    group.finish();
}

fn bench_individual_rules(c: &mut Criterion) {
    use rash::validation::rules::*;

    let mut group = c.benchmark_group("individual_rules");

    // Benchmark variable expansion validation
    group.bench_function("sc2086_quoted", |b| {
        let var = VariableExpansion::Quoted("USER".to_string());
        b.iter(|| {
            let _ = black_box(&var).validate();
        });
    });

    group.bench_function("sc2086_unquoted", |b| {
        let var = VariableExpansion::Unquoted("USER".to_string());
        b.iter(|| {
            let _ = black_box(&var).validate();
        });
    });

    // Benchmark command substitution validation
    group.bench_function("sc2046_quoted", |b| {
        let cmd = CommandSubstitution {
            command: "date".to_string(),
            context: SubstitutionContext::Quoted,
        };
        b.iter(|| {
            let _ = black_box(&cmd).validate();
        });
    });

    // Benchmark glob pattern validation
    group.bench_function("sc2035_safe", |b| {
        b.iter(|| {
            let _ = validate_glob_pattern(black_box("file.txt"));
        });
    });

    group.bench_function("sc2035_unsafe", |b| {
        b.iter(|| {
            let _ = validate_glob_pattern(black_box("-rf"));
        });
    });

    // Benchmark snippet validation
    group.bench_function("validate_all_clean", |b| {
        let snippet = "echo \"$USER\"\ncd /tmp || exit 1\nread -r var";
        b.iter(|| {
            let _ = validate_all(black_box(snippet));
        });
    });

    group.bench_function("validate_all_errors", |b| {
        let snippet = "echo `date`\ncd /tmp\nread var";
        b.iter(|| {
            let _ = validate_all(black_box(snippet));
        });
    });

    group.finish();
}

fn measure_validation_percentage(c: &mut Criterion) {
    let script = generate_test_script(1000);
    let config_no_validation = Config {
        validation_level: Some(ValidationLevel::None),
        ..Default::default()
    };
    let config_with_validation = Config {
        validation_level: Some(ValidationLevel::Minimal),
        ..Default::default()
    };

    // Measure without validation
    let start = Instant::now();
    for _ in 0..10 {
        let _ = transpile(&script, config_no_validation.clone());
    }
    let time_without = start.elapsed();

    // Measure with validation
    let start = Instant::now();
    for _ in 0..10 {
        let _ = transpile(&script, config_with_validation.clone());
    }
    let time_with = start.elapsed();

    let overhead_percentage = ((time_with.as_nanos() - time_without.as_nanos()) as f64
        / time_without.as_nanos() as f64)
        * 100.0;

    println!("Validation overhead: {:.2}%", overhead_percentage);

    // Assert that overhead is less than 1%
    assert!(
        overhead_percentage < 1.0,
        "Validation overhead ({:.2}%) exceeds 1% requirement",
        overhead_percentage
    );
}

criterion_group!(
    benches,
    bench_validation_overhead,
    bench_individual_rules,
    measure_validation_percentage
);
criterion_main!(benches);
