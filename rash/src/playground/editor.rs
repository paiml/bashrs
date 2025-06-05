use crate::models::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    pub trigger_pos: usize,
    pub prefix: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SnippetEngine {
    pub active_snippet: Option<String>,
    pub placeholders: Vec<(usize, usize)>, // (start, end) positions
    pub current_placeholder: usize,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
    pub mode: VisualMode,
}

#[derive(Debug, Clone)]
pub enum VisualMode {
    Character,
    Line,
    Block,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Delete,
    Change,
    Yank,
    Format,
}

/// Modal editing states for VI-style interface
#[derive(Debug, Clone)]
pub enum EditorMode {
    Normal {
        pending_operator: Option<Operator>,
        count: Option<usize>,
    },
    Insert {
        completion_ctx: CompletionContext,
        snippet_engine: SnippetEngine,
    },
    Visual {
        selection: Selection,
        mode: VisualMode,
    },
    Command {
        cmdline_buffer: String,
        history_index: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Action {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,

    // Editing
    InsertChar(char),
    DeleteChar,
    DeleteLine,
    NewLine,
    Undo,
    Redo,

    // Mode changes
    EnterInsertMode,
    EnterNormalMode,
    EnterVisualMode,
    EnterCommandMode,

    // File operations
    Save,
    Load,

    // Transpilation
    Transpile,
    ToggleWatch,

    // UI
    SwitchPanel,
    ToggleLayout,
    ShowHelp,

    // System
    Quit,
    ForceQuit,
}

#[cfg(feature = "playground")]
type KeyEvent = ratatui::crossterm::event::KeyEvent;

#[cfg(not(feature = "playground"))]
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[cfg(not(feature = "playground"))]
#[derive(Debug, Clone)]
pub enum KeyCode {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
}

#[cfg(not(feature = "playground"))]
#[derive(Debug, Clone)]
pub struct KeyModifiers;

#[derive(Debug)]
enum TrieResult {
    Match(Action),
    Prefix,
    NoMatch,
}

/// Trie node for efficient keymap matching
#[derive(Debug)]
struct TrieNode {
    action: Option<Action>,
    children: HashMap<String, TrieNode>,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            action: None,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, keys: &[String], action: Action) {
        if keys.is_empty() {
            self.action = Some(action);
            return;
        }

        let key = &keys[0];
        let child = self
            .children
            .entry(key.clone())
            .or_insert_with(TrieNode::new);
        child.insert(&keys[1..], action);
    }

    fn step(&self, key: &str) -> (Option<&TrieNode>, TrieResult) {
        if let Some(child) = self.children.get(key) {
            if child.action.is_some() {
                (
                    Some(child),
                    TrieResult::Match(child.action.clone().unwrap()),
                )
            } else {
                (Some(child), TrieResult::Prefix)
            }
        } else {
            (None, TrieResult::NoMatch)
        }
    }
}

/// Keymap engine with trie-based matching and timeout support
pub struct KeymapEngine {
    trie: TrieNode,
    current_node: Option<*const TrieNode>,
    timeout_ms: u64,
    sequence_start: Option<Instant>,
    vi_mode: bool,
}

unsafe impl Send for KeymapEngine {}
unsafe impl Sync for KeymapEngine {}

impl KeymapEngine {
    pub fn new(vi_mode: bool) -> Result<Self> {
        let mut engine = Self {
            trie: TrieNode::new(),
            current_node: None,
            timeout_ms: 500,
            sequence_start: None,
            vi_mode,
        };

        engine.setup_keymaps()?;
        Ok(engine)
    }

    fn setup_keymaps(&mut self) -> Result<()> {
        if self.vi_mode {
            self.setup_vi_keymaps()?;
        } else {
            self.setup_emacs_keymaps()?;
        }
        Ok(())
    }

    fn setup_vi_keymaps(&mut self) -> Result<()> {
        // Normal mode keymaps
        self.add_keymap(&["h"], Action::MoveLeft)?;
        self.add_keymap(&["j"], Action::MoveDown)?;
        self.add_keymap(&["k"], Action::MoveUp)?;
        self.add_keymap(&["l"], Action::MoveRight)?;
        self.add_keymap(&["w"], Action::MoveWordForward)?;
        self.add_keymap(&["b"], Action::MoveWordBackward)?;
        self.add_keymap(&["0"], Action::MoveLineStart)?;
        self.add_keymap(&["$"], Action::MoveLineEnd)?;
        self.add_keymap(&["g", "g"], Action::MoveFileStart)?;
        self.add_keymap(&["G"], Action::MoveFileEnd)?;

        self.add_keymap(&["i"], Action::EnterInsertMode)?;
        self.add_keymap(&["v"], Action::EnterVisualMode)?;
        self.add_keymap(&[":"], Action::EnterCommandMode)?;

        self.add_keymap(&["x"], Action::DeleteChar)?;
        self.add_keymap(&["d", "d"], Action::DeleteLine)?;
        self.add_keymap(&["u"], Action::Undo)?;
        self.add_keymap(&["C-r"], Action::Redo)?;

        self.add_keymap(&["q"], Action::Quit)?;

        Ok(())
    }

