//! MAKE010: Missing error handling (|| exit 1)
//!
//! **Rule**: Detect commands without error handling in recipes
//!
//! **Why this matters**:
//! By default, Make only stops on error if the recipe command returns non-zero.
//! However, some commands may fail silently or have side effects that should
//! stop the build. Adding `|| exit 1` ensures the build stops on failure.
//!
//! **Auto-fix**: Add `|| exit 1` to commands that should fail the build
//!
//! ## Examples
//!
//! ❌ **BAD** (no error handling):
//! ```makefile
//! restore:
//!     @if [ -f b ]; then \
//!         cp b a; \
//!         rm -f b; \
//!     fi
//! ```
//!
//! ✅ **GOOD** (with error handling):
//! ```makefile
//! restore:
//!     @if [ -f b ]; then \
//!         cp b a || exit 1; \
//!         rm -f b; \
//!     fi
//! ```
//!
//! ## GH-256 / GH-257 (PMAT-251)
//!
//! Two classes of false positive and one false negative were fixed here:
//!
//! 1. **String contents are not commands** (GH-256): `echo "Install with: cargo
//!    install tool"` used to flag `install`, which is just text inside a quoted
//!    argument. Command-name detection is now done with
//!    [`crate::linter::shell_words::simple_commands`], which is quote- and
//!    role-aware, instead of a raw `split_whitespace` over the line.
//! 2. **A subcommand is not the command** (GH-256): `cargo install x` used to
//!    flag `install`, even though the command word is `cargo`. Only the first
//!    word of a simple command (`WordRole::CommandName`) is ever considered.
//! 3. **A lone command on a plain recipe line needs no `|| exit 1`** (GH-256):
//!    when a critical command is the *only and last* simple command in its
//!    logical recipe (one physical line, or one `\`-continued block with no
//!    further command after it), that command's own exit status **is** the
//!    recipe line's exit status, so Make already aborts the target on
//!    failure. `|| exit 1` there is a no-op; MAKE010 now only fires on a
//!    critical command that is followed by another command in the same
//!    logical recipe — the case where the *later* command's exit status is
//!    what Make actually sees (see GH-257's `restore:` example).
//! 4. **A flag can already declare the tolerance being asked for** (GH-257):
//!    `rm -f`, `rm -rf`, `mkdir -p`, `ln -sf`, and `cp -f` are exempt
//!    regardless of position, because the flag itself states that the
//!    "error" MAKE010 is worried about (missing file, existing directory,
//!    existing link/dest) is not being treated as an error.

use crate::linter::shell_words::{simple_commands, ShellWord, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

#[cfg(test)]
#[path = "make010_tests_gh256_gh257.rs"]
mod tests_gh256_gh257;

/// Commands that should have error handling
const CRITICAL_COMMANDS: &[&str] = &[
    "cp", "mv", "rm", "install", "chmod", "chown", "ln", "mkdir", "curl", "wget", "git",
];

/// Check for missing error handling in recipe commands
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        if !lines[i].starts_with('\t') {
            i += 1;
            continue;
        }

        // Gather a continuation block: this recipe line plus any following
        // recipe lines joined to it by a trailing, unescaped backslash. Make
        // feeds the whole block to one shell invocation, so it is one
        // logical recipe for MAKE010's purposes (GH-256/GH-257).
        let block_start = i;
        let mut block_end = i;
        while ends_with_continuation(lines[block_end])
            && block_end + 1 < lines.len()
            && lines[block_end + 1].starts_with('\t')
        {
            block_end += 1;
        }

        check_block(&lines[block_start..=block_end], block_start, &mut result);
        i = block_end + 1;
    }

    result
}

/// True when a physical line ends in an unescaped `\`, meaning Make joins it
/// with the next recipe line into one logical recipe / shell invocation.
fn ends_with_continuation(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut count = 0;
    let mut idx = bytes.len();
    while idx > 0 && bytes[idx - 1] == b'\\' {
        count += 1;
        idx -= 1;
    }
    count % 2 == 1
}

/// Strip the trailing continuation backslash from a line that is joined to
/// the next one, so it does not become part of the joined shell script text.
fn strip_trailing_continuation(line: &str) -> &str {
    line.strip_suffix('\\').unwrap_or(line)
}

