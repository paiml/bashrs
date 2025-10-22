// SC2310: This function is called in a condition - set -e does not apply
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FUNCTION_DEF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:function\s+([a-zA-Z_][a-zA-Z0-9_]*)|([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\))").unwrap()
});

static FUNCTION_IN_CONDITION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:if|while|until)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

static SET_E: Lazy<Regex> = Lazy::new(|| Regex::new(r"set\s+-[a-zA-Z]*e").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    // Check if set -e is present
    let has_set_e = lines.iter().any(|line| SET_E.is_match(line));

    if !has_set_e {
        return result;
    }

    // Collect defined functions
    let mut functions = Vec::new();
    for line in &lines {
        if let Some(caps) = FUNCTION_DEF.captures(line) {
            // Check which capture group matched (function keyword or ())
            let func_name = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string());
            if let Some(name) = func_name {
                functions.push(name);
            }
        }
    }

    // Check if any defined function is used in a condition
    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if let Some(caps) = FUNCTION_IN_CONDITION.captures(line) {
            let func_name = &caps[1];
            if functions.contains(&func_name.to_string()) {
                let diagnostic = Diagnostic::new(
                    "SC2310",
                    Severity::Info,
                    format!("Function '{}' is called in a condition. set -e will not apply to commands inside it.", func_name),
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );
                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2310_function_in_if() {
        let code = r#"
set -e
myfunc() {
    false
}
if myfunc; then
    echo "ok"
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2310_no_set_e_ok() {
        let code = r#"
myfunc() {
    false
}
if myfunc; then
    echo "ok"
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2310_function_not_in_condition_ok() {
        let code = r#"
set -e
myfunc() {
    false
}
myfunc
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2310_while_condition() {
        let code = r#"
set -e
check() {
    return 1
}
while check; do
    echo "loop"
done
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2310_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2310_builtin_in_if_ok() {
        let code = r#"
set -e
if [ -f file ]; then
    echo "exists"
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2310_until_condition() {
        let code = r#"
set -e
ready() {
    return 1
}
until ready; do
    echo "waiting"
done
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2310_multiple_functions() {
        let code = r#"
set -e
func1() { return 0; }
func2() { return 1; }
if func1; then echo "1"; fi
if func2; then echo "2"; fi
"#;
        assert_eq!(check(code).diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2310_function_keyword() {
        let code = r#"
set -e
function myfunc {
    false
}
if myfunc; then
    echo "ok"
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2310_comment() {
        let code = r#"
set -e
myfunc() { false; }
# if myfunc; then echo "ok"; fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
