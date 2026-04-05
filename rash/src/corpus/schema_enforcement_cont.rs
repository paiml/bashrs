fn is_unquoted_var_at(bytes: &[u8], i: usize) -> bool {
    if i + 1 >= bytes.len() {
        return false;
    }
    let next = bytes[i + 1];
    // $( is subshell/arithmetic, not a bare variable
    if next == b'(' {
        return false;
    }
    next.is_ascii_alphabetic() || next == b'{' || next == b'_'
}

fn check_unquoted_expansion(line: &str) -> bool {
    if is_shell_assignment(line) {
        return false;
    }

    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double_quote => in_single_quote = !in_single_quote,
            b'"' if !in_single_quote => in_double_quote = !in_double_quote,
            b'$' if !in_single_quote && !in_double_quote => {
                if is_unquoted_var_at(bytes, i) {
                    return true;
                }
            }
            b'\\' if !in_single_quote => i += 1,
            _ => {}
        }
        i += 1;
    }
    false
}

/// Assignment operators in Makefiles
const MAKE_ASSIGN_OPS: &[&str] = &[":=", "?=", "+=", "="];

/// Extract a variable name from a Makefile assignment line
fn extract_make_var(line: &str) -> Option<String> {
    if line.starts_with('\t') || line.starts_with('#') {
        return None;
    }
    for op in MAKE_ASSIGN_OPS {
        if let Some(pos) = line.find(op) {
            let name = line[..pos].trim().to_string();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(name);
            }
        }
    }
    None
}

/// Check if a Makefile line is a space-indented recipe (GRAM-003)
fn is_space_indented_recipe(line: &str, in_recipe: bool) -> bool {
    in_recipe
        && !line.starts_with('\t')
        && !line.trim().is_empty()
        && (line.starts_with("    ") || line.starts_with("  "))
}

/// Validate Makefile output (§11.8.2)
fn validate_makefile_entry(
    entry: &CorpusEntry,
    violations: &mut Vec<GrammarViolation>,
    layers_passed: &mut Vec<ValidationLayer>,
) {
    let output = &entry.expected_output;

    if !output.is_empty() {
        layers_passed.push(ValidationLayer::Lexical);
    }

    let mut in_recipe = false;

    for (i, line) in output.lines().enumerate() {
        if line.trim().is_empty() {
            in_recipe = false;
            continue;
        }
        if line.starts_with('#') {
            continue;
        }

        // Detect target rules
        if !line.starts_with('\t') && line.contains(':') && !line.contains(":=") {
            in_recipe = true;
            continue;
        }

        if is_space_indented_recipe(line, in_recipe) {
            violations.push(GrammarViolation {
                category: GrammarCategory::TabSpaceConfusion,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Recipe line uses spaces instead of tab".into(),
            });
        }
    }

    // Collect defined variables (GRAM-005 preparation)
    let _defined_vars: Vec<String> = output.lines().filter_map(extract_make_var).collect();

    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Syntactic)
    {
        layers_passed.push(ValidationLayer::Syntactic);
    }
    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Semantic)
    {
        layers_passed.push(ValidationLayer::Semantic);
    }
}

/// Validate Dockerfile output (§11.8.3)
fn validate_dockerfile_entry(
    entry: &CorpusEntry,
    violations: &mut Vec<GrammarViolation>,
    layers_passed: &mut Vec<ValidationLayer>,
) {
    let output = &entry.expected_output;

    // L1: Lexical
    let l1_pass = !output.is_empty();
    if l1_pass {
        layers_passed.push(ValidationLayer::Lexical);
    }

    let instructions: Vec<&str> = output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();

    // GRAM-007: First instruction must be FROM (or ARG before FROM)
    if let Some(first) = instructions.first() {
        let upper = first.trim().to_uppercase();
        if !upper.starts_with("FROM") && !upper.starts_with("ARG") {
            violations.push(GrammarViolation {
                category: GrammarCategory::MissingFrom,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: 1,
                message: "Dockerfile must start with FROM (or ARG before FROM)".into(),
            });
        }
    }

    // GRAM-004: Shell form in CMD/ENTRYPOINT
    for (i, line) in output.lines().enumerate() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if (upper.starts_with("CMD ") || upper.starts_with("ENTRYPOINT ")) && !trimmed.contains('[')
        {
            violations.push(GrammarViolation {
                category: GrammarCategory::ShellFormCmd,
                layer: ValidationLayer::Semantic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Use exec form [\"cmd\", \"arg\"] instead of shell form".into(),
            });
        }
    }

    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Syntactic)
    {
        layers_passed.push(ValidationLayer::Syntactic);
    }
    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Semantic)
    {
        layers_passed.push(ValidationLayer::Semantic);
    }
}

