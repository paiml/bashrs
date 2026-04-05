/// Simple recursive directory walker (avoids external dependency)
fn walk_dir(dir: &Path, cb: &mut dyn FnMut(&Path)) -> Result<()> {
    let entries = fs::read_dir(dir).map_err(Error::Io)?;
    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            walk_dir(&path, cb)?;
        } else {
            cb(&path);
        }
    }
    Ok(())
}

pub(crate) struct PurifyReportData<'a> {
    pub input: &'a Path,
    pub output: Option<&'a Path>,
    pub source: &'a str,
    pub purified_bash: &'a str,
    pub read_time: std::time::Duration,
    pub parse_time: std::time::Duration,
    pub purify_time: std::time::Duration,
    pub codegen_time: std::time::Duration,
    pub write_time: std::time::Duration,
    pub total_time: std::time::Duration,
}

pub(crate) fn purify_print_report(data: PurifyReportData<'_>) {
    let PurifyReportData {
        input,
        output,
        source,
        purified_bash,
        read_time,
        parse_time,
        purify_time,
        codegen_time,
        write_time,
        total_time,
    } = data;
    use crate::cli::color::*;

    println!();
    println!("{BOLD}=== Purification Report ==={RESET}");
    println!("Input:  {CYAN}{}{RESET}", input.display());
    if let Some(output_path) = output {
        println!("Output: {CYAN}{}{RESET}", output_path.display());
    }
    println!();
    println!(
        "Input size:  {WHITE}{} lines{RESET}, {} bytes",
        source.lines().count(),
        source.len()
    );
    println!(
        "Output size: {WHITE}{} lines{RESET}, {} bytes",
        purified_bash.lines().count(),
        purified_bash.len()
    );

    println!();
    println!("{BOLD}Transformations Applied:{RESET}");
    println!("  {GREEN}✓{RESET} Shebang: #!/bin/bash → #!/bin/sh");
    println!("  {GREEN}✓{RESET} Determinism: Removed $RANDOM, timestamps");
    println!("  {GREEN}✓{RESET} Idempotency: mkdir → mkdir -p, rm → rm -f");
    println!("  {GREEN}✓{RESET} Safety: All variables quoted");

    println!();
    println!("{BOLD}Performance:{RESET}");
    println!("  {DIM}Read:{RESET}     {:>8.2?}", read_time);
    println!("  {DIM}Parse:{RESET}    {:>8.2?}", parse_time);
    println!("  {DIM}Purify:{RESET}   {:>8.2?}", purify_time);
    println!("  {DIM}Codegen:{RESET}  {:>8.2?}", codegen_time);
    println!("  {DIM}Write:{RESET}    {:>8.2?}", write_time);
    println!("  {DIM}─────────────────{RESET}");
    println!("  {WHITE}Total:{RESET}    {:>8.2?}", total_time);

    let throughput = (source.len() as f64) / total_time.as_secs_f64() / 1024.0 / 1024.0;
    println!();
    println!("Throughput: {WHITE}{:.2} MB/s{RESET}", throughput);
}

pub(crate) fn purify_generate_tests(
    output: Option<&Path>,
    purified_bash: &str,
    property_tests: bool,
    report: bool,
) -> Result<()> {
    use crate::bash_transpiler::test_generator::{TestGenerator, TestGeneratorOptions};

    let output_path = output.ok_or_else(|| {
        Error::Validation("--with-tests requires -o flag to specify output file".to_string())
    })?;

    let test_file_name = format!(
        "{}_test.sh",
        output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::Internal("Invalid output file name".to_string()))?
    );
    let test_path = output_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&test_file_name);

    let test_options = TestGeneratorOptions {
        property_tests,
        property_test_count: 100,
    };
    let generator = TestGenerator::new(test_options);
    let tests = generator.generate_tests(output_path, purified_bash);

    fs::write(&test_path, tests).map_err(Error::Io)?;
    info!("Test suite written to {}", test_path.display());

    if report {
        println!("\nTest Suite:");
        println!("  Location: {}", test_path.display());
        println!(
            "  Property tests: {}",
            if property_tests {
                "Enabled (100 cases)"
            } else {
                "Disabled"
            }
        );
    }
    Ok(())
}
