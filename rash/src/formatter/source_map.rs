//! Source mapping with character-level precision

use crate::formatter::types::*;
use std::collections::BTreeMap;

/// B+ tree for O(log n) point queries, O(k + log n) range queries
/// Simplified implementation using BTreeMap for now
#[derive(Debug, Clone)]
pub struct BPlusTree<K, V> {
    map: BTreeMap<K, V>,
}

impl<K: Ord + Clone, V: Clone> BPlusTree<K, V> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    pub fn search(&self, key: K) -> Option<V> {
        // Find the largest key <= search key
        self.map.range(..=key).next_back().map(|(_, v)| v.clone())
    }

    pub fn range_query(&self, start: K, end: K) -> Vec<(K, V)> {
        self.map
            .range(start..end)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<K: Ord + Clone, V: Clone> Default for BPlusTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Compressed span deltas for memory efficiency
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpanDelta {
    /// Starting position delta from previous span
    pub start_delta: u32,
    /// Length of the span
    pub length: u16,
    /// Transform ID that created this span
    pub transform_id: u16,
}

/// Character-level bidirectional source mapping
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// B+ tree for forward mapping (original -> formatted)
    forward: BPlusTree<CharPos, CharPos>,

    /// B+ tree for reverse mapping (formatted -> original)
    reverse: BPlusTree<CharPos, CharPos>,

    /// Compressed span deltas for memory efficiency
    deltas: Vec<SpanDelta>,

    /// Token boundaries for precise error reporting
    token_boundaries: Vec<TokenBoundary>,
}

#[derive(Debug, Clone)]
struct TokenBoundary {
    start: CharPos,
    end: CharPos,
    _token_type: TokenType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Word,
    Operator,
    String,
    Comment,
    Whitespace,
}

impl SourceMap {
    /// Create a new source map
    pub fn new() -> Self {
        Self {
            forward: BPlusTree::new(),
            reverse: BPlusTree::new(),
            deltas: Vec::new(),
            token_boundaries: Vec::new(),
        }
    }

    /// Create identity mapping for unchanged source
    pub fn identity(source_len: usize) -> Self {
        let mut map = Self::new();

        // Add identity mappings at start and end
        map.forward.insert(CharPos(0), CharPos(0));
        map.forward.insert(CharPos(source_len), CharPos(source_len));
        map.reverse.insert(CharPos(0), CharPos(0));
        map.reverse.insert(CharPos(source_len), CharPos(source_len));

        // Add single token boundary covering entire source
        map.token_boundaries.push(TokenBoundary {
            start: CharPos(0),
            end: CharPos(source_len),
            _token_type: TokenType::Word,
        });

        map
    }

    /// Add a mapping between original and formatted positions
    pub fn add_mapping(&mut self, original: CharPos, formatted: CharPos) {
        self.forward.insert(original, formatted);
        self.reverse.insert(formatted, original);
    }

    /// Add a span delta for compressed storage
    pub fn add_span_delta(&mut self, delta: SpanDelta) {
        self.deltas.push(delta);
    }

    /// Add token boundary information
    pub fn add_token_boundary(&mut self, start: CharPos, end: CharPos, token_type: TokenType) {
        self.token_boundaries.push(TokenBoundary {
            start,
            end,
            _token_type: token_type,
        });
    }

    /// Try to resolve position via identity mapping optimization.
    /// Returns Some(pos) if the forward map is a 2-entry identity mapping, None otherwise.
    fn try_identity_resolve(&self, pos: CharPos) -> Option<CharPos> {
        if self.forward.map.len() != 2 {
            return None;
        }
        let keys: Vec<_> = self.forward.map.keys().collect();
        if keys.len() != 2 || keys[0] != &CharPos(0) {
            return None;
        }
        let end_key = keys[1];
        let start_val = self.forward.map.get(&CharPos(0));
        let end_val = self.forward.map.get(end_key);
        if start_val == Some(&CharPos(0)) && end_val == Some(end_key) {
            Some(if pos.0 <= end_key.0 { pos } else { *end_key })
        } else {
            None
        }
    }

    /// Character-level precision with token boundary awareness
    pub fn resolve(&self, pos: CharPos) -> MappedPosition {
        let char_pos = self
            .try_identity_resolve(pos)
            .unwrap_or_else(|| self.forward.search(pos).unwrap_or(pos));

        let token_boundary = self.find_token_boundary(char_pos);

        MappedPosition {
            exact: char_pos,
            token_start: token_boundary.start,
            token_end: token_boundary.end,
        }
    }

