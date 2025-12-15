//! Terminal User Interface (TUI) for bashrs
//!
//! Provides a multi-panel terminal interface for interactive shell analysis
//! using ratatui/crossterm. Integrates with the existing REPL components.
//!
//! # Panels
//!
//! - **Editor**: Script input/editing
//! - **Lint Results**: Real-time lint diagnostics
//! - **Purified**: Deterministic/idempotent output
//! - **Quality**: Coverage, mutation, grades
//! - **Status Bar**: Mode, test count, stats
//!
//! # Modes
//!
//! - Normal: Direct editing
//! - Purify: Show purification transforms
//! - Lint: Highlight lint issues
//! - Debug: Step-through execution
//! - Explain: Educational explanations
//! - Fuzz: Monte Carlo edge case detection
//!
//! # Testing
//!
//! TUI is tested via probar (jugar-probar) with:
//! - GUI coverage tracking (95%+ target)
//! - Frame capture assertions
//! - Deterministic replay
//! - Monte Carlo fuzzing

#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod events;
#[cfg(feature = "tui")]
mod panels;
#[cfg(feature = "tui")]
mod ui;

#[cfg(feature = "tui")]
pub use app::{App, AppMode, AppState};
#[cfg(feature = "tui")]
pub use events::{Event, EventHandler};
#[cfg(feature = "tui")]
pub use panels::{Panel, PanelKind};
#[cfg(feature = "tui")]
pub use ui::render;

/// Run the TUI application
#[cfg(feature = "tui")]
pub fn run() -> anyhow::Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::prelude::*;
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let result = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    #[test]
    fn test_tui_module_compiles() {
        // Verify module structure compiles
        assert!(true);
    }
}
