/// Generate warnings for patterns that might need manual review
fn generate_warnings(patterns: &[BashPattern]) -> Vec<String> {
    let mut warnings = Vec::new();

    for pattern in patterns {
        match pattern {
            BashPattern::SudoCommand { command } => {
                warnings.push(format!("Sudo command may need manual review: {}", command));
            }
            BashPattern::Script { content } => {
                if content.contains("eval") {
                    warnings.push(format!(
                        "eval usage detected - potential security risk: {}",
                        content
                    ));
                }
                if content.contains("$RANDOM") || content.contains("$$") {
                    warnings.push(format!("Non-deterministic pattern detected: {}", content));
                }
            }
            _ => {}
        }
    }

    warnings
}

/// Convert a bash script file to installer project
pub fn convert_file_to_project(input: &Path, output_dir: &Path) -> Result<ConversionResult> {
    // Read the bash script
    let script = std::fs::read_to_string(input).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", input.display(), e),
        ))
    })?;

    // Get project name from output directory
    let name = output_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("converted-installer");

    // Convert the script
    let result = convert_bash_to_installer(&script, name)?;

    // Create output directory structure
    std::fs::create_dir_all(output_dir).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create output directory: {}", e),
        ))
    })?;

    std::fs::create_dir_all(output_dir.join("templates")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create templates directory: {}", e),
        ))
    })?;

    std::fs::create_dir_all(output_dir.join("tests")).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create tests directory: {}", e),
        ))
    })?;

    // Write installer.toml
    std::fs::write(output_dir.join("installer.toml"), &result.installer_toml).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write installer.toml: {}", e),
        ))
    })?;

    // Write template files
    for template in &result.templates {
        std::fs::write(
            output_dir.join("templates").join(&template.name),
            &template.content,
        )
        .map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write template {}: {}", template.name, e),
            ))
        })?;
    }

    Ok(result)
}

#[cfg(test)]
#[path = "from_bash_tests_installer_11.rs"]
mod tests_extracted;
