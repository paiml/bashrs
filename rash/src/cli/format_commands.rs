use crate::models::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn format_command(
    inputs: &[PathBuf],
    check: bool,
    dry_run: bool,
    output: Option<&Path>,
) -> Result<()> {
    let mut all_formatted = true;

    for input_path in inputs {
        let (source, formatted) = format_read_and_format(input_path)?;

        if check {
            if !format_check_file(input_path, &source, &formatted) {
                all_formatted = false;
            }
        } else if dry_run {
            format_dry_run_file(input_path, &source, &formatted);
        } else {
            format_apply_file(input_path, &source, &formatted, output)?;
        }
    }

    if check && !all_formatted {
        return Err(Error::Internal(
            "Files are not properly formatted. Run without --check to fix.".to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn format_read_and_format(input_path: &Path) -> Result<(String, String)> {
    use crate::bash_quality::Formatter;

    let config = format_load_config(input_path);
    let mut formatter = Formatter::with_config(config);

    let source = fs::read_to_string(input_path)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input_path.display(), e)))?;
    let formatted = formatter.format_source(&source).map_err(|e| {
        Error::Internal(format!("Failed to format {}: {}", input_path.display(), e))
    })?;

    Ok((source, formatted))
}

pub(crate) fn format_load_config(input_path: &Path) -> crate::bash_quality::FormatterConfig {
    use crate::bash_quality::FormatterConfig;

    if let Some(parent) = input_path.parent() {
        let script_dir_config = parent.join(".bashrs-fmt.toml");
        if script_dir_config.exists() {
            return FormatterConfig::from_file(&script_dir_config).unwrap_or_default();
        }
    }
    FormatterConfig::from_file(".bashrs-fmt.toml").unwrap_or_default()
}

pub(crate) fn format_check_file(input_path: &Path, source: &str, formatted: &str) -> bool {
    if source.trim() == formatted.trim() {
        println!("✓ {} is properly formatted", input_path.display());
        true
    } else {
        println!("✗ {} is not properly formatted", input_path.display());
        false
    }
}

pub(crate) fn format_dry_run_file(input_path: &Path, source: &str, formatted: &str) {
    println!("Would format: {}", input_path.display());
    if source.trim() != formatted.trim() {
        println!("  Changes detected");
    } else {
        println!("  No changes needed");
    }
}

pub(crate) fn format_apply_file(
    input_path: &Path,
    _source: &str,
    formatted: &str,
    output: Option<&Path>,
) -> Result<()> {
    if let Some(out_path) = output {
        fs::write(out_path, formatted).map_err(|e| {
            Error::Internal(format!("Failed to write {}: {}", out_path.display(), e))
        })?;
        println!(
            "✓ Formatted {} -> {}",
            input_path.display(),
            out_path.display()
        );
    } else {
        fs::write(input_path, formatted).map_err(|e| {
            Error::Internal(format!("Failed to write {}: {}", input_path.display(), e))
        })?;
        println!("✓ Formatted {}", input_path.display());
    }
    Ok(())
}
