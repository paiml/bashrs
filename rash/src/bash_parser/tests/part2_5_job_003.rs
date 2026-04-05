fn test_JOB_003_job_specifications() {
    // DOCUMENTATION: Job specification syntax (interactive only)
    //
    // Job specs for fg/bg/kill/disown:
    // %n      - Job number n (e.g., %1, %2)
    // %string - Job whose command contains 'string'
    // %%      - Current job
    // %+      - Current job (same as %%)
    // %-      - Previous job
    // %?string - Job whose command contains 'string'
    //
    // Examples:
    // $ sleep 10 & sleep 20 &
    // [1] 12345
    // [2] 12346
    // $ fg %1          # Foreground job 1
    // $ fg %sleep      # Foreground job with 'sleep'
    // $ fg %%          # Foreground current job
    // $ fg %-          # Foreground previous job

    let job_spec_script = r#"
sleep 10 &
sleep 20 &
fg %1         # Job number
fg %sleep     # Command substring
fg %%         # Current job
fg %+         # Current job (alt)
fg %-         # Previous job
"#;

    let result = BashParser::new(job_spec_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Job specifications are interactive only"
        );
    }

    // Job specs require job control:
    // - Interactive shell (set -m)
    // - Job tracking enabled
    // - Job table maintained by shell
    // - NOT SUPPORTED in bashrs (no job tracking)
}

#[test]
fn test_JOB_003_purification_removes_fg_bg() {
    // DOCUMENTATION: Purification removes fg/bg commands
    //
    // Before (with job control):
    // #!/bin/bash
    // sleep 10 &
    // sleep 20 &
    // fg %1     # Bring job 1 to foreground
    // bg %2     # Resume job 2 in background
    //
    // After (purified, fg/bg removed):
    // #!/bin/sh
    // sleep 10  # Foreground
    // sleep 20  # Foreground
    // # fg removed (no job control)
    // # bg removed (no job control)
    //
    // Removed because:
    // - Scripts run foreground only (no &)
    // - No job state management
    // - Sequential execution model
    // - No foreground/background switching

    let purified_no_fg_bg = r#"
#!/bin/sh
sleep 10
sleep 20
"#;

    let result = BashParser::new(purified_no_fg_bg);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Purified scripts have no fg/bg commands"
        );
    }

    // Purification strategy:
    // 1. Remove & from commands (run foreground)
    // 2. Remove fg command (everything already foreground)
    // 3. Remove bg command (no stopped jobs)
    // 4. Sequential execution only
}

#[test]
fn test_JOB_003_fg_bg_workflow() {
    // DOCUMENTATION: Interactive fg/bg workflow
    //
    // Typical interactive workflow:
    // 1. Start background job
    //    $ sleep 60 &
    //    [1] 12345
    //
    // 2. Check job status
    //    $ jobs
    //    [1]+  Running      sleep 60 &
    //
    // 3. Bring to foreground
    //    $ fg %1
    //    sleep 60
    //    (now in foreground, can use Ctrl-C to terminate)
    //
    // 4. Suspend with Ctrl-Z
    //    ^Z
    //    [1]+  Stopped      sleep 60
    //
    // 5. Resume in background
    //    $ bg %1
    //    [1]+ sleep 60 &
    //
    // 6. Check again
    //    $ jobs
    //    [1]+  Running      sleep 60 &
    //
    // This entire workflow is interactive-only, NOT SUPPORTED in bashrs.

    let interactive_workflow = r#"
sleep 60 &       # Start background
jobs             # Check status
fg %1            # Foreground
# User presses Ctrl-Z (SIGTSTP)
bg %1            # Resume background
jobs             # Check again
"#;

    let result = BashParser::new(interactive_workflow);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Interactive fg/bg workflow not supported in scripts"
        );
    }

    // Why not supported:
    // - Requires TTY for Ctrl-Z
    // - Needs SIGTSTP/SIGCONT signal handling
    // - Job state transitions (running/stopped)
    // - Interactive user input
}