    /// Reverse mapping from formatted to original
    pub fn resolve_reverse(&self, pos: CharPos) -> MappedPosition {
        // Find the nearest mapped position key
        let char_pos = self
            .reverse
            .map
            .range(..=pos)
            .next_back()
            .map_or(pos, |(k, _)| *k);
        let token_boundary = self.find_token_boundary_reverse(char_pos);

        MappedPosition {
            exact: char_pos,
            token_start: token_boundary.start,
            token_end: token_boundary.end,
        }
    }

    /// Find token boundary containing the given position
    fn find_token_boundary(&self, pos: CharPos) -> TokenBoundary {
        self.token_boundaries
            .iter()
            .find(|boundary| boundary.start <= pos && pos < boundary.end)
            .cloned()
            .unwrap_or(TokenBoundary {
                start: pos,
                end: CharPos(pos.0 + 1),
                _token_type: TokenType::Word,
            })
    }

    /// Find token boundary in reverse mapping
    fn find_token_boundary_reverse(&self, pos: CharPos) -> TokenBoundary {
        // For reverse mapping, we need to find the corresponding original token
        // This is simplified - would need more sophisticated logic in practice
        self.find_token_boundary(pos)
    }

    /// Get all mappings in a range
    pub fn mappings_in_range(&self, start: CharPos, end: CharPos) -> Vec<(CharPos, CharPos)> {
        self.forward.range_query(start, end)
    }

