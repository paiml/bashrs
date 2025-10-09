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

        process::exit(1);
    }
}