    fn setup_emacs_keymaps(&mut self) -> Result<()> {
        // Emacs-style keymaps
        self.add_keymap(&["C-f"], Action::MoveRight)?;
        self.add_keymap(&["C-b"], Action::MoveLeft)?;
        self.add_keymap(&["C-n"], Action::MoveDown)?;
        self.add_keymap(&["C-p"], Action::MoveUp)?;
        self.add_keymap(&["C-a"], Action::MoveLineStart)?;
        self.add_keymap(&["C-e"], Action::MoveLineEnd)?;
        self.add_keymap(&["M-f"], Action::MoveWordForward)?;
        self.add_keymap(&["M-b"], Action::MoveWordBackward)?;

        self.add_keymap(&["C-d"], Action::DeleteChar)?;
        self.add_keymap(&["C-k"], Action::DeleteLine)?;
        self.add_keymap(&["C-z"], Action::Undo)?;
        self.add_keymap(&["C-y"], Action::Redo)?;

        self.add_keymap(&["C-c", "C-c"], Action::Quit)?;

        Ok(())
    }

    fn add_keymap(&mut self, keys: &[&str], action: Action) -> Result<()> {
        let key_strings: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
        self.trie.insert(&key_strings, action);
        Ok(())
    }

    pub fn process_key(&mut self, key_event: KeyEvent, mode: &EditorMode) -> Option<Action> {
        let key_str = self.key_event_to_string(key_event);

        // Check for timeout
        if let Some(start) = self.sequence_start {
            if start.elapsed() > Duration::from_millis(self.timeout_ms) {
                self.reset();
            }
        }

        let current = if let Some(node_ptr) = self.current_node {
            unsafe { &*node_ptr }
        } else {
            &self.trie
        };

        let (next_node, result) = current.step(&key_str);

        match result {
            TrieResult::Match(action) => {
                self.reset();
                Some(action)
            }
            TrieResult::Prefix => {
                self.current_node = next_node.map(|n| n as *const _);
                if self.sequence_start.is_none() {
                    self.sequence_start = Some(Instant::now());
                }
                None
            }
            TrieResult::NoMatch => {
                self.reset();
                self.fallback_action(key_event, mode)
            }
        }
    }

    fn reset(&mut self) {
        self.current_node = None;
        self.sequence_start = None;
    }

    fn key_event_to_string(&self, key_event: KeyEvent) -> String {
        #[cfg(feature = "playground")]
        {
            use ratatui::crossterm::event::{KeyCode, KeyModifiers};

            let mut result = String::new();

            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                result.push_str("C-");
            }
            if key_event.modifiers.contains(KeyModifiers::ALT) {
                result.push_str("M-");
            }
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                result.push_str("S-");
            }

            match key_event.code {
                KeyCode::Char(c) => result.push(c),
                KeyCode::Esc => result.push_str("Esc"),
                KeyCode::Enter => result.push_str("Enter"),
                KeyCode::Backspace => result.push_str("Backspace"),
                KeyCode::Delete => result.push_str("Delete"),
                KeyCode::Up => result.push_str("Up"),
                KeyCode::Down => result.push_str("Down"),
                KeyCode::Left => result.push_str("Left"),
                KeyCode::Right => result.push_str("Right"),
                _ => result.push_str("Unknown"),
            }

            result
        }

        #[cfg(not(feature = "playground"))]
        {
            match key_event.code {
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Esc => "Esc".to_string(),
                KeyCode::Enter => "Enter".to_string(),
                KeyCode::Backspace => "Backspace".to_string(),
                KeyCode::Delete => "Delete".to_string(),
                KeyCode::Up => "Up".to_string(),
                KeyCode::Down => "Down".to_string(),
                KeyCode::Left => "Left".to_string(),
                KeyCode::Right => "Right".to_string(),
            }
        }
    }

    fn fallback_action(&self, key_event: KeyEvent, mode: &EditorMode) -> Option<Action> {
        #[cfg(feature = "playground")]
        {
            use ratatui::crossterm::event::KeyCode;

            match mode {
                EditorMode::Insert { .. } => match key_event.code {
                    KeyCode::Char(c) => Some(Action::InsertChar(c)),
                    KeyCode::Enter => Some(Action::NewLine),
                    KeyCode::Backspace => Some(Action::DeleteChar),
                    _ => None,
                },
                _ => None,
            }
        }

        #[cfg(not(feature = "playground"))]
        {
            match mode {
                EditorMode::Insert { .. } => match key_event.code {
                    KeyCode::Char(c) => Some(Action::InsertChar(c)),
                    KeyCode::Enter => Some(Action::NewLine),
                    KeyCode::Backspace => Some(Action::DeleteChar),
                    _ => None,
                },
                _ => None,
            }
        }
    }
}
