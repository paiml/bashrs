// Parallel safety analysis for Makefiles (Sprint 83)
//
// Detects race conditions, missing dependencies, recursive make calls,
// and directory creation races in parallel builds.

use super::{MakeAst, MakeItem, Transformation};

/// Check if Makefile has .NOTPARALLEL directive
pub(super) fn has_notparallel_directive(ast: &MakeAst) -> bool {
    ast.items
        .iter()
        .any(|item| matches!(item, MakeItem::Target { name, .. } if name == ".NOTPARALLEL"))
}

/// Collect all targets for analysis
pub(super) fn collect_targets(ast: &MakeAst) -> Vec<(&String, &Vec<String>)> {
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
pub(super) fn try_extract_output_redirect(recipe: &str) -> Option<String> {
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
pub(super) fn try_extract_cat_input(recipe: &str) -> Option<String> {
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
pub(super) fn is_automatic_variable(filename: &str) -> bool {
    matches!(filename, "$<" | "$@" | "$^" | "$?" | "$*" | "$+")
}

/// Check if target has a specific prerequisite
pub(super) fn target_has_prerequisite(ast: &MakeAst, target_name: &str, prerequisite: &str) -> bool {
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
pub(super) fn detect_missing_file_dependencies(
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

pub(super) fn analyze_parallel_safety(ast: &MakeAst) -> Vec<Transformation> {
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
