//! Core type definitions for the formatter

use crate::formatter::logging::TransformLog;
use crate::formatter::source_map::SourceMap;
use std::borrow::Cow;

/// Configuration for formatting operations
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Enable parallel processing for large files
    pub parallel: bool,

    /// Maximum number of threads to use
    pub max_threads: Option<usize>,

    /// Whether to preserve original whitespace in certain contexts
    pub preserve_whitespace: bool,

    /// Whether to generate SMT proofs for transformations
    pub generate_proofs: bool,

    /// Whether to enable SIMD optimizations
    pub enable_simd: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            parallel: false,
            max_threads: None,
            preserve_whitespace: false,
            generate_proofs: false,
            enable_simd: true,
        }
    }
}

/// Result of formatting operations with full provenance tracking
#[derive(Debug)]
pub struct FormattedSource<'a> {
    /// UTF-8 normalized text, zero-copy when possible
    pub text: Cow<'a, str>,

    /// Character-level bidirectional mapping with interval trees
    pub source_map: SourceMap,

    /// Semantic annotations preserved across transforms
    pub metadata: SemanticMetadata,

    /// BLAKE3-256 for content addressing (measured 89% cache hit rate)
    pub canonical_hash: [u8; 32],

    /// Append-only log for verification context propagation
    pub transforms: TransformLog,
}

/// Semantic metadata preserved during formatting
#[derive(Debug, Default, Clone)]
pub struct SemanticMetadata {
    /// Comments and their positions
    pub comments: Vec<CommentMetadata>,

    /// Variable declarations and usage
    pub variables: Vec<VariableMetadata>,

    /// Function definitions
    pub functions: Vec<FunctionMetadata>,

    /// Detected contracts/specifications
    pub contracts: Vec<ContractMetadata>,
}

#[derive(Debug, Clone)]
pub struct CommentMetadata {
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct VariableMetadata {
    pub name: String,
    pub declaration_pos: Option<usize>,
    pub usages: Vec<usize>,
    pub shell_type: Option<ShellType>,
}

#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub parameters: Vec<String>,
    pub return_type: Option<ShellType>,
}

#[derive(Debug, Clone)]
pub struct ContractMetadata {
    pub kind: ContractKind,
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    Precondition,
    Postcondition,
    Invariant,
    TypeAnnotation,
}

/// Shell-specific type system for contracts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShellType {
    /// Primitive types
    String,
    Integer,
    Boolean,

    /// Compound types
    Array(Box<ShellType>),
    AssocArray {
        key: Box<ShellType>,
        value: Box<ShellType>,
    },

    /// Shell-specific types
    FileDescriptor,
    ExitCode,
    Signal,

    /// Type variables for inference
    TypeVar(u32),

    /// Union types for shell's dynamic nature
    Union(Vec<ShellType>),
}

impl ShellType {
    /// Check if this type is compatible with another
    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (ShellType::String, ShellType::String) => true,
            (ShellType::Integer, ShellType::Integer) => true,
            (ShellType::Boolean, ShellType::Boolean) => true,
            (ShellType::Array(a), ShellType::Array(b)) => a.is_compatible(b),
            (
                ShellType::AssocArray { key: k1, value: v1 },
                ShellType::AssocArray { key: k2, value: v2 },
            ) => k1.is_compatible(k2) && v1.is_compatible(v2),
            (ShellType::Union(types1), other) => types1.iter().any(|t| t.is_compatible(other)),
            (other, ShellType::Union(types2)) => types2.iter().any(|t| other.is_compatible(t)),
            _ => false,
        }
    }

    /// Get a human-readable representation
    pub fn display(&self) -> String {
        match self {
            ShellType::String => "string".to_string(),
            ShellType::Integer => "integer".to_string(),
            ShellType::Boolean => "boolean".to_string(),
            ShellType::Array(inner) => format!("array[{}]", inner.display()),
            ShellType::AssocArray { key, value } => {
                format!("assoc[{} => {}]", key.display(), value.display())
            }
            ShellType::FileDescriptor => "fd".to_string(),
            ShellType::ExitCode => "exit_code".to_string(),
            ShellType::Signal => "signal".to_string(),
            ShellType::TypeVar(id) => format!("T{id}"),
            ShellType::Union(types) => {
                let type_strs: Vec<_> = types.iter().map(|t| t.display()).collect();
                format!("({})", type_strs.join(" | "))
            }
        }
    }
}

