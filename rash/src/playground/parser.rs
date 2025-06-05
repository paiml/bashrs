use crate::models::{Error, Result};
use std::time::Instant;

/// Tree-sitter based incremental parser for Rust code
pub struct IncrementalParser {
    #[cfg(feature = "playground")]
    parser: tree_sitter::Parser,

    #[cfg(feature = "playground")]
    #[allow(dead_code)]
    query_cache: std::collections::HashMap<&'static str, tree_sitter::Query>,

    edit_distance_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct ParseDelta {
    pub duration: std::time::Duration,
    pub changed_ranges: Vec<AstRange>,
    #[cfg(feature = "playground")]
    pub tree: tree_sitter::Tree,
}

#[derive(Debug, Clone)]
pub struct AstRange {
    pub start: usize,
    pub end: usize,
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: String,
    pub start: usize,
    pub end: usize,
}

#[cfg(feature = "playground")]
pub struct TextEdit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
}

impl IncrementalParser {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "playground")]
        {
            let parser = tree_sitter::Parser::new();

            // Skip language setup for now
            // In production, we'd load the Rust grammar

            Ok(Self {
                parser,
                query_cache: std::collections::HashMap::new(),
                edit_distance_threshold: 1000,
            })
        }

        #[cfg(not(feature = "playground"))]
        {
            Ok(Self {
                edit_distance_threshold: 1000,
            })
        }
    }

    #[cfg(feature = "playground")]
    pub fn parse_initial(&mut self, text: &str) -> Result<tree_sitter::Tree> {
        // Set up the Rust language grammar
        #[cfg(feature = "playground")]
        {
            extern crate tree_sitter_rust;
            if self.parser.language().is_none() {
                self.parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| {
                        Error::Internal(format!("Failed to set Rust language grammar: {e}"))
                    })?;
            }
        }

        let tree = self
            .parser
            .parse(text, None)
            .ok_or_else(|| Error::Internal("Failed to parse Rust code".to_string()))?;
        Ok(tree)
    }

    #[cfg(feature = "playground")]
    pub fn apply_edit(
        &mut self,
        tree: &mut tree_sitter::Tree,
        edit: &TextEdit,
        rope: &ropey::Rope,
    ) -> Result<ParseDelta> {
        let start = Instant::now();

        // Convert rope coordinates to tree-sitter format
        let ts_edit = tree_sitter::InputEdit {
            start_byte: edit.start_byte,
            old_end_byte: edit.old_end_byte,
            new_end_byte: edit.new_end_byte,
            start_position: self.byte_to_point(rope, edit.start_byte),
            old_end_position: self.byte_to_point(rope, edit.old_end_byte),
            new_end_position: self.byte_to_point(rope, edit.new_end_byte),
        };

        tree.edit(&ts_edit);

        // Incremental parse with rope as input
        let new_tree = self
            .parser
            .parse(rope.to_string(), Some(tree))
            .ok_or_else(|| Error::Internal("Incremental parse failed".to_string()))?;

        // Compute changed nodes via tree diff
        let changes = tree
            .changed_ranges(&new_tree)
            .map(|range| AstRange {
                start: range.start_byte,
                end: range.end_byte,
                nodes: self.collect_nodes_in_range(&new_tree, range.start_byte, range.end_byte),
            })
            .collect();

        Ok(ParseDelta {
            duration: start.elapsed(),
            changed_ranges: changes,
            tree: new_tree,
        })
    }

    #[cfg(feature = "playground")]
    fn byte_to_point(&self, rope: &ropey::Rope, byte_offset: usize) -> tree_sitter::Point {
        let line = rope.byte_to_line(byte_offset.min(rope.len_bytes()));
        let line_start = rope.line_to_byte(line);
        let column = byte_offset.saturating_sub(line_start);

        tree_sitter::Point { row: line, column }
    }

    #[cfg(feature = "playground")]
    fn collect_nodes_in_range(
        &self,
        tree: &tree_sitter::Tree,
        start: usize,
        end: usize,
    ) -> Vec<AstNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.walk();

        // Walk the tree and collect nodes in range
        self.collect_nodes_recursive(&mut cursor, start, end, &mut nodes);

        nodes
    }

    #[cfg(feature = "playground")]
    #[allow(clippy::only_used_in_recursion)]
    fn collect_nodes_recursive(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        start: usize,
        end: usize,
        nodes: &mut Vec<AstNode>,
    ) {
        let node = cursor.node();
        let node_start = node.start_byte();
        let node_end = node.end_byte();

        // Skip nodes outside range
        if node_end < start || node_start > end {
            return;
        }

        // Add node if it's within range
        nodes.push(AstNode {
            kind: node.kind().to_string(),
            start: node_start,
            end: node_end,
        });

        // Recurse to children
        if cursor.goto_first_child() {
            loop {
                self.collect_nodes_recursive(cursor, start, end, nodes);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    /// Check if we should do a full reparse based on edit distance
    pub fn should_full_reparse(&self, edit_size: usize) -> bool {
        edit_size > self.edit_distance_threshold
    }
}

// For now, we'll use a simple parser without tree-sitter-rust
// In production, we'd add tree-sitter-rust as a dependency
#[cfg(feature = "playground")]
impl IncrementalParser {
    #[allow(dead_code)]
    fn setup_language(parser: &mut tree_sitter::Parser) -> Result<()> {
        #[cfg(feature = "playground")]
        {
            extern crate tree_sitter_rust;
            parser
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .map_err(|e| {
                    Error::Internal(format!("Failed to set Rust language grammar: {e}"))
                })?;
        }
        Ok(())
    }
}