/// Strip exactly one leading tab (the Make recipe-line delimiter) and, for
/// the first line of a recipe only, Make's own `@`/`-`/`+` prefix characters
/// (silent / ignore-errors / always-run) — none of these ever reach the
/// shell, so they must not confuse command-name detection.
fn strip_recipe_prefix(line: &str, is_first_line: bool) -> (&str, usize) {
    let after_tab = line.strip_prefix('\t').unwrap_or(line);
    let tab_len = line.len() - after_tab.len();
    if !is_first_line {
        return (after_tab, tab_len);
    }
    let trimmed = after_tab.trim_start_matches(['@', '-', '+']);
    let prefix_len = after_tab.len() - trimmed.len();
    (trimmed, tab_len + prefix_len)
}

/// A byte of `joined` maps back to the physical line (index into
/// `block_lines`) and 0-based byte column in that line's *full* original
/// text (tab and all) that produced it.
type BlockOffsets = Vec<(usize, usize)>;

/// Join a recipe's `\`-continued physical lines into one shell-script string,
/// stripping the leading tab (and, on the first line, Make's own `@`/`-`/`+`
/// prefix) and the trailing continuation backslashes — none of which ever
/// reach the shell — while recording where every byte came from so
/// diagnostics can still point at real source locations.
fn join_block(block_lines: &[&str]) -> (String, BlockOffsets) {
    let last = block_lines.len() - 1;
    let mut joined = String::new();
    let mut offsets = BlockOffsets::new();

    for (k, &full_line) in block_lines.iter().enumerate() {
        let is_last = k == last;
        let (stripped, removed) = strip_recipe_prefix(full_line, k == 0);
        let content = if is_last {
            stripped
        } else {
            strip_trailing_continuation(stripped)
        };

        offsets.extend((0..content.len()).map(|byte_i| (k, removed + byte_i)));
        joined.push_str(content);

        if !is_last {
            joined.push(' ');
            offsets.push((k, full_line.len()));
        }
    }

    (joined, offsets)
}

/// Build the MAKE010 diagnostic for one flagged command, or `None` if its
/// command-name word can't be located (never happens for a real
/// [`WordRole::CommandName`], but the lookups stay total rather than
/// panicking on malformed input).
fn diagnose_command(
    name: &str,
    cmd: &crate::linter::shell_words::SimpleCommand,
    offsets: &BlockOffsets,
    block_lines: &[&str],
    block_start_line: usize,
) -> Option<Diagnostic> {
    let name_word = cmd.words.iter().find(|w| w.role == WordRole::CommandName)?;
    let &(blk_idx, orig_col0) = offsets.get(name_word.col.saturating_sub(1))?;

    let line_num = block_start_line + blk_idx + 1;
    let orig_line = block_lines[blk_idx];
    let span = Span::new(
        line_num,
        orig_col0 + 1,
        line_num,
        orig_col0 + 1 + name_word.raw.len(),
    );
    let fix_replacement = format!("{} || exit 1", orig_line.trim_start());

    Some(
        Diagnostic::new(
            "MAKE010",
            Severity::Warning,
            format!("Command '{name}' missing error handling - consider adding '|| exit 1'"),
            span,
        )
        .with_fix(Fix::new(&fix_replacement)),
    )
}

/// True when `cmd` is a critical command that MAKE010 should flag: it names
/// a [`CRITICAL_COMMANDS`] entry, it has no GH-257 tolerant flag, and it is
/// not the last command in its logical recipe (GH-256 point 3: the last
/// command's exit status is what Make actually sees, so nothing masks its
/// failure there).
fn should_flag(
    cmd: &crate::linter::shell_words::SimpleCommand,
    is_last_command: bool,
) -> Option<&str> {
    let name = cmd.name.as_deref()?;
    if !CRITICAL_COMMANDS.contains(&name) || is_last_command || has_tolerant_flag(name, &cmd.words)
    {
        return None;
    }
    Some(name)
}

/// Analyze one logical recipe (one or more `\`-continued physical lines) and
/// append any MAKE010 findings to `result`.
fn check_block(block_lines: &[&str], block_start_line: usize, result: &mut LintResult) {
    let (joined, offsets) = join_block(block_lines);
    if joined.trim().is_empty() || has_error_handling(&joined) {
        return;
    }

    let commands = simple_commands(&joined);
    let named: Vec<&crate::linter::shell_words::SimpleCommand> =
        commands.iter().filter(|c| c.name.is_some()).collect();
    let Some(last_named_idx) = named.len().checked_sub(1) else {
        return;
    };

    for (idx, cmd) in named.iter().enumerate() {
        let Some(name) = should_flag(cmd, idx == last_named_idx) else {
            continue;
        };
        if let Some(diag) = diagnose_command(name, cmd, &offsets, block_lines, block_start_line) {
            result.add(diag);
        }
    }
}

