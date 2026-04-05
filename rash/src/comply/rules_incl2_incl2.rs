fn is_unquoted_var_expansion(chars: &[char], j: usize, trimmed: &str) -> bool {
    j + 1 < chars.len()
        && chars[j + 1].is_alphabetic()
        && chars[j + 1] != '('
        && chars[j + 1] != '{'
        && !trimmed[..j].ends_with("((")
}

fn check_posix_patterns(content: &str) -> RuleResult {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
            continue;
        }
        check_posix_line(trimmed, i, &mut violations);
    }

    RuleResult {
        rule: RuleId::Posix,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_posix_line(trimmed: &str, i: usize, violations: &mut Vec<Violation>) {
    // Bash-specific shebang (only on first line)
    if i == 0 && trimmed.starts_with("#!/bin/bash") {
        violations.push(Violation {
            rule: RuleId::Posix,
            line: Some(1),
            message: "Non-POSIX shebang: use #!/bin/sh".to_string(),
        });
    }

    // Table-driven POSIX violation checks (all use line i+1)
    let posix_checks: ComplianceCheck<'_> = &[
        (
            &|t| t.contains("declare -a") || t.contains("declare -A"),
            "Bash-specific: declare -a/-A (arrays)",
        ),
        (
            &|t| t.contains("[[") && t.contains("]]"),
            "Bash-specific: [[ ]] (use [ ] for POSIX)",
        ),
        (
            &|t| is_function_keyword(t),
            "Bash-specific: function keyword (use name() for POSIX)",
        ),
        (
            &|t| is_standalone_arithmetic(t),
            "Bash-specific: (( )) arithmetic (use $(( )) or [ ] for POSIX)",
        ),
        (
            &|t| t.contains("<<<"),
            "Bash-specific: <<< here-string (use echo | or heredoc for POSIX)",
        ),
        (
            &|t| is_select_statement(t),
            "Bash-specific: select statement",
        ),
        (
            &|t| has_bash_parameter_expansion(t),
            "Bash-specific: ${var/...} pattern substitution",
        ),
        (
            &|t| has_bash_case_modification(t),
            "Bash-specific: ${var,,} or ${var^^} case modification",
        ),
        (
            &|t| t.contains("set -o pipefail") || t.contains("set -euo pipefail"),
            "Bash-specific: set -o pipefail (not in POSIX)",
        ),
        (
            &|t| is_bash_redirect(t),
            "Bash-specific: &> redirect (use >file 2>&1 for POSIX)",
        ),
    ];

    for (check_fn, message) in posix_checks {
        if check_fn(trimmed) {
            violations.push(Violation {
                rule: RuleId::Posix,
                line: Some(i + 1),
                message: message.to_string(),
            });
        }
    }
}

/// Detect `function name` syntax (bash-specific, POSIX uses `name()`)
fn is_function_keyword(trimmed: &str) -> bool {
    if let Some(rest) = trimmed.strip_prefix("function ") {
        // Must be followed by a function name (word chars)
        let rest = rest.trim_start();
        return rest
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_');
    }
    false
}

/// Detect standalone (( )) arithmetic (not $(( )) which is POSIX)
fn is_standalone_arithmetic(trimmed: &str) -> bool {
    // Standalone (( starts at beginning or after ; && ||
    if trimmed.starts_with("((") {
        return true;
    }
    for sep in &["; ", "&& ", "|| "] {
        if let Some(pos) = trimmed.find(sep) {
            let after = trimmed[pos + sep.len()..].trim_start();
            if after.starts_with("((") {
                return true;
            }
        }
    }
    false
}

/// Detect `select` statement (bash-specific)
fn is_select_statement(trimmed: &str) -> bool {
    trimmed.starts_with("select ") && trimmed.contains(" in ")
}

/// Detect ${var/pattern/repl} or ${var//pattern/repl} pattern substitution
/// Must not flag POSIX expansions like ${var:-/path}, ${var#*/}, ${var%/*}
fn has_bash_parameter_expansion(trimmed: &str) -> bool {
    for (start, _) in brace_expansion_starts(trimmed) {
        if is_pattern_substitution(trimmed.as_bytes(), start) {
            return true;
        }
    }
    false
}

/// Check if a ${...} starting at `start` (after `${`) is a pattern substitution
fn is_pattern_substitution(bytes: &[u8], start: usize) -> bool {
    let mut j = start;
    // Skip past variable name (alphanumeric + underscore)
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    // Pattern substitution: first non-varname char is /
    j < bytes.len() && bytes[j] == b'/'
}

/// Detect ${var,,} ${var^^} case modification (bash 4.0+)
fn has_bash_case_modification(trimmed: &str) -> bool {
    for (start, _) in brace_expansion_starts(trimmed) {
        if is_case_modification(trimmed.as_bytes(), start) {
            return true;
        }
    }
    false
}

/// Check if a ${...} starting at `start` (after `${`) contains ,, or ^^
fn is_case_modification(bytes: &[u8], start: usize) -> bool {
    let mut j = start;
    while j + 1 < bytes.len() && bytes[j] != b'}' {
        if is_case_mod_operator(bytes[j], bytes[j + 1]) {
            return true;
        }
        j += 1;
    }
    false
}

