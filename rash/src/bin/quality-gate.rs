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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let complexity_threshold: u32 = args
        .iter()
        .position(|arg| arg == "--complexity-threshold")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let cognitive_threshold: u32 = args
        .iter()
        .position(|arg| arg == "--cognitive-threshold")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(15);

    let dead_code_threshold: u32 = args
        .iter()
        .position(|arg| arg == "--dead-code-threshold")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let mut all_passed = true;

    // Check complexity if report exists
    if let Ok(content) = fs::read_to_string("complexity-current.json") {
        if let Ok(report) = serde_json::from_str::<ComplexityReport>(&content) {
            let complexity_violations: Vec<_> = report
                .files
                .iter()
                .filter(|f| f.max_cyclomatic > complexity_threshold)
                .collect();

            let cognitive_violations: Vec<_> = report
                .files
                .iter()
                .filter(|f| f.max_cognitive > cognitive_threshold)
                .collect();

            if !complexity_violations.is_empty() {
                eprintln!("❌ Cyclomatic complexity threshold violations:");
                for v in complexity_violations {
                    eprintln!("  {} - cyclomatic: {}", v.file_path, v.max_cyclomatic);
                }
                all_passed = false;
            }

            if !cognitive_violations.is_empty() {
                eprintln!("❌ Cognitive complexity threshold violations:");
                for v in cognitive_violations {
                    eprintln!("  {} - cognitive: {}", v.file_path, v.max_cognitive);
                }
                all_passed = false;
            }
        }
    }

    // Check dead code if report exists
    if let Ok(content) = fs::read_to_string("deadcode-current.json") {
        if let Ok(report) = serde_json::from_str::<DeadCodeReport>(&content) {
            let violations: Vec<_> = report
                .files
                .iter()
                .filter(|f| f.dead_code_count > dead_code_threshold)
                .collect();

            if !violations.is_empty() {
                eprintln!("❌ Dead code threshold violations:");
                for v in violations {
                    eprintln!("  {} - dead code items: {}", v.file_path, v.dead_code_count);
                }
                all_passed = false;
            }
        }
    }

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
