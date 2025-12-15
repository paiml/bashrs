//! TUI Rendering
//!
//! Renders the multi-panel TUI layout using ratatui.

// Layout indexing is safe - we create exact arrays we index into
#![allow(clippy::indexing_slicing)]

use super::app::{App, AppState, FocusedPanel};
use crate::linter::Severity;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Render the full TUI frame
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    render_title_bar(frame, chunks[0], app);
    render_main_content(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);

    // Render overlays
    if app.state == AppState::ShowingHelp {
        render_help_overlay(frame, area);
    } else if app.state == AppState::ConfirmingQuit {
        render_quit_dialog(frame, area);
    }
}

/// Render title bar
fn render_title_bar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let title = format!(
        " bashrs TUI v{} │ Mode: {} │ [F1:Help] [q:Quit]",
        env!("CARGO_PKG_VERSION"),
        app.mode.name()
    );
    let title_widget =
        Paragraph::new(title).style(Style::default().bg(Color::Blue).fg(Color::White).bold());
    frame.render_widget(title_widget, area);
}

/// Render main content area with 4 panels
fn render_main_content(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // Split into 2x2 grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    // Render panels
    render_editor_panel(frame, top_cols[0], app);
    render_lint_panel(frame, top_cols[1], app);
    render_purified_panel(frame, bottom_cols[0], app);
    render_quality_panel(frame, bottom_cols[1], app);
}

/// Render editor panel
fn render_editor_panel(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focused == FocusedPanel::Editor;
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" EDITOR [Tab] ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if app.editor_content.is_empty() {
        "Enter bash script here...\n\nPress F2 to lint\nPress F3 to purify".to_string()
    } else {
        app.editor_content.clone()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render lint results panel
fn render_lint_panel(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focused == FocusedPanel::LintResults;
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" LINT RESULTS [F2] ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem<'_>> = if app.diagnostics.is_empty() {
        vec![ListItem::new("No issues found").style(Style::default().fg(Color::Green))]
    } else {
        app.diagnostics
            .iter()
            .map(|d| {
                let style = match d.severity {
                    Severity::Error => Style::default().fg(Color::Red),
                    Severity::Warning => Style::default().fg(Color::Yellow),
                    Severity::Info => Style::default().fg(Color::Cyan),
                    Severity::Note => Style::default().fg(Color::Gray),
                    Severity::Perf => Style::default().fg(Color::Magenta),
                    Severity::Risk => Style::default().fg(Color::Red),
                };
                ListItem::new(format!(
                    "[{}] {}:{} - {}",
                    d.code, d.span.start_line, d.span.start_col, d.message
                ))
                .style(style)
            })
            .collect()
    };

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Render purified output panel
fn render_purified_panel(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focused == FocusedPanel::Purified;
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" PURIFIED [F3] ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if app.purified_output.is_empty() {
        "Purified output will appear here...\n\nDeterministic, idempotent POSIX sh".to_string()
    } else {
        app.purified_output.clone()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);
}

/// Render quality metrics panel
fn render_quality_panel(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focused == FocusedPanel::Quality;
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" QUALITY [F4] ")
        .borders(Borders::ALL)
        .border_style(border_style);

    // Generate progress bar
    let coverage_bar = progress_bar(app.coverage, 20);
    let score_bar = progress_bar(app.quality_score, 20);

    let content = format!(
        "Coverage: {:>5.1}% {}\n\
         Score:    {:>5.1}  {}\n\
         Issues:   {:>5}\n\
         Edges:    {:>5}",
        app.coverage,
        coverage_bar,
        app.quality_score,
        score_bar,
        app.diagnostics.len(),
        app.edge_cases.len()
    );

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

/// Render status bar
fn render_status_bar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mode_keys = "1:Normal 2:Purify 3:Lint 4:Debug 5:Explain 6:Fuzz";
    let status = format!(" {} │ {} │ {:?}", app.status, mode_keys, app.focused);
    let status_widget =
        Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(status_widget, area);
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame<'_>, area: Rect) {
    let help_text = r#"
    bashrs TUI Help
    ═══════════════

    Navigation:
      Tab        Cycle panel focus
      F1         Toggle help
      q          Quit

    Modes (number keys):
      1          Normal mode
      2          Purify mode
      3          Lint mode
      4          Debug mode
      5          Explain mode
      6          Fuzz mode

    Actions:
      F2         Run linter
      F3         Run purifier
      F4         Show quality metrics

    Editor:
      Type       Enter text
      Backspace  Delete character
      Enter      New line
      Esc        Clear focus

    Press any key to close...
    "#;

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let centered = centered_rect(60, 80, area);
    frame.render_widget(ratatui::widgets::Clear, centered);
    frame.render_widget(
        Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: false }),
        centered,
    );
}

/// Render quit confirmation dialog
fn render_quit_dialog(frame: &mut Frame<'_>, area: Rect) {
    let dialog = Paragraph::new(" Quit bashrs TUI? (y/n) ")
        .block(
            Block::default()
                .title(" Confirm ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Red).fg(Color::White)),
        )
        .alignment(Alignment::Center);

    let centered = centered_rect(30, 20, area);
    frame.render_widget(ratatui::widgets::Clear, centered);
    frame.render_widget(dialog, centered);
}

/// Create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Generate ASCII progress bar
fn progress_bar(value: f64, width: usize) -> String {
    let filled = ((value / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0.0, 10), "[░░░░░░░░░░]");
        assert_eq!(progress_bar(50.0, 10), "[█████░░░░░]");
        assert_eq!(progress_bar(100.0, 10), "[██████████]");
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);
        assert!(centered.x > 0);
        assert!(centered.y > 0);
        assert!(centered.width < 100);
        assert!(centered.height < 100);
    }
}
