use bashrs::cli::{execute_command, Cli};
use bashrs::models::Diagnostic;
use clap::Parser;
use std::error::Error;
use std::process;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = execute_command(cli) {
        // Create rich diagnostic from error
        let diagnostic = Diagnostic::from_error(&error, None);

        // Print formatted diagnostic
        eprintln!("{}", diagnostic);

        // Optional: Print original error chain for debugging
        if std::env::var("RASH_DEBUG").is_ok() {
            eprintln!("\nDebug trace:");
            eprintln!("  {error}");
            let mut source = error.source();
            while let Some(err) = source {
                eprintln!("  Caused by: {err}");
                source = err.source();
            }
        }

        // Issue #6: Different exit codes based on error type
        // Exit 1: General errors (lint failures, validation errors, etc.)
        // Exit 2: Tool failures (I/O errors, invalid arguments, etc.)
        let exit_code = match error {
            bashrs::models::Error::Io(_) => 2, // File not found, permission denied, etc.
            bashrs::models::Error::Parse(_) => 2, // Invalid input
            bashrs::models::Error::Internal(_) => 2, // Tool failure
            _ => 1, // Lint failures, validation errors, etc.
        };

        process::exit(exit_code);
    }
}
