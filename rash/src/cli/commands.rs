#[cfg(feature = "oracle")]
use crate::cli::args::ExplainErrorFormat;
use crate::cli::args::{CompileRuntime, ContainerFormatArg, InspectionFormat};
#[cfg(feature = "oracle")]
use crate::cli::logic::extract_exit_code;
use crate::cli::logic::{is_shell_script_file, normalize_shell_script};
// Test-only imports from crate::cli::logic (needed by command_tests.rs via `super::*`)
#[cfg(test)]
use crate::cli::args::{ConfigOutputFormat, LintFormat, MakeOutputFormat};
#[cfg(test)]
use crate::cli::logic::{
    add_no_install_recommends, add_package_manager_cleanup, convert_add_to_copy_if_local,
    find_devcontainer_json as logic_find_devcontainer_json, format_timestamp, generate_diff_lines,
    hex_encode, pin_base_image_version, truncate_str,
};
use crate::cli::{Cli, Commands, CommandsExt};
use crate::models::{Config, Error, Result};
use crate::models::{ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;
use crate::{check, transpile};
use std::fs;
use std::path::Path;
use tracing::{info, warn};
#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;
#[cfg(test)]
#[path = "command_tests_analysis.rs"]
mod command_tests_analysis;
#[cfg(test)]
#[path = "command_tests_corpus1.rs"]
mod command_tests_corpus1;
#[cfg(test)]
#[path = "command_tests_corpus2.rs"]
mod command_tests_corpus2;
#[cfg(test)]
#[path = "command_tests_display.rs"]
mod command_tests_display;
#[cfg(test)]
#[path = "command_tests_gates.rs"]
mod command_tests_gates;

#[cfg(test)]
#[path = "command_tests_corpus3.rs"]
mod command_tests_corpus3;

#[cfg(test)]
#[path = "command_tests_corpus_cov.rs"]
mod command_tests_corpus_cov;

#[cfg(test)]
#[path = "command_tests_lint_cov.rs"]
mod command_tests_lint_cov;

#[cfg(test)]
#[path = "command_tests_installer_cov.rs"]
mod command_tests_installer_cov;

// ---------------------------------------------------------------------------
// Extracted command modules (thin dispatch -> dedicated files)
// ---------------------------------------------------------------------------

// Lint, purify, format, playbook, mutate, simulate command modules
#[path = "format_commands.rs"]
mod format_cmds;
#[path = "lint_commands.rs"]
mod lint_cmds;
#[path = "mutate_commands.rs"]
mod mutate_cmds;
#[path = "playbook_commands.rs"]
mod playbook_cmds;
#[path = "purify_commands.rs"]
mod purify_cmds;
#[path = "simulate_commands.rs"]
mod simulate_cmds;

// Re-import so existing dispatch calls and tests still work
use format_cmds::format_command;
use lint_cmds::{lint_command, LintCommandOptions};
use mutate_cmds::mutate_command;
use playbook_cmds::playbook_command;
use purify_cmds::{purify_command, PurifyCommandOptions};
use simulate_cmds::simulate_command;
#[path = "adversarial_commands.rs"]
mod adversarial_cmds;
#[path = "chat_inference.rs"]
pub(crate) mod chat_inference;
#[path = "classify_commands.rs"]
pub(crate) mod classify_cmds;
#[path = "explain_command.rs"]
mod explain_cmds;
#[path = "fix_command.rs"]
mod fix_cmds;
#[path = "safety_check_command.rs"]
mod safety_check_cmds;

// Quality command modules
#[path = "audit_commands.rs"]
mod audit_commands;
#[path = "cfg_commands.rs"]
mod cfg_cmds;
#[path = "coverage_commands.rs"]
mod coverage_commands;
#[path = "score_commands.rs"]
mod score_commands;
#[path = "test_commands.rs"]
mod test_commands;

#[cfg(test)]
use audit_commands::audit_command;
#[cfg(test)]
use coverage_commands::coverage_command;
#[cfg(test)]
use score_commands::score_command;
#[cfg(test)]
use test_commands::test_command;

// Gate, make, devcontainer, config, comply command modules
#[path = "comply_commands.rs"]
mod comply_cmds;
#[path = "config_commands.rs"]
mod config_cmds;
#[path = "devcontainer_commands.rs"]
mod devcontainer_cmds;
#[path = "gate_commands.rs"]
mod gate_cmds;
#[path = "make_commands.rs"]
mod make_cmds;

// Corpus command modules (25 files).
// Module names must match the `super::xxx` references used inside these files.
#[path = "corpus_advanced_commands.rs"]
pub(super) mod corpus_advanced_commands;
#[path = "corpus_analysis_commands.rs"]
pub(super) mod corpus_analysis_commands;
#[path = "corpus_b2_commands.rs"]
pub(super) mod corpus_b2_commands;
#[path = "corpus_b2_fix_commands.rs"]
pub(super) mod corpus_b2_fix_commands;
#[path = "corpus_compare_commands.rs"]
pub(super) mod corpus_compare_commands;
#[path = "corpus_config_commands.rs"]
pub(super) mod corpus_config_commands;
#[path = "corpus_convergence_commands.rs"]
pub(super) mod corpus_convergence_commands;
#[path = "corpus_core_commands.rs"]
mod corpus_core_cmds;
#[path = "corpus_decision_commands.rs"]
pub(super) mod corpus_decision_commands;
#[path = "corpus_diag_commands.rs"]
pub(super) mod corpus_diag_commands;
#[path = "corpus_diff_commands.rs"]
pub(super) mod corpus_diff_commands;
#[path = "corpus_display_commands.rs"]
pub(super) mod corpus_display_commands;
#[path = "corpus_entry_commands.rs"]
pub(super) mod corpus_entry_commands;
#[path = "corpus_expansion_commands.rs"]
pub(super) mod corpus_expansion_commands;
#[path = "corpus_failure_commands.rs"]
pub(super) mod corpus_failure_commands;
#[path = "corpus_gate_commands.rs"]
pub(super) mod corpus_gate_commands;
#[path = "corpus_metrics_commands.rs"]
pub(super) mod corpus_metrics_commands;
#[path = "corpus_ml_commands.rs"]
pub(super) mod corpus_ml_commands;
#[path = "corpus_ops_commands.rs"]
pub(super) mod corpus_ops_commands;
#[path = "corpus_pipeline_commands.rs"]
pub(super) mod corpus_pipeline_commands;
#[path = "corpus_ranking_commands.rs"]
pub(super) mod corpus_ranking_commands;
#[path = "corpus_report_commands.rs"]
pub(super) mod corpus_report_commands;
#[path = "corpus_score_print_commands.rs"]
pub(super) mod corpus_score_print_commands;
#[path = "corpus_ssb_commands.rs"]
pub(super) mod corpus_ssb_commands;
#[path = "corpus_tier_commands.rs"]
pub(super) mod corpus_tier_commands;
#[path = "corpus_time_commands.rs"]
pub(super) mod corpus_time_commands;
#[path = "corpus_viz_commands.rs"]
pub(super) mod corpus_viz_commands;
#[path = "corpus_weight_commands.rs"]
pub(super) mod corpus_weight_commands;

// Re-export convert_lint_format at module scope (needed by lint_cmds via super::)
use make_cmds::convert_lint_format;

// Re-exports needed only by tests (command_tests.rs and inline test modules use `super::*`)
#[cfg(test)]
use config_cmds::{
    config_analyze_command, config_lint_command, count_duplicate_path_entries,
    handle_output_to_file, should_output_to_stdout,
};
#[cfg(test)]
use make_cmds::{make_lint_command, make_parse_command, make_purify_command, run_filtered_lint};
// Dockerfile and installer are sibling modules declared in cli/mod.rs.
// Re-export their public functions so command_tests.rs (`super::*`) can reach them.
#[cfg(test)]
use super::dockerfile_commands::{
    dockerfile_lint_command, dockerfile_purify_command, purify_dockerfile,
    DockerfilePurifyCommandArgs,
};
#[cfg(test)]
use super::dockerfile_profile_commands::{
    dockerfile_profile_command, dockerfile_size_check_command, estimate_build_time,
};
#[cfg(test)]
use super::dockerfile_validate_commands::dockerfile_full_validate_command;
#[cfg(test)]
use super::installer_commands::parse_public_key;

pub fn execute_command(cli: Cli) -> Result<()> {
    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Error::Internal(format!("Failed to initialize logging: {e}")))?;

    dispatch_command(
        cli.command,
        cli.target,
        cli.verify,
        cli.validation,
        cli.strict,
    )
}