/// Validate all entries in a corpus registry (§11.8.4)
pub fn validate_corpus(registry: &CorpusRegistry) -> SchemaReport {
    let results: Vec<SchemaResult> = registry.entries.iter().map(validate_entry).collect();

    let total_entries = results.len();
    let valid_entries = results.iter().filter(|r| r.valid).count();
    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();

    // Count violations by category
    let mut category_counts = std::collections::HashMap::new();
    for result in &results {
        for v in &result.violations {
            *category_counts.entry(v.category).or_insert(0usize) += 1;
        }
    }

    let mut violations_by_category: Vec<(GrammarCategory, usize)> =
        category_counts.into_iter().collect();
    violations_by_category.sort_by(|a, b| b.1.cmp(&a.1));

    SchemaReport {
        results,
        total_entries,
        valid_entries,
        total_violations,
        violations_by_category,
    }
}

/// Format the schema validation report as a table
pub fn format_schema_report(report: &SchemaReport) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(72);

    out.push_str(&format!(
        "{}\n{:<12} {:<14} {:<10} {:<10} {}\n{}\n",
        line, "Format", "Entries", "Valid", "Violations", "Pass Rate", line,
    ));

    // Per-format summary
    for format in &[
        CorpusFormat::Bash,
        CorpusFormat::Makefile,
        CorpusFormat::Dockerfile,
    ] {
        let fmt_results: Vec<&SchemaResult> = report
            .results
            .iter()
            .filter(|r| r.format == *format)
            .collect();
        let total = fmt_results.len();
        let valid = fmt_results.iter().filter(|r| r.valid).count();
        let violations: usize = fmt_results.iter().map(|r| r.violations.len()).sum();
        let rate = if total > 0 {
            (valid as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let fmt_name = match format {
            CorpusFormat::Bash => "Bash",
            CorpusFormat::Makefile => "Makefile",
            CorpusFormat::Dockerfile => "Dockerfile",
        };

        out.push_str(&format!(
            "{:<12} {:<14} {:<10} {:<10} {:.1}%\n",
            fmt_name, total, valid, violations, rate,
        ));
    }

    out.push_str(&line);
    out.push('\n');
    out.push_str(&format!(
        "{:<12} {:<14} {:<10} {:<10} {:.1}%\n",
        "Total",
        report.total_entries,
        report.valid_entries,
        report.total_violations,
        report.pass_rate(),
    ));

    out
}

/// Format grammar errors grouped by category
pub fn format_grammar_errors(report: &SchemaReport) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(72);

    out.push_str(&format!(
        "{}\n{:<12} {:<36} {:<8} {}\n{}\n",
        line, "Code", "Description", "Count", "Format", line,
    ));

    // Show all categories, even those with 0 violations
    for cat in GrammarCategory::all() {
        let count = report
            .violations_by_category
            .iter()
            .find(|(c, _)| c == cat)
            .map_or(0, |(_, n)| *n);

        let fmt_name = match cat.applicable_format() {
            CorpusFormat::Bash => "Bash",
            CorpusFormat::Makefile => "Makefile",
            CorpusFormat::Dockerfile => "Dockerfile",
        };

        out.push_str(&format!(
            "{:<12} {:<36} {:<8} {}\n",
            cat.code(),
            cat.description(),
            count,
            fmt_name,
        ));
    }

    out.push_str(&line);
    out.push('\n');

    // Show entries with violations
    let entries_with_violations: Vec<&SchemaResult> =
        report.results.iter().filter(|r| !r.valid).collect();

    if entries_with_violations.is_empty() {
        out.push_str("No grammar violations found.\n");
    } else {
        out.push_str(&format!(
            "\nEntries with violations ({}):\n",
            entries_with_violations.len()
        ));
        for result in entries_with_violations.iter().take(20) {
            out.push_str(&format!(
                "  {} ({}): {} violation(s)\n",
                result.entry_id,
                result.format,
                result.violations.len(),
            ));
            for v in &result.violations {
                out.push_str(&format!(
                    "    L{}: {} ({})\n",
                    v.line, v.message, v.category
                ));
            }
        }
        if entries_with_violations.len() > 20 {
            out.push_str(&format!(
                "  ... and {} more entries\n",
                entries_with_violations.len() - 20
            ));
        }
    }

    out
}