fn is_case_mod_operator(a: u8, b: u8) -> bool {
    (a == b',' && b == b',') || (a == b'^' && b == b'^')
}

/// Yield (content_start, brace_pos) for each `${` in the string
fn brace_expansion_starts(s: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let bytes = s.as_bytes();
    (0..bytes.len().saturating_sub(1)).filter_map(move |i| {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            Some((i + 2, i))
        } else {
            None
        }
    })
}

/// Detect &> redirect (bash-specific combined stdout+stderr redirect)
fn is_bash_redirect(trimmed: &str) -> bool {
    // &> or &>> (but not >&2 which is POSIX fd redirect)
    if let Some(pos) = trimmed.find("&>") {
        // Make sure it's not >&N (fd redirect) — check preceding char
        if pos > 0 && trimmed.as_bytes()[pos - 1] == b'>' {
            // This is >&... which could be POSIX >&2
            return false;
        }
        return true;
    }
    false
}

fn check_shellcheck_patterns(content: &str) -> RuleResult {
    // Lightweight shellcheck-equivalent patterns
    // Full shellcheck integration deferred to Phase 2
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        check_shellcheck_line(trimmed, i + 1, &mut violations);
    }

    RuleResult {
        rule: RuleId::ShellCheck,
        passed: violations.is_empty(),
        violations,
    }
}

fn check_shellcheck_line(trimmed: &str, line_num: usize, violations: &mut Vec<Violation>) {
    // SC2006: Use $(...) instead of backticks
    if trimmed.contains('`') && !trimmed.contains("\\`") {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2006: Use $(...) instead of backticks".to_string(),
        });
    }

    // SC2115: Use "${var:?}" to fail if empty
    if is_dangerous_rm_rf(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2115: Dangerous rm -rf with variable path".to_string(),
        });
    }

    // SC2164: cd without || exit (silent failure)
    if is_bare_cd(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2164: Use cd ... || exit in case cd fails".to_string(),
        });
    }

    // SC2162: read without -r (mangles backslashes)
    if is_read_without_r(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2162: read without -r mangles backslashes".to_string(),
        });
    }

    // SC2181: if [ $? ... ] instead of direct command check
    if is_dollar_question_check(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2181: Check exit code directly with if cmd, not $?".to_string(),
        });
    }

    // SC2012: for f in $(ls ...) — use glob instead
    if is_ls_iteration(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2012: Use glob instead of ls to iterate files".to_string(),
        });
    }

    // SC2035: Use ./* glob to avoid filenames starting with -
    if is_bare_glob(trimmed) {
        violations.push(Violation {
            rule: RuleId::ShellCheck,
            line: Some(line_num),
            message: "SC2035: Use ./* instead of * to avoid - filename issues".to_string(),
        });
    }
}

fn is_dangerous_rm_rf(trimmed: &str) -> bool {
    (trimmed.starts_with("rm -rf /") || trimmed.starts_with("rm -rf \"/$"))
        && trimmed.contains('$')
        && !trimmed.contains(":?")
}

/// SC2164: cd without error handling (|| exit, || return, || die)
fn is_bare_cd(trimmed: &str) -> bool {
    if !trimmed.starts_with("cd ") {
        return false;
    }
    // Allow: cd ... || exit, cd ... || return, cd ... && ..., cd alone (cd ~)
    let has_error_handling = trimmed.contains("|| exit")
        || trimmed.contains("|| return")
        || trimmed.contains("|| die")
        || trimmed.contains("|| {");
    // cd to home (just "cd" or "cd ~") is always safe
    let is_safe_cd = trimmed == "cd" || trimmed == "cd ~";
    !has_error_handling && !is_safe_cd
}

/// SC2162: read without -r flag
fn is_read_without_r(trimmed: &str) -> bool {
    if !trimmed.starts_with("read ") && !trimmed.contains("| read ") {
        return false;
    }
    // Extract the read command portion
    let read_part = if let Some(pos) = trimmed.find("| read ") {
        &trimmed[pos + 2..]
    } else {
        trimmed
    };
    // Check if -r is present (as a flag, not part of a variable name)
    !read_part
        .split_whitespace()
        .any(|w| w == "-r" || w.starts_with("-r") && w.len() > 2 && w.as_bytes()[2] != b' ')
}

/// SC2181: if [ $? ... ] pattern
fn is_dollar_question_check(trimmed: &str) -> bool {
    trimmed.contains("$?") && (trimmed.contains("[ $?") || trimmed.contains("[$?"))
}

/// SC2012: for f in $(ls ...) iteration
fn is_ls_iteration(trimmed: &str) -> bool {
    trimmed.contains("$(ls") || trimmed.contains("`ls ")
}

/// SC2035: Bare * glob in command arguments (use ./* instead)
fn is_bare_glob(trimmed: &str) -> bool {
    // Only flag: for f in *; (bare * as sole glob, not *.txt or ./*)
    if trimmed.starts_with("for ") {
        if let Some(pos) = trimmed.find(" in ") {
            let after_in = trimmed[pos + 4..].trim_start();
            // Bare * followed by ; or end-of-line
            return after_in.starts_with("*;") || after_in.starts_with("* ;") || after_in == "*";
        }
    }
    false
}

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
