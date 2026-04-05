use crate::cli::args::WatchCommand;
use crate::cli::Commands;
use crate::models::{Error, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tracing::info;

#[cfg(test)]
#[path = "watch_commands_tests.rs"]
mod tests;

/// Entry point for the watch command — delegates from dispatch_command.
pub(crate) fn watch_command(command: Commands) -> Result<()> {
    match command {
        Commands::Watch {
            paths,
            command: watch_cmd,
            debounce,
            extensions,
            clear,
            fail_fast,
        } => run_watch(&paths, &watch_cmd, debounce, &extensions, clear, fail_fast),
        _ => unreachable!("watch_command called with non-watch command"),
    }
}

/// Parse comma-separated extension list into a set.
fn parse_extensions(ext_str: &str) -> Vec<String> {
    ext_str
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check if a path matches the watched extensions.
fn matches_extensions(path: &Path, extensions: &[String]) -> bool {
    // Special-case extensionless filenames like Makefile, Dockerfile
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let name_lower = name.to_lowercase();
        for ext in extensions {
            if name_lower == *ext {
                return true;
            }
        }
    }
    // Check file extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        extensions.contains(&ext_lower)
    } else {
        false
    }
}

/// Collect all matching files from watched paths for the initial run.
fn collect_files(paths: &[PathBuf], extensions: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if matches_extensions(path, extensions) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            if let Ok(entries) = glob::glob(&format!("{}/**/*", path.display())) {
                for entry in entries.flatten() {
                    if entry.is_file() && matches_extensions(&entry, extensions) {
                        files.push(entry);
                    }
                }
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Get the bashrs binary path for subprocess execution.
fn bashrs_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("bashrs"))
}

/// Build the subcommand name string.
fn command_name(watch_cmd: &WatchCommand) -> &'static str {
    match watch_cmd {
        WatchCommand::Lint => "lint",
        WatchCommand::Format => "format",
        WatchCommand::Test => "test",
        WatchCommand::Score => "score",
        WatchCommand::SafetyCheck => "safety-check",
        WatchCommand::Audit => "audit",
    }
}

/// Run the appropriate subcommand on the given files as a subprocess.
///
/// Uses subprocess execution to isolate from `process::exit()` calls in
/// lint/format commands. Returns true if the subcommand succeeded.
fn run_subcommand(watch_cmd: &WatchCommand, files: &[PathBuf]) -> bool {
    if files.is_empty() {
        eprintln!("  No matching files found.");
        return true;
    }

    let exe = bashrs_exe();
    let cmd_name = command_name(watch_cmd);

    let mut cmd = std::process::Command::new(&exe);
    cmd.arg(cmd_name);

    // Format check mode for format command
    if matches!(watch_cmd, WatchCommand::Format) {
        cmd.arg("--check");
    }

    // Add file args
    for file in files {
        cmd.arg(file);
    }

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("\x1b[32m  \u{2713} All checks passed\x1b[0m");
                true
            } else {
                // Non-zero exit is expected (lint found issues, etc.)
                false
            }
        }
        Err(e) => {
            eprintln!("\x1b[31m  \u{2717} Failed to run bashrs {cmd_name}: {e}\x1b[0m");
            false
        }
    }
}

/// Print the watch header with timestamp.
fn print_header(watch_cmd: &WatchCommand, file_count: usize) {
    let now = chrono::Local::now();
    println!(
        "\x1b[1m[{time}] bashrs watch \u{2192} {cmd} ({n} file{s})\x1b[0m",
        time = now.format("%H:%M:%S"),
        cmd = command_name(watch_cmd),
        n = file_count,
        s = if file_count == 1 { "" } else { "s" },
    );
}

/// Main watch loop: setup watcher, debounce, re-run on changes.
fn run_watch(
    paths: &[PathBuf],
    watch_cmd: &WatchCommand,
    debounce_ms: u64,
    extensions: &str,
    clear: bool,
    fail_fast: bool,
) -> Result<()> {
    let exts = parse_extensions(extensions);
    let files = collect_files(paths, &exts);

    println!(
        "\x1b[1;36mbashrs watch\x1b[0m v{ver}",
        ver = env!("CARGO_PKG_VERSION")
    );
    println!(
        "  Watching {n} path{s} for changes (extensions: {exts})",
        n = paths.len(),
        s = if paths.len() == 1 { "" } else { "s" },
        exts = extensions,
    );
    println!(
        "  Debounce: {debounce_ms}ms | Command: \x1b[1m{cmd}\x1b[0m",
        cmd = command_name(watch_cmd),
    );
    println!("  Press Ctrl+C to stop.\n");

    // Initial run
    if clear {
        print!("\x1b[2J\x1b[H");
    }
    print_header(watch_cmd, files.len());
    let ok = run_subcommand(watch_cmd, &files);
    if fail_fast && !ok {
        return Err(Error::Internal("Watch: initial run failed with --fail-fast".into()));
    }
    println!();

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let debounce_duration = Duration::from_millis(debounce_ms);

    let mut debouncer = new_debouncer(debounce_duration, tx).map_err(|e| {
        Error::Internal(format!("Failed to create file watcher: {e}"))
    })?;

    for path in paths {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        info!("Watching path: {}", canonical.display());
        debouncer
            .watcher()
            .watch(&canonical, RecursiveMode::Recursive)
            .map_err(|e| {
                Error::Internal(format!("Failed to watch {}: {e}", path.display()))
            })?;
    }

    // Event loop with coalescing: drain all pending events before re-running
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let mut changed: Vec<PathBuf> = events
                    .iter()
                    .filter(|e| e.kind == DebouncedEventKind::Any)
                    .map(|e| e.path.clone())
                    .filter(|p| matches_extensions(p, &exts))
                    .collect();

                // Drain any additional pending events to coalesce rapid changes
                while let Ok(Ok(more_events)) = rx.try_recv() {
                    for ev in more_events {
                        if ev.kind == DebouncedEventKind::Any
                            && matches_extensions(&ev.path, &exts)
                        {
                            changed.push(ev.path);
                        }
                    }
                }

                if changed.is_empty() {
                    continue;
                }

                // Deduplicate
                changed.sort();
                changed.dedup();

                if clear {
                    print!("\x1b[2J\x1b[H");
                }

                // Re-collect all matching files (some might have been added/removed)
                let all_files = collect_files(paths, &exts);
                print_header(watch_cmd, all_files.len());

                // Show which files changed
                for p in &changed {
                    println!(
                        "  \x1b[33m\u{25cf}\x1b[0m {}",
                        p.strip_prefix(std::env::current_dir().unwrap_or_default())
                            .unwrap_or(p)
                            .display()
                    );
                }

                let ok = run_subcommand(watch_cmd, &all_files);
                if fail_fast && !ok {
                    return Err(Error::Internal(
                        "Watch: run failed with --fail-fast".into(),
                    ));
                }
                println!();
            }
            Ok(Err(errs)) => {
                eprintln!("\x1b[31mWatch error: {errs:?}\x1b[0m");
            }
            Err(e) => {
                return Err(Error::Internal(format!("Watch channel closed: {e}")));
            }
        }
    }
}
