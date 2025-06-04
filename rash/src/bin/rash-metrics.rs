use std::fs;
use std::path::Path;

fn analyze_directory(path: &Path) -> (usize, usize, usize, usize) {
    let mut total_lines = 0;
    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.to_str().unwrap_or("").contains("target") {
                let (tl, cl, cm, bl) = analyze_directory(&path);
                total_lines += tl;
                code_lines += cl;
                comment_lines += cm;
                blank_lines += bl;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        total_lines += 1;
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            blank_lines += 1;
                        } else if trimmed.starts_with("//")
                            || trimmed.starts_with("/*")
                            || trimmed.starts_with("*")
                        {
                            comment_lines += 1;
                        } else {
                            code_lines += 1;
                        }
                    }
                }
            }
        }
    }

    (total_lines, code_lines, comment_lines, blank_lines)
}

fn main() {
    println!("\n## RASH Custom Metrics\n");

    let (total, code, comments, blank) = analyze_directory(Path::new("rash/src"));

    println!("### Code Statistics");
    println!("- Total Lines: {}", total);
    println!(
        "- Code Lines: {} ({:.1}%)",
        code,
        (code as f64 / total as f64) * 100.0
    );
    println!(
        "- Comment Lines: {} ({:.1}%)",
        comments,
        (comments as f64 / total as f64) * 100.0
    );
    println!(
        "- Blank Lines: {} ({:.1}%)",
        blank,
        (blank as f64 / total as f64) * 100.0
    );

    println!("\n### Module Analysis");
    let modules = ["ast", "cli", "emitter", "ir", "services", "verifier"];
    for module in &modules {
        let module_path = Path::new("rash/src").join(module);
        if module_path.exists() {
            let (mt, mc, _, _) = analyze_directory(&module_path);
            println!("- {}: {} lines ({} code)", module, mt, mc);
        }
    }

    println!("\n### Quality Indicators");
    let comment_ratio = comments as f64 / code as f64;
    println!("- Comment to Code Ratio: {:.2}", comment_ratio);
    println!("- Average File Size: ~{} lines", total / 50); // Rough estimate

    if comment_ratio < 0.1 {
        println!("⚠️  Low comment ratio - consider adding more documentation");
    } else {
        println!("✅ Good documentation coverage");
    }
}
