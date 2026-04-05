#![allow(clippy::unwrap_used)]

use super::*;

// ---------------------------------------------------------------------------
// Unit tests for helper functions
// ---------------------------------------------------------------------------

#[test]
fn test_pmat209_parse_extensions_basic() {
    let exts = parse_extensions("sh,bash");
    assert_eq!(exts, vec!["sh", "bash"]);
}

#[test]
fn test_pmat209_parse_extensions_whitespace() {
    let exts = parse_extensions(" sh , bash , zsh ");
    assert_eq!(exts, vec!["sh", "bash", "zsh"]);
}

#[test]
fn test_pmat209_parse_extensions_empty() {
    let exts = parse_extensions("");
    assert!(exts.is_empty());
}

#[test]
fn test_pmat209_parse_extensions_case_insensitive() {
    let exts = parse_extensions("SH,Bash,ZSH");
    assert_eq!(exts, vec!["sh", "bash", "zsh"]);
}

#[test]
fn test_pmat209_parse_extensions_with_makefile() {
    let exts = parse_extensions("sh,makefile,dockerfile");
    assert_eq!(exts, vec!["sh", "makefile", "dockerfile"]);
}

#[test]
fn test_pmat209_parse_extensions_trailing_comma() {
    let exts = parse_extensions("sh,bash,");
    assert_eq!(exts, vec!["sh", "bash"]);
}

// ---------------------------------------------------------------------------
// matches_extensions tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat209_matches_extensions_by_extension() {
    let exts = vec!["sh".to_string(), "bash".to_string()];
    assert!(matches_extensions(Path::new("script.sh"), &exts));
    assert!(matches_extensions(Path::new("deploy.bash"), &exts));
    assert!(!matches_extensions(Path::new("main.rs"), &exts));
}

#[test]
fn test_pmat209_matches_extensions_by_filename() {
    let exts = vec!["sh".to_string(), "makefile".to_string()];
    assert!(matches_extensions(Path::new("Makefile"), &exts));
    assert!(matches_extensions(Path::new("/project/Makefile"), &exts));
}

#[test]
fn test_pmat209_matches_extensions_dockerfile() {
    let exts = vec!["dockerfile".to_string()];
    assert!(matches_extensions(Path::new("Dockerfile"), &exts));
    assert!(!matches_extensions(Path::new("docker-compose.yml"), &exts));
}

#[test]
fn test_pmat209_matches_extensions_case_insensitive() {
    let exts = vec!["sh".to_string()];
    assert!(matches_extensions(Path::new("script.SH"), &exts));
    assert!(matches_extensions(Path::new("script.Sh"), &exts));
}

#[test]
fn test_pmat209_matches_extensions_no_extension() {
    let exts = vec!["sh".to_string()];
    assert!(!matches_extensions(Path::new("README"), &exts));
}

#[test]
fn test_pmat209_matches_extensions_empty_list() {
    let exts: Vec<String> = vec![];
    assert!(!matches_extensions(Path::new("script.sh"), &exts));
}

// ---------------------------------------------------------------------------
// collect_files tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat209_collect_files_single_file() {
    let tmp = tempfile::NamedTempFile::with_suffix(".sh").unwrap();
    let path = tmp.path().to_path_buf();
    let exts = vec!["sh".to_string()];
    let files = collect_files(&[path.clone()], &exts);
    assert_eq!(files, vec![path]);
}

#[test]
fn test_pmat209_collect_files_wrong_extension() {
    let tmp = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    let path = tmp.path().to_path_buf();
    let exts = vec!["sh".to_string()];
    let files = collect_files(&[path], &exts);
    assert!(files.is_empty());
}

#[test]
fn test_pmat209_collect_files_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.sh"), "#!/bin/sh\necho a").unwrap();
    std::fs::write(dir.path().join("b.sh"), "#!/bin/sh\necho b").unwrap();
    std::fs::write(dir.path().join("c.rs"), "fn main() {}").unwrap();

    let exts = vec!["sh".to_string()];
    let files = collect_files(&[dir.path().to_path_buf()], &exts);
    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|f| f.extension().unwrap() == "sh"));
}

#[test]
fn test_pmat209_collect_files_deduplicates() {
    let tmp = tempfile::NamedTempFile::with_suffix(".sh").unwrap();
    let path = tmp.path().to_path_buf();
    let exts = vec!["sh".to_string()];
    let files = collect_files(&[path.clone(), path.clone()], &exts);
    assert_eq!(files.len(), 1);
}

// ---------------------------------------------------------------------------
// command_name tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat209_command_name_all_variants() {
    assert_eq!(command_name(&WatchCommand::Lint), "lint");
    assert_eq!(command_name(&WatchCommand::Format), "format");
    assert_eq!(command_name(&WatchCommand::Test), "test");
    assert_eq!(command_name(&WatchCommand::Score), "score");
    assert_eq!(command_name(&WatchCommand::SafetyCheck), "safety-check");
    assert_eq!(command_name(&WatchCommand::Audit), "audit");
}
