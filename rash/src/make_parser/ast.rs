//! AST (Abstract Syntax Tree) for GNU Makefiles
//!
//! This module defines the AST structure for representing parsed Makefiles.
//! The design follows the specification in:
//! `docs/specification/lint-purify-test-write-Makefile-document-gnu-guide.md`

use std::fmt;

/// Root AST node representing a complete Makefile
#[derive(Debug, Clone, PartialEq)]
pub struct MakeAst {
    /// All items in the Makefile (targets, variables, conditionals, etc.)
    pub items: Vec<MakeItem>,
    /// Metadata about the Makefile
    pub metadata: MakeMetadata,
}

/// Metadata about the parsed Makefile
#[derive(Debug, Clone, PartialEq)]
pub struct MakeMetadata {
    /// Source file path (if available)
    pub source_file: Option<String>,
    /// Number of lines in the source
    pub line_count: usize,
    /// Parse time in milliseconds
    pub parse_time_ms: u64,
}

impl MakeMetadata {
    /// Create default metadata
    pub fn new() -> Self {
        Self {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        }
    }

    /// Create metadata with line count
    pub fn with_line_count(line_count: usize) -> Self {
        Self {
            source_file: None,
            line_count,
            parse_time_ms: 0,
        }
    }
}

impl Default for MakeMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Makefile constructs (targets, variables, conditionals, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum MakeItem {
    /// A target with prerequisites and recipe
    ///
    /// Example:
    /// ```makefile
    /// build: src/main.c src/util.c
    ///     gcc -o build src/main.c src/util.c
    /// ```
    Target {
        /// Target name (e.g., "build", "test", "clean")
        name: String,
        /// List of prerequisites (targets or files this depends on)
        prerequisites: Vec<String>,
        /// Recipe lines (commands to execute, tab-indented)
        recipe: Vec<String>,
        /// Whether this target is marked as .PHONY
        phony: bool,
        /// Source location
        span: Span,
    },

    /// A variable assignment
    ///
    /// Example:
    /// ```makefile
    /// CC := gcc
    /// CFLAGS = -O2 -Wall
    /// ```
    Variable {
        /// Variable name
        name: String,
        /// Variable value
        value: String,
        /// Variable flavor (=, :=, ?=, +=, !=)
        flavor: VarFlavor,
        /// Source location
        span: Span,
    },

    /// A pattern rule
    ///
    /// Example:
    /// ```makefile
    /// %.o: %.c
    ///     $(CC) -c $< -o $@
    /// ```
    PatternRule {
        /// Target pattern (e.g., "%.o")
        target_pattern: String,
        /// Prerequisite patterns
        prereq_patterns: Vec<String>,
        /// Recipe lines
        recipe: Vec<String>,
        /// Source location
        span: Span,
    },

    /// A conditional block (ifeq, ifdef, etc.)
    ///
    /// Example:
    /// ```makefile
    /// ifeq ($(DEBUG),1)
    /// CFLAGS = -g
    /// else
    /// CFLAGS = -O2
    /// endif
    /// ```
    Conditional {
        /// Condition type
        condition: MakeCondition,
        /// Items in the "then" branch
        then_items: Vec<MakeItem>,
        /// Items in the "else" branch (if present)
        else_items: Option<Vec<MakeItem>>,
        /// Source location
        span: Span,
    },

    /// An include directive
    ///
    /// Example:
    /// ```makefile
    /// include common.mk
    /// -include optional.mk
    /// ```
    Include {
        /// File path to include
        path: String,
        /// Whether this is optional (-include)
        optional: bool,
        /// Source location
        span: Span,
    },

    /// A function call
    ///
    /// Example:
    /// ```makefile
    /// SOURCES := $(wildcard src/*.c)
    /// OBJS := $(patsubst %.c,%.o,$(SOURCES))
    /// ```
    FunctionCall {
        /// Function name (e.g., "wildcard", "patsubst")
        name: String,
        /// Function arguments
        args: Vec<String>,
        /// Source location
        span: Span,
    },

    /// A comment line
    ///
    /// Example:
    /// ```makefile
    /// # This is a comment
    /// ```
    Comment {
        /// Comment text (without the # prefix)
        text: String,
        /// Source location
        span: Span,
    },
}

/// Variable assignment flavors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarFlavor {
    /// Recursive assignment (=) - expanded when used
    Recursive,
    /// Simple assignment (:=) - expanded immediately (PREFERRED)
    Simple,
    /// Conditional assignment (?=) - only if not already defined
    Conditional,
    /// Append (+=) - add to existing value
    Append,
    /// Shell assignment (!=) - execute shell command
    Shell,
}

impl fmt::Display for VarFlavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarFlavor::Recursive => write!(f, "="),
            VarFlavor::Simple => write!(f, ":="),
            VarFlavor::Conditional => write!(f, "?="),
            VarFlavor::Append => write!(f, "+="),
            VarFlavor::Shell => write!(f, "!="),
        }
    }
}

/// Conditional types in Makefiles
#[derive(Debug, Clone, PartialEq, Eq)]
/// Makefile conditional directives - names match Make syntax exactly
#[allow(clippy::enum_variant_names)]
pub enum MakeCondition {
    /// ifeq ($(VAR),value)
    IfEq(String, String),
    /// ifneq ($(VAR),value)
    IfNeq(String, String),
    /// ifdef VAR
    IfDef(String),
    /// ifndef VAR
    IfNdef(String),
}

/// Source location information
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Span {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
}

impl Span {
    /// Create a dummy span (for testing or when location is unknown)
    pub fn dummy() -> Self {
        Span {
            start: 0,
            end: 0,
            line: 0,
        }
    }

    /// Create a span with specific values
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Span { start, end, line }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::dummy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_flavor_display() {
        assert_eq!(format!("{}", VarFlavor::Recursive), "=");
        assert_eq!(format!("{}", VarFlavor::Simple), ":=");
        assert_eq!(format!("{}", VarFlavor::Conditional), "?=");
        assert_eq!(format!("{}", VarFlavor::Append), "+=");
        assert_eq!(format!("{}", VarFlavor::Shell), "!=");
    }

    #[test]
    fn test_span_dummy() {
        let span = Span::dummy();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 0);
        assert_eq!(span.line, 0);
    }

    #[test]
    fn test_metadata_default() {
        let meta = MakeMetadata::default();
        assert_eq!(meta.source_file, None);
        assert_eq!(meta.line_count, 0);
        assert_eq!(meta.parse_time_ms, 0);
    }
}
