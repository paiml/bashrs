/// Look up base image size from known sizes
fn lookup_base_image_size(image: &str) -> u64 {
    // Try exact match first
    for (name, size) in BASE_IMAGE_SIZES {
        if image == *name {
            return *size;
        }
    }

    // Try prefix match (for tagged versions)
    let image_base = image.split(':').next().unwrap_or(image);
    for (name, size) in BASE_IMAGE_SIZES {
        let name_base = name.split(':').next().unwrap_or(name);
        if image_base == name_base {
            return *size;
        }
    }

    0
}

/// Estimate apt-get packages size and check for bloat
fn estimate_apt_size(
    cmd: &str,
    line: usize,
    total: &mut u64,
    notes: &mut Vec<String>,
    bloat: &mut Vec<BloatPattern>,
) {
    let packages = extract_apt_packages(cmd);
    for pkg in &packages {
        let pkg_size = lookup_package_size(pkg);
        if pkg_size > 0 {
            *total += pkg_size;
            notes.push(format!("{}: ~{}MB", pkg, pkg_size / 1_000_000));
        }
    }
    if !cmd.contains("--no-install-recommends") && !packages.is_empty() {
        bloat.push(BloatPattern {
            code: "SIZE002".to_string(),
            description: "apt-get install without --no-install-recommends".to_string(),
            line,
            wasted_bytes: 100_000_000,
            remediation: "Add '--no-install-recommends' to apt-get install".to_string(),
        });
    }
}

/// Estimate pip packages size and check for bloat
fn estimate_pip_size(
    cmd: &str,
    line: usize,
    total: &mut u64,
    notes: &mut Vec<String>,
    bloat: &mut Vec<BloatPattern>,
) {
    let packages = extract_pip_packages(cmd);
    for pkg in &packages {
        let pkg_size = lookup_package_size(pkg);
        if pkg_size > 0 {
            *total += pkg_size;
            notes.push(format!("{}: ~{}MB", pkg, pkg_size / 1_000_000));
        }
    }
    if !cmd.contains("--no-cache-dir") {
        bloat.push(BloatPattern {
            code: "SIZE003".to_string(),
            description: "pip install without --no-cache-dir".to_string(),
            line,
            wasted_bytes: 50_000_000,
            remediation: "Add '--no-cache-dir' to pip install".to_string(),
        });
    }
}

/// Estimate npm packages size and check for bloat
fn estimate_npm_size(
    cmd: &str,
    line: usize,
    total: &mut u64,
    notes: &mut Vec<String>,
    bloat: &mut Vec<BloatPattern>,
) {
    *total += 200_000_000;
    notes.push("npm dependencies".to_string());
    if !cmd.contains("--production") && !cmd.contains("ci") {
        bloat.push(BloatPattern {
            code: "SIZE004".to_string(),
            description: "npm install includes dev dependencies".to_string(),
            line,
            wasted_bytes: 100_000_000,
            remediation: "Use 'npm ci --only=production' for smaller image".to_string(),
        });
    }
}

/// Estimate size of a RUN layer
fn estimate_run_layer_size(cmd: &str, line: usize) -> (u64, Option<String>, Vec<BloatPattern>) {
    let mut total: u64 = 0;
    let mut notes = Vec::new();
    let mut bloat = Vec::new();

    if cmd.contains("apt-get install") || cmd.contains("apt install") {
        estimate_apt_size(cmd, line, &mut total, &mut notes, &mut bloat);
    }
    if cmd.contains("pip install") || cmd.contains("pip3 install") {
        estimate_pip_size(cmd, line, &mut total, &mut notes, &mut bloat);
    }
    if cmd.contains("npm install") || cmd.contains("npm i ") {
        estimate_npm_size(cmd, line, &mut total, &mut notes, &mut bloat);
    }

    if total == 0 {
        total = 10_000_000;
    }

    let notes_str = if notes.is_empty() {
        None
    } else {
        Some(notes.join(", "))
    };

    (total, notes_str, bloat)
}

