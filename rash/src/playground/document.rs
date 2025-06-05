use crate::models::{Error, Result};
use crate::playground::parser::{IncrementalParser, TextEdit};

/// Document management with CRDT-like properties
pub struct DocumentStore {
    #[cfg(feature = "playground")]
    rope: ropey::Rope,

    #[cfg(feature = "playground")]
    syntax_tree: Option<tree_sitter::Tree>,
    
    #[cfg(feature = "playground")]
    parser: IncrementalParser,

    version: u64,

    #[cfg(not(feature = "playground"))]
    content: String,
}

impl DocumentStore {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "playground")]
        {
            let parser = IncrementalParser::new()?;
            Ok(Self {
                rope: ropey::Rope::new(),
                syntax_tree: None,
                parser,
                version: 0,
            })
        }

        #[cfg(not(feature = "playground"))]
        {
            Ok(Self {
                content: String::new(),
                version: 0,
            })
        }
    }

    pub fn load_content(&mut self, content: &str) -> Result<()> {
        #[cfg(feature = "playground")]
        {
            self.rope = ropey::Rope::from_str(content);
            // Parse initial content with tree-sitter
            self.syntax_tree = Some(self.parser.parse_initial(content)?);
            self.version += 1;
        }

        #[cfg(not(feature = "playground"))]
        {
            self.content = content.to_string();
            self.version += 1;
        }

        Ok(())
    }

    pub fn get_content(&self) -> String {
        #[cfg(feature = "playground")]
        {
            self.rope.to_string()
        }

        #[cfg(not(feature = "playground"))]
        {
            self.content.clone()
        }
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }

    #[cfg(feature = "playground")]
    pub fn get_rope(&self) -> &ropey::Rope {
        &self.rope
    }

    #[cfg(feature = "playground")]
    pub fn apply_edit(&mut self, start: usize, end: usize, text: &str) -> Result<()> {
        if start > self.rope.len_bytes() || end > self.rope.len_bytes() || start > end {
            return Err(Error::Internal("Invalid edit range".to_string()));
        }

        // Apply incremental parsing if we have a syntax tree
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = TextEdit {
                start_byte: start,
                old_end_byte: end,
                new_end_byte: start + text.len(),
            };
            
            // Apply the edit to the rope first
            self.rope.remove(start..end);
            self.rope.insert(start, text);
            
            // Then update the syntax tree incrementally
            let parse_delta = self.parser.apply_edit(tree, &edit, &self.rope)?;
            self.syntax_tree = Some(parse_delta.tree);
        } else {
            // No syntax tree yet, just update the rope
            self.rope.remove(start..end);
            self.rope.insert(start, text);
            
            // Parse from scratch
            let content = self.rope.to_string();
            self.syntax_tree = Some(self.parser.parse_initial(&content)?);
        }
        
        self.version += 1;
        Ok(())
    }

    #[cfg(feature = "playground")]
    pub fn insert_text(&mut self, pos: usize, text: &str) -> Result<()> {
        if pos > self.rope.len_bytes() {
            return Err(Error::Internal("Insert position out of bounds".to_string()));
        }

        self.rope.insert(pos, text);
        self.version += 1;
        self.syntax_tree = None;

        Ok(())
    }

    #[cfg(feature = "playground")]
    pub fn delete_range(&mut self, start: usize, end: usize) -> Result<()> {
        if start > self.rope.len_bytes() || end > self.rope.len_bytes() || start > end {
            return Err(Error::Internal("Invalid delete range".to_string()));
        }

        self.rope.remove(start..end);
        self.version += 1;
        self.syntax_tree = None;

        Ok(())
    }

    #[cfg(feature = "playground")]
    pub fn get_line_count(&self) -> usize {
        self.rope.len_lines()
    }

    #[cfg(feature = "playground")]
    pub fn get_line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            return None;
        }

        let line = self.rope.line(line_idx);
        Some(line.to_string())
    }

    #[cfg(feature = "playground")]
    pub fn byte_to_line_col(&self, byte_pos: usize) -> (usize, usize) {
        if byte_pos > self.rope.len_bytes() {
            let last_line = self.rope.len_lines().saturating_sub(1);
            let last_line_len = self.rope.line(last_line).len_bytes();
            return (last_line, last_line_len);
        }

        let line_idx = self.rope.byte_to_line(byte_pos);
        let line_start = self.rope.line_to_byte(line_idx);
        let col_idx = byte_pos - line_start;

        (line_idx, col_idx)
    }

    #[cfg(feature = "playground")]
    pub fn line_col_to_byte(&self, line: usize, col: usize) -> usize {
        if line >= self.rope.len_lines() {
            return self.rope.len_bytes();
        }

        let line_start = self.rope.line_to_byte(line);
        let line_len = self.rope.line(line).len_bytes();

        line_start + col.min(line_len)
    }
}