    /// Get statistics about the mapping
    pub fn stats(&self) -> SourceMapStats {
        SourceMapStats {
            forward_entries: self.forward.map.len(),
            reverse_entries: self.reverse.map.len(),
            span_deltas: self.deltas.len(),
            token_boundaries: self.token_boundaries.len(),
            memory_usage_bytes: self.estimate_memory_usage(),
        }
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimation
        let forward_size = self.forward.map.len() * (std::mem::size_of::<CharPos>() * 2);
        let reverse_size = self.reverse.map.len() * (std::mem::size_of::<CharPos>() * 2);
        let deltas_size = self.deltas.len() * std::mem::size_of::<SpanDelta>();
        let boundaries_size = self.token_boundaries.len() * std::mem::size_of::<TokenBoundary>();

        forward_size + reverse_size + deltas_size + boundaries_size
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about source map memory usage and performance
#[derive(Debug, Clone)]
pub struct SourceMapStats {
    pub forward_entries: usize,
    pub reverse_entries: usize,
    pub span_deltas: usize,
    pub token_boundaries: usize,
    pub memory_usage_bytes: usize,
}

impl SourceMapStats {
    /// Calculate compression ratio compared to naive mapping
    pub fn compression_ratio(&self, source_chars: usize) -> f64 {
        let naive_size = source_chars * std::mem::size_of::<CharPos>() * 2; // Forward + reverse
        if naive_size == 0 {
            1.0
        } else {
            self.memory_usage_bytes as f64 / naive_size as f64
        }
    }

    /// Get human-readable memory usage
    pub fn memory_usage_human(&self) -> String {
        let bytes = self.memory_usage_bytes;
        if bytes < 1024 {
            format!("{bytes} B")
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Builder for constructing source maps incrementally
pub struct SourceMapBuilder {
    map: SourceMap,
    last_original: CharPos,
    last_formatted: CharPos,
    has_mappings: bool,
}

impl SourceMapBuilder {
    pub fn new() -> Self {
        Self {
            map: SourceMap::new(),
            last_original: CharPos(0),
            last_formatted: CharPos(0),
            has_mappings: false,
        }
    }

    /// Add a character-level mapping
    pub fn add_char_mapping(&mut self, original: CharPos, formatted: CharPos) {
        self.map.add_mapping(original, formatted);

        // Calculate delta if this is not the first mapping
        if self.has_mappings {
            let start_delta = original.0.saturating_sub(self.last_original.0) as u32;
            let length = formatted.0.saturating_sub(self.last_formatted.0) as u16;

            // Add span delta only if it makes sense (non-zero length)
            if length > 0 {
                self.map.add_span_delta(SpanDelta {
                    start_delta,
                    length,
                    transform_id: 0, // Would be set by caller
                });
            }
        }

        self.last_original = original;
        self.last_formatted = formatted;
        self.has_mappings = true;
    }

    /// Add a range mapping
    pub fn add_range_mapping(
        &mut self,
        original_start: CharPos,
        original_end: CharPos,
        formatted_start: CharPos,
        formatted_end: CharPos,
    ) {
        self.add_char_mapping(original_start, formatted_start);
        self.add_char_mapping(original_end, formatted_end);
    }

    /// Add token boundary
    pub fn add_token(&mut self, start: CharPos, end: CharPos, token_type: TokenType) {
        self.map.add_token_boundary(start, end, token_type);
    }

    /// Build the final source map
    pub fn build(self) -> SourceMap {
        self.map
    }
}

impl Default for SourceMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bplus_tree_operations() {
        let mut tree = BPlusTree::new();
        tree.insert(CharPos(10), CharPos(20));
        tree.insert(CharPos(5), CharPos(15));
        tree.insert(CharPos(15), CharPos(25));

        // Test exact match
        assert_eq!(tree.search(CharPos(10)), Some(CharPos(20)));

        // Test range query
        let results = tree.range_query(CharPos(5), CharPos(15));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_source_map_identity() {
        let map = SourceMap::identity(100);

        let mapped = map.resolve(CharPos(50));
        assert_eq!(mapped.exact, CharPos(50));
        assert_eq!(mapped.token_start, CharPos(0));
        assert_eq!(mapped.token_end, CharPos(100));
    }

    #[test]
    fn test_source_map_mappings() {
        let mut map = SourceMap::new();
        map.add_mapping(CharPos(10), CharPos(20));
        map.add_mapping(CharPos(30), CharPos(35));

        let mapped = map.resolve(CharPos(25));
        assert_eq!(mapped.exact, CharPos(20)); // Should find the largest key <= 25
    }

    #[test]
    fn test_source_map_reverse() {
        let mut map = SourceMap::new();
        map.add_mapping(CharPos(10), CharPos(20));
        map.add_mapping(CharPos(30), CharPos(35));

        let mapped = map.resolve_reverse(CharPos(25));
        assert_eq!(mapped.exact, CharPos(20));
    }

    #[test]
    fn test_source_map_builder() {
        let mut builder = SourceMapBuilder::new();
        builder.add_char_mapping(CharPos(0), CharPos(0));
        builder.add_char_mapping(CharPos(10), CharPos(15));
        builder.add_char_mapping(CharPos(20), CharPos(30));

        let map = builder.build();
        let stats = map.stats();

        assert_eq!(stats.forward_entries, 3);
        assert_eq!(stats.reverse_entries, 3);
        assert_eq!(stats.span_deltas, 2); // Two deltas from three mappings
    }

    #[test]
    fn test_span_delta_creation() {
        let delta = SpanDelta {
            start_delta: 10,
            length: 5,
            transform_id: 1,
        };

        assert_eq!(delta.start_delta, 10);
        assert_eq!(delta.length, 5);
        assert_eq!(delta.transform_id, 1);
    }

    #[test]
    fn test_token_boundaries() {
        let mut map = SourceMap::new();
        map.add_token_boundary(CharPos(0), CharPos(5), TokenType::Word);
        map.add_token_boundary(CharPos(5), CharPos(6), TokenType::Whitespace);
        map.add_token_boundary(CharPos(6), CharPos(11), TokenType::String);

        let boundary = map.find_token_boundary(CharPos(3));
        assert_eq!(boundary.start, CharPos(0));
        assert_eq!(boundary.end, CharPos(5));
        assert_eq!(boundary._token_type, TokenType::Word);

        let boundary2 = map.find_token_boundary(CharPos(8));
        assert_eq!(boundary2.start, CharPos(6));
        assert_eq!(boundary2.end, CharPos(11));
        assert_eq!(boundary2._token_type, TokenType::String);
    }

    #[test]
    fn test_source_map_stats() {
        let mut builder = SourceMapBuilder::new();
        for i in 0..100 {
            builder.add_char_mapping(CharPos(i), CharPos(i + 10));
        }

        let map = builder.build();
        let stats = map.stats();

        assert_eq!(stats.forward_entries, 100);
        assert!(stats.memory_usage_bytes > 0);

        let compression = stats.compression_ratio(100);
        assert!(compression > 0.0);

        let human_readable = stats.memory_usage_human();
        assert!(human_readable.contains("B") || human_readable.contains("KB"));
    }

    #[test]
    fn test_mappings_in_range() {
        let mut map = SourceMap::new();
        map.add_mapping(CharPos(5), CharPos(10));
        map.add_mapping(CharPos(15), CharPos(20));
        map.add_mapping(CharPos(25), CharPos(30));

        let mappings = map.mappings_in_range(CharPos(10), CharPos(20));
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0], (CharPos(15), CharPos(20)));
    }
}
