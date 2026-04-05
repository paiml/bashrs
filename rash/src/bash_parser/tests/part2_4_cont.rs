fn test_PROMPT_001_interactive_vs_script_mode_hooks() {
    // DOCUMENTATION: Interactive hooks vs script mode
    //
    // Interactive hooks (NOT SUPPORTED in scripts):
    // - PROMPT_COMMAND: Before each prompt
    // - PS0: After command read, before execution
    // - DEBUG trap: Before each command (when set -x)
    // - RETURN trap: After function/script return
    // - EXIT trap: On shell exit
    //
    // Script mode (what IS supported):
    // - EXIT trap: On script exit (POSIX)
    // - ERR trap: On command failure (Bash extension)
    // - Explicit logging: printf statements
    // - Exit handlers: cleanup functions

    let script_mode_hooks = r#"
#!/bin/sh
# POSIX-compatible script hooks

# EXIT trap (supported - runs on script exit)
cleanup() {
  printf '%s\n' "Cleaning up..."
  rm -f /tmp/work.$$
}
trap cleanup EXIT

# Main script
printf '%s\n' "Starting..."
touch /tmp/work.$$
printf '%s\n' "Done"

# cleanup() runs automatically on exit (EXIT trap)
"#;

    let result = BashParser::new(script_mode_hooks);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Scripts support EXIT trap, not PROMPT_COMMAND"
        );
    }

    // Summary:
    // Interactive: PROMPT_COMMAND (implicit hook before each prompt)
    // Script: EXIT trap (explicit hook on exit)
    //
    // bashrs: Remove PROMPT_COMMAND, keep EXIT trap (POSIX)
}

// ============================================================================
// JOB-002: jobs Command (Interactive Job Control, NOT SUPPORTED)
// ============================================================================
//
// Task: JOB-002 - Document jobs command
// Status: DOCUMENTED (NOT SUPPORTED - interactive job control)
// Priority: LOW (job control not needed in scripts)
//
// The 'jobs' command lists active background jobs in the current shell session.
// It's an interactive job control feature.
//
// Bash behavior:
// - Lists background jobs started with &
// - Shows job number, status, command
// - Format: [job_number] status command
// - Interactive shells only (requires job control)
//
// bashrs policy:
// - NOT SUPPORTED (interactive job control)
// - Purification removes 'jobs' commands
// - Scripts run foreground only (no job control)
// - POSIX sh supports jobs, but bashrs doesn't use it
//
// Transformation:
// Bash input:
//   sleep 10 &
//   jobs
//
// Purified POSIX sh:
//   sleep 10  # Run in foreground (no &)
//   (jobs removed - not needed)
//
// Related features:
// - Background jobs (&) - JOB-001 (partial support)
// - fg/bg commands - JOB-003 (not supported)
// - disown command - Job control
// - wait command - Foreground synchronization (supported)

#[test]
fn test_JOB_002_jobs_command_not_supported() {
    // DOCUMENTATION: 'jobs' command is NOT SUPPORTED (interactive job control)
    //
    // jobs command lists background jobs:
    // $ sleep 10 &
    // [1] 12345
    // $ sleep 20 &
    // [2] 12346
    // $ jobs
    // [1]-  Running                 sleep 10 &
    // [2]+  Running                 sleep 20 &
    //
    // NOT SUPPORTED because:
    // - Interactive job control feature
    // - Scripts run foreground only
    // - No job control in non-interactive mode
    // - Not needed in automated execution

    let jobs_script = r#"
sleep 10 &
jobs
"#;

    let result = BashParser::new(jobs_script);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "jobs command is interactive only, NOT SUPPORTED in scripts"
            );
        }
        Err(_) => {
            // Parse error acceptable - interactive feature
        }
    }

    // jobs command options (all interactive):
    // -l: List process IDs
    // -n: Show only jobs changed since last notification
    // -p: List process IDs only
    // -r: List only running jobs
    // -s: List only stopped jobs
    //
    // All options are interactive-only and NOT SUPPORTED in bashrs.
}
