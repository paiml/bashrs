//! TUI Application State
//!
//! Core application state machine for the bashrs TUI.

use crate::linter::Diagnostic;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use std::time::Duration;

/// TUI operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Normal,
    Purify,
    Lint,
    Debug,
    Explain,
    Fuzz,
}

impl AppMode {
    /// Get mode display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Purify => "Purify",
            Self::Lint => "Lint",
            Self::Debug => "Debug",
            Self::Explain => "Explain",
            Self::Fuzz => "Fuzz",
        }
    }

    /// Get mode keybinding hint
    pub fn key(&self) -> char {
        match self {
            Self::Normal => '1',
            Self::Purify => '2',
            Self::Lint => '3',
            Self::Debug => '4',
            Self::Explain => '5',
            Self::Fuzz => '6',
        }
    }
}

/// Application state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Idle,
    Editing,
    Linting,
    Purifying,
    ShowingResults,
    ShowingHelp,
    ConfirmingQuit,
}

/// Focused panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Editor,
    LintResults,
    Purified,
    Quality,
}

/// Main TUI application
pub struct App {
    /// Current mode
    pub mode: AppMode,
    /// Current state
    pub state: AppState,
    /// Focused panel
    pub focused: FocusedPanel,
    /// Editor content
    pub editor_content: String,
    /// Cursor position in editor
    pub cursor_pos: usize,
    /// Lint diagnostics
    pub diagnostics: Vec<Diagnostic>,
    /// Purified output
    pub purified_output: String,
    /// Quality metrics
    pub quality_score: f64,
    /// Coverage percentage
    pub coverage: f64,
    /// Edge cases found
    pub edge_cases: Vec<String>,
    /// Should quit
    pub should_quit: bool,
    /// Status message
    pub status: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create new application
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            state: AppState::Idle,
            focused: FocusedPanel::Editor,
            editor_content: String::new(),
            cursor_pos: 0,
            diagnostics: Vec::new(),
            purified_output: String::new(),
            quality_score: 0.0,
            coverage: 0.0,
            edge_cases: Vec::new(),
            should_quit: false,
            status: "Ready".to_string(),
        }
    }

    /// Run the application main loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| super::ui::render(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Handle input events
    fn handle_events(&mut self) -> anyhow::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key.code);
                }
            }
        }
        Ok(())
    }

    /// Handle key press
    fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            AppState::ConfirmingQuit => self.handle_quit_confirm(key),
            AppState::ShowingHelp => self.handle_help(key),
            _ => self.handle_normal(key),
        }
    }

    /// Handle keys in normal state
    fn handle_normal(&mut self, key: KeyCode) {
        match key {
            // Quit
            KeyCode::Char('q') => {
                self.state = AppState::ConfirmingQuit;
                self.status = "Quit? (y/n)".to_string();
            }
            // Help
            KeyCode::F(1) => {
                self.state = AppState::ShowingHelp;
                self.status = "Help - press any key to close".to_string();
            }
            // Mode switching
            KeyCode::Char('1') => self.set_mode(AppMode::Normal),
            KeyCode::Char('2') => self.set_mode(AppMode::Purify),
            KeyCode::Char('3') => self.set_mode(AppMode::Lint),
            KeyCode::Char('4') => self.set_mode(AppMode::Debug),
            KeyCode::Char('5') => self.set_mode(AppMode::Explain),
            KeyCode::Char('6') => self.set_mode(AppMode::Fuzz),
            // Panel focus
            KeyCode::Tab => self.cycle_focus(),
            // Run lint
            KeyCode::F(2) => self.run_lint(),
            // Run purify
            KeyCode::F(3) => self.run_purify(),
            // Show quality
            KeyCode::F(4) => self.show_quality(),
            // Editor input
            KeyCode::Char(c) if self.focused == FocusedPanel::Editor => {
                self.editor_content.push(c);
                self.cursor_pos += 1;
                self.state = AppState::Editing;
            }
            KeyCode::Backspace if self.focused == FocusedPanel::Editor => {
                if !self.editor_content.is_empty() {
                    self.editor_content.pop();
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
            }
            KeyCode::Enter if self.focused == FocusedPanel::Editor => {
                self.editor_content.push('\n');
                self.cursor_pos += 1;
            }
            KeyCode::Esc => {
                self.state = AppState::Idle;
                self.status = "Ready".to_string();
            }
            _ => {}
        }
    }

    /// Handle quit confirmation
    fn handle_quit_confirm(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.should_quit = true;
            }
            _ => {
                self.state = AppState::Idle;
                self.status = "Ready".to_string();
            }
        }
    }

    /// Handle help screen
    fn handle_help(&mut self, _key: KeyCode) {
        self.state = AppState::Idle;
        self.status = "Ready".to_string();
    }

    /// Set mode
    fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        self.status = format!("Mode: {}", mode.name());
    }

    /// Cycle panel focus
    fn cycle_focus(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Editor => FocusedPanel::LintResults,
            FocusedPanel::LintResults => FocusedPanel::Purified,
            FocusedPanel::Purified => FocusedPanel::Quality,
            FocusedPanel::Quality => FocusedPanel::Editor,
        };
        self.status = format!("Focus: {:?}", self.focused);
    }

    /// Run linting on editor content
    fn run_lint(&mut self) {
        self.state = AppState::Linting;
        self.status = "Linting...".to_string();

        // Use existing linter infrastructure
        use crate::linter::lint_shell;
        let result = lint_shell(&self.editor_content);
        self.diagnostics = result.diagnostics;

        self.state = AppState::ShowingResults;
        self.status = format!("{} issues found", self.diagnostics.len());
    }

    /// Run purification on editor content
    fn run_purify(&mut self) {
        self.state = AppState::Purifying;
        self.status = "Purifying...".to_string();

        // Use existing purifier infrastructure
        use crate::repl::purifier::purify_bash;
        match purify_bash(&self.editor_content) {
            Ok(result) => {
                self.purified_output = result;
                self.status = "Purified successfully".to_string();
            }
            Err(e) => {
                self.purified_output = format!("Error: {}", e);
                self.status = "Purification failed".to_string();
            }
        }

        self.state = AppState::ShowingResults;
    }

    /// Show quality metrics
    fn show_quality(&mut self) {
        self.status = format!(
            "Coverage: {:.1}% | Score: {:.1}",
            self.coverage, self.quality_score
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.state, AppState::Idle);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_mode_switching() {
        let mut app = App::new();
        app.set_mode(AppMode::Lint);
        assert_eq!(app.mode, AppMode::Lint);
    }

    #[test]
    fn test_focus_cycling() {
        let mut app = App::new();
        assert_eq!(app.focused, FocusedPanel::Editor);
        app.cycle_focus();
        assert_eq!(app.focused, FocusedPanel::LintResults);
        app.cycle_focus();
        assert_eq!(app.focused, FocusedPanel::Purified);
        app.cycle_focus();
        assert_eq!(app.focused, FocusedPanel::Quality);
        app.cycle_focus();
        assert_eq!(app.focused, FocusedPanel::Editor);
    }

    #[test]
    fn test_quit_confirm() {
        let mut app = App::new();
        app.handle_key(KeyCode::Char('q'));
        assert_eq!(app.state, AppState::ConfirmingQuit);
        app.handle_key(KeyCode::Char('n'));
        assert_eq!(app.state, AppState::Idle);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_app_mode_names() {
        assert_eq!(AppMode::Normal.name(), "Normal");
        assert_eq!(AppMode::Purify.name(), "Purify");
        assert_eq!(AppMode::Lint.name(), "Lint");
    }
}
