// CLI Logic - Dockerfile Processing
//
// Dockerfile purification, analysis, and build-related helper functions.

use crate::models::{Error, Result};
use std::path::{Path, PathBuf};

// =============================================================================
// DOCKERFILE PURIFICATION LOGIC
// =============================================================================

/// Convert ADD to COPY for local files (DOCKER006)
///
/// Keep ADD for:
/// - URLs (http://, https://)
/// - Tarballs (.tar, .tar.gz, .tgz, .tar.bz2, .tar.xz) which ADD auto-extracts
pub fn convert_add_to_copy_if_local(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Must be an ADD directive
    if !trimmed.starts_with("ADD ") {
        return line.to_string();
    }

    // Extract the source path (first argument after ADD)
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let source = match parts.get(1) {
        Some(s) => *s,
        None => return line.to_string(), // Malformed ADD directive
    };

    // Check if source is a URL
    if source.starts_with("http://") || source.starts_with("https://") {
        return line.to_string(); // Keep ADD for URLs
    }

    // Check if source is a tarball (which ADD auto-extracts)
    let is_tarball = source.ends_with(".tar")
        || source.ends_with(".tar.gz")
        || source.ends_with(".tgz")
        || source.ends_with(".tar.bz2")
        || source.ends_with(".tar.xz")
        || source.ends_with(".tar.Z");

    if is_tarball {
        return line.to_string(); // Keep ADD for tarballs
    }

    // It's a local file - convert ADD to COPY
    line.replacen("ADD ", "COPY ", 1)
}

/// Add --no-install-recommends flag to apt-get install commands (DOCKER005)
pub fn add_no_install_recommends(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Check if already has --no-install-recommends
    if line.contains("--no-install-recommends") {
        return line.to_string();
    }

    // Must contain apt-get install
    if !line.contains("apt-get install") {
        return line.to_string();
    }

    let mut result = line.to_string();

    // Replace "apt-get install -y " (with -y flag)
    result = result.replace(
        "apt-get install -y ",
        "apt-get install -y --no-install-recommends ",
    );

    // Replace remaining "apt-get install "
    if !result.contains("--no-install-recommends") {
        result = result.replace(
            "apt-get install ",
            "apt-get install --no-install-recommends ",
        );
    }

    // Handle edge case: "apt-get install" at end of line (no trailing space)
    if !result.contains("--no-install-recommends") && result.trim_end().ends_with("apt-get install")
    {
        result = result.trim_end().to_string() + " --no-install-recommends ";
    }

    result
}

/// Add cleanup commands for package managers (DOCKER003)
pub fn add_package_manager_cleanup(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Check if cleanup already present
    if line.contains("/var/lib/apt/lists") || line.contains("/var/cache/apk") {
        return line.to_string();
    }

    // Detect apt/apt-get commands
    if line.contains("apt-get install") || line.contains("apt install") {
        return format!("{} && rm -rf /var/lib/apt/lists/*", line.trim_end());
    }

    // Detect apk commands
    if line.contains("apk add") {
        return format!("{} && rm -rf /var/cache/apk/*", line.trim_end());
    }

    line.to_string()
}

/// Pin unpinned base images to stable versions (DOCKER002)
pub fn pin_base_image_version(line: &str) -> String {
    let trimmed = line.trim();

    // Must be a FROM line
    if !trimmed.starts_with("FROM ") {
        return line.to_string();
    }

    // Parse FROM line
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let image_part = match parts.get(1) {
        Some(img) => *img,
        None => return line.to_string(), // Malformed FROM line
    };

    // Parse registry prefix
    let (registry_prefix, image_with_tag) = if let Some(slash_pos) = image_part.find('/') {
        let prefix_part = &image_part[..slash_pos];
        if prefix_part.contains('.') || prefix_part == "localhost" {
            (Some(prefix_part), &image_part[slash_pos + 1..])
        } else {
            (None, image_part)
        }
    } else {
        (None, image_part)
    };

    // Split image into name and tag
    let (image_name, tag) = if let Some(colon_pos) = image_with_tag.find(':') {
        let name = &image_with_tag[..colon_pos];
        let tag = &image_with_tag[colon_pos + 1..];
        (name, Some(tag))
    } else {
        (image_with_tag, None)
    };

    // Determine if pinning is needed
    let needs_pinning = tag.is_none() || tag == Some("latest");

    if !needs_pinning {
        return line.to_string(); // Already has specific version
    }

    // Map common images to stable versions
    let pinned_tag = match image_name {
        "ubuntu" => "22.04",
        "debian" => "12-slim",
        "alpine" => "3.19",
        "node" => "20-alpine",
        "python" => "3.11-slim",
        "rust" => "1.75-alpine",
        "nginx" => "1.25-alpine",
        "postgres" => "16-alpine",
        "redis" => "7-alpine",
        _ => return line.to_string(), // Unknown image
    };

    // Reconstruct FROM line with pinned version
    let pinned_image = if let Some(prefix) = registry_prefix {
        format!("{}/{}:{}", prefix, image_name, pinned_tag)
    } else {
        format!("{}:{}", image_name, pinned_tag)
    };

    // Preserve "AS <name>" if present
    if parts.len() > 2 {
        let rest = parts.get(2..).map(|s| s.join(" ")).unwrap_or_default();
        format!("FROM {} {}", pinned_image, rest)
    } else {
        format!("FROM {}", pinned_image)
    }
}

