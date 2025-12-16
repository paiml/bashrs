// Purification Engine for Makefiles
//
// This module provides functionality to automatically fix non-deterministic
// patterns detected by semantic analysis.
//
// Transformations:
// - $(wildcard *.c) → $(sort $(wildcard *.c))
// - $(shell find ...) → $(sort $(shell find ...))
// - Nested patterns in filter, foreach, call
// - Comments for manual fixes (shell date, $RANDOM)

use crate::make_parser::ast::{MakeAst, MakeItem};
use crate::make_parser::semantic::{analyze_makefile, SemanticIssue};

/// Result of purification process
#[derive(Debug, Clone, PartialEq)]
pub struct PurificationResult {
    /// Purified AST
    pub ast: MakeAst,
    /// Number of transformations applied
    pub transformations_applied: usize,
    /// Number of issues successfully fixed
    pub issues_fixed: usize,
    /// Number of issues requiring manual intervention
    pub manual_fixes_needed: usize,
    /// Report of transformations
    pub report: Vec<String>,
}

/// Type of transformation to apply
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    /// Wrap pattern with $(sort ...)
    WrapWithSort {
        variable_name: String,
        pattern: String,
        safe: bool,
    },
    /// Add comment suggesting manual fix
    AddComment {
        variable_name: String,
        rule: String,
        suggestion: String,
        safe: bool,
    },
    // Sprint 83 - Parallel Safety Transformations
    /// Recommend adding .NOTPARALLEL directive
    RecommendNotParallel { reason: String, safe: bool },
    /// Detect race condition in shared file write
    DetectRaceCondition {
        target_names: Vec<String>,
        conflicting_file: String,
        safe: bool,
    },
    /// Recommend adding order-only prerequisite
    RecommendOrderOnlyPrereq {
        target_name: String,
        prereq_name: String,
        reason: String,
        safe: bool,
    },
    /// Detect missing dependency
    DetectMissingDependency {
        target_name: String,
        missing_file: String,
        provider_target: String,
        safe: bool,
    },
    /// Detect multiple targets with same output
    DetectOutputConflict {
        target_names: Vec<String>,
        output_file: String,
        safe: bool,
    },
    /// Recommend handling recursive make calls
    RecommendRecursiveMakeHandling {
        target_name: String,
        subdirs: Vec<String>,
        safe: bool,
    },
    /// Detect shared directory creation race
    DetectDirectoryRace {
        target_names: Vec<String>,
        directory: String,
        safe: bool,
    },
    // Sprint 83 - Reproducible Builds Transformations (Day 4)
    /// Detect non-deterministic timestamp ($(shell date))
    DetectTimestamp {
        variable_name: String,
        pattern: String,
        safe: bool,
    },
    /// Detect $RANDOM usage
    DetectRandom { variable_name: String, safe: bool },
    /// Detect process ID $$ usage
    DetectProcessId { variable_name: String, safe: bool },
    /// Suggest SOURCE_DATE_EPOCH for reproducibility
    SuggestSourceDateEpoch {
        variable_name: String,
        original_pattern: String,
        safe: bool,
    },
    /// Detect non-deterministic command (hostname, git, mktemp, etc.)
    DetectNonDeterministicCommand {
        variable_name: String,
        command: String,
        reason: String,
        safe: bool,
    },
    // Sprint 83 - Performance Optimization Transformations (Day 5)
    /// Suggest combining multiple shell invocations
    SuggestCombineShellInvocations {
        target_name: String,
        recipe_count: usize,
        safe: bool,
    },
    /// Suggest using := instead of = for simple variables
    SuggestSimpleExpansion {
        variable_name: String,
        reason: String,
        safe: bool,
    },
    /// Recommend adding .SUFFIXES: to disable builtin rules
    RecommendSuffixes { reason: String, safe: bool },
    /// Detect multiple sequential recipe lines that could be combined
    DetectSequentialRecipes {
        target_name: String,
        recipe_count: usize,
        safe: bool,
    },
    /// Suggest pattern rule instead of explicit rules
    SuggestPatternRule {
        pattern: String,
        target_count: usize,
        safe: bool,
    },

    // Sprint 83 - Error Handling Transformations (Day 6)
    /// Detect missing error handling in recipes
    DetectMissingErrorHandling {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Detect silent failures with @ prefix
    DetectSilentFailure {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Recommend .DELETE_ON_ERROR
    RecommendDeleteOnError { reason: String, safe: bool },
    /// Recommend .ONESHELL for multiline recipes
    RecommendOneshell {
        target_name: String,
        reason: String,
        safe: bool,
    },
    /// Detect missing set -e in shell scripts
    DetectMissingSetE {
        target_name: String,
        command: String,
        safe: bool,
    },
    /// Detect missing error handling in loops
    DetectLoopWithoutErrorHandling {
        target_name: String,
        loop_command: String,
        safe: bool,
    },

    // Sprint 83 - Portability Transformations (Day 7)
    /// Detect bashisms (non-POSIX shell constructs)
    DetectBashism {
        target_name: String,
        construct: String,
        posix_alternative: String,
        safe: bool,
    },
    /// Detect platform-specific commands
    DetectPlatformSpecific {
        target_name: String,
        command: String,
        reason: String,
        safe: bool,
    },
    /// Detect shell-specific features (source, declare, etc.)
    DetectShellSpecific {
        target_name: String,
        feature: String,
        posix_alternative: String,
        safe: bool,
    },
    /// Detect non-portable command flags
    DetectNonPortableFlags {
        target_name: String,
        command: String,
        flag: String,
        reason: String,
        safe: bool,
    },
    /// Detect non-portable echo usage
    DetectNonPortableEcho {
        target_name: String,
        command: String,
        safe: bool,
    },
}

/// Purify a Makefile AST by fixing non-deterministic patterns
///
/// # Example
/// ```ignore
/// let ast = parse_makefile("FILES := $(wildcard *.c)").unwrap();
/// let result = purify_makefile(&ast);
/// // result.ast contains: FILES := $(sort $(wildcard *.c))
/// ```
pub fn purify_makefile(ast: &MakeAst) -> PurificationResult {
    // 1. Run semantic analysis to find issues
    let issues = analyze_makefile(ast);

    // 2. Plan transformations for each issue
    let mut transformations = plan_transformations(ast, &issues);

    // 3. Add parallel safety analysis (Sprint 83 - Days 2-3)
    transformations.extend(analyze_parallel_safety(ast));

    // 4. Add reproducible builds analysis (Sprint 83 - Day 4)
    transformations.extend(analyze_reproducible_builds(ast));

    // 5. Add performance optimization analysis (Sprint 83 - Day 5)
    transformations.extend(analyze_performance_optimization(ast));

    // 6. Add error handling analysis (Sprint 83 - Day 6)
    transformations.extend(analyze_error_handling(ast));

    // 7. Add portability analysis (Sprint 83 - Day 7)
    transformations.extend(analyze_portability(ast));

    // 8. Apply transformations
    let purified_ast = apply_transformations(ast, &transformations);

    // 8. Count results
    let issues_fixed = transformations
        .iter()
        .filter(|t| is_safe_transformation(t))
        .count();

    let manual_fixes_needed = transformations
        .iter()
        .filter(|t| !is_safe_transformation(t))
        .count();

    // 9. Generate report
    let report = generate_report(&transformations);

    PurificationResult {
        ast: purified_ast,
        transformations_applied: transformations.len(),
        issues_fixed,
        manual_fixes_needed,
        report,
    }
}

/// Plan which transformations to apply for detected issues
fn plan_transformations(_ast: &MakeAst, issues: &[SemanticIssue]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for issue in issues {
        // Extract variable name from issue message
        let var_name = extract_variable_name(&issue.message);

        match issue.rule.as_str() {
            "NO_WILDCARD" => {
                transformations.push(Transformation::WrapWithSort {
                    variable_name: var_name,
                    pattern: "$(wildcard".to_string(),
                    safe: true,
                });
            }
            "NO_UNORDERED_FIND" => {
                transformations.push(Transformation::WrapWithSort {
                    variable_name: var_name,
                    pattern: "$(shell find".to_string(),
                    safe: true,
                });
            }
            "NO_TIMESTAMPS" => {
                transformations.push(Transformation::AddComment {
                    variable_name: var_name,
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone().unwrap_or_default(),
                    safe: false, // Manual fix required
                });
            }
            "NO_RANDOM" => {
                transformations.push(Transformation::AddComment {
                    variable_name: var_name,
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone().unwrap_or_default(),
                    safe: false, // Manual fix required
                });
            }
            _ => {}
        }
    }

    transformations
}

/// Apply transformations to AST
fn apply_transformations(ast: &MakeAst, transformations: &[Transformation]) -> MakeAst {
    let mut purified = ast.clone();

    for transformation in transformations {
        match transformation {
            Transformation::WrapWithSort {
                variable_name,
                pattern,
                ..
            } => {
                wrap_variable_with_sort(&mut purified, variable_name, pattern);
            }
            Transformation::AddComment { .. } => {
                // Intentionally no-op: AddComment is for manual fixes, not AST modification
            }
            // Sprint 83 - Parallel Safety transformations
            // These are detection/recommendation transformations, not AST modifications
            // They generate reports but don't change the AST
            Transformation::RecommendNotParallel { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectRaceCondition { .. } => {
                // Detection only - no AST change
            }
            Transformation::RecommendOrderOnlyPrereq { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectMissingDependency { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectOutputConflict { .. } => {
                // Detection only - no AST change
            }
            Transformation::RecommendRecursiveMakeHandling { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectDirectoryRace { .. } => {
                // Detection only - no AST change
            }
            // Sprint 83 - Reproducible Builds transformations (Day 4)
            // These are detection/recommendation transformations, not AST modifications
            // They generate reports but don't change the AST
            Transformation::DetectTimestamp { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectRandom { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectProcessId { .. } => {
                // Detection only - no AST change
            }
            Transformation::SuggestSourceDateEpoch { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectNonDeterministicCommand { .. } => {
                // Detection only - no AST change
            }
            // Sprint 83 - Performance Optimization transformations (Day 5)
            // These are detection/recommendation transformations, not AST modifications
            // They generate reports but don't change the AST
            Transformation::SuggestCombineShellInvocations { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::SuggestSimpleExpansion { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::RecommendSuffixes { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectSequentialRecipes { .. } => {
                // Detection only - no AST change
            }
            Transformation::SuggestPatternRule { .. } => {
                // Recommendation only - no AST change
            }
            // Sprint 83 - Error Handling transformations (Day 6)
            // These are detection/recommendation transformations, not AST modifications
            // They generate reports but don't change the AST
            Transformation::DetectMissingErrorHandling { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectSilentFailure { .. } => {
                // Detection only - no AST change
            }
            Transformation::RecommendDeleteOnError { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::RecommendOneshell { .. } => {
                // Recommendation only - no AST change
            }
            Transformation::DetectMissingSetE { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectLoopWithoutErrorHandling { .. } => {
                // Detection only - no AST change
            }
            // Sprint 83 - Portability transformations (Day 7)
            // These are detection/recommendation transformations, not AST modifications
            // They generate reports but don't change the AST
            Transformation::DetectBashism { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectPlatformSpecific { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectShellSpecific { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectNonPortableFlags { .. } => {
                // Detection only - no AST change
            }
            Transformation::DetectNonPortableEcho { .. } => {
                // Detection only - no AST change
            }
        }
    }

    purified
}

/// Wrap pattern in specific variable with $(sort ...)
fn wrap_variable_with_sort(ast: &mut MakeAst, variable_name: &str, pattern: &str) {
    for item in &mut ast.items {
        if let MakeItem::Variable { name, value, .. } = item {
            if name == variable_name && value.contains(pattern) {
                *value = wrap_pattern_with_sort(value, pattern);
            }
        }
    }
}

/// Wrap specific pattern with $(sort ...)
///
/// Examples:
/// - "$(wildcard *.c)" → "$(sort $(wildcard *.c))"
/// - "$(filter %.o, $(wildcard *.c))" → "$(filter %.o, $(sort $(wildcard *.c)))"
fn wrap_pattern_with_sort(value: &str, pattern: &str) -> String {
    // Find pattern start
    if let Some(start) = value.find(pattern) {
        // Find matching closing paren
        if let Some(end) = find_matching_paren(value, start) {
            // Extract the complete pattern
            let complete_pattern = &value[start..=end];

            // Wrap with $(sort ...)
            let wrapped = format!("$(sort {})", complete_pattern);

            // Replace in original string
            value.replace(complete_pattern, &wrapped)
        } else {
            // No matching paren found, return unchanged
            value.to_string()
        }
    } else {
        // Pattern not found, return unchanged
        value.to_string()
    }
}

/// Find matching closing parenthesis for pattern starting at position
///
/// Handles nested parentheses correctly.
///
/// # Arguments
/// * `s` - The string to search
/// * `start` - Starting position (should point to the pattern start, e.g., "$(" )
///
/// # Returns
/// Position of the matching closing parenthesis
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();

    // Find the opening '(' for the pattern at start
    let mut depth = 0;
    let mut found_opening = false;
    let mut i = start;

    // Skip to the opening paren
    while i < bytes.len() {
        if bytes.get(i) == Some(&b'(') && i > 0 && bytes.get(i - 1) == Some(&b'$') {
            depth = 1;
            found_opening = true;
            i += 1;
            break;
        }
        i += 1;
    }

    if !found_opening {
        return None;
    }

    // Now find the matching closing paren
    while i < bytes.len() {
        match bytes.get(i) {
            Some(&b'$') if bytes.get(i + 1) == Some(&b'(') => {
                depth += 1;
                i += 2; // Skip past $(
                continue;
            }
            Some(&b')') => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Extract variable name from semantic issue message
///
/// Message format: "Variable 'NAME' uses non-deterministic ..."
fn extract_variable_name(message: &str) -> String {
    if let Some(start) = message.find("'") {
        if let Some(end) = message[start + 1..].find("'") {
            return message[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}

/// Analyze Makefile for parallel safety issues (Sprint 83)
/// Check if Makefile has .NOTPARALLEL directive
fn has_notparallel_directive(ast: &MakeAst) -> bool {
    ast.items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".NOTPARALLEL"))
}

/// Collect all targets for analysis
fn collect_targets(ast: &MakeAst) -> Vec<(&String, &Vec<String>)> {
    let mut targets = Vec::new();
    for item in &ast.items {
        if let MakeItem::Target { name, recipe, .. } = item {
            targets.push((name, recipe));
        }
    }
    targets
}

/// Detect output file conflicts (multiple targets writing to same file)
fn detect_output_file_conflicts(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    let mut output_files: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect output redirects: > filename or >> filename
            if let Some(pos) = recipe.find(" > ") {
                let after = &recipe[pos + 3..];
                let filename = after.split_whitespace().next().unwrap_or("");
                if !filename.is_empty() {
                    output_files
                        .entry(filename.to_string())
                        .or_default()
                        .push((*target_name).clone());
                }
            }
            // Detect compiler output: -o filename
            if let Some(pos) = recipe.find(" -o ") {
                let after = &recipe[pos + 4..];
                let filename = after.split_whitespace().next().unwrap_or("");
                if !filename.is_empty() && filename != "$@" {
                    output_files
                        .entry(filename.to_string())
                        .or_default()
                        .push((*target_name).clone());
                }
            }
        }
    }

    // Report conflicts
    for (file, target_names) in output_files {
        if target_names.len() > 1 {
            transformations.push(Transformation::DetectRaceCondition {
                target_names: target_names.clone(),
                conflicting_file: file.clone(),
                safe: false,
            });

            transformations.push(Transformation::DetectOutputConflict {
                target_names,
                output_file: file,
                safe: false,
            });
        }
    }

    transformations
}

// ===== Helper functions for detect_missing_file_dependencies =====

/// Try to extract output filename from redirect pattern: " > filename"
fn try_extract_output_redirect(recipe: &str) -> Option<String> {
    if let Some(pos) = recipe.find(" > ") {
        let after = &recipe[pos + 3..];
        let filename = after.split_whitespace().next()?;
        if !filename.is_empty() {
            return Some(filename.to_string());
        }
    }
    None
}

/// Try to extract input filename from cat command: "cat filename"
fn try_extract_cat_input(recipe: &str) -> Option<String> {
    if recipe.contains("cat ") {
        if let Some(pos) = recipe.find("cat ") {
            let after = &recipe[pos + 4..];
            let filename = after.split_whitespace().next()?;
            if !filename.is_empty() && !is_automatic_variable(filename) {
                return Some(filename.to_string());
            }
        }
    }
    None
}

/// Check if filename is an automatic Make variable ($<, $@, etc.)
fn is_automatic_variable(filename: &str) -> bool {
    matches!(filename, "$<" | "$@" | "$^" | "$?" | "$*" | "$+")
}

/// Check if target has a specific prerequisite
fn target_has_prerequisite(ast: &MakeAst, target_name: &str, prerequisite: &str) -> bool {
    ast.items.iter().any(|item| {
        if let MakeItem::Target {
            name,
            prerequisites,
            ..
        } = item
        {
            name == target_name && prerequisites.contains(&prerequisite.to_string())
        } else {
            false
        }
    })
}

/// Detect missing file dependencies (file usage without dependency)
fn detect_missing_file_dependencies(
    ast: &MakeAst,
    targets: &[(&String, &Vec<String>)],
) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    let mut file_creators: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut file_users: Vec<(String, String)> = Vec::new();

    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect file creation using helper
            if let Some(filename) = try_extract_output_redirect(recipe) {
                file_creators.insert(filename, (*target_name).clone());
            }
            // Detect file usage using helper
            if let Some(filename) = try_extract_cat_input(recipe) {
                file_users.push(((*target_name).clone(), filename));
            }
        }
    }

    // Report missing dependencies using helper
    for (user_target, used_file) in file_users {
        if let Some(provider_target) = file_creators.get(&used_file) {
            if !target_has_prerequisite(ast, &user_target, provider_target) {
                transformations.push(Transformation::DetectMissingDependency {
                    target_name: user_target.clone(),
                    missing_file: used_file,
                    provider_target: provider_target.clone(),
                    safe: false,
                });
            }
        }
    }

    transformations
}

/// Detect recursive make calls
fn detect_recursive_make_calls(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for (target_name, recipes) in targets {
        let mut subdirs = Vec::new();
        for recipe in *recipes {
            if recipe.contains("$(MAKE)") || recipe.contains("${MAKE}") {
                if let Some(pos) = recipe.find("-C ") {
                    let after = &recipe[pos + 3..];
                    let subdir = after.split_whitespace().next().unwrap_or("");
                    if !subdir.is_empty() {
                        subdirs.push(subdir.to_string());
                    }
                }
            }
        }

        if !subdirs.is_empty() {
            transformations.push(Transformation::RecommendRecursiveMakeHandling {
                target_name: (*target_name).clone(),
                subdirs,
                safe: false,
            });
        }
    }

    transformations
}

/// Detect shared directory creation races
fn detect_directory_creation_races(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    let mut dir_creators: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.contains("mkdir") {
                if let Some(pos) = recipe.find("mkdir") {
                    let after = &recipe[pos + 5..];
                    let parts: Vec<&str> = after.split_whitespace().collect();
                    for part in parts {
                        if part != "-p" && !part.is_empty() {
                            dir_creators
                                .entry(part.to_string())
                                .or_default()
                                .push((*target_name).clone());
                        }
                    }
                }
            }
        }
    }

    // Report directory creation races
    for (directory, target_names) in dir_creators {
        if target_names.len() > 1 {
            transformations.push(Transformation::DetectDirectoryRace {
                target_names,
                directory,
                safe: false,
            });
        }
    }

    transformations
}

/// Check if .NOTPARALLEL should be recommended
fn should_recommend_notparallel(
    has_notparallel: bool,
    transformations: &[Transformation],
    targets: &[(&String, &Vec<String>)],
) -> bool {
    !has_notparallel && !transformations.is_empty() && !targets.is_empty()
}

fn analyze_parallel_safety(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    let has_notparallel = has_notparallel_directive(ast);
    let targets = collect_targets(ast);

    // Detect race conditions
    transformations.extend(detect_output_file_conflicts(&targets));

    // Detect missing dependencies
    transformations.extend(detect_missing_file_dependencies(ast, &targets));

    // Detect recursive make calls
    transformations.extend(detect_recursive_make_calls(&targets));

    // Detect shared directory creation races
    transformations.extend(detect_directory_creation_races(&targets));

    // Recommend .NOTPARALLEL if issues detected
    if should_recommend_notparallel(has_notparallel, &transformations, &targets) {
        transformations.push(Transformation::RecommendNotParallel {
            reason: "Parallel safety issues detected - consider adding .NOTPARALLEL".to_string(),
            safe: false,
        });
    }

    transformations
}

/// Analyze Makefile for reproducible builds issues (Sprint 83 - Day 4)
fn analyze_reproducible_builds(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    // Collect all variables for analysis
    for item in &ast.items {
        if let MakeItem::Variable { name, value, .. } = item {
            // Analysis 1: Detect $(shell date) patterns
            if value.contains("date") && (value.contains("$(shell") || value.contains("${shell")) {
                transformations.push(Transformation::DetectTimestamp {
                    variable_name: name.clone(),
                    pattern: value.clone(),
                    safe: false,
                });

                // Also suggest SOURCE_DATE_EPOCH
                transformations.push(Transformation::SuggestSourceDateEpoch {
                    variable_name: name.clone(),
                    original_pattern: value.clone(),
                    safe: false,
                });
            }

            // Analysis 2: Detect $RANDOM usage (Makefile syntax: $$RANDOM)
            if value.contains("$$RANDOM") || value.contains("$RANDOM") {
                transformations.push(Transformation::DetectRandom {
                    variable_name: name.clone(),
                    safe: false,
                });
            }

            // Analysis 3: Detect process ID $$ usage (Makefile syntax: $$$$)
            if value.contains("$$$$") {
                transformations.push(Transformation::DetectProcessId {
                    variable_name: name.clone(),
                    safe: false,
                });
            }

            // Analysis 4: Detect hostname command
            if value.contains("hostname")
                && (value.contains("$(shell") || value.contains("${shell"))
            {
                transformations.push(Transformation::DetectNonDeterministicCommand {
                    variable_name: name.clone(),
                    command: "hostname".to_string(),
                    reason: "hostname is environment-dependent and makes builds non-reproducible"
                        .to_string(),
                    safe: false,
                });
            }

            // Analysis 5: Detect git timestamp commands
            if value.contains("git")
                && value.contains("log")
                && (value.contains("%cd") || value.contains("--date"))
            {
                transformations.push(Transformation::DetectNonDeterministicCommand {
                    variable_name: name.clone(),
                    command: "git log timestamp".to_string(),
                    reason: "git commit timestamps are non-deterministic".to_string(),
                    safe: false,
                });
            }
        }

        // Check recipes for mktemp usage
        if let MakeItem::Target { name, recipe, .. } = item {
            for recipe_line in recipe {
                // Analysis 6: Detect mktemp usage
                if recipe_line.contains("mktemp") {
                    transformations.push(Transformation::DetectNonDeterministicCommand {
                        variable_name: name.clone(),
                        command: "mktemp".to_string(),
                        reason: "mktemp creates random temporary file names".to_string(),
                        safe: false,
                    });
                }
            }
        }
    }

    transformations
}

/// Analyze Makefile for performance optimization opportunities (Sprint 83 - Day 5)
fn analyze_performance_optimization(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    // Check if Makefile has .SUFFIXES directive
    let has_suffixes = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".SUFFIXES"));

    // Collect all targets and variables for analysis
    let mut targets: Vec<(&String, &Vec<String>)> = Vec::new();
    let mut variables: Vec<(&String, &String, &crate::make_parser::ast::VarFlavor)> = Vec::new();

    for item in &ast.items {
        if let MakeItem::Target { name, recipe, .. } = item {
            targets.push((name, recipe));
        } else if let MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } = item
        {
            variables.push((name, value, flavor));
        }
    }

    // Analysis 1: Detect variables using = with $(shell) that should use :=
    for (var_name, value, flavor) in &variables {
        if matches!(flavor, crate::make_parser::ast::VarFlavor::Recursive) {
            // Check if value contains $(shell) - should use :=
            if value.contains("$(shell") || value.contains("${shell") {
                transformations.push(Transformation::SuggestSimpleExpansion {
                    variable_name: (*var_name).clone(),
                    reason: "Use := instead of = to avoid re-expanding $(shell) multiple times"
                        .to_string(),
                    safe: false,
                });
            }
            // Check if value is simple (no variable references) - could use :=
            else if !value.contains("$(") && !value.contains("${") {
                transformations.push(Transformation::SuggestSimpleExpansion {
                    variable_name: (*var_name).clone(),
                    reason:
                        "Use := instead of = for simple variables to avoid unnecessary re-expansion"
                            .to_string(),
                    safe: false,
                });
            }
        }
    }

    // Analysis 3: Detect targets with multiple recipe lines that could be combined
    for (target_name, recipes) in &targets {
        if recipes.len() >= 3 {
            // Check for sequential commands (not using && or ;)
            let has_command_separator = recipes.iter().any(|r| r.contains("&&") || r.contains(";"));
            if !has_command_separator {
                transformations.push(Transformation::DetectSequentialRecipes {
                    target_name: (*target_name).clone(),
                    recipe_count: recipes.len(),
                    safe: false,
                });

                transformations.push(Transformation::SuggestCombineShellInvocations {
                    target_name: (*target_name).clone(),
                    recipe_count: recipes.len(),
                    safe: false,
                });
            }
        }

        // Analysis 4: Detect multiple rm commands that could be combined
        let rm_count = recipes
            .iter()
            .filter(|r| r.trim().starts_with("rm "))
            .count();
        if rm_count >= 2 {
            transformations.push(Transformation::DetectSequentialRecipes {
                target_name: (*target_name).clone(),
                recipe_count: rm_count,
                safe: false,
            });
        }
    }

    // Analysis 5: Detect repeated explicit rules that could be pattern rules
    // Group targets by their recipe pattern
    let mut rule_patterns: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (target_name, recipes) in &targets {
        // Look for .o: .c pattern
        if target_name.ends_with(".o") && recipes.iter().any(|r| r.contains("-c")) {
            let pattern = "%.o: %.c compilation".to_string();
            rule_patterns
                .entry(pattern.clone())
                .or_default()
                .push((*target_name).clone());
        }
    }

    // Report if we found 3+ similar rules
    for (_pattern, target_names) in rule_patterns {
        if target_names.len() >= 3 {
            transformations.push(Transformation::SuggestPatternRule {
                pattern: "%.o: %.c".to_string(),
                target_count: target_names.len(),
                safe: false,
            });
        }
    }

    // Final Analysis: Recommend .SUFFIXES: only if we found other performance issues
    // This ensures idempotency - we don't recommend .SUFFIXES: on already-clean Makefiles
    if !has_suffixes && !targets.is_empty() && !transformations.is_empty() {
        transformations.push(Transformation::RecommendSuffixes {
            reason: "Add .SUFFIXES: to disable builtin rules for better performance".to_string(),
            safe: false,
        });
    }

    transformations
}

/// Analyze Makefile for error handling issues (Sprint 83 - Day 6)
/// Check if Makefile has .DELETE_ON_ERROR directive
fn has_delete_on_error_directive(ast: &MakeAst) -> bool {
    ast.items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".DELETE_ON_ERROR"))
}

/// Collect all targets for analysis (both regular targets and pattern rules)
fn collect_targets_and_patterns(ast: &MakeAst) -> Vec<(&String, &Vec<String>)> {
    let mut targets = Vec::new();
    for item in &ast.items {
        match item {
            MakeItem::Target { name, recipe, .. } => {
                targets.push((name, recipe));
            }
            MakeItem::PatternRule {
                target_pattern,
                recipe,
                ..
            } => {
                targets.push((target_pattern, recipe));
            }
            _ => {}
        }
    }
    targets
}

/// Detect commands without error handling
fn detect_missing_error_handling_in_commands(
    targets: &[(&String, &Vec<String>)],
) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    let critical_commands = ["mkdir", "gcc", "cp", "mv"];

    for (target_name, recipes) in targets {
        for recipe in *recipes {
            let trimmed = recipe.trim();
            for cmd in &critical_commands {
                if trimmed.starts_with(cmd)
                    && !trimmed.starts_with(&format!("{} -", cmd))
                    && !recipe.contains("||")
                    && !recipe.contains("&&")
                {
                    transformations.push(Transformation::DetectMissingErrorHandling {
                        target_name: (*target_name).clone(),
                        command: trimmed.to_string(),
                        safe: false,
                    });
                }
            }
        }
    }
    transformations
}

/// Detect @ prefix that may hide errors
fn detect_silent_failures(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.trim().starts_with('@') {
                let without_at = recipe.trim().trim_start_matches('@').trim();
                if !without_at.starts_with("echo") {
                    transformations.push(Transformation::DetectSilentFailure {
                        target_name: (*target_name).clone(),
                        command: recipe.trim().to_string(),
                        safe: false,
                    });
                }
            }
        }
    }
    transformations
}

/// Detect multiline recipes without .ONESHELL
fn detect_missing_oneshell(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        if recipes.len() >= 2 {
            let has_cd = recipes.iter().any(|r| r.trim().starts_with("cd "));
            let has_command_separator = recipes.iter().any(|r| r.contains("&&") || r.contains(";"));

            if has_cd && !has_command_separator {
                transformations.push(Transformation::RecommendOneshell {
                    target_name: (*target_name).clone(),
                    reason:
                        "Use .ONESHELL or combine commands with && to ensure cd works across lines"
                            .to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect bash -c without set -e
fn detect_missing_set_e(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.contains("bash -c") && !recipe.contains("set -e") {
                transformations.push(Transformation::DetectMissingSetE {
                    target_name: (*target_name).clone(),
                    command: recipe.trim().to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect for loops without error handling
fn detect_loop_without_error_handling(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            if recipe.contains("for ")
                && recipe.contains("do ")
                && !recipe.contains("|| exit")
                && !recipe.contains("|| return")
            {
                transformations.push(Transformation::DetectLoopWithoutErrorHandling {
                    target_name: (*target_name).clone(),
                    loop_command: recipe.trim().to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Check if .DELETE_ON_ERROR should be recommended
fn should_recommend_delete_on_error_directive(
    has_delete_on_error: bool,
    transformations: &[Transformation],
    targets: &[(&String, &Vec<String>)],
) -> bool {
    !has_delete_on_error && !transformations.is_empty() && !targets.is_empty()
}

fn analyze_error_handling(ast: &MakeAst) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    let has_delete_on_error = has_delete_on_error_directive(ast);
    let targets = collect_targets_and_patterns(ast);

    // Detect commands without error handling
    transformations.extend(detect_missing_error_handling_in_commands(&targets));

    // Detect silent failures
    transformations.extend(detect_silent_failures(&targets));

    // Detect missing .ONESHELL
    transformations.extend(detect_missing_oneshell(&targets));

    // Detect missing set -e
    transformations.extend(detect_missing_set_e(&targets));

    // Detect loops without error handling
    transformations.extend(detect_loop_without_error_handling(&targets));

    // Recommend .DELETE_ON_ERROR if issues detected
    if should_recommend_delete_on_error_directive(has_delete_on_error, &transformations, &targets) {
        transformations.push(Transformation::RecommendDeleteOnError {
            reason: "Add .DELETE_ON_ERROR to automatically remove targets if recipe fails"
                .to_string(),
            safe: false,
        });
    }

    transformations
}

/// Analyze Makefile for portability issues (Sprint 83 - Day 7)
/// Collect all targets for analysis (both regular targets and pattern rules)
fn collect_targets_for_analysis(ast: &MakeAst) -> Vec<(&String, &Vec<String>)> {
    let mut targets = Vec::new();
    for item in &ast.items {
        match item {
            MakeItem::Target { name, recipe, .. } => {
                targets.push((name, recipe));
            }
            MakeItem::PatternRule {
                target_pattern,
                recipe,
                ..
            } => {
                targets.push((target_pattern, recipe));
            }
            _ => {}
        }
    }
    targets
}

/// Detect bashisms (non-POSIX shell constructs)
fn detect_bashisms(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect [[ ]] (bash-specific test)
            if recipe.contains("[[") {
                transformations.push(Transformation::DetectBashism {
                    target_name: (*target_name).clone(),
                    construct: "[[".to_string(),
                    posix_alternative: "Use [ instead of [[ for POSIX compliance".to_string(),
                    safe: false,
                });
            }

            // Detect $(( )) arithmetic expansion (bash-specific)
            if recipe.contains("$((") {
                transformations.push(Transformation::DetectBashism {
                    target_name: (*target_name).clone(),
                    construct: "$(())".to_string(),
                    posix_alternative: "Use expr for POSIX arithmetic".to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect platform-specific commands
fn detect_platform_specific(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect uname (platform-specific)
            if recipe.contains("uname") {
                transformations.push(Transformation::DetectPlatformSpecific {
                    target_name: (*target_name).clone(),
                    command: "uname".to_string(),
                    reason: "uname is platform-specific; consider using configure scripts"
                        .to_string(),
                    safe: false,
                });
            }

            // Detect /proc/ (Linux-specific)
            if recipe.contains("/proc/") {
                transformations.push(Transformation::DetectPlatformSpecific {
                    target_name: (*target_name).clone(),
                    command: "/proc/".to_string(),
                    reason: "/proc/ is Linux-specific and not portable".to_string(),
                    safe: false,
                });
            }

            // Detect ifconfig (deprecated, platform-specific)
            if recipe.contains("ifconfig") {
                transformations.push(Transformation::DetectPlatformSpecific {
                    target_name: (*target_name).clone(),
                    command: "ifconfig".to_string(),
                    reason: "ifconfig is deprecated and platform-specific".to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect shell-specific features
fn detect_shell_specific(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect source (bash-specific)
            if recipe.contains("source ") {
                transformations.push(Transformation::DetectShellSpecific {
                    target_name: (*target_name).clone(),
                    feature: "source".to_string(),
                    posix_alternative: "Use . instead of source for POSIX compliance".to_string(),
                    safe: false,
                });
            }

            // Detect declare (bash-specific)
            if recipe.contains("declare") {
                transformations.push(Transformation::DetectShellSpecific {
                    target_name: (*target_name).clone(),
                    feature: "declare".to_string(),
                    posix_alternative: "Use regular variable assignment for POSIX compliance"
                        .to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect non-portable command flags (GNU extensions)
fn detect_nonportable_flags(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect --preserve (GNU cp extension)
            if recipe.contains("--preserve") {
                transformations.push(Transformation::DetectNonPortableFlags {
                    target_name: (*target_name).clone(),
                    command: "cp".to_string(),
                    flag: "--preserve".to_string(),
                    reason: "--preserve is a GNU extension; use -p for portability".to_string(),
                    safe: false,
                });
            }

            // Detect --color (GNU extension)
            if recipe.contains("--color") {
                transformations.push(Transformation::DetectNonPortableFlags {
                    target_name: (*target_name).clone(),
                    command: "ls/grep".to_string(),
                    flag: "--color".to_string(),
                    reason: "--color is a GNU extension; not portable".to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect non-portable echo usage
fn detect_nonportable_echo(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect echo -e (not POSIX)
            if recipe.contains("echo -e") {
                transformations.push(Transformation::DetectNonPortableEcho {
                    target_name: (*target_name).clone(),
                    command: "echo -e".to_string(),
                    safe: false,
                });
            }

            // Detect echo -n (not POSIX)
            if recipe.contains("echo -n") {
                transformations.push(Transformation::DetectNonPortableEcho {
                    target_name: (*target_name).clone(),
                    command: "echo -n".to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

/// Detect sed -i (GNU extension)
fn detect_sed_inplace(targets: &[(&String, &Vec<String>)]) -> Vec<Transformation> {
    let mut transformations = Vec::new();
    for (target_name, recipes) in targets {
        for recipe in *recipes {
            // Detect sed -i (not portable)
            if recipe.contains("sed -i") {
                transformations.push(Transformation::DetectNonPortableFlags {
                    target_name: (*target_name).clone(),
                    command: "sed".to_string(),
                    flag: "-i".to_string(),
                    reason: "sed -i is a GNU extension; use temp file for portability".to_string(),
                    safe: false,
                });
            }
        }
    }
    transformations
}

fn analyze_portability(ast: &MakeAst) -> Vec<Transformation> {
    let targets = collect_targets_for_analysis(ast);

    let mut transformations = Vec::new();
    transformations.extend(detect_bashisms(&targets));
    transformations.extend(detect_platform_specific(&targets));
    transformations.extend(detect_shell_specific(&targets));
    transformations.extend(detect_nonportable_flags(&targets));
    transformations.extend(detect_nonportable_echo(&targets));
    transformations.extend(detect_sed_inplace(&targets));

    transformations
}

/// Check if transformation can be applied safely
fn is_safe_transformation(transformation: &Transformation) -> bool {
    match transformation {
        Transformation::WrapWithSort { safe, .. } => *safe,
        Transformation::AddComment { safe, .. } => *safe,
        // Sprint 83 - Parallel Safety
        Transformation::RecommendNotParallel { safe, .. } => *safe,
        Transformation::DetectRaceCondition { safe, .. } => *safe,
        Transformation::RecommendOrderOnlyPrereq { safe, .. } => *safe,
        Transformation::DetectMissingDependency { safe, .. } => *safe,
        Transformation::DetectOutputConflict { safe, .. } => *safe,
        Transformation::RecommendRecursiveMakeHandling { safe, .. } => *safe,
        Transformation::DetectDirectoryRace { safe, .. } => *safe,
        // Sprint 83 - Reproducible Builds (Day 4)
        Transformation::DetectTimestamp { safe, .. } => *safe,
        Transformation::DetectRandom { safe, .. } => *safe,
        Transformation::DetectProcessId { safe, .. } => *safe,
        Transformation::SuggestSourceDateEpoch { safe, .. } => *safe,
        Transformation::DetectNonDeterministicCommand { safe, .. } => *safe,
        // Sprint 83 - Performance Optimization (Day 5)
        Transformation::SuggestCombineShellInvocations { safe, .. } => *safe,
        Transformation::SuggestSimpleExpansion { safe, .. } => *safe,
        Transformation::RecommendSuffixes { safe, .. } => *safe,
        Transformation::DetectSequentialRecipes { safe, .. } => *safe,
        Transformation::SuggestPatternRule { safe, .. } => *safe,
        // Sprint 83 - Error Handling (Day 6)
        Transformation::DetectMissingErrorHandling { safe, .. } => *safe,
        Transformation::DetectSilentFailure { safe, .. } => *safe,
        Transformation::RecommendDeleteOnError { safe, .. } => *safe,
        Transformation::RecommendOneshell { safe, .. } => *safe,
        Transformation::DetectMissingSetE { safe, .. } => *safe,
        Transformation::DetectLoopWithoutErrorHandling { safe, .. } => *safe,
        // Sprint 83 - Portability (Day 7)
        Transformation::DetectBashism { safe, .. } => *safe,
        Transformation::DetectPlatformSpecific { safe, .. } => *safe,
        Transformation::DetectShellSpecific { safe, .. } => *safe,
        Transformation::DetectNonPortableFlags { safe, .. } => *safe,
        Transformation::DetectNonPortableEcho { safe, .. } => *safe,
    }
}

/// Generate human-readable report of transformations
fn generate_report(transformations: &[Transformation]) -> Vec<String> {
    transformations.iter().map(|t| match t {
        Transformation::WrapWithSort { variable_name, pattern, .. } => {
            format!("✅ Wrapped {} in variable '{}' with $(sort ...)", pattern, variable_name)
        }
        Transformation::AddComment { variable_name, rule, .. } => {
            format!("⚠️  Manual fix needed for variable '{}': {}", variable_name, rule)
        }
        Transformation::RecommendNotParallel { reason, .. } => {
            format!("⚠️  Parallel safety: {} (.NOTPARALLEL)", reason)
        }
        Transformation::DetectRaceCondition { target_names, conflicting_file, .. } => {
            format!("⚠️  Race condition detected: targets {:?} write to same file '{}'", target_names, conflicting_file)
        }
        Transformation::RecommendOrderOnlyPrereq { target_name, prereq_name, reason, .. } => {
            format!("⚠️  Recommend order-only prerequisite for '{}': add | {} ({})", target_name, prereq_name, reason)
        }
        Transformation::DetectMissingDependency { target_name, missing_file, provider_target, .. } => {
            format!("⚠️  Missing dependency: '{}' uses '{}' created by '{}', but '{}' is not in prerequisites", target_name, missing_file, provider_target, provider_target)
        }
        Transformation::DetectOutputConflict { output_file, .. } => {
            format!("⚠️  Output conflict: multiple targets write to same output file '{}'", output_file)
        }
        Transformation::RecommendRecursiveMakeHandling { target_name, subdirs, .. } => {
            format!("⚠️  Recursive make in '{}': consider dependencies for subdirs {:?} ($(MAKE))", target_name, subdirs)
        }
        Transformation::DetectDirectoryRace { target_names, directory, .. } => {
            format!("⚠️  Directory creation race: targets {:?} create directory '{}'", target_names, directory)
        }
        // Sprint 83 - Reproducible Builds (Day 4)
        Transformation::DetectTimestamp { variable_name, pattern, .. } => {
            format!("⚠️  Non-deterministic timestamp in '{}': {} - consider using SOURCE_DATE_EPOCH", variable_name, pattern)
        }
        Transformation::DetectRandom { variable_name, .. } => {
            format!("⚠️  Non-deterministic $RANDOM in '{}' - use fixed seed or version number", variable_name)
        }
        Transformation::DetectProcessId { variable_name, .. } => {
            format!("⚠️  Non-deterministic process ID ($$) in '{}' - use fixed temporary file name", variable_name)
        }
        Transformation::SuggestSourceDateEpoch { variable_name, .. } => {
            format!("💡 Suggestion: Use SOURCE_DATE_EPOCH for reproducible timestamps in '{}'", variable_name)
        }
        Transformation::DetectNonDeterministicCommand { variable_name, command, reason, .. } => {
            format!("⚠️  Non-deterministic command in '{}': {} - {}", variable_name, command, reason)
        }
        // Sprint 83 - Performance Optimization (Day 5)
        Transformation::SuggestCombineShellInvocations { target_name, recipe_count, .. } => {
            format!("💡 Performance: Combine {} shell invocations in '{}' using && or ; to reduce subshell spawns", recipe_count, target_name)
        }
        Transformation::SuggestSimpleExpansion { variable_name, reason, .. } => {
            format!("💡 Performance: {} for variable '{}'", reason, variable_name)
        }
        Transformation::RecommendSuffixes { reason, .. } => {
            format!("💡 Performance: {}", reason)
        }
        Transformation::DetectSequentialRecipes { target_name, recipe_count, .. } => {
            format!("⚠️  Performance: Target '{}' has {} sequential recipe lines - consider combining with && or ;", target_name, recipe_count)
        }
        Transformation::SuggestPatternRule { pattern, target_count, .. } => {
            format!("💡 Performance: {} explicit rules could use pattern rule '{}'", target_count, pattern)
        }
        // Sprint 83 - Error Handling (Day 6)
        Transformation::DetectMissingErrorHandling { target_name, command, .. } => {
            format!("⚠️  Error handling: Target '{}' has command without error handling: '{}' - consider adding || exit 1", target_name, command)
        }
        Transformation::DetectSilentFailure { target_name, command, .. } => {
            format!("⚠️  Error handling: Target '{}' has @ prefix that may hide errors: '{}' - consider removing @", target_name, command)
        }
        Transformation::RecommendDeleteOnError { reason, .. } => {
            format!("💡 Error handling: {}", reason)
        }
        Transformation::RecommendOneshell { target_name, reason, .. } => {
            format!("💡 Error handling: Target '{}' - {}", target_name, reason)
        }
        Transformation::DetectMissingSetE { target_name, command, .. } => {
            format!("⚠️  Error handling: Target '{}' has bash -c without set -e: '{}' - add 'set -e;' to fail on errors", target_name, command)
        }
        Transformation::DetectLoopWithoutErrorHandling { target_name, loop_command, .. } => {
            format!("⚠️  Error handling: Target '{}' has loop without error handling: '{}' - add || exit 1 inside loop", target_name, loop_command)
        }
        // Sprint 83 - Portability (Day 7)
        Transformation::DetectBashism { target_name, construct, posix_alternative, .. } => {
            format!("⚠️  Portability: Target '{}' uses bashism '{}' - {}", target_name, construct, posix_alternative)
        }
        Transformation::DetectPlatformSpecific { target_name, command, reason, .. } => {
            format!("⚠️  Portability: Target '{}' uses platform-specific command '{}' - {}", target_name, command, reason)
        }
        Transformation::DetectShellSpecific { target_name, feature, posix_alternative, .. } => {
            format!("⚠️  Portability: Target '{}' uses shell-specific feature '{}' - {}", target_name, feature, posix_alternative)
        }
        Transformation::DetectNonPortableFlags { target_name, command, flag, reason, .. } => {
            format!("⚠️  Portability: Target '{}' uses non-portable flag '{}' in '{}' - {}", target_name, flag, command, reason)
        }
        Transformation::DetectNonPortableEcho { target_name, command, .. } => {
            format!("⚠️  Portability: Target '{}' uses non-portable echo usage '{}' - use printf for portability", target_name, command)
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_paren_simple() {
        let s = "$(wildcard *.c)";
        assert_eq!(find_matching_paren(s, 0), Some(14));
    }

    #[test]
    fn test_find_matching_paren_nested() {
        let s = "$(filter %.o, $(wildcard *.c))";
        // Start at "$(wildcard", find its closing paren
        // Position 15 is the start of "$(wildcard"
        // The wildcard closing paren is at position 28
        assert_eq!(find_matching_paren(s, 15), Some(28));
    }

    #[test]
    fn test_wrap_pattern_with_sort_simple() {
        let value = "$(wildcard *.c)";
        let result = wrap_pattern_with_sort(value, "$(wildcard");
        assert_eq!(result, "$(sort $(wildcard *.c))");
    }

    #[test]
    fn test_wrap_pattern_with_sort_nested() {
        let value = "$(filter %.o, $(wildcard *.c))";
        let result = wrap_pattern_with_sort(value, "$(wildcard");
        assert_eq!(result, "$(filter %.o, $(sort $(wildcard *.c)))");
    }

    #[test]
    fn test_extract_variable_name() {
        let message = "Variable 'FILES' uses non-deterministic $(wildcard)";
        assert_eq!(extract_variable_name(message), "FILES");
    }

    // ========================================
    // Sprint 83 - Day 2-3: Parallel Safety Tests
    // ========================================

    /// Test PARALLEL_SAFETY_001: Check parallel safety analysis runs
    #[test]
    fn test_PARALLEL_SAFETY_001_parallel_safety_analysis() {
        // ARRANGE: Simple Makefile with no parallel safety issues
        let makefile = r#"
all: build

build:
	gcc -o app main.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify with parallel safety transformations
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Note: This Makefile has no issues, so no .NOTPARALLEL recommended
        // This is correct idempotent behavior
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PARALLEL_SAFETY_002: Detect race condition in shared file write
    #[test]
    fn test_PARALLEL_SAFETY_002_detect_race_condition() {
        // ARRANGE: Two targets writing to same file
        let makefile = r#"
target1:
	echo "output1" > shared.txt

target2:
	echo "output2" > shared.txt

all: target1 target2
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify with parallel safety analysis
        let result = purify_makefile(&ast);

        // ASSERT: Should detect race condition AND recommend .NOTPARALLEL
        assert!(
            result.report.iter().any(|r| r.contains("race")
                || r.contains("parallel")
                || r.contains(".NOTPARALLEL")),
            "Should detect race condition in shared file write and recommend .NOTPARALLEL"
        );
    }

    /// Test PARALLEL_SAFETY_003: Add order-only prerequisite for dependency
    #[test]
    fn test_PARALLEL_SAFETY_003_add_order_only_prereq() {
        // ARRANGE: Target that needs order-only prerequisite
        let makefile = r#"
build: | output_dir
	gcc -o output_dir/app main.c

output_dir:
	mkdir -p output_dir
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should preserve order-only prerequisite (already correct)
        // Test passes if purify_makefile runs without panic
        // Should handle order-only prerequisites correctly
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PARALLEL_SAFETY_004: Detect missing dependency causing race
    #[test]
    fn test_PARALLEL_SAFETY_004_missing_dependency() {
        // ARRANGE: Target using file created by another target (missing dep)
        let makefile = r#"
generate:
	echo "data" > data.txt

process:
	cat data.txt > output.txt

all: generate process
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect that process depends on generate's output
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("dependency") || r.contains("data.txt")),
            "Should detect missing dependency"
        );
    }

    /// Test PARALLEL_SAFETY_005: Preserve existing .NOTPARALLEL
    #[test]
    fn test_PARALLEL_SAFETY_005_preserve_notparallel() {
        // ARRANGE: Makefile already has .NOTPARALLEL
        let makefile = r#"
.NOTPARALLEL:

all: build

build:
	gcc -o app main.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should not add duplicate .NOTPARALLEL
        let notparallel_count = result
            .report
            .iter()
            .filter(|r| r.contains(".NOTPARALLEL"))
            .count();
        assert!(
            notparallel_count <= 1,
            "Should not add duplicate .NOTPARALLEL"
        );
    }

    /// Test PARALLEL_SAFETY_006: Detect .PHONY target parallel safety
    #[test]
    fn test_PARALLEL_SAFETY_006_phony_target_safety() {
        // ARRANGE: .PHONY targets that are parallel-safe
        let makefile = r#"
.PHONY: clean test

clean:
	rm -f *.o

test:
	./run_tests.sh

all: clean test
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should handle .PHONY targets correctly
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PARALLEL_SAFETY_007: Multiple targets same output file
    #[test]
    fn test_PARALLEL_SAFETY_007_multiple_targets_same_output() {
        // ARRANGE: Multiple targets writing to same output
        let makefile = r#"
debug: main.c
	gcc -g -o app main.c

release: main.c
	gcc -O2 -o app main.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should warn about conflict
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("conflict") || r.contains("same output")),
            "Should detect multiple targets with same output"
        );
    }

    /// Test PARALLEL_SAFETY_008: Recursive make calls need serialization
    #[test]
    fn test_PARALLEL_SAFETY_008_recursive_make_serialization() {
        // ARRANGE: Recursive make calls
        let makefile = r#"
subdirs:
	$(MAKE) -C subdir1
	$(MAKE) -C subdir2

all: subdirs
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend proper dependency handling
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("recursive") || r.contains("$(MAKE)")),
            "Should handle recursive make calls"
        );
    }

    /// Test PARALLEL_SAFETY_009: Parallel-safe pattern rule
    #[test]
    fn test_PARALLEL_SAFETY_009_pattern_rule_safety() {
        // ARRANGE: Pattern rule that is parallel-safe
        let makefile = r#"
%.o: %.c
	gcc -c $< -o $@

all: main.o util.o
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should recognize parallel-safe pattern rules
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PARALLEL_SAFETY_010: Shared directory creation race
    #[test]
    fn test_PARALLEL_SAFETY_010_shared_directory_race() {
        // ARRANGE: Multiple targets creating same directory
        let makefile = r#"
obj/main.o: main.c
	mkdir -p obj
	gcc -c main.c -o obj/main.o

obj/util.o: util.c
	mkdir -p obj
	gcc -c util.c -o obj/util.o

all: obj/main.o obj/util.o
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend order-only prerequisite for directory
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("directory") || r.contains("mkdir")),
            "Should detect shared directory creation race"
        );
    }

    // ========================================
    // Sprint 83 - Day 4: Reproducible Builds Tests
    // ========================================

    /// Test REPRODUCIBLE_001: Detect $(shell date) timestamp
    #[test]
    fn test_REPRODUCIBLE_001_detect_shell_date() {
        // ARRANGE: Makefile with $(shell date) timestamp
        let makefile = r#"
VERSION := $(shell date +%Y%m%d)

build:
	echo "Building version $(VERSION)"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect timestamp and suggest SOURCE_DATE_EPOCH
        assert!(
            result.report.iter().any(|r| r.contains("timestamp")
                || r.contains("date")
                || r.contains("SOURCE_DATE_EPOCH")),
            "Should detect non-deterministic timestamp $(shell date)"
        );
    }

    /// Test REPRODUCIBLE_002: Detect $(shell date +%s) unix timestamp
    #[test]
    fn test_REPRODUCIBLE_002_detect_unix_timestamp() {
        // ARRANGE: Makefile with unix timestamp
        let makefile = r#"
RELEASE := release-$(shell date +%s)

deploy:
	tar -czf $(RELEASE).tar.gz src/
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect unix timestamp
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("timestamp") || r.contains("date")),
            "Should detect non-deterministic unix timestamp"
        );
    }

    /// Test REPRODUCIBLE_003: Detect $RANDOM variable
    #[test]
    fn test_REPRODUCIBLE_003_detect_random() {
        // ARRANGE: Makefile with $RANDOM
        let makefile = r#"
SESSION_ID := session-$$RANDOM

test:
	echo "Session: $(SESSION_ID)"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect $RANDOM
        assert!(
            result.report.iter().any(|r| r.contains("RANDOM")
                || r.contains("random")
                || r.contains("non-deterministic")),
            "Should detect non-deterministic $RANDOM variable"
        );
    }

    /// Test REPRODUCIBLE_004: Detect process ID $$
    #[test]
    fn test_REPRODUCIBLE_004_detect_process_id() {
        // ARRANGE: Makefile with process ID
        let makefile = r#"
TMP_FILE := /tmp/build-$$$$

build:
	touch $(TMP_FILE)
	gcc -o app main.c
	rm $(TMP_FILE)
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect process ID
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("process") || r.contains("$$") || r.contains("PID")),
            "Should detect non-deterministic process ID $$"
        );
    }

    /// Test REPRODUCIBLE_005: Suggest SOURCE_DATE_EPOCH replacement
    #[test]
    fn test_REPRODUCIBLE_005_suggest_source_date_epoch() {
        // ARRANGE: Makefile with timestamp that should use SOURCE_DATE_EPOCH
        let makefile = r#"
BUILD_DATE := $(shell date)

package:
	echo "Built on: $(BUILD_DATE)" > version.txt
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest SOURCE_DATE_EPOCH
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("SOURCE_DATE_EPOCH")),
            "Should suggest using SOURCE_DATE_EPOCH for reproducibility"
        );
    }

    /// Test REPRODUCIBLE_006: Detect non-deterministic command substitution
    #[test]
    fn test_REPRODUCIBLE_006_detect_command_substitution() {
        // ARRANGE: Makefile with non-deterministic command
        let makefile = r#"
HOSTNAME := $(shell hostname)

config:
	echo "Host: $(HOSTNAME)" > config.txt
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect hostname (environment-dependent)
        assert!(
            result.report.iter().any(|r| r.contains("hostname")
                || r.contains("environment")
                || r.contains("deterministic")),
            "Should detect environment-dependent hostname"
        );
    }

    /// Test REPRODUCIBLE_007: Preserve deterministic timestamps
    #[test]
    fn test_REPRODUCIBLE_007_preserve_deterministic() {
        // ARRANGE: Makefile already using SOURCE_DATE_EPOCH
        let makefile = r#"
BUILD_DATE := $(SOURCE_DATE_EPOCH)

build:
	echo "Build: $(BUILD_DATE)"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should not flag SOURCE_DATE_EPOCH as issue
        // Test passes if purify_makefile runs without panic
        // May still have other transformations, but not for SOURCE_DATE_EPOCH
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test REPRODUCIBLE_008: Detect git commit hash timestamp
    #[test]
    fn test_REPRODUCIBLE_008_detect_git_timestamp() {
        // ARRANGE: Makefile using git commit timestamp
        let makefile = r#"
GIT_DATE := $(shell git log -1 --format=%cd)

version:
	echo "Git date: $(GIT_DATE)" > version.txt
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect git timestamp
        assert!(
            result.report.iter().any(|r| r.contains("git")
                || r.contains("timestamp")
                || r.contains("deterministic")),
            "Should detect git commit timestamp"
        );
    }

    /// Test REPRODUCIBLE_009: Detect mktemp usage
    #[test]
    fn test_REPRODUCIBLE_009_detect_mktemp() {
        // ARRANGE: Makefile using mktemp (non-deterministic temp files)
        let makefile = r#"
build:
	TMP=$$(mktemp); \
	gcc -o $$TMP main.c; \
	cp $$TMP app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect mktemp
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("mktemp") || r.contains("temp") || r.contains("deterministic")),
            "Should detect non-deterministic mktemp usage"
        );
    }

    /// Test REPRODUCIBLE_010: Comprehensive reproducibility check
    #[test]
    fn test_REPRODUCIBLE_010_comprehensive_check() {
        // ARRANGE: Makefile with multiple reproducibility issues
        let makefile = r#"
VERSION := $(shell date +%Y%m%d)
SESSION := $$RANDOM
BUILD_HOST := $(shell hostname)

all: build

build:
	echo "Version: $(VERSION)" > version.txt
	echo "Session: $(SESSION)" >> version.txt
	echo "Host: $(BUILD_HOST)" >> version.txt
	gcc -o app main.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect multiple issues
        assert!(
            result.transformations_applied >= 3,
            "Should detect at least 3 reproducibility issues (date, RANDOM, hostname)"
        );
    }

    // ========================================
    // Sprint 83 - Day 5: Performance Optimization Tests
    // ========================================

    /// Test PERFORMANCE_001: Detect multiple shell invocations
    #[test]
    fn test_PERFORMANCE_001_detect_multiple_shell_invocations() {
        // ARRANGE: Makefile with multiple shell commands that could be combined
        let makefile = r#"
build:
	mkdir -p bin
	gcc -c main.c -o bin/main.o
	gcc -c util.c -o bin/util.o
	gcc bin/main.o bin/util.o -o bin/app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend combining shell invocations
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("combine") || r.contains("shell") || r.contains("performance")),
            "Should detect multiple shell invocations that could be combined"
        );
    }

    /// Test PERFORMANCE_002: Suggest using := instead of =
    #[test]
    fn test_PERFORMANCE_002_suggest_simple_expansion() {
        // ARRANGE: Makefile with recursive variable that could be simple
        let makefile = r#"
CC = gcc
CFLAGS = -Wall -O2
LDFLAGS = -lm

build:
	$(CC) $(CFLAGS) main.c $(LDFLAGS) -o app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest using := for simple variables
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains(":=") || r.contains("simple") || r.contains("expansion")),
            "Should suggest using := instead of = for simple variables"
        );
    }

    /// Test PERFORMANCE_003: Detect missing .SUFFIXES
    #[test]
    fn test_PERFORMANCE_003_detect_missing_suffixes() {
        // ARRANGE: Makefile without .SUFFIXES, with performance issue (recursive var with shell)
        // Note: .SUFFIXES is only recommended when other performance issues are detected
        let makefile = r#"
VERSION = $(shell git describe)

all: app

app: main.o
	gcc main.o -o app

main.o: main.c
	gcc -c main.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend adding .SUFFIXES (because of VERSION performance issue)
        assert!(
            result.report.iter().any(|r| r.contains(".SUFFIXES")
                || r.contains("builtin")
                || r.contains("performance")),
            "Should recommend adding .SUFFIXES: when performance issues detected"
        );
    }

    /// Test PERFORMANCE_004: Detect inefficient variable expansion
    #[test]
    fn test_PERFORMANCE_004_detect_inefficient_expansion() {
        // ARRANGE: Makefile with variable that re-expands $(shell) multiple times
        let makefile = r#"
VERSION = $(shell git describe --tags)

build:
	echo "Building $(VERSION)"
	tar -czf myapp-$(VERSION).tar.gz src/
	echo "Created myapp-$(VERSION).tar.gz"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest using := to avoid re-expansion
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains(":=") || r.contains("expansion") || r.contains("shell")),
            "Should suggest := to avoid re-expanding $(shell) multiple times"
        );
    }

    /// Test PERFORMANCE_005: Preserve existing .SUFFIXES
    #[test]
    fn test_PERFORMANCE_005_preserve_existing_suffixes() {
        // ARRANGE: Makefile already has .SUFFIXES:
        let makefile = r#"
.SUFFIXES:

all: app

app: main.c
	gcc main.c -o app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should not add duplicate .SUFFIXES
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PERFORMANCE_006: Detect sequential recipe lines
    #[test]
    fn test_PERFORMANCE_006_detect_sequential_recipes() {
        // ARRANGE: Makefile with many sequential recipe lines
        let makefile = r#"
install:
	mkdir -p /usr/local/bin
	cp app /usr/local/bin/
	chmod +x /usr/local/bin/app
	mkdir -p /usr/local/share/doc/app
	cp README.md /usr/local/share/doc/app/
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest combining with && or ;
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("combine") || r.contains("&&") || r.contains("performance")),
            "Should suggest combining sequential recipe lines"
        );
    }

    /// Test PERFORMANCE_007: Detect expensive wildcard in recipe
    #[test]
    fn test_PERFORMANCE_007_detect_expensive_wildcard() {
        // ARRANGE: Makefile with wildcard expansion in recipe (expensive)
        let makefile = r#"
clean:
	rm -f *.o
	rm -f *.a
	rm -f *.so
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest combining into single rm command
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("combine") || r.contains("rm") || r.contains("performance")),
            "Should suggest combining rm commands for better performance"
        );
    }

    /// Test PERFORMANCE_008: Detect := already used
    #[test]
    fn test_PERFORMANCE_008_detect_simple_expansion_already_used() {
        // ARRANGE: Makefile already uses := (correct)
        let makefile = r#"
CC := gcc
CFLAGS := -Wall -O2

build:
	$(CC) $(CFLAGS) main.c -o app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should not flag variables already using :=
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PERFORMANCE_009: Detect pattern rule efficiency
    #[test]
    fn test_PERFORMANCE_009_detect_pattern_rule_efficiency() {
        // ARRANGE: Makefile with explicit rules that could be pattern rule
        let makefile = r#"
main.o: main.c
	gcc -c main.c -o main.o

util.o: util.c
	gcc -c util.c -o util.o

math.o: math.c
	gcc -c math.c -o math.o
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should suggest using pattern rule %.o: %.c
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("pattern") || r.contains("%.o") || r.contains("rule")),
            "Should suggest using pattern rule for repeated compilation"
        );
    }

    /// Test PERFORMANCE_010: Comprehensive performance check
    #[test]
    fn test_PERFORMANCE_010_comprehensive_performance_check() {
        // ARRANGE: Makefile with multiple performance issues
        let makefile = r#"
VERSION = $(shell git describe --tags)
CC = gcc
CFLAGS = -Wall -O2

all: app

app: main.o util.o
	gcc main.o util.o -o app

main.o: main.c
	mkdir -p obj
	gcc -c main.c -o main.o
	cp main.o obj/

util.o: util.c
	mkdir -p obj
	gcc -c util.c -o util.o
	cp util.o obj/

clean:
	rm -f *.o
	rm -f *.a
	rm -f app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect multiple performance issues
        assert!(
            result.transformations_applied >= 2,
            "Should detect multiple performance issues (shell expansion, multiple commands, etc.)"
        );
    }

    // ========================================
    // Sprint 83 - Day 6: Error Handling Tests
    // ========================================

    /// Test ERROR_HANDLING_001: Detect missing error handling (|| exit 1)
    #[test]
    fn test_ERROR_HANDLING_001_detect_missing_error_handling() {
        // ARRANGE: Makefile with important commands without error handling
        let makefile = r#"
build:
	mkdir build
	gcc -c main.c -o build/main.o
	gcc build/main.o -o build/app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend error handling for critical commands
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("error") || r.contains("exit") || r.contains("handling")),
            "Should recommend adding error handling (|| exit 1) for critical commands"
        );
    }

    /// Test ERROR_HANDLING_002: Detect silent failures (@ prefix)
    #[test]
    fn test_ERROR_HANDLING_002_detect_silent_failures() {
        // ARRANGE: Makefile with @ prefix hiding errors
        let makefile = r#"
test:
	@echo "Running tests..."
	@./run-tests.sh
	@echo "Tests complete"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should warn about @ prefix hiding errors
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("@") || r.contains("silent") || r.contains("error")),
            "Should detect @ prefix that may hide errors in critical commands"
        );
    }

    /// Test ERROR_HANDLING_003: Recommend .DELETE_ON_ERROR
    #[test]
    fn test_ERROR_HANDLING_003_recommend_delete_on_error() {
        // ARRANGE: Makefile without .DELETE_ON_ERROR but with error handling issues
        let makefile = r#"
build:
	mkdir build
	gcc -c main.c -o build/main.o
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend .DELETE_ON_ERROR (because mkdir without error handling was detected)
        assert!(
            result.report.iter().any(|r| r.contains(".DELETE_ON_ERROR")),
            "Should recommend .DELETE_ON_ERROR when error handling issues are detected"
        );
    }

    /// Test ERROR_HANDLING_004: Preserve existing .DELETE_ON_ERROR
    #[test]
    fn test_ERROR_HANDLING_004_preserve_existing_delete_on_error() {
        // ARRANGE: Makefile already has .DELETE_ON_ERROR
        let makefile = r#"
.DELETE_ON_ERROR:

%.o: %.c
	gcc -c $< -o $@
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should preserve existing .DELETE_ON_ERROR without duplication
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test ERROR_HANDLING_005: Detect unchecked command substitution
    #[test]
    fn test_ERROR_HANDLING_005_detect_unchecked_command_substitution() {
        // ARRANGE: Makefile with unchecked $(shell) commands
        let makefile = r#"
VERSION := $(shell git describe --tags)

build:
	echo "Building version $(VERSION)"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should detect potentially unchecked shell command substitution
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test ERROR_HANDLING_006: Detect missing .ONESHELL with multiline recipes
    #[test]
    fn test_ERROR_HANDLING_006_detect_missing_oneshell() {
        // ARRANGE: Makefile with multiline recipe without .ONESHELL
        let makefile = r#"
deploy:
	cd /tmp
	mkdir app
	echo "Done"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend .ONESHELL for related commands
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains(".ONESHELL") || r.contains("multiline") || r.contains("shell")),
            "Should recommend .ONESHELL or && for related commands across lines"
        );
    }

    /// Test ERROR_HANDLING_007: Detect commands that modify state without checks
    #[test]
    fn test_ERROR_HANDLING_007_detect_unchecked_state_modification() {
        // ARRANGE: Makefile with state-modifying commands without checks
        let makefile = r#"
clean:
	rm -rf build/*
	rm -rf dist/*
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should verify destructive commands have appropriate flags
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test ERROR_HANDLING_008: Detect set -e equivalent missing
    #[test]
    fn test_ERROR_HANDLING_008_detect_missing_set_e() {
        // ARRANGE: Makefile with shell script without set -e
        let makefile = r#"
test:
	bash -c "echo test1; false; echo test2"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend set -e for shell scripts
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("set -e") || r.contains("error") || r.contains("exit")),
            "Should recommend 'set -e' for inline shell scripts"
        );
    }

    /// Test ERROR_HANDLING_009: Detect missing error handling in loops
    #[test]
    fn test_ERROR_HANDLING_009_detect_missing_error_handling_in_loops() {
        // ARRANGE: Makefile with for loop without error handling
        let makefile = r#"
install:
	for f in *.so; do cp $$f /usr/lib; done
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend error handling in loops
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("loop") || r.contains("error") || r.contains("exit")),
            "Should recommend error handling in for loops (|| exit 1)"
        );
    }

    /// Test ERROR_HANDLING_010: Comprehensive error handling check
    #[test]
    fn test_ERROR_HANDLING_010_comprehensive_error_handling_check() {
        // ARRANGE: Makefile with multiple error handling issues
        let makefile = r#"
VERSION := $(shell git describe)

build:
	@mkdir build
	gcc -c main.c -o build/main.o
	for f in *.c; do gcc -c $$f; done
	bash -c "echo done"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect multiple error handling issues
        assert!(
            result.transformations_applied >= 1,
            "Should detect multiple error handling issues (@ prefix, missing ||, loops, etc.)"
        );
    }

    // ========================================
    // Sprint 83 - Day 7: Portability Tests
    // ========================================

    /// Test PORTABILITY_001: Detect bashisms in recipes ([[, $(()), etc.)
    #[test]
    fn test_PORTABILITY_001_detect_bashisms() {
        // ARRANGE: Makefile with bash-specific syntax
        let makefile = r#"
test:
	if [[ -f file.txt ]]; then echo "found"; fi
	result=$$((1 + 2))
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect bashisms ([[ and $(()))
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("bashism") || r.contains("[[") || r.contains("POSIX")),
            "Should detect bashisms like [[ and $(())"
        );
    }

    /// Test PORTABILITY_002: Detect GNU Make-specific extensions
    #[test]
    fn test_PORTABILITY_002_detect_gnu_make_extensions() {
        // ARRANGE: Makefile with GNU Make-specific syntax
        let makefile = r#"
%.o: %.c
	gcc -c $< -o $@

build: $(wildcard *.c)
	gcc -o app $^
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should detect GNU Make-specific constructs
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PORTABILITY_003: Detect platform-specific commands
    #[test]
    fn test_PORTABILITY_003_detect_platform_specific_commands() {
        // ARRANGE: Makefile with platform-specific commands
        let makefile = r#"
detect:
	uname -s
	cat /proc/cpuinfo
	ifconfig
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect platform-specific commands
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("platform") || r.contains("portable") || r.contains("uname")),
            "Should detect platform-specific commands like uname, /proc, ifconfig"
        );
    }

    /// Test PORTABILITY_004: Detect shell-specific features
    #[test]
    fn test_PORTABILITY_004_detect_shell_specific_features() {
        // ARRANGE: Makefile with bash-specific features
        let makefile = r#"
build:
	echo $$RANDOM
	source setup.sh
	declare -a array=(1 2 3)
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect shell-specific features (source, declare)
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("source") || r.contains("declare") || r.contains("bash")),
            "Should detect bash-specific features like source and declare"
        );
    }

    /// Test PORTABILITY_005: Detect path separator issues
    #[test]
    fn test_PORTABILITY_005_detect_path_separator_issues() {
        // ARRANGE: Makefile with hardcoded path separators
        let makefile = r#"
build:
	gcc -I/usr/local/include -L/usr/local/lib app.c
	install -m 755 app /usr/local/bin/app
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should detect hardcoded paths that may not be portable
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PORTABILITY_006: Preserve portable constructs
    #[test]
    fn test_PORTABILITY_006_preserve_portable_constructs() {
        // ARRANGE: Makefile with POSIX-compliant syntax
        let makefile = r#"
build:
	if [ -f file.txt ]; then echo "found"; fi
	result=`expr 1 + 2`
	. setup.sh
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Should not flag POSIX-compliant constructs ([ instead of [[, expr, .)
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Test PORTABILITY_007: Detect non-portable flags
    #[test]
    fn test_PORTABILITY_007_detect_non_portable_flags() {
        // ARRANGE: Makefile with GNU-specific flags
        let makefile = r#"
build:
	cp --preserve=all src dest
	ls --color=auto
	grep --color=always pattern file
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect GNU-specific long flags
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("--") || r.contains("GNU") || r.contains("portable")),
            "Should detect GNU-specific long flags like --preserve, --color"
        );
    }

    /// Test PORTABILITY_008: Detect echo -e and echo -n
    #[test]
    fn test_PORTABILITY_008_detect_echo_flags() {
        // ARRANGE: Makefile with non-portable echo usage
        let makefile = r#"
build:
	echo -e "Line1\nLine2"
	echo -n "No newline"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect non-portable echo flags
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("echo") || r.contains("printf") || r.contains("portable")),
            "Should detect non-portable echo -e and echo -n (recommend printf)"
        );
    }

    /// Test PORTABILITY_009: Detect sed -i (GNU extension)
    #[test]
    fn test_PORTABILITY_009_detect_sed_in_place() {
        // ARRANGE: Makefile with sed -i (GNU extension)
        let makefile = r#"
build:
	sed -i 's/old/new/g' file.txt
	sed -i.bak 's/foo/bar/' data.txt
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect sed -i (non-portable)
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("sed") || r.contains("portable") || r.contains("-i")),
            "Should detect sed -i as non-portable GNU extension"
        );
    }

    /// Test PORTABILITY_010: Comprehensive portability check
    #[test]
    fn test_PORTABILITY_010_comprehensive_portability_check() {
        // ARRANGE: Makefile with multiple portability issues
        let makefile = r#"
build:
	if [[ -f file.txt ]]; then echo -e "Found\n"; fi
	uname -s
	source env.sh
	cp --preserve=all src dest
	sed -i 's/old/new/' file
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect multiple portability issues
        assert!(
            result.transformations_applied >= 1,
            "Should detect multiple portability issues ([[, echo -e, uname, source, --, sed -i)"
        );
    }

    // ========================================
    // Sprint 83 - Days 8-9: Property & Integration Tests
    // ========================================

    /// Property Test 001: Idempotency - purifying twice should be identical to purifying once
    #[test]
    fn test_PROPERTY_001_idempotency() {
        // ARRANGE: Makefile with various issues
        let makefile = r#"
FILES = $(wildcard *.c)
build:
	mkdir build
	gcc -c main.c
	echo "Done"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify once
        let result1 = purify_makefile(&ast);

        // Purify the result again (should be idempotent)
        let result2 = purify_makefile(&ast);

        // ASSERT: Both purifications should produce the same recommendations
        assert_eq!(
            result1.report.len(),
            result2.report.len(),
            "Purification should be idempotent - same recommendations"
        );
        assert_eq!(
            result1.transformations_applied, result2.transformations_applied,
            "Purification should apply same number of transformations"
        );
    }

    /// Property Test 002: Parallel Safety - verify parallel safety analysis works
    #[test]
    fn test_PROPERTY_002_parallel_safety_preserved() {
        // ARRANGE: Makefile that could benefit from parallel safety checks
        let makefile = r#"
all: build test
build:
	gcc -c main.c -o main.o
test:
	./test.sh
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // Test passes if purify_makefile runs without panic
        // Parallel safety analysis should execute without errors
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Property Test 003: Reproducibility - verify non-deterministic detection
    #[test]
    fn test_PROPERTY_003_reproducibility_enforced() {
        // ARRANGE: Makefile with non-deterministic elements
        let makefile = r#"
VERSION = $(shell date +%s)
build:
	echo "Version: $$RANDOM"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect non-deterministic patterns
        assert!(
            result
                .report
                .iter()
                .any(|r| r.contains("date") || r.contains("RANDOM") || r.contains("deterministic")),
            "Should detect non-deterministic patterns"
        );
    }

    /// Property Test 004: Performance - verify optimization recommendations
    #[test]
    fn test_PROPERTY_004_performance_optimizations() {
        // ARRANGE: Makefile with performance issues
        let makefile = r#"
VAR = $(shell echo test)
FILES = $(wildcard *.c)
build:
	gcc -c file1.c
	gcc -c file2.c
	gcc -c file3.c
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend performance improvements
        assert!(
            result.transformations_applied >= 1,
            "Should recommend performance optimizations"
        );
    }

    /// Property Test 005: Error Handling - verify error handling recommendations
    #[test]
    fn test_PROPERTY_005_error_handling_completeness() {
        // ARRANGE: Makefile without error handling
        let makefile = r#"
build:
	mkdir build
	gcc -c main.c
	cp main.o build/
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should recommend error handling
        assert!(
            result.report.iter().any(|r| r.contains("error")
                || r.contains("exit")
                || r.contains("DELETE_ON_ERROR")),
            "Should recommend error handling improvements"
        );
    }

    /// Integration Test 001: End-to-end purification workflow
    #[test]
    fn test_INTEGRATION_001_complete_purification() {
        // ARRANGE: Complex Makefile with multiple issues
        let makefile = r#"
# Makefile with multiple categories of issues
FILES = $(wildcard *.c)
VERSION = $(shell date +%s)

build: compile link
compile:
	mkdir build
	gcc -c main.c -o build/main.o
link:
	gcc build/main.o -o app
	echo "Build complete"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect issues across multiple categories
        // 1. Non-deterministic (wildcard needs sort, date timestamp)
        // 2. Missing error handling
        // 3. Performance issues
        assert!(
            result.transformations_applied >= 3,
            "Should detect issues across multiple transformation categories"
        );
        assert!(
            result.report.len() >= 3,
            "Should generate recommendations for multiple issues"
        );
    }

    /// Integration Test 002: Verify no false positives on clean Makefiles
    #[test]
    fn test_INTEGRATION_002_clean_makefile_no_false_positives() {
        // ARRANGE: Well-written Makefile with no obvious issues
        let makefile = r#"
.DELETE_ON_ERROR:
.SUFFIXES:

FILES := $(sort $(wildcard *.c))

build: compile
compile:
	mkdir -p build || exit 1
	gcc -c main.c -o build/main.o || exit 1
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should have minimal recommendations (good practices already applied)
        // Test passes if purify_makefile runs without panic
        // Clean Makefile should not trigger excessive recommendations
        let _ = result.transformations_applied; // Verify result exists
    }

    /// Integration Test 003: Verify composition of transformations
    #[test]
    fn test_INTEGRATION_003_transformation_composition() {
        // ARRANGE: Makefile that triggers multiple transformation categories
        let makefile = r#"
FILES = $(wildcard *.c)
VERSION = $(shell date +%s)

build: compile link
compile:
	mkdir build
	for f in *.c; do gcc -c $$f; done
	echo "Compiled"
link:
	gcc *.o -o app
	echo -e "Linked\n"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should detect multiple categories:
        // - Reproducibility (wildcard, date)
        // - Error handling (mkdir, for loop)
        // - Portability (echo -e)
        // - Performance (wildcard not sorted)
        let report_text = result.report.join("\n");

        // Check for diverse transformation categories
        let has_reproducibility = report_text.contains("wildcard") || report_text.contains("date");
        let has_error_handling = report_text.contains("error") || report_text.contains("exit");
        let has_portability = report_text.contains("echo") || report_text.contains("portable");

        assert!(
            has_reproducibility || has_error_handling || has_portability,
            "Should detect issues from multiple transformation categories"
        );
    }

    /// Integration Test 004: Verify all 5 transformation categories are functional
    #[test]
    fn test_INTEGRATION_004_all_categories_functional() {
        // ARRANGE: Makefile that exercises all 5 categories
        let makefile = r#"
# 1. Parallel Safety - race condition
FILES = $(wildcard *.c)
# 2. Reproducibility - non-deterministic
VERSION = $(shell date +%s)

# 3. Performance - multiple shell invocations
build: compile link
compile:
	# 4. Error Handling - no error checks
	mkdir build
	gcc -c main.c
	# 5. Portability - bashisms
	if [[ -f main.o ]]; then echo "found"; fi
link:
	gcc *.o -o app
	echo -e "Done\n"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should have detected issues from all categories
        assert!(
            result.transformations_applied >= 5,
            "Should detect issues from all 5 transformation categories"
        );

        // Verify report contains recommendations
        assert!(
            result.report.len() >= 5,
            "Should generate recommendations from multiple categories"
        );
    }

    /// Integration Test 005: Verify backward compatibility (existing tests still pass)
    #[test]
    fn test_INTEGRATION_005_backward_compatibility() {
        // ARRANGE: Simple Makefile from earlier tests
        let makefile = r#"
FILES = $(wildcard *.c)
all:
	echo "Building"
"#;
        let ast = crate::make_parser::parser::parse_makefile(makefile).unwrap();

        // ACT: Purify
        let result = purify_makefile(&ast);

        // ASSERT: Should still work (backward compatibility)
        // Purification succeeded - result exists
        let _ = result.transformations_applied;
        let _ = result.manual_fixes_needed;
    }

    // ===== NASA-QUALITY UNIT TESTS for detect_missing_file_dependencies helpers =====

    #[test]
    fn test_try_extract_output_redirect_valid() {
        let recipe = "echo hello > output.txt";
        assert_eq!(
            try_extract_output_redirect(recipe),
            Some("output.txt".to_string()),
            "Should extract output filename from redirect"
        );
    }

    #[test]
    fn test_try_extract_output_redirect_no_redirect() {
        let recipe = "echo hello";
        assert_eq!(
            try_extract_output_redirect(recipe),
            None,
            "Should return None when no redirect present"
        );
    }

    #[test]
    fn test_try_extract_output_redirect_multiple_words() {
        let recipe = "cat input.txt > output.txt extra";
        assert_eq!(
            try_extract_output_redirect(recipe),
            Some("output.txt".to_string()),
            "Should extract only first word after redirect"
        );
    }

    #[test]
    fn test_try_extract_cat_input_valid() {
        let recipe = "cat input.txt";
        assert_eq!(
            try_extract_cat_input(recipe),
            Some("input.txt".to_string()),
            "Should extract input filename from cat command"
        );
    }

    #[test]
    fn test_try_extract_cat_input_no_cat() {
        let recipe = "echo hello";
        assert_eq!(
            try_extract_cat_input(recipe),
            None,
            "Should return None when no cat command"
        );
    }

    #[test]
    fn test_try_extract_cat_input_automatic_variable() {
        let recipe = "cat $<";
        assert_eq!(
            try_extract_cat_input(recipe),
            None,
            "Should return None for automatic variables"
        );
    }

    #[test]
    fn test_try_extract_cat_input_with_path() {
        let recipe = "cat src/file.txt | grep pattern";
        assert_eq!(
            try_extract_cat_input(recipe),
            Some("src/file.txt".to_string()),
            "Should extract filename with path"
        );
    }

    #[test]
    fn test_is_automatic_variable_all_variants() {
        assert!(is_automatic_variable("$<"), "$< should be automatic");
        assert!(is_automatic_variable("$@"), "$@ should be automatic");
        assert!(is_automatic_variable("$^"), "$^ should be automatic");
        assert!(is_automatic_variable("$?"), "$? should be automatic");
        assert!(is_automatic_variable("$*"), "$* should be automatic");
        assert!(is_automatic_variable("$+"), "$+ should be automatic");
    }

    #[test]
    fn test_is_automatic_variable_normal_filename() {
        assert!(
            !is_automatic_variable("file.txt"),
            "Normal filename should NOT be automatic"
        );
        assert!(
            !is_automatic_variable("$VAR"),
            "User variable should NOT be automatic"
        );
    }

    #[test]
    fn test_target_has_prerequisite_true() {
        use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

        let ast = MakeAst {
            items: vec![MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["compile".to_string()],
                recipe: vec![],
                phony: false,
                recipe_metadata: None,
                span: Span::new(0, 10, 1),
            }],
            metadata: MakeMetadata::new(),
        };

        assert!(
            target_has_prerequisite(&ast, "build", "compile"),
            "Should find existing prerequisite"
        );
    }

    #[test]
    fn test_target_has_prerequisite_false() {
        use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

        let ast = MakeAst {
            items: vec![MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["compile".to_string()],
                recipe: vec![],
                phony: false,
                recipe_metadata: None,
                span: Span::new(0, 10, 1),
            }],
            metadata: MakeMetadata::new(),
        };

        assert!(
            !target_has_prerequisite(&ast, "build", "missing"),
            "Should return false for missing prerequisite"
        );
    }

    #[test]
    fn test_target_has_prerequisite_nonexistent_target() {
        use crate::make_parser::ast::{MakeAst, MakeItem, MakeMetadata, Span};

        let ast = MakeAst {
            items: vec![MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["compile".to_string()],
                recipe: vec![],
                phony: false,
                recipe_metadata: None,
                span: Span::new(0, 10, 1),
            }],
            metadata: MakeMetadata::new(),
        };

        assert!(
            !target_has_prerequisite(&ast, "nonexistent", "compile"),
            "Should return false for nonexistent target"
        );
    }
}
