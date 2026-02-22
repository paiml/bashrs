use crate::cli::args::{DevContainerCommands, LintFormat};
use crate::cli::logic::find_devcontainer_json as logic_find_devcontainer_json;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub(crate) fn handle_devcontainer_command(command: DevContainerCommands) -> Result<()> {
    match command {
        DevContainerCommands::Validate {
            path,
            format,
            lint_dockerfile,
            list_rules,
        } => devcontainer_validate(&path, format, lint_dockerfile, list_rules),
    }
}

fn devcontainer_validate(
    path: &Path,
    format: LintFormat,
    lint_dockerfile: bool,
    list_rules: bool,
) -> Result<()> {
    use crate::linter::output::{write_results, OutputFormat};
    use crate::linter::rules::devcontainer::{list_devcontainer_rules, validate_devcontainer};

    if list_rules {
        println!("Available DEVCONTAINER rules:\n");
        for (code, desc) in list_devcontainer_rules() {
            println!("  {}: {}", code, desc);
        }
        return Ok(());
    }

    info!("Validating devcontainer at {}", path.display());
    let devcontainer_path = logic_find_devcontainer_json(path)?;
    info!("Found devcontainer.json at {}", devcontainer_path.display());

    let content = fs::read_to_string(&devcontainer_path).map_err(Error::Io)?;
    let result = validate_devcontainer(&content)
        .map_err(|e| Error::Validation(format!("Invalid devcontainer.json: {}", e)))?;

    let output_format = match format {
        LintFormat::Human => OutputFormat::Human,
        LintFormat::Json => OutputFormat::Json,
        LintFormat::Sarif => OutputFormat::Sarif,
    };

    let mut stdout = std::io::stdout();
    write_results(
        &mut stdout,
        &result,
        output_format,
        devcontainer_path.to_str().unwrap_or("devcontainer.json"),
    )
    .map_err(Error::Io)?;

    if lint_dockerfile {
        lint_referenced_dockerfile(&content, &devcontainer_path, format)?;
    }

    let has_errors = result
        .diagnostics
        .iter()
        .any(|d| d.severity == crate::linter::Severity::Error);
    if has_errors {
        Err(Error::Validation(
            "devcontainer.json validation failed".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Lint the Dockerfile referenced in a devcontainer.json build section
fn lint_referenced_dockerfile(
    content: &str,
    devcontainer_path: &Path,
    format: LintFormat,
) -> Result<()> {
    let json = match crate::linter::rules::devcontainer::parse_jsonc(content) {
        Ok(j) => j,
        Err(_) => return Ok(()),
    };

    let dockerfile = json
        .get("build")
        .and_then(|b| b.get("dockerfile"))
        .and_then(|v| v.as_str());

    let dockerfile = match dockerfile {
        Some(d) => d,
        None => return Ok(()),
    };

    let dockerfile_path = devcontainer_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(dockerfile);

    if dockerfile_path.exists() {
        info!(
            "Linting referenced Dockerfile: {}",
            dockerfile_path.display()
        );
        crate::cli::dockerfile_commands::dockerfile_lint_command(&dockerfile_path, format, None)?;
    } else {
        warn!(
            "Referenced Dockerfile not found: {}",
            dockerfile_path.display()
        );
    }

    Ok(())
}