/// Purify a Dockerfile source (pure function - no I/O)
///
/// Applies the following transformations:
/// - DOCKER002: Pin unpinned base images to stable versions
/// - DOCKER003: Add package manager cleanup
/// - DOCKER005: Add --no-install-recommends to apt-get
/// - DOCKER006: Convert ADD to COPY for local files
/// - Add non-root USER directive before CMD/ENTRYPOINT
pub fn purify_dockerfile_source(source: &str, skip_user: bool) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut purified = Vec::new();

    // Check if USER directive already exists
    let has_user = lines.iter().any(|line| line.trim().starts_with("USER "));
    let is_scratch = lines
        .iter()
        .any(|line| line.trim().starts_with("FROM scratch"));

    // Find CMD/ENTRYPOINT position
    let cmd_pos = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("CMD ") || trimmed.starts_with("ENTRYPOINT ")
    });

    // Build purified Dockerfile
    for (i, line) in lines.iter().enumerate() {
        // Check if we should add USER before CMD/ENTRYPOINT
        if !skip_user && !has_user && !is_scratch && Some(i) == cmd_pos {
            purified.push(String::new());
            purified.push("# Security: Run as non-root user".to_string());
            purified.push("RUN groupadd -r appuser && useradd -r -g appuser appuser".to_string());
            purified.push("USER appuser".to_string());
            purified.push(String::new());
        }

        // DOCKER002: Pin unpinned base images
        let mut processed_line = if line.trim().starts_with("FROM ") {
            pin_base_image_version(line)
        } else {
            line.to_string()
        };

        // DOCKER006: Convert ADD to COPY for local files
        if line.trim().starts_with("ADD ") {
            processed_line = convert_add_to_copy_if_local(&processed_line);
        }

        // DOCKER005: Add --no-install-recommends to apt-get install
        if line.trim().starts_with("RUN ") && processed_line.contains("apt-get install") {
            processed_line = add_no_install_recommends(&processed_line);
        }

        // DOCKER003: Add apt/apk cleanup
        if line.trim().starts_with("RUN ") {
            processed_line = add_package_manager_cleanup(&processed_line);
        }

        purified.push(processed_line);
    }

    purified.join("\n")
}

/// Check if Dockerfile has USER directive
pub fn dockerfile_has_user_directive(source: &str) -> bool {
    source.lines().any(|line| line.trim().starts_with("USER "))
}

/// Check if Dockerfile uses scratch base
pub fn dockerfile_is_scratch(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim().starts_with("FROM scratch"))
}

/// Find CMD or ENTRYPOINT line number (0-indexed)
pub fn dockerfile_find_cmd_line(source: &str) -> Option<usize> {
    source.lines().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("CMD ") || trimmed.starts_with("ENTRYPOINT ")
    })
}

/// Find devcontainer.json in standard locations
pub fn find_devcontainer_json(path: &Path) -> Result<PathBuf> {
    // If path is a file, use it directly
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    // Search standard locations
    let candidates = [
        path.join(".devcontainer/devcontainer.json"),
        path.join(".devcontainer.json"),
    ];

    if let Some(found) = candidates.iter().find(|c| c.exists()) {
        return Ok(found.clone());
    }

    // Check for .devcontainer/<folder>/devcontainer.json
    if let Some(found) = find_devcontainer_in_subdirs(path) {
        return Ok(found);
    }

    Err(Error::Validation(format!(
        "No devcontainer.json found in {}. Expected locations:\n  \
         - .devcontainer/devcontainer.json\n  \
         - .devcontainer.json\n  \
         - .devcontainer/<folder>/devcontainer.json",
        path.display()
    )))
}

