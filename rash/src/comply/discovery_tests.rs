#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::discovery::*;
use super::config::Scope;
use std::path::{Path, PathBuf};

// ============================================================================
// ArtifactKind Display coverage
// ============================================================================

#[test]
fn test_artifact_kind_display_shell_script() {
    assert_eq!(format!("{}", ArtifactKind::ShellScript), "shell");
}

#[test]
fn test_artifact_kind_display_makefile() {
    assert_eq!(format!("{}", ArtifactKind::Makefile), "makefile");
}

#[test]
fn test_artifact_kind_display_dockerfile() {
    assert_eq!(format!("{}", ArtifactKind::Dockerfile), "dockerfile");
}

#[test]
fn test_artifact_kind_display_shell_config() {
    assert_eq!(format!("{}", ArtifactKind::ShellConfig), "config");
}

#[test]
fn test_artifact_kind_display_workflow() {
    assert_eq!(format!("{}", ArtifactKind::Workflow), "workflow");
}

#[test]
fn test_artifact_kind_display_devcontainer() {
    assert_eq!(format!("{}", ArtifactKind::DevContainer), "devcontainer");
}

// ============================================================================
// Artifact construction and equality
// ============================================================================

#[test]
fn test_artifact_new() {
    let a = Artifact::new(
        PathBuf::from("scripts/deploy.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    assert_eq!(a.path, PathBuf::from("scripts/deploy.sh"));
    assert_eq!(a.scope, Scope::Project);
    assert_eq!(a.kind, ArtifactKind::ShellScript);
}

#[test]
fn test_artifact_eq() {
    let a1 = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let a2 = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    assert_eq!(a1, a2);
}

#[test]
fn test_artifact_ne_different_path() {
    let a1 = Artifact::new(
        PathBuf::from("a.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let a2 = Artifact::new(
        PathBuf::from("b.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    assert_ne!(a1, a2);
}

#[test]
fn test_artifact_ne_different_scope() {
    let a1 = Artifact::new(
        PathBuf::from("a.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let a2 = Artifact::new(
        PathBuf::from("a.sh"),
        Scope::User,
        ArtifactKind::ShellScript,
    );
    assert_ne!(a1, a2);
}

#[test]
fn test_artifact_ne_different_kind() {
    let a1 = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let a2 = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    assert_ne!(a1, a2);
}

#[test]
fn test_artifact_clone() {
    let a = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_artifact_debug() {
    let a = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let debug = format!("{a:?}");
    assert!(debug.contains("test.sh"));
    assert!(debug.contains("ShellScript"));
}

// ============================================================================
// Artifact::display_name
// ============================================================================

#[test]
fn test_display_name_project_scope() {
    let a = Artifact::new(
        PathBuf::from("scripts/deploy.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    assert_eq!(a.display_name(), "scripts/deploy.sh");
}

#[test]
fn test_display_name_system_scope() {
    let a = Artifact::new(
        PathBuf::from("/etc/profile"),
        Scope::System,
        ArtifactKind::ShellConfig,
    );
    assert_eq!(a.display_name(), "/etc/profile");
}

#[test]
fn test_display_name_user_scope_with_home() {
    // This test depends on HOME being set, which it almost always is
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(&home).join(".bashrc");
        let a = Artifact::new(path.clone(), Scope::User, ArtifactKind::ShellConfig);
        let display = a.display_name();
        assert!(
            display.starts_with("~/"),
            "expected ~/ prefix, got: {display}"
        );
        assert!(display.contains(".bashrc"));
    }
}

#[test]
fn test_display_name_user_scope_without_home_prefix() {
    // Path not under HOME falls back to full path display
    let a = Artifact::new(
        PathBuf::from("/opt/custom/.zshrc"),
        Scope::User,
        ArtifactKind::ShellConfig,
    );
    let display = a.display_name();
    assert_eq!(display, "/opt/custom/.zshrc");
}

// ============================================================================
// classify: filename-based classification
// ============================================================================

#[test]
fn test_classify_sh_extension() {
    let kind = classify(Path::new("deploy.sh"));
    assert_eq!(kind, Some(ArtifactKind::ShellScript));
}

#[test]
fn test_classify_bash_extension() {
    let kind = classify(Path::new("setup.bash"));
    assert_eq!(kind, Some(ArtifactKind::ShellScript));
}

#[test]
fn test_classify_makefile() {
    assert_eq!(classify(Path::new("Makefile")), Some(ArtifactKind::Makefile));
}

#[test]
fn test_classify_makefile_lowercase() {
    assert_eq!(classify(Path::new("makefile")), Some(ArtifactKind::Makefile));
}

#[test]
fn test_classify_gnumakefile() {
    assert_eq!(
        classify(Path::new("GNUmakefile")),
        Some(ArtifactKind::Makefile)
    );
}

#[test]
fn test_classify_mk_extension() {
    assert_eq!(classify(Path::new("rules.mk")), Some(ArtifactKind::Makefile));
}

#[test]
fn test_classify_dockerfile() {
    assert_eq!(
        classify(Path::new("Dockerfile")),
        Some(ArtifactKind::Dockerfile)
    );
}

#[test]
fn test_classify_dockerfile_with_suffix() {
    assert_eq!(
        classify(Path::new("Dockerfile.prod")),
        Some(ArtifactKind::Dockerfile)
    );
}

#[test]
fn test_classify_devcontainer_json() {
    assert_eq!(
        classify(Path::new("devcontainer.json")),
        Some(ArtifactKind::DevContainer)
    );
}

#[test]
fn test_classify_docker_compose_yml() {
    assert_eq!(
        classify(Path::new("docker-compose.yml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_docker_compose_yaml() {
    assert_eq!(
        classify(Path::new("docker-compose.yaml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_github_workflow() {
    assert_eq!(
        classify(Path::new(".github/workflows/ci.yml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_github_workflow_yaml() {
    assert_eq!(
        classify(Path::new(".github/workflows/deploy.yaml")),
        Some(ArtifactKind::Workflow)
    );
}

#[test]
fn test_classify_shell_configs() {
    assert_eq!(
        classify(Path::new(".zshrc")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(Path::new(".bashrc")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(Path::new(".bash_profile")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(Path::new(".profile")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(Path::new(".zprofile")),
        Some(ArtifactKind::ShellConfig)
    );
    assert_eq!(
        classify(Path::new(".zshenv")),
        Some(ArtifactKind::ShellConfig)
    );
}

#[test]
fn test_classify_unknown_file() {
    // A .txt file with no shebang (file doesn't exist on disk)
    assert_eq!(classify(Path::new("notes.txt")), None);
}

#[test]
fn test_classify_no_filename() {
    // Root path has no file_name
    // On Unix, "/" has no file_name -> returns None
    // But Path::new("") has no file_name either
    let result = classify(Path::new(""));
    assert!(result.is_none());
}

#[test]
fn test_classify_yaml_not_in_workflow_location() {
    // A .yml file NOT in .github/workflows and not docker-compose -> not a workflow
    let result = classify(Path::new("config.yml"));
    assert!(result.is_none());
}

// ============================================================================
// ArtifactKind copy and clone
// ============================================================================

#[test]
fn test_artifact_kind_copy() {
    let kind = ArtifactKind::ShellScript;
    let kind2 = kind; // Copy
    assert_eq!(kind, kind2);
}

#[test]
fn test_artifact_kind_clone() {
    let kind = ArtifactKind::Makefile;
    let kind2 = kind.clone();
    assert_eq!(kind, kind2);
}

#[test]
fn test_artifact_kind_eq() {
    assert_eq!(ArtifactKind::Dockerfile, ArtifactKind::Dockerfile);
    assert_ne!(ArtifactKind::Dockerfile, ArtifactKind::Makefile);
}

// ============================================================================
// Scope Display coverage
// ============================================================================

#[test]
fn test_scope_display_project() {
    assert_eq!(format!("{}", Scope::Project), "project");
}

#[test]
fn test_scope_display_user() {
    assert_eq!(format!("{}", Scope::User), "user");
}

#[test]
fn test_scope_display_system() {
    assert_eq!(format!("{}", Scope::System), "system");
}

// ============================================================================
// discover function with nonexistent project path
// ============================================================================

#[test]
fn test_discover_project_nonexistent_path() {
    let artifacts = discover(Path::new("/nonexistent/path/12345"), Scope::Project);
    assert!(artifacts.is_empty());
}

#[test]
fn test_discover_system_scope() {
    // System scope checks known paths like /etc/profile
    let artifacts = discover(Path::new("/nonexistent"), Scope::System);
    // May or may not find system files depending on the environment,
    // but should not panic
    for a in &artifacts {
        assert_eq!(a.scope, Scope::System);
        assert_eq!(a.kind, ArtifactKind::ShellConfig);
    }
}

#[test]
fn test_discover_user_scope() {
    let artifacts = discover(Path::new("/nonexistent"), Scope::User);
    // All user artifacts should be ShellConfig with User scope
    for a in &artifacts {
        assert_eq!(a.scope, Scope::User);
        assert_eq!(a.kind, ArtifactKind::ShellConfig);
    }
}

#[test]
fn test_discover_all_nonexistent_path() {
    let artifacts = discover_all(Path::new("/nonexistent/path/12345"));
    // Should include user + system artifacts even though project has none
    // No panic expected
    for a in &artifacts {
        assert!(
            a.scope == Scope::Project
                || a.scope == Scope::User
                || a.scope == Scope::System
        );
    }
}

// ============================================================================
// PzshInfo struct
// ============================================================================

#[test]
fn test_pzsh_info_debug_and_clone() {
    let info = PzshInfo {
        version: "1.0.0".to_string(),
    };
    let info2 = info.clone();
    assert_eq!(info.version, info2.version);
    let debug = format!("{info:?}");
    assert!(debug.contains("1.0.0"));
}

// ============================================================================
// classify_by_name edge cases
// ============================================================================

#[test]
fn test_classify_uppercase_makefile_variant() {
    // "MAKEFILE" lowercases to "makefile"
    assert_eq!(
        classify(Path::new("MAKEFILE")),
        Some(ArtifactKind::Makefile)
    );
}

#[test]
fn test_classify_dockerfile_variant_case() {
    // "dockerfile" (lowercase) starts_with "dockerfile"
    assert_eq!(
        classify(Path::new("dockerfile")),
        Some(ArtifactKind::Dockerfile)
    );
}

#[test]
fn test_classify_dockerfile_multi_stage() {
    assert_eq!(
        classify(Path::new("Dockerfile.build")),
        Some(ArtifactKind::Dockerfile)
    );
}