fn dispatch_command(
    command: Commands,
    target: ShellDialect,
    verify: VerificationLevel,
    validation: ValidationLevel,
    strict: bool,
) -> Result<()> {
    match command {
        // Core transpilation commands
        Commands::Build { .. }
        | Commands::Check { .. }
        | Commands::Init { .. }
        | Commands::Verify { .. }
        | Commands::Inspect { .. }
        | Commands::Compile { .. } => dispatch_core(command, target, verify, validation, strict),

        // Analysis and transformation commands
        Commands::Lint { .. }
        | Commands::Purify { .. }
        | Commands::SafetyCheck { .. }
        | Commands::Explain { .. }
        | Commands::Fix { .. }
        | Commands::Classify { .. }
        | Commands::Format { .. } => dispatch_analysis(command, target, verify, validation, strict),

        // Quality and testing commands
        Commands::Test { .. }
        | Commands::Score { .. }
        | Commands::Audit { .. }
        | Commands::Coverage { .. }
        | Commands::Ext(CommandsExt::Bench { .. })
        | Commands::Ext(CommandsExt::Mutate { .. })
        | Commands::Ext(CommandsExt::Simulate { .. })
        | Commands::Gate { .. }
        | Commands::Ext(CommandsExt::Cfg { .. }) => dispatch_quality(command),

        // Delegated subcommand groups
        Commands::Make { command: cmd } => make_cmds::handle_make_command(cmd),
        Commands::Dockerfile { command: cmd } => {
            super::dockerfile_commands::handle_dockerfile_command(cmd)
        }
        Commands::Devcontainer { command: cmd } => {
            devcontainer_cmds::handle_devcontainer_command(cmd)
        }
        Commands::Config { command: cmd } => config_cmds::handle_config_command(cmd),
        Commands::Ext(CommandsExt::Installer { command: cmd }) => {
            super::installer_commands::handle_installer_command(cmd)
        }
        Commands::Comply { command: cmd } => comply_cmds::handle_comply_command(cmd),
        Commands::Corpus { command: cmd } => corpus_core_cmds::handle_corpus_command(cmd),

        // Interactive, misc, and generation commands
        Commands::Repl { .. }
        | Commands::Ext(CommandsExt::Playbook { .. })
        | Commands::Ext(CommandsExt::GenerateAdversarial { .. }) => dispatch_interactive(command),

        #[cfg(feature = "tui")]
        Commands::Tui => crate::tui::run()
            .map_err(|e| crate::models::Error::Io(std::io::Error::other(e.to_string()))),

        #[cfg(feature = "oracle")]
        Commands::Ext(CommandsExt::ExplainError { .. }) => dispatch_interactive(command),
    }
}