/// Search .devcontainer subdirectories for devcontainer.json
fn find_devcontainer_in_subdirs(path: &Path) -> Option<PathBuf> {
    let devcontainer_dir = path.join(".devcontainer");
    let entries = std::fs::read_dir(&devcontainer_dir).ok()?;
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().join("devcontainer.json"))
        .find(|c| c.exists())
}

/// Parse custom size limit from string (e.g., "2GB", "500MB")
pub fn parse_size_limit(s: &str) -> Option<u64> {
    let s = s.to_uppercase();
    if s.ends_with("GB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000_000.0) as u64)
    } else if s.ends_with("MB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000.0) as u64)
    } else if s.ends_with("KB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000.0) as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Estimate build time based on layer complexity
pub fn estimate_build_time_seconds(
    layer_count: usize,
    total_size: u64,
    has_apt: bool,
    has_pip: bool,
    has_npm: bool,
) -> u64 {
    let mut total_seconds = 0u64;

    // Base time for each layer
    total_seconds += layer_count as u64;

    // Add time based on size (1 second per 100MB)
    total_seconds += total_size / 100_000_000;

    // Add extra time for known slow operations
    if has_apt {
        total_seconds += 10;
    }
    if has_pip {
        total_seconds += 5;
    }
    if has_npm {
        total_seconds += 5;
    }

    total_seconds
}

/// Format build time as human-readable string
pub fn format_build_time(seconds: u64) -> String {
    if seconds < 60 {
        format!("~{}s", seconds)
    } else {
        format!("~{}m {}s", seconds / 60, seconds % 60)
    }
}

/// Parse size string like "10GB" or "500MB" into bytes
pub fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    if s.ends_with("GB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000_000.0) as u64)
    } else if s.ends_with("MB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000.0) as u64)
    } else if s.ends_with("KB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000.0) as u64)
    } else if s.ends_with('B') {
        s[..s.len() - 1].trim().parse::<u64>().ok()
    } else {
        // Try parsing as raw bytes
        s.parse::<u64>().ok()
    }
}

/// Format build time estimate from layer data (pure function)
pub fn format_build_time_estimate(
    layer_count: usize,
    total_size_bytes: u64,
    has_apt: bool,
    has_pip: bool,
    has_npm: bool,
) -> String {
    let seconds =
        estimate_build_time_seconds(layer_count, total_size_bytes, has_apt, has_pip, has_npm);
    if seconds < 60 {
        format!("~{}s", seconds)
    } else {
        format!("~{}m {}s", seconds / 60, seconds % 60)
    }
}

/// Check if size exceeds limit
pub fn size_exceeds_limit(size_bytes: u64, limit_bytes: u64) -> bool {
    size_bytes > limit_bytes
}

/// Calculate size percentage of limit
pub fn size_percentage_of_limit(size_bytes: u64, limit_bytes: u64) -> f64 {
    if limit_bytes == 0 {
        100.0
    } else {
        (size_bytes as f64 / limit_bytes as f64) * 100.0
    }
}

/// Determine if layer contains slow operation
pub fn layer_has_slow_operation(content: &str) -> (bool, bool, bool) {
    let lower = content.to_lowercase();
    (
        lower.contains("apt-get install") || lower.contains("apt install"),
        lower.contains("pip install") || lower.contains("pip3 install"),
        lower.contains("npm install") || lower.contains("yarn install"),
    )
}

/// Format size comparison for display
pub fn format_size_comparison(actual_bytes: u64, limit_bytes: u64) -> String {
    let actual_gb = actual_bytes as f64 / 1_000_000_000.0;
    let limit_gb = limit_bytes as f64 / 1_000_000_000.0;
    let percentage = size_percentage_of_limit(actual_bytes, limit_bytes);

    if actual_bytes > limit_bytes {
        format!("✗ EXCEEDS LIMIT: {:.2}GB > {:.0}GB", actual_gb, limit_gb)
    } else {
        format!(
            "✓ Within limit: {:.2}GB / {:.0}GB ({:.0}%)",
            actual_gb, limit_gb, percentage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== DOCKERFILE LOGIC TESTS =====

    #[test]
    fn test_convert_add_to_copy_local_file() {
        assert_eq!(
            convert_add_to_copy_if_local("ADD file.txt /app/"),
            "COPY file.txt /app/"
        );
    }

    #[test]
    fn test_convert_add_to_copy_preserves_url() {
        let line = "ADD https://example.com/file.tar.gz /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_convert_add_to_copy_preserves_tarball() {
        let line = "ADD archive.tar.gz /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);

        let line2 = "ADD data.tgz /app/";
        assert_eq!(convert_add_to_copy_if_local(line2), line2);
    }

    #[test]
    fn test_convert_add_to_copy_preserves_comment() {
        let line = "# ADD file.txt /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_convert_add_to_copy_non_add_line() {
        let line = "COPY file.txt /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_add_no_install_recommends() {
        assert_eq!(
            add_no_install_recommends("RUN apt-get install -y curl"),
            "RUN apt-get install -y --no-install-recommends curl"
        );
    }

    #[test]
    fn test_add_no_install_recommends_already_present() {
        let line = "RUN apt-get install -y --no-install-recommends curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_no_install_recommends_without_y() {
        assert_eq!(
            add_no_install_recommends("RUN apt-get install curl"),
            "RUN apt-get install --no-install-recommends curl"
        );
    }

    #[test]
    fn test_add_no_install_recommends_comment() {
        let line = "# apt-get install curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_no_install_recommends_non_apt() {
        let line = "RUN yum install curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_apt() {
        assert_eq!(
            add_package_manager_cleanup("RUN apt-get install -y curl"),
            "RUN apt-get install -y curl && rm -rf /var/lib/apt/lists/*"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apk() {
        assert_eq!(
            add_package_manager_cleanup("RUN apk add curl"),
            "RUN apk add curl && rm -rf /var/cache/apk/*"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_already_present() {
        let line = "RUN apt-get install curl && rm -rf /var/lib/apt/lists/*";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_comment() {
        let line = "# apt-get install curl";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_other_command() {
        let line = "RUN echo hello";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_pin_base_image_ubuntu() {
        assert_eq!(pin_base_image_version("FROM ubuntu"), "FROM ubuntu:22.04");
    }

    #[test]
    fn test_pin_base_image_latest() {
        assert_eq!(
            pin_base_image_version("FROM alpine:latest"),
            "FROM alpine:3.19"
        );
    }

    #[test]
    fn test_pin_base_image_already_pinned() {
        let line = "FROM python:3.9";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_pin_base_image_with_as() {
        assert_eq!(
            pin_base_image_version("FROM node AS builder"),
            "FROM node:20-alpine AS builder"
        );
    }

    #[test]
    fn test_pin_base_image_with_registry() {
        assert_eq!(
            pin_base_image_version("FROM docker.io/ubuntu"),
            "FROM docker.io/ubuntu:22.04"
        );
    }

    #[test]
    fn test_pin_base_image_unknown() {
        let line = "FROM mycompany/myimage";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_pin_base_image_non_from_line() {
        let line = "RUN echo hello";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_parse_size_limit_gb() {
        assert_eq!(parse_size_limit("2GB"), Some(2_000_000_000));
        assert_eq!(parse_size_limit("1.5gb"), Some(1_500_000_000));
    }

    #[test]
    fn test_parse_size_limit_mb() {
        assert_eq!(parse_size_limit("500MB"), Some(500_000_000));
        assert_eq!(parse_size_limit("100mb"), Some(100_000_000));
    }

    #[test]
    fn test_parse_size_limit_kb() {
        assert_eq!(parse_size_limit("1000KB"), Some(1_000_000));
    }

    #[test]
    fn test_parse_size_limit_invalid() {
        assert_eq!(parse_size_limit("invalid"), None);
        assert_eq!(parse_size_limit(""), None);
    }

    #[test]
    fn test_estimate_build_time_basic() {
        let seconds = estimate_build_time_seconds(5, 0, false, false, false);
        assert_eq!(seconds, 5); // 1 second per layer
    }

    #[test]
    fn test_estimate_build_time_with_size() {
        let seconds = estimate_build_time_seconds(2, 500_000_000, false, false, false);
        assert_eq!(seconds, 2 + 5); // 2 layers + 5 seconds for 500MB
    }

    #[test]
    fn test_estimate_build_time_with_package_managers() {
        let seconds = estimate_build_time_seconds(1, 0, true, true, true);
        assert_eq!(seconds, 1 + 10 + 5 + 5); // 1 layer + apt + pip + npm
    }

    #[test]
    fn test_format_build_time_seconds() {
        assert_eq!(format_build_time(30), "~30s");
        assert_eq!(format_build_time(59), "~59s");
    }

    #[test]
    fn test_format_build_time_minutes() {
        assert_eq!(format_build_time(60), "~1m 0s");
        assert_eq!(format_build_time(90), "~1m 30s");
        assert_eq!(format_build_time(125), "~2m 5s");
    }

    #[test]
    fn test_find_devcontainer_json_not_found() {
        let result = find_devcontainer_json(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No devcontainer.json found"));
    }

    // ===== DOCKERFILE PURIFICATION TESTS =====

    #[test]
    fn test_purify_dockerfile_source_basic() {
        let source = "FROM ubuntu\nRUN apt-get install -y curl\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should pin ubuntu version
        assert!(purified.contains("ubuntu:22.04"));
        // Should add --no-install-recommends
        assert!(purified.contains("--no-install-recommends"));
        // Should add cleanup
        assert!(purified.contains("rm -rf /var/lib/apt/lists"));
        // Should add USER directive
        assert!(purified.contains("USER appuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_skip_user() {
        let source = "FROM ubuntu\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, true);

        // Should NOT add USER directive when skip_user is true
        assert!(!purified.contains("USER appuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_existing_user() {
        let source = "FROM ubuntu\nUSER myuser\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should NOT add another USER directive
        assert!(!purified.contains("USER appuser"));
        assert!(purified.contains("USER myuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_scratch() {
        let source = "FROM scratch\nCOPY binary /\nCMD [\"/binary\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should NOT add USER directive for scratch images
        assert!(!purified.contains("USER appuser"));
    }

    #[test]
    fn test_dockerfile_has_user_directive() {
        assert!(dockerfile_has_user_directive(
            "FROM ubuntu\nUSER root\nCMD bash"
        ));
        assert!(!dockerfile_has_user_directive("FROM ubuntu\nCMD bash"));
        assert!(dockerfile_has_user_directive("USER nobody"));
    }

    #[test]
    fn test_dockerfile_is_scratch() {
        assert!(dockerfile_is_scratch("FROM scratch\nCOPY app /"));
        assert!(!dockerfile_is_scratch("FROM ubuntu\nCOPY app /"));
        assert!(dockerfile_is_scratch("  FROM scratch  "));
    }

    #[test]
    fn test_dockerfile_find_cmd_line() {
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nRUN apt update\nCMD bash"),
            Some(2)
        );
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nENTRYPOINT [\"app\"]"),
            Some(1)
        );
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nRUN apt update"),
            None
        );
    }

    // ===== ESTIMATE BUILD TIME TESTS =====

    #[test]
    fn test_estimate_build_time_small() {
        // 10 layers, 100MB, no package managers
        let time = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        assert!(time >= 10); // at least 10 seconds for layers
    }

    #[test]
    fn test_estimate_build_time_large() {
        // 20 layers, 1GB, with package managers
        let time = estimate_build_time_seconds(20, 1_000_000_000, true, true, true);
        assert!(time > 30); // layers + size + package manager overhead
    }

    #[test]
    fn test_estimate_build_time_with_apt() {
        let no_apt = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_apt = estimate_build_time_seconds(10, 100_000_000, true, false, false);
        assert!(with_apt > no_apt);
    }

    #[test]
    fn test_estimate_build_time_with_pip() {
        let no_pip = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_pip = estimate_build_time_seconds(10, 100_000_000, false, true, false);
        assert!(with_pip > no_pip);
    }

    #[test]
    fn test_estimate_build_time_with_npm() {
        let no_npm = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_npm = estimate_build_time_seconds(10, 100_000_000, false, false, true);
        assert!(with_npm > no_npm);
    }

    // ===== SIZE PARSING TESTS =====

    #[test]
    fn test_parse_size_string_gb() {
        assert_eq!(parse_size_string("1GB"), Some(1_000_000_000));
        assert_eq!(parse_size_string("2.5GB"), Some(2_500_000_000));
        assert_eq!(parse_size_string("10GB"), Some(10_000_000_000));
    }

    #[test]
    fn test_parse_size_string_mb() {
        assert_eq!(parse_size_string("100MB"), Some(100_000_000));
        assert_eq!(parse_size_string("500MB"), Some(500_000_000));
        assert_eq!(parse_size_string("1.5MB"), Some(1_500_000));
    }

    #[test]
    fn test_parse_size_string_kb() {
        assert_eq!(parse_size_string("1KB"), Some(1_000));
        assert_eq!(parse_size_string("500KB"), Some(500_000));
    }

    #[test]
    fn test_parse_size_string_bytes() {
        assert_eq!(parse_size_string("1000B"), Some(1000));
        assert_eq!(parse_size_string("1000"), Some(1000));
    }

    #[test]
    fn test_parse_size_string_case_insensitive() {
        assert_eq!(parse_size_string("1gb"), Some(1_000_000_000));
        assert_eq!(parse_size_string("1Gb"), Some(1_000_000_000));
        assert_eq!(parse_size_string("1mb"), Some(1_000_000));
    }

    #[test]
    fn test_parse_size_string_with_spaces() {
        assert_eq!(parse_size_string("  1GB  "), Some(1_000_000_000));
        // "500 MB" works because the number part gets trimmed after extracting
        assert_eq!(parse_size_string("500 MB"), Some(500_000_000));
    }

    #[test]
    fn test_parse_size_string_invalid() {
        assert_eq!(parse_size_string("invalid"), None);
        assert_eq!(parse_size_string(""), None);
        assert_eq!(parse_size_string("GB"), None);
    }

    // ===== BUILD TIME ESTIMATE TESTS =====

    #[test]
    fn test_format_build_time_estimate_seconds() {
        let result = format_build_time_estimate(5, 100_000_000, false, false, false);
        assert!(result.starts_with("~"));
        assert!(result.contains('s'));
    }

    #[test]
    fn test_format_build_time_estimate_minutes() {
        // Large enough to be over 60 seconds
        let result = format_build_time_estimate(50, 5_000_000_000, true, true, true);
        assert!(result.contains('m'));
    }

    // ===== SIZE LIMIT TESTS =====

    #[test]
    fn test_size_exceeds_limit() {
        assert!(size_exceeds_limit(2_000_000_000, 1_000_000_000));
        assert!(!size_exceeds_limit(500_000_000, 1_000_000_000));
        assert!(!size_exceeds_limit(1_000_000_000, 1_000_000_000));
    }

    #[test]
    fn test_size_percentage_of_limit() {
        assert_eq!(size_percentage_of_limit(500_000_000, 1_000_000_000), 50.0);
        assert_eq!(
            size_percentage_of_limit(1_000_000_000, 1_000_000_000),
            100.0
        );
        assert_eq!(size_percentage_of_limit(250_000_000, 1_000_000_000), 25.0);
    }

    #[test]
    fn test_size_percentage_of_limit_zero() {
        assert_eq!(size_percentage_of_limit(100, 0), 100.0);
    }

    // ===== LAYER OPERATION TESTS =====

    #[test]
    fn test_layer_has_slow_operation_apt() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN apt-get install -y curl");
        assert!(apt);
        assert!(!pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_pip() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN pip install requests");
        assert!(!apt);
        assert!(pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_npm() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN npm install express");
        assert!(!apt);
        assert!(!pip);
        assert!(npm);
    }

    #[test]
    fn test_layer_has_slow_operation_yarn() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN yarn install");
        assert!(!apt);
        assert!(!pip);
        assert!(npm); // yarn counts as npm-like
    }

    #[test]
    fn test_layer_has_slow_operation_none() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN echo hello");
        assert!(!apt);
        assert!(!pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_multiple() {
        let (apt, pip, npm) =
            layer_has_slow_operation("RUN apt-get install && pip install && npm install");
        assert!(apt);
        assert!(pip);
        assert!(npm);
    }

    // ===== SIZE COMPARISON TESTS =====

    #[test]
    fn test_format_size_comparison_within_limit() {
        let result = format_size_comparison(500_000_000, 1_000_000_000);
        assert!(result.contains("✓"));
        assert!(result.contains("Within limit"));
        assert!(result.contains("50%"));
    }

    #[test]
    fn test_format_size_comparison_exceeds() {
        let result = format_size_comparison(2_000_000_000, 1_000_000_000);
        assert!(result.contains("✗"));
        assert!(result.contains("EXCEEDS"));
    }
}
