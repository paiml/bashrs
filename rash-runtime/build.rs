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
    let mut validator = SyntaxValidator::new();
    validator.validate(content)
}

struct SyntaxValidator {
    single_quote: bool,
    double_quote: bool,
    escape_next: bool,
    paren_count: i32,
    brace_count: i32,
    bracket_count: i32,
}

impl SyntaxValidator {
    fn new() -> Self {
        Self {
            single_quote: false,
            double_quote: false,
            escape_next: false,
            paren_count: 0,
            brace_count: 0,
            bracket_count: 0,
        }
    }

    fn validate(&mut self, content: &str) -> Result<()> {
        for ch in content.chars() {
            if self.escape_next {
                self.escape_next = false;
                continue;
            }

            self.process_char(ch);
        }

        self.check_final_state()
    }

    fn process_char(&mut self, ch: char) {
        if self.handle_escape(ch) {
            return;
        }

        if self.handle_quotes(ch) {
            return;
        }

        self.handle_brackets(ch);
    }

    fn handle_escape(&mut self, ch: char) -> bool {
        if ch == '\\' && !self.single_quote {
            self.escape_next = true;
            return true;
        }
        false
    }

    fn handle_quotes(&mut self, ch: char) -> bool {
        match ch {
            '\'' if !self.double_quote && !self.escape_next => {
                self.single_quote = !self.single_quote;
                true
            }
            '"' if !self.single_quote && !self.escape_next => {
                self.double_quote = !self.double_quote;
                true
            }
            _ => false,
        }
    }

    fn handle_brackets(&mut self, ch: char) {
        if self.single_quote || self.double_quote {
            return;
        }

        match ch {
            '(' => self.paren_count += 1,
            ')' => self.paren_count -= 1,
            '{' => self.brace_count += 1,
            '}' => self.brace_count -= 1,
            '[' => self.bracket_count += 1,
            ']' => self.bracket_count -= 1,
            _ => {}
        }
    }

    fn check_final_state(&self) -> Result<()> {
        if self.single_quote {
            anyhow::bail!("Unclosed single quote in runtime");
        }
        if self.double_quote {
            anyhow::bail!("Unclosed double quote in runtime");
        }
        if self.paren_count != 0 {
            anyhow::bail!("Unmatched parentheses in runtime: {}", self.paren_count);
        }
        if self.brace_count != 0 {
            anyhow::bail!("Unmatched braces in runtime: {}", self.brace_count);
        }
        if self.bracket_count != 0 {
            anyhow::bail!("Unmatched brackets in runtime: {}", self.bracket_count);
        }
        Ok(())
    }
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
