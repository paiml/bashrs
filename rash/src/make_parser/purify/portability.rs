// Portability analysis for Makefiles (Sprint 83 - Day 7)
//
// Detects non-portable constructs: bashisms, platform-specific commands,
// shell-specific features, GNU extension flags, non-portable echo/sed usage.

use super::{MakeAst, MakeItem, Transformation};

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

pub(super) fn analyze_portability(ast: &MakeAst) -> Vec<Transformation> {
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
