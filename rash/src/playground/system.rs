use crate::models::{Error, Result};
use crate::playground::{
    ComputationGraph, DocumentStore, EditorMode, KeymapEngine, RenderPipeline,
};
use std::sync::Arc;
use std::time::Instant;

/// Core playground system implementing reactive architecture
pub struct PlaygroundSystem {
    /// Document management with CRDT-like properties
    document_store: DocumentStore,

    /// Lock-free incremental computation graph
    computation_dag: Arc<ComputationGraph>,

    /// Zero-copy rendering pipeline
    render_pipeline: RenderPipeline,

    /// Current editor mode and keymap
    editor_mode: EditorMode,
    keymap_engine: KeymapEngine,

    /// VI-style editing enabled
    vi_mode: bool,

    /// Session metrics
    session_start: Instant,
    frame_count: u64,
}

impl PlaygroundSystem {
    /// Create a new playground system
    pub fn new(vi_mode: bool) -> Result<Self> {
        let document_store = DocumentStore::new()?;
        let computation_dag = Arc::new(ComputationGraph::new()?);
        let render_pipeline = RenderPipeline::new()?;
        let keymap_engine = KeymapEngine::new(vi_mode)?;

        Ok(Self {
            document_store,
            computation_dag,
            render_pipeline,
            editor_mode: if vi_mode {
                EditorMode::Normal {
                    pending_operator: None,
                    count: None,
                }
            } else {
                EditorMode::Insert {
                    completion_ctx: Default::default(),
                    snippet_engine: Default::default(),
                }
            },
            keymap_engine,
            vi_mode,
            session_start: Instant::now(),
            frame_count: 0,
        })
    }

    /// Load content into the playground
    pub fn load_content(&mut self, content: &str) -> Result<()> {
        self.document_store.load_content(content)?;

        // Mark entire document as dirty for recomputation
        self.computation_dag.mark_full_rebuild();

        Ok(())
    }

    /// Restore session from URL-encoded state
    pub fn restore_from_url(&mut self, _url: &str) -> Result<()> {
        // TODO: Implement URL state decoding
        Err(Error::Internal(
            "URL restoration not yet implemented".to_string(),
        ))
    }

    /// Run the playground main loop
    pub fn run(&mut self) -> Result<()> {
        #[cfg(feature = "playground")]
        {
            use ratatui::{backend::CrosstermBackend, Terminal};
            use std::io;

            // Setup terminal
            ratatui::crossterm::terminal::enable_raw_mode()
                .map_err(|e| Error::Internal(format!("Failed to enable raw mode: {e}")))?;

            let mut stdout = io::stdout();
            ratatui::crossterm::execute!(
                stdout,
                ratatui::crossterm::terminal::EnterAlternateScreen,
                ratatui::crossterm::event::EnableMouseCapture
            )
            .map_err(|e| Error::Internal(format!("Failed to setup terminal: {e}")))?;

            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)
                .map_err(|e| Error::Internal(format!("Failed to create terminal: {e}")))?;

            // Run main loop
            let result = self.main_loop(&mut terminal);

            // Cleanup terminal
            ratatui::crossterm::terminal::disable_raw_mode()
                .map_err(|e| Error::Internal(format!("Failed to disable raw mode: {e}")))?;

            ratatui::crossterm::execute!(
                terminal.backend_mut(),
                ratatui::crossterm::terminal::LeaveAlternateScreen,
                ratatui::crossterm::event::DisableMouseCapture
            )
            .map_err(|e| Error::Internal(format!("Failed to cleanup terminal: {e}")))?;

            terminal
                .show_cursor()
                .map_err(|e| Error::Internal(format!("Failed to show cursor: {e}")))?;

            result
        }