/// Extract package names from apt-get install command
fn extract_apt_packages(cmd: &str) -> Vec<String> {
    let mut packages = Vec::new();

    // Find the install command and extract packages
    if let Some(idx) = cmd.find("install") {
        let after_install = &cmd[idx + 7..];
        for word in after_install.split_whitespace() {
            // Skip flags
            if word.starts_with('-') || word.starts_with('\\') {
                continue;
            }
            // Skip operators
            if word == "&&" || word == "||" || word == ";" {
                break;
            }
            // Skip -y which might come after install
            if word == "-y" {
                continue;
            }
            packages.push(word.to_string());
        }
    }

    packages
}

/// Extract package names from pip install command
fn extract_pip_packages(cmd: &str) -> Vec<String> {
    let mut packages = Vec::new();

    // Find pip install and extract packages
    let install_patterns = ["pip install", "pip3 install"];
    for pattern in &install_patterns {
        if let Some(idx) = cmd.find(pattern) {
            let after_install = &cmd[idx + pattern.len()..];
            for word in after_install.split_whitespace() {
                // Skip flags
                if word.starts_with('-') || word.starts_with('\\') {
                    continue;
                }
                // Skip operators
                if word == "&&" || word == "||" || word == ";" {
                    break;
                }
                // Skip requirement files
                if word.ends_with(".txt") {
                    continue;
                }
                packages.push(word.to_string());
            }
        }
    }

    packages
}

/// Look up package size from known sizes
fn lookup_package_size(package: &str) -> u64 {
    let package_lower = package.to_lowercase();

    for (name, size) in PACKAGE_SIZES {
        if package_lower == *name || package_lower.contains(name) {
            return *size;
        }
    }

    0
}

