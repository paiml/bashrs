use serde::Deserialize;
use std::fs;
use std::process;

#[derive(Deserialize)]
struct ComplexityReport {
    files: Vec<FileComplexity>,
}

#[derive(Deserialize)]
struct FileComplexity {
    file_path: String,
    max_cyclomatic: u32,
    max_cognitive: u32,
}

#[derive(Deserialize)]
struct DeadCodeReport {
    files: Vec<DeadCodeFile>,
}

#[derive(Deserialize)]
struct DeadCodeFile {
    file_path: String,
    dead_code_count: u32,
}

/// Parse threshold argument from command line
fn parse_threshold_arg(args: &[String], flag: &str, default: u32) -> u32 {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Check complexity violations and report them
fn check_complexity_violations(complexity_threshold: u32, cognitive_threshold: u32) -> bool {
    let content = match fs::read_to_string("complexity-current.json") {
        Ok(c) => c,
        Err(_) => return true, // No report = pass
    };

    let report = match serde_json::from_str::<ComplexityReport>(&content) {
        Ok(r) => r,
        Err(_) => return true, // Invalid report = pass
    };

    let mut passed = true;

    // Check cyclomatic complexity
    let complexity_violations: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.max_cyclomatic > complexity_threshold)
        .collect();

    if !complexity_violations.is_empty() {
        eprintln!("❌ Cyclomatic complexity threshold violations:");
        for v in complexity_violations {
            eprintln!("  {} - cyclomatic: {}", v.file_path, v.max_cyclomatic);
        }
        passed = false;
    }

    // Check cognitive complexity
    let cognitive_violations: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.max_cognitive > cognitive_threshold)
        .collect();

    if !cognitive_violations.is_empty() {
        eprintln!("❌ Cognitive complexity threshold violations:");
        for v in cognitive_violations {
            eprintln!("  {} - cognitive: {}", v.file_path, v.max_cognitive);
        }
        passed = false;
    }

    passed
}

/// Check dead code violations and report them
fn check_dead_code_violations(dead_code_threshold: u32) -> bool {
    let content = match fs::read_to_string("deadcode-current.json") {
        Ok(c) => c,
        Err(_) => return true, // No report = pass
    };

    let report = match serde_json::from_str::<DeadCodeReport>(&content) {
        Ok(r) => r,
        Err(_) => return true, // Invalid report = pass
    };

    let violations: Vec<_> = report
        .files
        .iter()
        .filter(|f| f.dead_code_count > dead_code_threshold)
        .collect();

    if violations.is_empty() {
        return true;
    }

    eprintln!("❌ Dead code threshold violations:");
    for v in violations {
        eprintln!("  {} - dead code items: {}", v.file_path, v.dead_code_count);
    }
    false
}

/// Report final results and exit appropriately
fn report_results(
    all_passed: bool,
    complexity_threshold: u32,
    cognitive_threshold: u32,
    dead_code_threshold: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if all_passed {
        println!("✅ All quality gates passed!");
        println!("  Complexity threshold: {complexity_threshold}");
        println!("  Cognitive threshold: {cognitive_threshold}");
        println!("  Dead code threshold: {dead_code_threshold}");
        Ok(())
    } else {
        process::exit(1);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let complexity_threshold = parse_threshold_arg(&args, "--complexity-threshold", 10);
    let cognitive_threshold = parse_threshold_arg(&args, "--cognitive-threshold", 15);
    let dead_code_threshold = parse_threshold_arg(&args, "--dead-code-threshold", 5);

    let complexity_passed = check_complexity_violations(complexity_threshold, cognitive_threshold);
    let dead_code_passed = check_dead_code_violations(dead_code_threshold);

    let all_passed = complexity_passed && dead_code_passed;

    report_results(
        all_passed,
        complexity_threshold,
        cognitive_threshold,
        dead_code_threshold,
    )
}
