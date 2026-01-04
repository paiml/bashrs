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

/// Metadata about recipe formatting (line continuations, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeMetadata {
    /// Original line breaks in the recipe (indices where continuations occurred)
    /// Each entry contains:
    /// - character position in the concatenated recipe line
    /// - original indentation of the continued line
    pub line_breaks: Vec<(usize, String)>,
}

impl RecipeMetadata {
    /// Create empty recipe metadata
    pub fn new() -> Self {
        Self {
            line_breaks: Vec::new(),
        }
    }

    /// Create metadata with line breaks
    pub fn with_breaks(line_breaks: Vec<(usize, String)>) -> Self {
        Self { line_breaks }
    }
}

impl Default for RecipeMetadata {
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
        /// Recipe formatting metadata (line continuations, etc.)
        recipe_metadata: Option<RecipeMetadata>,
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
        /// Recipe formatting metadata (line continuations, etc.)
        recipe_metadata: Option<RecipeMetadata>,
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
    fn test_span_new() {
        let span = Span::new(10, 20, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.line, 5);
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 0);
        assert_eq!(span.line, 0);
    }

    #[test]
    fn test_metadata_new() {
        let meta = MakeMetadata::new();
        assert_eq!(meta.source_file, None);
        assert_eq!(meta.line_count, 0);
        assert_eq!(meta.parse_time_ms, 0);
    }

    #[test]
    fn test_metadata_default() {
        let meta = MakeMetadata::default();
        assert_eq!(meta.source_file, None);
        assert_eq!(meta.line_count, 0);
        assert_eq!(meta.parse_time_ms, 0);
    }

    #[test]
    fn test_metadata_with_line_count() {
        let meta = MakeMetadata::with_line_count(42);
        assert_eq!(meta.source_file, None);
        assert_eq!(meta.line_count, 42);
        assert_eq!(meta.parse_time_ms, 0);
    }

    #[test]
    fn test_recipe_metadata_new() {
        let meta = RecipeMetadata::new();
        assert!(meta.line_breaks.is_empty());
    }

    #[test]
    fn test_recipe_metadata_default() {
        let meta = RecipeMetadata::default();
        assert!(meta.line_breaks.is_empty());
    }

    #[test]
    fn test_recipe_metadata_with_breaks() {
        let breaks = vec![(10, "  ".to_string()), (25, "\t".to_string())];
        let meta = RecipeMetadata::with_breaks(breaks.clone());
        assert_eq!(meta.line_breaks.len(), 2);
        assert_eq!(meta.line_breaks[0], (10, "  ".to_string()));
        assert_eq!(meta.line_breaks[1], (25, "\t".to_string()));
    }

    #[test]
    fn test_make_ast_creation() {
        let ast = MakeAst {
            items: vec![],
            metadata: MakeMetadata::default(),
        };
        assert!(ast.items.is_empty());
        assert_eq!(ast.metadata.line_count, 0);
    }

    #[test]
    fn test_make_item_target() {
        let target = MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["src/main.c".to_string()],
            recipe: vec!["gcc -o build src/main.c".to_string()],
            phony: false,
            recipe_metadata: None,
            span: Span::dummy(),
        };
        if let MakeItem::Target {
            name,
            phony,
            recipe,
            ..
        } = target
        {
            assert_eq!(name, "build");
            assert!(!phony);
            assert_eq!(recipe.len(), 1);
        }
    }

    #[test]
    fn test_make_item_target_phony() {
        let target = MakeItem::Target {
            name: "clean".to_string(),
            prerequisites: vec![],
            recipe: vec!["rm -rf *.o".to_string()],
            phony: true,
            recipe_metadata: Some(RecipeMetadata::new()),
            span: Span::new(0, 50, 1),
        };
        if let MakeItem::Target {
            phony,
            recipe_metadata,
            ..
        } = target
        {
            assert!(phony);
            assert!(recipe_metadata.is_some());
        }
    }

    #[test]
    fn test_make_item_variable() {
        let var = MakeItem::Variable {
            name: "CC".to_string(),
            value: "gcc".to_string(),
            flavor: VarFlavor::Simple,
            span: Span::new(0, 10, 1),
        };
        if let MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } = var
        {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
            assert_eq!(flavor, VarFlavor::Simple);
        }
    }

    #[test]
    fn test_make_item_pattern_rule() {
        let rule = MakeItem::PatternRule {
            target_pattern: "%.o".to_string(),
            prereq_patterns: vec!["%.c".to_string()],
            recipe: vec!["$(CC) -c $< -o $@".to_string()],
            recipe_metadata: None,
            span: Span::dummy(),
        };
        if let MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } = rule
        {
            assert_eq!(target_pattern, "%.o");
            assert_eq!(prereq_patterns, vec!["%.c"]);
        }
    }

    #[test]
    fn test_make_item_conditional() {
        let cond = MakeItem::Conditional {
            condition: MakeCondition::IfEq("$(DEBUG)".to_string(), "1".to_string()),
            then_items: vec![],
            else_items: Some(vec![]),
            span: Span::dummy(),
        };
        if let MakeItem::Conditional {
            condition,
            else_items,
            ..
        } = cond
        {
            assert!(matches!(condition, MakeCondition::IfEq(_, _)));
            assert!(else_items.is_some());
        }
    }

    #[test]
    fn test_make_item_include() {
        let incl = MakeItem::Include {
            path: "common.mk".to_string(),
            optional: false,
            span: Span::new(0, 20, 1),
        };
        if let MakeItem::Include { path, optional, .. } = incl {
            assert_eq!(path, "common.mk");
            assert!(!optional);
        }
    }

    #[test]
    fn test_make_item_include_optional() {
        let incl = MakeItem::Include {
            path: "optional.mk".to_string(),
            optional: true,
            span: Span::dummy(),
        };
        if let MakeItem::Include { optional, .. } = incl {
            assert!(optional);
        }
    }

    #[test]
    fn test_make_item_function_call() {
        let func = MakeItem::FunctionCall {
            name: "wildcard".to_string(),
            args: vec!["src/*.c".to_string()],
            span: Span::dummy(),
        };
        if let MakeItem::FunctionCall { name, args, .. } = func {
            assert_eq!(name, "wildcard");
            assert_eq!(args.len(), 1);
        }
    }

    #[test]
    fn test_make_item_comment() {
        let comment = MakeItem::Comment {
            text: "This is a comment".to_string(),
            span: Span::new(0, 20, 3),
        };
        if let MakeItem::Comment { text, span } = comment {
            assert_eq!(text, "This is a comment");
            assert_eq!(span.line, 3);
        }
    }

    #[test]
    fn test_make_condition_variants() {
        let ifeq = MakeCondition::IfEq("a".to_string(), "b".to_string());
        let ifneq = MakeCondition::IfNeq("c".to_string(), "d".to_string());
        let ifdef = MakeCondition::IfDef("VAR".to_string());
        let ifndef = MakeCondition::IfNdef("OTHER".to_string());

        assert!(matches!(ifeq, MakeCondition::IfEq(_, _)));
        assert!(matches!(ifneq, MakeCondition::IfNeq(_, _)));
        assert!(matches!(ifdef, MakeCondition::IfDef(_)));
        assert!(matches!(ifndef, MakeCondition::IfNdef(_)));
    }

    #[test]
    fn test_var_flavor_equality() {
        assert_eq!(VarFlavor::Recursive, VarFlavor::Recursive);
        assert_ne!(VarFlavor::Recursive, VarFlavor::Simple);
    }

    #[test]
    fn test_make_ast_clone() {
        let ast = MakeAst {
            items: vec![MakeItem::Comment {
                text: "test".to_string(),
                span: Span::dummy(),
            }],
            metadata: MakeMetadata::with_line_count(10),
        };
        let cloned = ast.clone();
        assert_eq!(cloned.items.len(), 1);
        assert_eq!(cloned.metadata.line_count, 10);
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(0, 10, 1);
        let span2 = Span::new(0, 10, 1);
        let span3 = Span::new(0, 10, 2);
        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_span_copy() {
        let span1 = Span::new(5, 15, 3);
        let span2 = span1; // Copy
        assert_eq!(span1, span2);
    }

    #[test]
    fn test_make_metadata_full() {
        let mut meta = MakeMetadata::new();
        meta.source_file = Some("Makefile".to_string());
        meta.line_count = 100;
        meta.parse_time_ms = 5;
        assert_eq!(meta.source_file, Some("Makefile".to_string()));
        assert_eq!(meta.line_count, 100);
        assert_eq!(meta.parse_time_ms, 5);
    }
}
