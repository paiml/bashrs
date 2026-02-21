//! Artifact discovery across project, user, and system scopes

use super::config::Scope;
use std::path::{Path, PathBuf};

/// A discovered shell artifact
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub path: PathBuf,
    pub scope: Scope,
    pub kind: ArtifactKind,
}

/// Classification of shell artifacts
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    ShellScript,
    Makefile,
    Dockerfile,
    ShellConfig,
    Workflow,
    DevContainer,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactKind::ShellScript => write!(f, "shell"),
            ArtifactKind::Makefile => write!(f, "makefile"),
            ArtifactKind::Dockerfile => write!(f, "dockerfile"),
            ArtifactKind::ShellConfig => write!(f, "config"),
            ArtifactKind::Workflow => write!(f, "workflow"),
            ArtifactKind::DevContainer => write!(f, "devcontainer"),
        }
    }
}

impl Artifact {
    pub fn new(path: PathBuf, scope: Scope, kind: ArtifactKind) -> Self {
        Self { path, scope, kind }
    }

    /// Display name relative to project or with ~ for user scope
    pub fn display_name(&self) -> String {
        match self.scope {
            Scope::User => {
                if let Some(home) = dirs_home() {
                    if let Ok(rel) = self.path.strip_prefix(&home) {
                        return format!("~/{}", rel.display());
                    }
                }
                self.path.display().to_string()
            }
            _ => self.path.display().to_string(),
        }
    }
}

/// Project scope glob patterns for artifact discovery
const PROJECT_PATTERNS: &[(&str, ArtifactKind)] = &[
    ("*.sh", ArtifactKind::ShellScript),
    ("scripts/**/*.sh", ArtifactKind::ShellScript),
    ("bin/**/*.sh", ArtifactKind::ShellScript),
    ("hooks/**/*.sh", ArtifactKind::ShellScript),
    ("Makefile", ArtifactKind::Makefile),
    ("makefile", ArtifactKind::Makefile),
    ("GNUmakefile", ArtifactKind::Makefile),
    ("*.mk", ArtifactKind::Makefile),
    ("Dockerfile", ArtifactKind::Dockerfile),
    ("Dockerfile.*", ArtifactKind::Dockerfile),
    ("docker-compose.yml", ArtifactKind::Workflow),
    ("docker-compose.yaml", ArtifactKind::Workflow),
    (".github/workflows/*.yml", ArtifactKind::Workflow),
    (".github/workflows/*.yaml", ArtifactKind::Workflow),
    (
        ".devcontainer/devcontainer.json",
        ArtifactKind::DevContainer,
    ),
];

/// User scope known paths
const USER_CONFIGS: &[&str] = &[
    ".zshrc",
    ".bashrc",
    ".bash_profile",
    ".profile",
    ".zprofile",
    ".zshenv",
    ".zlogout",
    ".bash_logout",
];

/// System scope known paths (read-only audit)
const SYSTEM_CONFIGS: &[&str] = &[
    "/etc/profile",
    "/etc/bash.bashrc",
    "/etc/zsh/zshrc",
    "/etc/zsh/zshenv",
    "/etc/environment",
];

/// Discover artifacts in a given scope
pub fn discover(project_path: &Path, scope: Scope) -> Vec<Artifact> {
    match scope {
        Scope::Project => discover_project(project_path),
        Scope::User => discover_user(),
        Scope::System => discover_system(),
    }
}

/// Discover all artifacts across all scopes
pub fn discover_all(project_path: &Path) -> Vec<Artifact> {
    let mut artifacts = discover_project(project_path);
    artifacts.extend(discover_user());
    artifacts.extend(discover_system());
    artifacts
}

