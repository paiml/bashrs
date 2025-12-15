//! TUI Panel Components
//!
//! Individual panel widgets for the TUI.

/// Panel kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Editor,
    LintResults,
    Purified,
    Quality,
    EdgeCases,
    Status,
}

impl PanelKind {
    /// Get panel title
    pub fn title(&self) -> &'static str {
        match self {
            Self::Editor => "EDITOR",
            Self::LintResults => "LINT RESULTS",
            Self::Purified => "PURIFIED",
            Self::Quality => "QUALITY",
            Self::EdgeCases => "EDGE CASES",
            Self::Status => "STATUS",
        }
    }

    /// Get panel keybinding
    pub fn keybinding(&self) -> Option<&'static str> {
        match self {
            Self::Editor => Some("Tab"),
            Self::LintResults => Some("F2"),
            Self::Purified => Some("F3"),
            Self::Quality => Some("F4"),
            Self::EdgeCases => Some("F5"),
            Self::Status => None,
        }
    }
}

/// Panel state
#[derive(Debug, Clone, Default)]
pub struct Panel {
    /// Panel kind
    pub kind: Option<PanelKind>,
    /// Is focused
    pub focused: bool,
    /// Content
    pub content: String,
    /// Scroll offset
    pub scroll: u16,
}

impl Panel {
    /// Create new panel
    pub fn new(kind: PanelKind) -> Self {
        Self {
            kind: Some(kind),
            focused: false,
            content: String::new(),
            scroll: 0,
        }
    }

    /// Set content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_kind_title() {
        assert_eq!(PanelKind::Editor.title(), "EDITOR");
        assert_eq!(PanelKind::LintResults.title(), "LINT RESULTS");
    }

    #[test]
    fn test_panel_kind_keybinding() {
        assert_eq!(PanelKind::Editor.keybinding(), Some("Tab"));
        assert_eq!(PanelKind::Status.keybinding(), None);
    }

    #[test]
    fn test_panel_scroll() {
        let mut panel = Panel::new(PanelKind::Editor);
        assert_eq!(panel.scroll, 0);

        panel.scroll_down();
        assert_eq!(panel.scroll, 1);

        panel.scroll_up();
        assert_eq!(panel.scroll, 0);

        // Can't scroll below 0
        panel.scroll_up();
        assert_eq!(panel.scroll, 0);
    }

    #[test]
    fn test_panel_content() {
        let mut panel = Panel::new(PanelKind::Editor);
        panel.set_content("test content");
        assert_eq!(panel.content, "test content");
    }
}