/// GH-257: a flag that already declares the tolerance MAKE010 is asking for.
/// `rm -f`/`-rf`, `mkdir -p`, `cp -f`, and `ln -sf` name a failure mode
/// (missing file, existing directory/link/destination) that the command is
/// explicitly told to treat as a non-error — asking for `|| exit 1` on top
/// requests the opposite of what the flag was written for.
fn has_tolerant_flag(cmd: &str, words: &[ShellWord]) -> bool {
    let (short_char, long_form) = match cmd {
        "rm" | "cp" | "ln" => ('f', "--force"),
        "mkdir" => ('p', "--parents"),
        _ => return false,
    };

    words.iter().any(|w| {
        if w.role != WordRole::Argument {
            return false;
        }
        if w.literal == long_form {
            return true;
        }
        w.literal.starts_with('-') && !w.literal.starts_with("--") && w.literal.contains(short_char)
    })
}

/// Check if a recipe line already has error handling
/// GH-209: this used to be a substring test for the literal `"|| exit"`, so the
/// compound form
///
/// ```make
/// curl -fsSL "$(URL)" | tar xz || { echo "✗ download failed"; exit 1; }
/// ```
///
/// was reported as missing error handling — even though it handles errors
/// strictly better than a bare `|| exit 1`, since the user learns what failed.
/// The offered autofix appended a second `|| exit 1` after a block that already
/// exits, which is dead code.
///
/// Now: find the `||` and inspect its tail, so any failure branch that
/// terminates counts — `|| exit 1`, `|| { …; exit 1; }`, `|| return 1`, and the
/// common `die`/`fail`/`abort` helpers.
fn has_error_handling(recipe: &str) -> bool {
    if recipe.contains("set -e") || recipe.contains("&&") {
        return true;
    }

    let Some(idx) = recipe.find("||") else {
        return false;
    };
    let tail = &recipe[idx + 2..];

    if tail.contains("exit") || tail.contains("return") {
        return true;
    }

    // `|| die "msg"` and `|| { fail …` — the brace may be its own token
    // (`|| { fail`) or glued on (`||{fail`), so strip it before the first word
    // rather than from it.
    tail.trim_start()
        .trim_start_matches('{')
        .split_whitespace()
        .next()
        .is_some_and(|w| matches!(w, "die" | "fail" | "abort" | "error" | "bail"))
}

