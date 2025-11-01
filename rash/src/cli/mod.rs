pub mod args;
pub mod bench;
pub mod commands;

#[cfg(test)]
mod tests;

pub use args::{Cli, Commands, ConfigCommands, ConfigOutputFormat};
pub use commands::execute_command;
