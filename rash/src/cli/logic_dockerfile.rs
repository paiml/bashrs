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
#[path = "logic_dockerfile_tests_extracted.rs"]
mod tests_extracted;