fn discover_project(project_path: &Path) -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    for (pattern, kind) in PROJECT_PATTERNS {
        let full_pattern = project_path.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();
        if let Ok(paths) = glob::glob(&pattern_str) {
            for entry in paths.flatten() {
                if entry.is_file() {
                    // Store relative path for project scope
                    let rel = entry
                        .strip_prefix(project_path)
                        .unwrap_or(&entry)
                        .to_path_buf();
                    let artifact = Artifact::new(rel, Scope::Project, *kind);
                    if !artifacts.iter().any(|a: &Artifact| a.path == artifact.path) {
                        artifacts.push(artifact);
                    }
                }
            }
        }
    }

    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    artifacts
}

fn discover_user() -> Vec<Artifact> {
    let mut artifacts = Vec::new();
    let home = match dirs_home() {
        Some(h) => h,
        None => return artifacts,
    };

    for config_name in USER_CONFIGS {
        let path = home.join(config_name);
        if path.exists() {
            artifacts.push(Artifact::new(path, Scope::User, ArtifactKind::ShellConfig));
        }
    }

    // pzsh config
    let pzsh_config = home.join(".config/pzsh/config.toml");
    if pzsh_config.exists() {
        artifacts.push(Artifact::new(
            pzsh_config,
            Scope::User,
            ArtifactKind::ShellConfig,
        ));
    }

    artifacts
}

fn discover_system() -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    for path_str in SYSTEM_CONFIGS {
        let path = PathBuf::from(path_str);
        if path.exists() {
            artifacts.push(Artifact::new(
                path,
                Scope::System,
                ArtifactKind::ShellConfig,
            ));
        }
    }

    artifacts
}

/// Classify a file path into an artifact kind
pub fn classify(path: &Path) -> Option<ArtifactKind> {
    let name = path.file_name()?.to_str()?;
    let name_lower = name.to_lowercase();

    classify_by_name(&name_lower, name, path).or_else(|| classify_by_shebang(path))
}

/// Classify by filename patterns
fn classify_by_name(name_lower: &str, name: &str, path: &Path) -> Option<ArtifactKind> {
    // Build tools
    if name_lower == "makefile" || name_lower == "gnumakefile" || name_lower.ends_with(".mk") {
        return Some(ArtifactKind::Makefile);
    }
    if name_lower.starts_with("dockerfile") {
        return Some(ArtifactKind::Dockerfile);
    }
    if name_lower == "devcontainer.json" {
        return Some(ArtifactKind::DevContainer);
    }
    // Workflow files (YAML in known locations)
    if is_workflow_yaml(name_lower, path) {
        return Some(ArtifactKind::Workflow);
    }
    // Shell scripts by extension
    if name_lower.ends_with(".sh") || name_lower.ends_with(".bash") {
        return Some(ArtifactKind::ShellScript);
    }
    // User shell config files
    const CONFIG_NAMES: &[&str] = &[
        ".zshrc",
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".zprofile",
        ".zshenv",
    ];
    if CONFIG_NAMES.contains(&name) {
        return Some(ArtifactKind::ShellConfig);
    }
    None
}

fn is_workflow_yaml(name_lower: &str, path: &Path) -> bool {
    (name_lower.ends_with(".yml") || name_lower.ends_with(".yaml"))
        && (path.to_string_lossy().contains(".github/workflows")
            || name_lower.starts_with("docker-compose"))
}

/// Classify by shebang line for scripts without extension
fn classify_by_shebang(path: &Path) -> Option<ArtifactKind> {
    let content = std::fs::read_to_string(path).ok()?;
    const SHELL_SHEBANGS: &[&str] = &[
        "#!/bin/sh",
        "#!/bin/bash",
        "#!/usr/bin/env bash",
        "#!/usr/bin/env sh",
    ];
    if SHELL_SHEBANGS.iter().any(|s| content.starts_with(s)) {
        return Some(ArtifactKind::ShellScript);
    }
    None
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Check if pzsh is available on this system
pub fn detect_pzsh() -> Option<PzshInfo> {
    let output = std::process::Command::new("pzsh")
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version = version_str.trim().strip_prefix("pzsh ")?.to_string();

    Some(PzshInfo { version })
}

/// pzsh integration info
#[derive(Clone, Debug)]
pub struct PzshInfo {
    pub version: String,
}