        #[cfg(not(feature = "playground"))]
        Err(Error::Internal(
            "Playground feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "playground")]
    fn main_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        use ratatui::crossterm::event::{self, Event};
        use std::time::Duration;

        loop {
            // Render frame
            let frame_start = Instant::now();
            terminal
                .draw(|f| self.render_frame(f))
                .map_err(|e| Error::Internal(format!("Render failed: {e}")))?;

            self.frame_count += 1;
            let render_time = frame_start.elapsed();

            // Performance monitoring - warn if frame took too long
            if render_time > Duration::from_millis(16) {
                tracing::warn!("Slow frame: {:?} (target: 16ms)", render_time);
            }

            // Handle input with timeout for smooth animation
            if event::poll(Duration::from_millis(16))
                .map_err(|e| Error::Internal(format!("Event poll failed: {e}")))?
            {
                match event::read()
                    .map_err(|e| Error::Internal(format!("Event read failed: {e}")))?
                {
                    Event::Key(key_event) => {
                        if self.handle_key_event(key_event)? {
                            break; // Exit requested
                        }
                    }
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height)?;
                    }
                    _ => {}
                }
            }

            // Process any pending computations
            self.computation_dag.process_pending()?;
        }

        Ok(())
    }

    #[cfg(feature = "playground")]
    fn render_frame(&mut self, f: &mut ratatui::Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Style},
            widgets::{Block, Borders, Paragraph},
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 2), // Source editor
                Constraint::Ratio(1, 2), // Output panel
            ])
            .split(f.area());

        // Render source editor
        let source_content = self.document_store.get_content();
        let source_widget = Paragraph::new(source_content)
            .block(Block::default().title("Rust Source").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(source_widget, chunks[0]);

        // Render transpiled output
        let output_content = self.get_transpiled_output();
        let output_widget = Paragraph::new(output_content)
            .block(Block::default().title("Shell Output").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green));

        f.render_widget(output_widget, chunks[1]);

        // Render status line
        self.render_status_line(f, f.area());
    }

    #[cfg(feature = "playground")]
    fn render_status_line(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::{
            layout::Rect,
            style::{Color, Style},
            text::Line,
            widgets::Paragraph,
        };

        let status_area = Rect {
            x: area.x,
            y: area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        let mode_str = match &self.editor_mode {
            EditorMode::Normal { .. } => "NORMAL",
            EditorMode::Insert { .. } => "INSERT",
            EditorMode::Visual { .. } => "VISUAL",
            EditorMode::Command { .. } => "COMMAND",
        };

        let fps = if self.frame_count > 0 {
            self.frame_count as f64 / self.session_start.elapsed().as_secs_f64()
        } else {
            0.0
        };

        let status_text = format!(
            " {} | FPS: {:.1} | Frame: {} ",
            mode_str, fps, self.frame_count
        );

        let status_widget = Paragraph::new(Line::from(status_text))
            .style(Style::default().bg(Color::Blue).fg(Color::White));

        f.render_widget(status_widget, status_area);
    }

    fn handle_key_event(&mut self, key_event: ratatui::crossterm::event::KeyEvent) -> Result<bool> {
        use ratatui::crossterm::event::KeyCode;

        // Global shortcuts
        match key_event.code {
            KeyCode::Esc if matches!(self.editor_mode, EditorMode::Insert { .. }) => {
                if self.vi_mode {
                    self.editor_mode = EditorMode::Normal {
                        pending_operator: None,
                        count: None,
                    };
                }
                return Ok(false);
            }
            KeyCode::Char('q') if matches!(self.editor_mode, EditorMode::Normal { .. }) => {
                return Ok(true); // Exit
            }
            _ => {}
        }

        // Process through keymap engine
        if let Some(action) = self.keymap_engine.process_key(key_event, &self.editor_mode) {
            self.execute_action(action)?;
        }

        Ok(false)
    }

    fn handle_resize(&mut self, _width: u16, _height: u16) -> Result<()> {
        // Mark render pipeline for full refresh
        self.render_pipeline.mark_full_refresh();
        Ok(())
    }

    fn execute_action(&mut self, _action: crate::playground::editor::Action) -> Result<()> {
        // TODO: Implement action execution
        Ok(())
    }

    fn get_transpiled_output(&self) -> String {
        // TODO: Get actual transpiled output from computation graph
        "// Transpiled shell output will appear here\necho \"Hello, World!\"".to_string()
    }
    
    /// Get access to document content for testing
    #[cfg(test)]
    pub fn get_document_content(&self) -> String {
        self.document_store.get_content()
    }
}