/// Check if a line is a variable assignment (`VAR="..."` / `VAR='...'`).
///
/// GH-256 (PMAT-251): `check()` no longer calls this directly — command-name
/// detection now goes through [`crate::linter::shell_words::simple_commands`],
/// whose `WordRole::AssignPrefix` handles assignment prefixes (and every
/// other word-position question) more generally and without the raw-text
/// quoting pitfalls this line-local heuristic had. Retained (and still
/// exercised by its own unit/property tests below) because it documents the
/// exact quoted-vs-unquoted assignment contract those tests pin down.
#[allow(dead_code)]
fn is_variable_assignment(line: &str) -> bool {
    // Pattern: VAR="..." or VAR='...'
    if let Some(eq_pos) = line.find('=') {
        let before_eq = &line[..eq_pos];
        // Variable name should be alphanumeric + underscore only
        let is_valid_var_name = before_eq
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$');

        if is_valid_var_name {
            let after_eq = &line[eq_pos + 1..];
            // Check if value is quoted
            return after_eq.starts_with('"') || after_eq.starts_with('\'');
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    // GH-256 (PMAT-251): a critical command that is the sole/last command in
    // its logical recipe is no longer flagged — its exit status IS the
    // recipe line's exit status, so Make already aborts on failure there.
    // See `test_MAKE010_detects_missing_error_handling` (now split into the
    // GH-256 no-warning case below) and `check_block`'s doc comment.

    #[test]
    fn test_MAKE010_detects_missing_error_handling() {
        // Not last: `chmod` follows, so a failed `cp` here would be masked
        // (the whole line's exit status becomes `chmod`'s), which is exactly
        // the case MAKE010 exists for.
        let makefile = "install:\n\tcp app /usr/bin/app; \\\n\tchmod +x /usr/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE010");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("error handling"));
        assert!(diag.message.contains("'cp'"));
    }

    #[test]
    fn test_MAKE010_no_warning_with_exit_handling() {
        let makefile = "install:\n\tcp app /usr/bin/app || exit 1";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_provides_fix() {
        let makefile = "install:\n\tcp app /usr/bin/app; \\\n\tchmod +x /usr/bin/app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("|| exit 1"));
    }

    #[test]
    fn test_MAKE010_detects_multiple_commands() {
        // Both `cp` and `mv` are followed by another command in the same
        // logical recipe; `chmod` is last, so it alone is exempt.
        let makefile =
            "install:\n\tcp app /usr/bin; \\\n\tmv app2 /usr/bin2; \\\n\tchmod +x /usr/bin/app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE010_no_warning_for_safe_commands() {
        let makefile = "build:\n\techo Building...";
        let result = check(makefile);

        // echo doesn't need error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_with_set_e() {
        let makefile = "install:\n\tset -e; cp app /usr/bin/app";
        let result = check(makefile);

        // set -e provides error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_detects_git_commands() {
        // `git` is not last (`echo` follows), so a failed pull would be
        // masked rather than caught by Make's own abort-on-error behavior.
        let makefile = "deploy:\n\tgit pull origin main; \\\n\techo done";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE010_no_warning_with_and_chaining() {
        let makefile = "deploy:\n\tgit pull origin main && make build";
        let result = check(makefile);

        // && chaining provides implicit error handling
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Issue #18: Tests for string literal detection

    #[test]
    fn test_MAKE010_no_warning_for_echo_with_command_keyword() {
        let makefile = "help:\n\t@echo \"Run: make install\"";
        let result = check(makefile);

        // Should NOT warn about 'install' in echo string
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_printf_with_command_keyword() {
        let makefile = "help:\n\t@printf 'Use: rm -rf /tmp\\n'";
        let result = check(makefile);

        // Should NOT warn about 'rm' in printf string
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_variable_assignment() {
        let makefile = "config:\n\t@MSG=\"install here\"";
        let result = check(makefile);

        // Should NOT warn about 'install' in variable assignment
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE010_no_warning_for_heredoc() {
        let makefile = "docs:\n\t@cat << EOF";
        let result = check(makefile);

        // Should NOT warn in heredoc context
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_is_variable_assignment_double_quotes() {
        assert!(is_variable_assignment("MSG=\"install here\""));
        assert!(is_variable_assignment("HELP=\"use rm -rf\""));
    }

    #[test]
    fn test_is_variable_assignment_single_quotes() {
        assert!(is_variable_assignment("MSG='install here'"));
        assert!(is_variable_assignment("HELP='use rm -rf'"));
    }

    #[test]
    fn test_is_variable_assignment_unquoted() {
        assert!(!is_variable_assignment("MSG=install"));
        assert!(!is_variable_assignment("HELP=rm"));
    }

    #[test]
    fn test_is_variable_assignment_shell_var() {
        assert!(is_variable_assignment("$$VAR=\"value\""));
    }

    #[test]
    fn test_is_not_variable_assignment() {
        assert!(!is_variable_assignment("echo test"));
        assert!(!is_variable_assignment("cargo install foo"));
    }

    // Property-based tests for Issue #18

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid command keywords
        fn command_keyword() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "install", "cp", "mv", "rm", "chmod", "chown", "ln", "mkdir", "curl", "wget", "git",
            ])
            .prop_map(|s| s.to_string())
        }

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            /// Property: echo/printf with command keywords should never trigger MAKE010
            #[test]
            fn prop_echo_with_command_never_warns(
                cmd in command_keyword(),
                prefix in prop::sample::select(vec!["echo", "printf"]),
                text in "[a-zA-Z0-9 ]+",
            ) {
                let recipe = format!("\t@{} \"{}. Use: {} here\"", prefix, text, cmd);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 for command in echo/printf
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "echo/printf with '{}' in string should not trigger MAKE010", cmd);
            }

            /// Property: Variable assignments with command keywords should never trigger MAKE010
            #[test]
            fn prop_variable_assignment_never_warns(
                cmd in command_keyword(),
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                text in "[a-zA-Z0-9 ]+",
            ) {
                let recipe = format!("\t@{}=\"{} {}\"", var_name, text, cmd);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 for command in variable assignment
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "Variable assignment with '{}' in value should not trigger MAKE010", cmd);
            }

            /// Property: an actual command that is NOT the last command in its
            /// recipe should always trigger MAKE010.
            ///
            /// GH-256 (PMAT-251) point 3: a critical command that is the sole/
            /// last command on a recipe line already gets Make's own
            /// abort-on-error behavior for free (its exit status IS the
            /// recipe line's exit status), so MAKE010 would be noise there.
            /// The property now follows each critical command with a
            /// trailing `true` so it is never last. `args` excludes `-` so it
            /// can never accidentally spell a GH-257 tolerant flag (`-f`,
            /// `-p`, …) and mask the very thing this property checks.
            #[test]
            fn prop_non_last_command_always_warns(
                cmd in command_keyword(),
                args in "[a-zA-Z0-9/.]+",
            ) {
                let recipe = format!("\t{} {}; \\\n\ttrue", cmd, args);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // SHOULD trigger MAKE010 for actual, non-last command
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 1,
                    "Actual non-last '{}' command without error handling should trigger MAKE010", cmd);
            }

            /// Property: that same critical command, when it IS the sole and
            /// last command on a plain (non-continued) recipe line, must
            /// never trigger MAKE010 (GH-256 point 3).
            #[test]
            fn prop_lone_last_command_never_warns(
                cmd in command_keyword(),
                args in "[a-zA-Z0-9/.]+",
            ) {
                let recipe = format!("\t{} {}", cmd, args);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "A lone last '{}' command on a plain recipe line should not trigger MAKE010", cmd);
            }

            /// Property: Commands with || exit 1 should never trigger MAKE010
            #[test]
            fn prop_command_with_error_handling_never_warns(
                cmd in command_keyword(),
                args in "[a-zA-Z0-9/._-]+",
            ) {
                let recipe = format!("\t{} {} || exit 1", cmd, args);
                let makefile = format!("target:\n{}", recipe);
                let result = check(&makefile);

                // Should NOT trigger MAKE010 when error handling present
                let make010_count = result.diagnostics.iter()
                    .filter(|d| d.code == "MAKE010")
                    .count();

                prop_assert_eq!(make010_count, 0,
                    "Command '{}' with || exit 1 should not trigger MAKE010", cmd);
            }

            /// Property: is_variable_assignment is deterministic
            #[test]
            fn prop_is_variable_assignment_deterministic(line in ".*") {
                let result1 = is_variable_assignment(&line);
                let result2 = is_variable_assignment(&line);
                prop_assert_eq!(result1, result2,
                    "is_variable_assignment should be deterministic");
            }

            /// Property: is_variable_assignment only accepts quoted values
            #[test]
            fn prop_is_variable_assignment_requires_quotes(
                var_name in "[A-Z][A-Z0-9_]{0,10}",
                value in "[a-zA-Z0-9 ]+",
            ) {
                let unquoted = format!("{}={}", var_name, value);
                let double_quoted = format!("{}=\"{}\"", var_name, value);
                let single_quoted = format!("{}='{}'", var_name, value);

                prop_assert!(!is_variable_assignment(&unquoted),
                    "Unquoted assignment should return false");
                prop_assert!(is_variable_assignment(&double_quoted),
                    "Double-quoted assignment should return true");
                prop_assert!(is_variable_assignment(&single_quoted),
                    "Single-quoted assignment should return true");
            }
        }
    }

    /// GH-209: `|| { echo ...; exit 1; }` IS error handling — better than a bare
    /// `|| exit 1`. The old substring test for the literal "|| exit" missed it,
    /// and the autofix appended a second `|| exit 1` after a block that exits.
    #[test]
    fn gh209_compound_error_handler_is_recognized() {
        assert!(has_error_handling(
            "curl -fsSL \"$(URL)\" | tar xz || { echo \"failed\"; exit 1; }"
        ));
        assert!(has_error_handling(
            "rm -rf \"$(DIR)\" || { echo \"rm failed\"; exit 1; }"
        ));
        assert!(has_error_handling("cmd || return 1"));
        assert!(has_error_handling("cmd || die \"nope\""));
        assert!(has_error_handling("cmd || { fail \"nope\"; }"));
    }

    /// And a recipe with NO handling must still be reported, or the fix is a
    /// false negative rather than a fix.
    #[test]
    fn gh209_unhandled_command_still_reported() {
        assert!(!has_error_handling("curl -fsSL \"$(URL)\" | tar xz"));
        let result = check("target:\n\tcurl -fsSL \"$(URL)\" | tar xz\n");
        assert!(
            !result.diagnostics.is_empty(),
            "genuinely unhandled curl must still fire"
        );
    }
}
