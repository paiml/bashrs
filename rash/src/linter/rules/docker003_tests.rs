//! Tests for DOCKER003: Missing apt-get cleanup
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::docker003;

#[test]
fn test_docker003_apt_install_without_cleanup() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\n";
    let result = docker003::check(source);
    assert!(!result.diagnostics.is_empty(), "Should flag apt-get install without cleanup");
}

#[test]
fn test_docker003_apt_install_with_cleanup() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\n";
    let result = docker003::check(source);
    assert!(result.diagnostics.is_empty(), "Should not flag with cleanup present");
}

#[test]
fn test_docker003_no_apt_install() {
    let source = "FROM ubuntu:22.04\nRUN echo hello\n";
    let result = docker003::check(source);
    assert!(result.diagnostics.is_empty(), "No apt-get install means no violation");
}

#[test]
fn test_docker003_multiline_run_with_apt_no_cleanup() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update \\\n    && apt-get install -y curl\n";
    let result = docker003::check(source);
    assert!(!result.diagnostics.is_empty(), "Multiline RUN without cleanup should be flagged");
}

#[test]
fn test_docker003_multiline_run_with_cleanup() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update \\\n    && apt-get install -y curl \\\n    && rm -rf /var/lib/apt/lists/*\n";
    let result = docker003::check(source);
    assert!(result.diagnostics.is_empty(), "Multiline RUN with cleanup is fine");
}

#[test]
fn test_docker003_multiple_run_commands() {
    let source = "FROM ubuntu:22.04\nRUN echo setup\nRUN apt-get install -y git\nRUN echo done\n";
    let result = docker003::check(source);
    assert!(!result.diagnostics.is_empty(), "Second RUN with apt-get should be flagged");
}

#[test]
fn test_docker003_apt_in_single_line_no_continuation() {
    let source = "FROM ubuntu:22.04\nRUN apt-get install -y python3\n";
    let result = docker003::check(source);
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn test_docker003_empty_dockerfile() {
    let source = "";
    let result = docker003::check(source);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_docker003_comment_only() {
    let source = "# just a comment\n";
    let result = docker003::check(source);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_docker003_diagnostic_message() {
    let source = "RUN apt-get install -y curl\n";
    let result = docker003::check(source);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("apt-get install"));
    assert!(result.diagnostics[0].message.contains("cleanup"));
}

#[test]
fn test_docker003_run_at_end_no_newline() {
    let source = "FROM ubuntu:22.04\nRUN apt-get update \\\n    && apt-get install -y vim";
    let result = docker003::check(source);
    assert!(!result.diagnostics.is_empty(), "Trailing multiline RUN should be checked");
}
