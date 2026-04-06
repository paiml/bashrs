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
