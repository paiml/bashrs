//! TUI Event Handling
//!
//! Event types and handlers for the TUI.

use crossterm::event::KeyCode;

/// TUI events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Key press event
    Key(KeyCode),
    /// Focus change event
    FocusEditor,
    /// Lint request
    LintRequest,
    /// Lint completed
    LintComplete,
    /// Lint error
    LintError(String),
    /// Purify request
    PurifyRequest,
    /// Purify completed
    PurifyComplete,
    /// Purify error
    PurifyError(String),
    /// Edge case found (during fuzzing)
    EdgeCaseFound(String),
    /// Quit request
    QuitRequest,
    /// Quit confirmed
    QuitConfirmed,
    /// Help toggle
    HelpToggle,
    /// Mode change
    ModeChange(u8),
    /// Tick (for async operations)
    Tick,
}

/// Event handler trait for testing
pub trait EventHandler {
    /// Handle an event
    fn handle(&mut self, event: Event);

    /// Check if event was handled
    fn was_handled(&self, event: &Event) -> bool;
}

/// Simple event recorder for testing (used by probar tests)
#[derive(Debug, Default)]
#[cfg(test)]
struct EventRecorder {
    events: Vec<Event>,
}

#[cfg(test)]
impl EventRecorder {
    /// Create new recorder
    fn new() -> Self {
        Self::default()
    }

    /// Get recorded events
    fn events(&self) -> &[Event] {
        &self.events
    }

    /// Clear recorded events
    fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
impl EventHandler for EventRecorder {
    fn handle(&mut self, event: Event) {
        self.events.push(event);
    }

    fn was_handled(&self, event: &Event) -> bool {
        self.events.contains(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_recorder() {
        let mut recorder = EventRecorder::new();

        recorder.handle(Event::FocusEditor);
        recorder.handle(Event::LintRequest);

        assert!(recorder.was_handled(&Event::FocusEditor));
        assert!(recorder.was_handled(&Event::LintRequest));
        assert!(!recorder.was_handled(&Event::QuitRequest));

        assert_eq!(recorder.events().len(), 2);

        recorder.clear();
        assert!(recorder.events().is_empty());
    }

    #[test]
    fn test_event_equality() {
        assert_eq!(Event::FocusEditor, Event::FocusEditor);
        assert_ne!(Event::FocusEditor, Event::LintRequest);
    }
}