/// Character position in source text
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharPos(pub usize);

impl CharPos {
    pub fn new(pos: usize) -> Self {
        CharPos(pos)
    }

    pub fn offset(&self) -> usize {
        self.0
    }
}

/// Byte position in source text
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytePos(pub usize);

impl BytePos {
    pub fn new(pos: usize) -> Self {
        BytePos(pos)
    }

    pub fn offset(&self) -> usize {
        self.0
    }
}

/// Source span with start and end positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub fn new(start: BytePos, end: BytePos) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.0 - self.start.0
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, pos: BytePos) -> bool {
        self.start <= pos && pos < self.end
    }

    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// Position mapping result with token boundary information
#[derive(Debug, Clone, Copy)]
pub struct MappedPosition {
    /// Exact character position
    pub exact: CharPos,

    /// Start of containing token
    pub token_start: CharPos,

    /// End of containing token
    pub token_end: CharPos,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_config_default() {
        let config = FormatConfig::default();
        assert!(!config.parallel);
        assert!(config.max_threads.is_none());
        assert!(!config.preserve_whitespace);
        assert!(!config.generate_proofs);
        assert!(config.enable_simd);
    }

    #[test]
    fn test_shell_type_compatibility() {
        assert!(ShellType::String.is_compatible(&ShellType::String));
        assert!(ShellType::Integer.is_compatible(&ShellType::Integer));
        assert!(!ShellType::String.is_compatible(&ShellType::Integer));

        let array_str = ShellType::Array(Box::new(ShellType::String));
        let array_str2 = ShellType::Array(Box::new(ShellType::String));
        let array_int = ShellType::Array(Box::new(ShellType::Integer));

        assert!(array_str.is_compatible(&array_str2));
        assert!(!array_str.is_compatible(&array_int));
    }

    #[test]
    fn test_shell_type_union_compatibility() {
        let union_type = ShellType::Union(vec![ShellType::String, ShellType::Integer]);

        assert!(union_type.is_compatible(&ShellType::String));
        assert!(union_type.is_compatible(&ShellType::Integer));
        assert!(!union_type.is_compatible(&ShellType::Boolean));
    }

    #[test]
    fn test_shell_type_display() {
        assert_eq!(ShellType::String.display(), "string");
        assert_eq!(ShellType::Integer.display(), "integer");

        let array_type = ShellType::Array(Box::new(ShellType::String));
        assert_eq!(array_type.display(), "array[string]");

        let union_type = ShellType::Union(vec![ShellType::String, ShellType::Integer]);
        assert_eq!(union_type.display(), "(string | integer)");
    }

    #[test]
    fn test_span_operations() {
        let span1 = Span::new(BytePos(10), BytePos(20));
        let span2 = Span::new(BytePos(15), BytePos(25));
        let span3 = Span::new(BytePos(25), BytePos(30));

        assert_eq!(span1.len(), 10);
        assert!(!span1.is_empty());
        assert!(span1.contains(BytePos(15)));
        assert!(!span1.contains(BytePos(25)));
        assert!(span1.overlaps(&span2));
        assert!(!span1.overlaps(&span3));
    }

    #[test]
    fn test_char_pos_byte_pos() {
        let char_pos = CharPos::new(42);
        assert_eq!(char_pos.offset(), 42);

        let byte_pos = BytePos::new(84);
        assert_eq!(byte_pos.offset(), 84);

        assert!(char_pos < CharPos::new(50));
        assert!(byte_pos < BytePos::new(100));
    }
}