/// Get formal grammar specification for a format (§11.8.1-11.8.3)
pub fn format_grammar_spec(format: CorpusFormat) -> String {
    match format {
        CorpusFormat::Bash => posix_grammar_spec(),
        CorpusFormat::Makefile => makefile_grammar_spec(),
        CorpusFormat::Dockerfile => dockerfile_grammar_spec(),
    }
}

fn posix_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("POSIX Shell Grammar (IEEE Std 1003.1-2017, Section 2)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
complete_command : list separator_op
               | list
               ;
list            : list separator_op and_or
               | and_or
               ;
and_or          : pipeline
               | and_or AND_IF linebreak pipeline
               | and_or OR_IF linebreak pipeline
               ;
pipeline        : pipe_sequence
               | Bang pipe_sequence
               ;
pipe_sequence   : command
               | pipe_sequence '|' linebreak command
               ;
command         : simple_command
               | compound_command
               | compound_command redirect_list
               | function_definition
               ;
simple_command  : cmd_prefix cmd_word cmd_suffix
               | cmd_prefix cmd_word
               | cmd_prefix
               | cmd_name cmd_suffix
               | cmd_name
               ;
compound_command: brace_group
               | subshell
               | for_clause
               | case_clause
               | if_clause
               | while_clause
               | until_clause
               ;

Validation Layers:
  L1: Lexical  — bashrs parser (ShellAst), token stream valid
  L2: Syntactic — shellcheck -s sh, POSIX grammar compliance
  L3: Semantic  — bashrs linter (SEC/DET/IDEM rules)
  L4: Behavioral — cross-shell execution (dash, bash, ash)
",
    );
    out
}

fn makefile_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("GNU Make Grammar (GNU Make Manual 4.4, Section 3.7)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
makefile     : (rule | assignment | directive | comment | empty_line)*
rule         : targets ':' prerequisites '\\n' recipe
targets      : target (' ' target)*
prerequisites: prerequisite (' ' prerequisite)*
recipe       : ('\\t' command '\\n')+
assignment   : variable assignment_op value
assignment_op: ':=' | '?=' | '+=' | '='
directive    : 'include' | 'ifeq' | 'ifdef' | 'define' | '.PHONY' | ...

Validation Layers:
  L1: Lexical  — tab-vs-space detection
  L2: Syntactic — make -n --warn-undefined-variables
  L3: Semantic  — bashrs Makefile linter (MAKE001-MAKE020)
  L4: Behavioral — make -n dry-run comparison
",
    );
    out
}

fn dockerfile_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("Dockerfile Grammar (Docker Engine v25+)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
dockerfile   : (instruction | comment | empty_line)*
instruction  : FROM from_args
             | RUN run_args
             | COPY copy_args
             | WORKDIR path
             | ENV env_args
             | EXPOSE port_spec
             | USER user_spec
             | CMD exec_or_shell
             | ENTRYPOINT exec_or_shell
             | ARG arg_spec
             | LABEL label_args
             | HEALTHCHECK healthcheck_args
             | ...
from_args    : ['--platform=' platform] image [':' tag | '@' digest] ['AS' name]
exec_or_shell: exec_form | shell_form
exec_form    : '[' string (',' string)* ']'
shell_form   : string

Validation Layers:
  L1: Lexical  — instruction keyword recognition
  L2: Syntactic — bashrs Dockerfile parser
  L3: Semantic  — bashrs Dockerfile linter (DOCKER001-012) + Hadolint
  L4: Behavioral — docker build --no-cache
",
    );
    out
}

#[cfg(test)]
#[path = "schema_enforcement_tests_extracted.rs"]
mod tests_extracted;
