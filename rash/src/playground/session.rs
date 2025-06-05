use crate::models::{Config, Error, Result};
// use crate::playground::editor::EditorMode;
use serde::{Deserialize, Serialize};

/// Session state for persistence and sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Document content
    #[serde(with = "rope_serde")]
    pub document: ropey::Rope,

    /// Cursor position
    pub cursor_position: CursorPosition,

    /// Layout configuration
    pub layout: LayoutStrategy,

    /// Transpiler configuration
    pub transpiler_config: Config,

    /// Compressed history for undo/redo
    #[serde(with = "compressed_history")]
    pub history: History,

    /// Session metrics
    pub session_metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutStrategy {
    Vertical { ratio: f32 },
    Horizontal { ratio: f32 },
    Tabbed { active: usize },
}

impl Default for LayoutStrategy {
    fn default() -> Self {
        Self::Vertical { ratio: 1.618 } // Golden ratio
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
    pub current_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub action: EditAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditAction {
    Insert {
        pos: usize,
        text: String,
    },
    Delete {
        start: usize,
        end: usize,
    },
    Replace {
        start: usize,
        end: usize,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_edits: usize,
    pub total_transpilations: usize,
    pub session_duration_secs: u64,
    pub avg_transpilation_time_ms: f64,
}

/// Minimal state for URL encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalState {
    pub source: String,
    pub config: Config,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            document: ropey::Rope::new(),
            cursor_position: CursorPosition {
                line: 0,
                column: 0,
                offset: 0,
            },
            layout: LayoutStrategy::default(),
            transpiler_config: Config::default(),
            history: History {
                entries: Vec::new(),
                current_index: 0,
            },
            session_metrics: SessionMetrics {
                total_edits: 0,
                total_transpilations: 0,
                session_duration_secs: 0,
                avg_transpilation_time_ms: 0.0,
            },
        }
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to URL-encoded string for sharing
    pub fn to_url(&self) -> Result<String> {
        let minimal = MinimalState {
            source: self.document.to_string(),
            config: self.transpiler_config.clone(),
        };

        // Serialize to JSON
        let json = serde_json::to_vec(&minimal)
            .map_err(|e| Error::Internal(format!("Failed to serialize state: {e}")))?;

        // Compress with Brotli
        let mut encoder = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
        std::io::Write::write_all(&mut encoder, &json)
            .map_err(|e| Error::Internal(format!("Compression failed: {e}")))?;
        let compressed_data = encoder.into_inner();

        // Base64 encode
        use base64::Engine;
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed_data);

        Ok(format!("https://play.rash-lang.org/?c={encoded}"))
    }

    /// Restore from URL-encoded string
    pub fn from_url(url: &str) -> Result<Self> {
        // Extract the encoded part
        let encoded = url
            .strip_prefix("https://play.rash-lang.org/?c=")
            .or_else(|| url.strip_prefix("?c="))
            .ok_or_else(|| Error::Internal("Invalid URL format".to_string()))?;

        // Base64 decode
        use base64::Engine;
        let compressed = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| Error::Internal(format!("Failed to decode base64: {e}")))?;

        // Decompress
        let mut decompressor = brotli::Decompressor::new(&compressed[..], 4096);
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decompressor, &mut decompressed)
            .map_err(|e| Error::Internal(format!("Failed to decompress: {e}")))?;

        // Deserialize
        let minimal: MinimalState = serde_json::from_slice(&decompressed)
            .map_err(|e| Error::Internal(format!("Failed to deserialize: {e}")))?;

        // Create full state from minimal
        let mut state = Self::new();
        state.document = ropey::Rope::from_str(&minimal.source);
        state.transpiler_config = minimal.config;

        Ok(state)
    }

    /// Save session to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Internal(format!("Failed to serialize session: {e}")))?;

        std::fs::write(path, json)
            .map_err(|e| Error::Internal(format!("Failed to write session file: {e}")))?;

        Ok(())
    }

    /// Load session from file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| Error::Internal(format!("Failed to read session file: {e}")))?;

        let state = serde_json::from_str(&json)
            .map_err(|e| Error::Internal(format!("Failed to deserialize session: {e}")))?;

        Ok(state)
    }
}

/// Custom serde implementation for Rope
mod rope_serde {
    use ropey::Rope;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(rope: &Rope, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as chunks for efficiency
        let chunks: Vec<&str> = rope.chunks().collect();
        chunks.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rope, D::Error>
    where
        D: Deserializer<'de>,
    {
        let chunks: Vec<String> = Vec::deserialize(deserializer)?;
        Ok(Rope::from_iter(chunks.iter().map(|s| s.as_str())))
    }
}

/// Custom serde for compressed history
mod compressed_history {
    use super::History;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(history: &History, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize last 100 entries to keep size reasonable
        let recent_entries = history
            .entries
            .iter()
            .rev()
            .take(100)
            .rev()
            .collect::<Vec<_>>();

        (recent_entries, history.current_index).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<History, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (entries, current_index): (Vec<_>, usize) = Deserialize::deserialize(deserializer)?;
        Ok(History {
            entries,
            current_index,
        })
    }
}
