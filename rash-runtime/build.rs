use anyhow::Result;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // Read the shell runtime library
    let runtime_path = Path::new("src/lib.sh");
    let runtime_content = fs::read_to_string(runtime_path)?;

    // Validate shell syntax (basic check)
    // validate_shell_syntax(&runtime_content)?;

    // Minify the runtime (remove comments and extra whitespace)
    let minified = minify_shell(&runtime_content);

    // Generate Rust code to embed the runtime
    let escaped_runtime = minified.replace('\\', "\\\\").replace('"', "\\\"");
    let output = format!(
        r#"/// Embedded Rash runtime library
pub const RUNTIME_LIBRARY: &str = "{escaped_runtime}";

pub fn get_runtime() -> &'static str {{
    RUNTIME_LIBRARY
}}
"#
    );

    // Write to output file
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("runtime.rs");
    fs::write(dest_path, output)?;

    // Tell cargo to rerun if the runtime changes
    println!("cargo:rerun-if-changed=src/lib.sh");

    Ok(())
}

#[allow(dead_code)]
fn validate_shell_syntax(content: &str) -> Result<()> {
    // Basic validation - check for balanced quotes and brackets
    let mut single_quote = false;
    let mut double_quote = false;
    let mut escape_next = false;
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut bracket_count = 0;

    for ch in content.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if !single_quote => escape_next = true,
            '\'' if !double_quote && !escape_next => single_quote = !single_quote,
            '"' if !single_quote && !escape_next => double_quote = !double_quote,
            '(' if !single_quote && !double_quote => paren_count += 1,
            ')' if !single_quote && !double_quote => paren_count -= 1,
            '{' if !single_quote && !double_quote => brace_count += 1,
            '}' if !single_quote && !double_quote => brace_count -= 1,
            '[' if !single_quote && !double_quote => bracket_count += 1,
            ']' if !single_quote && !double_quote => bracket_count -= 1,
            _ => {}
        }
    }

    if single_quote {
        anyhow::bail!("Unclosed single quote in runtime");
    }
    if double_quote {
        anyhow::bail!("Unclosed double quote in runtime");
    }
    if paren_count != 0 {
        anyhow::bail!("Unmatched parentheses in runtime: {}", paren_count);
    }
    if brace_count != 0 {
        anyhow::bail!("Unmatched braces in runtime: {}", brace_count);
    }
    if bracket_count != 0 {
        anyhow::bail!("Unmatched brackets in runtime: {}", bracket_count);
    }

    Ok(())
}

fn minify_shell(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            // Remove comments (but preserve shebang)
            if line.starts_with("#!") {
                line.to_string()
            } else if let Some(pos) = line.find('#') {
                // Check if # is inside quotes
                let before_hash = &line[..pos];
                let single_quotes = before_hash.chars().filter(|&c| c == '\'').count();
                let double_quotes = before_hash.chars().filter(|&c| c == '"').count();

                // If we're inside quotes, keep the line as-is
                if single_quotes % 2 != 0 || double_quotes % 2 != 0 {
                    line.trim().to_string()
                } else {
                    // Remove comment
                    line[..pos].trim().to_string()
                }
            } else {
                line.trim().to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
