use clap::Parser;
use rash::cli::{execute_command, Cli};
use std::error::Error;
use std::process;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = execute_command(cli) {
        eprintln!("Error: {error}");

        // Print error chain if available
        let mut source = error.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {err}");
            source = err.source();
        }

        process::exit(1);
    }
}
