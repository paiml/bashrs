#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_parser::error::SourceLocation;

    #[test]
    fn test_parse_empty_makefile() {
        let result = parse_makefile("");
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 0);
    }

    #[test]
    fn test_parse_target_with_recipe() {
        let makefile = "build:\n\tcargo build";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Target { name, recipe, .. } => {
                assert_eq!(name, "build");
                assert_eq!(recipe.len(), 1);
                assert_eq!(recipe[0], "cargo build");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_parse_target_no_prerequisites() {
        let makefile = "test:\n\tcargo test";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 0);
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_parse_multiple_targets() {
        let makefile = "build:\n\tcargo build\n\ntest:\n\tcargo test";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 2);
    }

    // ===== NASA-QUALITY UNIT TESTS for parse_makefile_items helpers =====

    #[test]
    fn test_should_skip_line_empty() {
        assert!(should_skip_line(""), "Empty line should be skipped");
        assert!(
            should_skip_line("   "),
            "Whitespace-only line should be skipped"
        );
        assert!(should_skip_line("\t"), "Tab-only line should be skipped");
    }

    #[test]
    fn test_should_skip_line_non_empty() {
        assert!(
            !should_skip_line("build:"),
            "Target line should NOT be skipped"
        );
        assert!(
            !should_skip_line("# comment"),
            "Comment should NOT be skipped"
        );
        assert!(
            !should_skip_line("VAR = value"),
            "Variable should NOT be skipped"
        );
    }

    #[test]
    fn test_try_parse_comment_valid() {
        let comment = try_parse_comment("# This is a comment", 1);
        assert!(comment.is_some(), "Should recognize comment line");

        if let Some(MakeItem::Comment { text, .. }) = comment {
            assert_eq!(
                text, "This is a comment",
                "Comment text should have # stripped and be trimmed"
            );
        } else {
            panic!("Expected Comment item");
        }
    }

    #[test]
    fn test_try_parse_comment_not_comment() {
        let comment = try_parse_comment("build:", 1);
        assert!(comment.is_none(), "Non-comment should return None");
    }

    #[test]
    fn test_try_parse_include_valid() {
        let result = try_parse_include("include foo.mk", 1);
        assert!(result.is_some(), "Should recognize include directive");

        if let Some(Ok(MakeItem::Include { path, .. })) = result {
            assert_eq!(path, "foo.mk");
        } else {
            panic!("Expected Include item");
        }
    }

    #[test]
    fn test_try_parse_include_not_include() {
        let result = try_parse_include("build:", 1);
        assert!(result.is_none(), "Non-include should return None");
    }

    #[test]
    fn test_try_parse_variable_valid() {
        let result = try_parse_variable("CC = gcc", 1);
        assert!(result.is_some(), "Should recognize variable assignment");

        if let Some(Ok(MakeItem::Variable { name, value, .. })) = result {
            assert_eq!(name, "CC");
            assert_eq!(value, "gcc");
        } else {
            panic!("Expected Variable item");
        }
    }

    #[test]
    fn test_try_parse_variable_not_variable() {
        let result = try_parse_variable("build:", 1);
        assert!(result.is_none(), "Non-variable should return None");
    }

    #[test]
    fn test_try_add_item_success() {
        let mut items = Vec::new();
        let item = MakeItem::Comment {
            text: "# test".to_string(),
            span: Span::new(0, 6, 1),
        };

        let result = try_add_item(&mut items, Ok(item));

        assert!(result.is_ok(), "Should successfully add item");
        assert_eq!(items.len(), 1, "Should have 1 item");
    }

    #[test]
    fn test_try_add_item_error() {
        let mut items = Vec::new();
        let error = MakeParseError::EmptyVariableName {
            location: SourceLocation::new(1),
        };

        let result = try_add_item(&mut items, Err(error));

        assert!(result.is_err(), "Should propagate error");
        assert_eq!(items.len(), 0, "Should not add any items on error");
        assert!(
            result.unwrap_err().contains("Empty variable name"),
            "Error message should be preserved"
        );
    }

    #[test]
    fn test_try_add_item_multiple_success() {
        let mut items = Vec::new();

        let item1 = MakeItem::Comment {
            text: "# comment 1".to_string(),
            span: Span::new(0, 11, 1),
        };
        let item2 = MakeItem::Comment {
            text: "# comment 2".to_string(),
            span: Span::new(0, 11, 2),
        };

        assert!(try_add_item(&mut items, Ok(item1)).is_ok());
        assert!(try_add_item(&mut items, Ok(item2)).is_ok());

        assert_eq!(items.len(), 2, "Should have 2 items");
    }

    #[test]
    fn test_try_parse_comment_trims_whitespace() {
        let comment = try_parse_comment("#   indented comment  ", 1);
        assert!(comment.is_some());

        if let Some(MakeItem::Comment { text, .. }) = comment {
            assert_eq!(
                text, "indented comment",
                "Whitespace should be trimmed (both leading and trailing)"
            );
        } else {
            panic!("Expected Comment item");
        }
    }
}