/// Check if Docker daemon is available
pub fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get actual image size from Docker
pub fn get_docker_image_size(image_name: &str) -> Option<u64> {
    let output = Command::new("docker")
        .args(["images", image_name, "--format", "{{.Size}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let size_str = String::from_utf8_lossy(&output.stdout);
    parse_docker_size(size_str.trim())
}

/// Parse Docker size string (e.g., "1.5GB", "500MB")
fn parse_docker_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.to_uppercase();

    // Try to parse number and unit
    let (num_str, multiplier) = if size_str.ends_with("GB") {
        (&size_str[..size_str.len() - 2], 1_000_000_000u64)
    } else if size_str.ends_with("MB") {
        (&size_str[..size_str.len() - 2], 1_000_000u64)
    } else if size_str.ends_with("KB") {
        (&size_str[..size_str.len() - 2], 1_000u64)
    } else if size_str.ends_with('B') {
        (&size_str[..size_str.len() - 1], 1u64)
    } else {
        return None;
    };

    num_str
        .trim()
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

/// Generate lint result from size estimate
pub fn size_estimate_to_lint_result(
    estimate: &SizeEstimate,
    profile: PlatformProfile,
    strict: bool,
) -> LintResult {
    let mut result = LintResult::new();

    // Add warnings from analysis
    for warning in &estimate.warnings {
        let span = Span::new(1, 1, 1, 1);
        result.add(Diagnostic::new("SIZE-INFO", Severity::Info, warning, span));
    }

    // Add bloat patterns as warnings
    for bloat in &estimate.bloat_patterns {
        let span = Span::new(bloat.line, 1, bloat.line, 1);
        let mut diag = Diagnostic::new(
            bloat.code.clone(),
            Severity::Warning,
            format!(
                "{} (~{}MB wasted)",
                bloat.description,
                bloat.wasted_bytes / 1_000_000
            ),
            span,
        );
        diag.fix = Some(Fix::new(bloat.remediation.clone()));
        result.add(diag);
    }

    // Check against platform limits
    let max_size = profile.max_size_bytes();
    let warning_threshold = (max_size as f64 * profile.size_warning_threshold()) as u64;

    if estimate.total_estimated > max_size {
        let severity = if strict {
            Severity::Error
        } else {
            Severity::Warning
        };
        let span = Span::new(1, 1, 1, 1);
        let mut diag = Diagnostic::new(
            "SIZE-LIMIT",
            severity,
            format!(
                "Estimated image size ({:.1}GB) exceeds platform limit ({:.1}GB)",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ),
            span,
        );
        diag.fix = Some(Fix::new(
            "Consider using a smaller base image or multi-stage build",
        ));
        result.add(diag);
    } else if estimate.total_estimated > warning_threshold {
        let span = Span::new(1, 1, 1, 1);
        let mut diag = Diagnostic::new(
            "SIZE-WARNING",
            Severity::Warning,
            format!(
                "Estimated image size ({:.1}GB) approaching platform limit ({:.1}GB)",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ),
            span,
        );
        diag.fix = Some(Fix::new("Consider optimizations to reduce image size"));
        result.add(diag);
    }

    result
}

/// Format size estimate as human-readable output
pub fn format_size_estimate(estimate: &SizeEstimate, verbose: bool) -> String {
    let mut output = String::new();

    output.push_str("Image Size Analysis\n");
    output.push_str("===================\n\n");

    // Base image
    output.push_str(&format!(
        "Base image: {} (~{:.1}GB)\n\n",
        estimate.base_image,
        estimate.base_image_size as f64 / 1_000_000_000.0
    ));

    // Layer breakdown
    if verbose {
        output.push_str("Layer Breakdown:\n");
        for layer in &estimate.layer_estimates {
            let size_str = if layer.estimated_size == 0 {
                "unknown".to_string()
            } else {
                format!("~{:.1}MB", layer.estimated_size as f64 / 1_000_000.0)
            };

            output.push_str(&format!(
                "  [{}] {} ({}) - line {}\n",
                layer.layer_num, layer.instruction, size_str, layer.line
            ));

            if let Some(notes) = &layer.notes {
                output.push_str(&format!("      {}\n", notes));
            }
        }
        output.push('\n');
    }

    // Total
    output.push_str(&format!(
        "Estimated total: {:.2}GB\n\n",
        estimate.total_estimated as f64 / 1_000_000_000.0
    ));

    // Bloat patterns
    if !estimate.bloat_patterns.is_empty() {
        output.push_str("Optimization Opportunities:\n");
        for bloat in &estimate.bloat_patterns {
            output.push_str(&format!(
                "  {} [line {}]: {} (~{}MB)\n",
                bloat.code,
                bloat.line,
                bloat.description,
                bloat.wasted_bytes / 1_000_000
            ));
            output.push_str(&format!("    Fix: {}\n", bloat.remediation));
        }
        output.push('\n');
    }

    // Warnings
    if !estimate.warnings.is_empty() {
        output.push_str("Warnings:\n");
        for warning in &estimate.warnings {
            output.push_str(&format!("  - {}\n", warning));
        }
    }

    output
}

/// Format size estimate as JSON
pub fn format_size_estimate_json(estimate: &SizeEstimate) -> String {
    let layers: Vec<HashMap<&str, serde_json::Value>> = estimate
        .layer_estimates
        .iter()
        .map(|l| {
            let mut map = HashMap::new();
            map.insert("layer_num", serde_json::json!(l.layer_num));
            map.insert("instruction", serde_json::json!(l.instruction));
            map.insert("line", serde_json::json!(l.line));
            map.insert("estimated_bytes", serde_json::json!(l.estimated_size));
            map.insert("notes", serde_json::json!(l.notes));
            map
        })
        .collect();

    let bloat: Vec<HashMap<&str, serde_json::Value>> = estimate
        .bloat_patterns
        .iter()
        .map(|b| {
            let mut map = HashMap::new();
            map.insert("code", serde_json::json!(b.code));
            map.insert("description", serde_json::json!(b.description));
            map.insert("line", serde_json::json!(b.line));
            map.insert("wasted_bytes", serde_json::json!(b.wasted_bytes));
            map.insert("remediation", serde_json::json!(b.remediation));
            map
        })
        .collect();

    let json = serde_json::json!({
        "base_image": estimate.base_image,
        "base_image_bytes": estimate.base_image_size,
        "total_estimated_bytes": estimate.total_estimated,
        "total_estimated_gb": estimate.total_estimated as f64 / 1_000_000_000.0,
        "layers": layers,
        "bloat_patterns": bloat,
        "warnings": estimate.warnings,
    });

    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

/// List all SIZE rules
pub fn list_size_rules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("SIZE001", "apt cache not cleaned after install"),
        ("SIZE002", "apt-get install without --no-install-recommends"),
        ("SIZE003", "pip install without --no-cache-dir"),
        ("SIZE004", "npm install includes dev dependencies"),
        ("SIZE-LIMIT", "Image size exceeds platform limit"),
        ("SIZE-WARNING", "Image size approaching platform limit"),
    ]
}