/// Dispatch core transpilation commands (Build, Check, Init, Verify, Inspect, Compile).
fn dispatch_core(
    command: Commands,
    target: ShellDialect,
    verify: VerificationLevel,
    validation: ValidationLevel,
    strict: bool,
) -> Result<()> {
    match command {
        Commands::Build {
            input,
            output,
            emit_proof,
            no_optimize,
        } => {
            let config = Config {
                target,
                verify,
                emit_proof,
                optimize: !no_optimize,
                validation_level: Some(validation),
                strict_mode: strict,
            };
            build_command(&input, &output, config)
        }
        Commands::Check { input } => check_command(&input),
        Commands::Init { path, name } => init_command(&path, name.as_deref()),
        Commands::Verify {
            rust_source,
            shell_script,
        } => verify_command(&rust_source, &shell_script, target, verify),
        Commands::Inspect {
            input,
            format,
            output,
            detailed,
        } => inspect_command(&input, format, output.as_deref(), detailed),
        Commands::Compile {
            rust_source,
            output,
            runtime,
            self_extracting,
            container,
            container_format,
        } => {
            let config = Config {
                target,
                verify,
                emit_proof: false,
                optimize: true,
                validation_level: Some(validation),
                strict_mode: strict,
            };
            handle_compile(
                &rust_source,
                &output,
                runtime,
                self_extracting,
                container,
                container_format,
                &config,
            )
        }
        _ => unreachable!("dispatch_core called with non-core command"),
    }
}

include!("commands_dispatch.rs");
