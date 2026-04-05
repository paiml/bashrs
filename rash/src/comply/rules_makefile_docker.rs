fn check_makefile_safety(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut declared_phony: Vec<String> = Vec::new();
    let mut defined_targets: Vec<(String, usize)> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Collect .PHONY declarations
        if trimmed.starts_with(".PHONY") {
            collect_phony_targets(trimmed, &mut declared_phony);
        }

        // Collect target definitions (name: ...)
        if is_target_definition(line) {
            if let Some(name) = extract_target_name(line) {
                defined_targets.push((name, i + 1));
            }
        }

        // Recipe-level checks (tab-indented lines)
        if line.starts_with('\t') {
            check_makefile_recipe(trimmed, i + 1, &mut violations);
        }
    }

    // Check for common targets missing .PHONY
    check_missing_phony(&declared_phony, &defined_targets, &mut violations);

    RuleResult {
        rule: RuleId::MakefileSafety,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_makefile_recipe(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // MAKE001: eval in recipe (injection risk)
    if trimmed.contains("eval ") {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE001: eval in Makefile recipe (injection risk)".to_string(),
        });
    }

    // MAKE002: bare 'make' instead of $(MAKE) in recipes
    if is_recursive_make_bare(trimmed) {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE002: Use $(MAKE) instead of bare make for recursive calls".to_string(),
        });
    }

    // MAKE003: rm -rf with variable in recipe (dangerous)
    if is_dangerous_recipe_rm(trimmed) {
        violations.push(Violation {
            rule: RuleId::MakefileSafety,
            line: Some(line_num),
            message: "MAKE003: Dangerous rm -rf with variable in recipe".to_string(),
        });
    }
}

/// Detect bare `make target` instead of `$(MAKE) target` in recipes
fn is_recursive_make_bare(trimmed: &str) -> bool {
    // Match: starts with make, or has `&& make`, `; make`, `|| make`
    let starts_bare = trimmed.starts_with("make ") || trimmed == "make";
    let has_chained =
        trimmed.contains("&& make ") || trimmed.contains("; make ") || trimmed.contains("|| make ");
    // Exclude $(MAKE) and ${MAKE} and @make (suppressed)
    let is_variable = trimmed.contains("$(MAKE)") || trimmed.contains("${MAKE}");
    (starts_bare || has_chained) && !is_variable
}

/// Detect rm -rf with unguarded variable expansion in recipes
fn is_dangerous_recipe_rm(trimmed: &str) -> bool {
    if !trimmed.contains("rm -rf") && !trimmed.contains("rm -fr") {
        return false;
    }
    // Check for variable in the rm path ($$var or $(VAR))
    // Only flag if the variable isn't guarded with :? or similar
    let has_var = trimmed.contains("$$") || trimmed.contains("$(");
    let has_guard = trimmed.contains(":?") || trimmed.contains("|| true");
    has_var && !has_guard
}

fn collect_phony_targets(line: &str, phonys: &mut Vec<String>) {
    // .PHONY: target1 target2 target3
    if let Some(targets) = line.strip_prefix(".PHONY:") {
        for target in targets.split_whitespace() {
            phonys.push(target.to_string());
        }
    } else if let Some(targets) = line.strip_prefix(".PHONY :") {
        for target in targets.split_whitespace() {
            phonys.push(target.to_string());
        }
    }
}

/// Check if a line is a target definition (not a variable assignment)
fn is_target_definition(line: &str) -> bool {
    // Target definitions start at column 0, contain `:`, are not variable assignments (=)
    if line.starts_with('\t') || line.starts_with(' ') || line.starts_with('#') {
        return false;
    }
    line.contains(':') && !line.contains('=') && !line.starts_with('.')
}

fn extract_target_name(line: &str) -> Option<String> {
    line.split(':').next().map(|s| s.trim().to_string())
}

/// Common targets that should always have .PHONY
const COMMON_PHONY_TARGETS: &[&str] = &[
    "all", "clean", "test", "build", "install", "check", "lint", "format", "help", "dist",
    "release", "deploy", "coverage", "bench",
];

fn check_missing_phony(
    phonys: &[String],
    targets: &[(String, usize)],
    violations: &mut Vec<Violation>,
) {
    for (name, line) in targets {
        if COMMON_PHONY_TARGETS.contains(&name.as_str()) && !phonys.contains(name) {
            violations.push(Violation {
                rule: RuleId::MakefileSafety,
                line: Some(*line),
                message: format!("MAKE004: Target '{}' should be declared .PHONY", name),
            });
        }
    }
}

fn check_dockerfile_best(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut has_user = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("USER ") {
            has_user = true;
        }

        check_dockerfile_line(trimmed, i + 1, &mut violations);
    }

    if !has_user && !content.is_empty() {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: None,
            message: "DOCKER010: Missing USER directive (runs as root)".to_string(),
        });
    }

    RuleResult {
        rule: RuleId::DockerfileBest,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_dockerfile_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // DOCKER008: ADD instead of COPY for local files
    if trimmed.starts_with("ADD ") && !trimmed.contains("http://") && !trimmed.contains("https://")
    {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER008: Use COPY instead of ADD for local files".to_string(),
        });
    }

    // DOCKER001: Untagged or :latest base image (non-deterministic)
    if is_unpinned_from(trimmed) {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER001: Pin base image version (avoid untagged or :latest)".to_string(),
        });
    }

    // DOCKER003: apt-get install without cleanup
    if is_apt_without_clean(trimmed) {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER003: apt-get install without cleanup (add rm -rf /var/lib/apt/lists/*)"
                .to_string(),
        });
    }

    // DOCKER004: EXPOSE with no port number
    if trimmed == "EXPOSE" {
        violations.push(Violation {
            rule: RuleId::DockerfileBest,
            line: Some(line_num),
            message: "DOCKER004: EXPOSE requires a port number".to_string(),
        });
    }
}

/// Detect FROM without a pinned tag (FROM image or FROM image:latest)
fn is_unpinned_from(trimmed: &str) -> bool {
    if !trimmed.starts_with("FROM ") {
        return false;
    }
    let image = trimmed[5..].split_whitespace().next().unwrap_or("");
    // Skip scratch (no tag needed) and $ARG references
    if image == "scratch" || image.starts_with('$') {
        return false;
    }
    // Check for tag: image:tag (but not image:latest)
    if let Some(tag) = image.split(':').nth(1) {
        tag == "latest"
    } else {
        // No tag at all — unpinned
        !image.contains('@') // @sha256: is pinned by digest
    }
}

/// Detect apt-get install without cleanup in the same RUN layer
fn is_apt_without_clean(trimmed: &str) -> bool {
    if !trimmed.contains("apt-get install") {
        return false;
    }
    // Check if cleanup is in the same RUN command
    !trimmed.contains("rm -rf /var/lib/apt")
        && !trimmed.contains("apt-get clean")
        && !trimmed.contains("apt-get autoremove")
}

fn check_config_hygiene(content: &str) -> RuleResult {
    let mut violations = Vec::new();
    let mut path_exports = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        // Detect PATH modifications
        if trimmed.contains("export PATH=") || trimmed.contains("PATH=") {
            path_exports.push(i + 1);
        }
    }

    // Multiple PATH exports suggest potential duplication
    if path_exports.len() > 3 {
        violations.push(Violation {
            rule: RuleId::ConfigHygiene,
            line: Some(path_exports[0]),
            message: format!(
                "PATH modified {} times (potential duplication)",
                path_exports.len()
            ),
        });
    }

    RuleResult {
        rule: RuleId::ConfigHygiene,
        passed: violations.is_empty(),
        violations,
    }
}
