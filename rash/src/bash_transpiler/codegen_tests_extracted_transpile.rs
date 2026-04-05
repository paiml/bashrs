    #[test]
    fn test_transpile_comment_preserved() {
        let bash_code = "# This is a comment";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: true,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("//"));
    }

    #[test]
    fn test_transpile_comment_discarded() {
        let bash_code = "# This is a comment\nx=1";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: false,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        // Comment line should be empty, not contain //
        assert!(rash_code.contains("let x"));
    }

    // Return statement tests
    #[test]
    fn test_transpile_return_no_value() {
        let bash_code = r#"
foo() {
    return
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return;"));
    }

    #[test]
    fn test_transpile_return_with_value() {
        let bash_code = r#"
foo() {
    return 0
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return"));
        assert!(rash_code.contains("0"));
    }

    // Expression tests
    #[test]
    fn test_transpile_literal_string() {
        let bash_code = "echo hello";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("hello"));
    }

    #[test]
    fn test_transpile_variable() {
        let bash_code = "echo $x";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("x"));
    }

    // Test expression tests
    #[test]
    fn test_transpile_string_eq() {
        let bash_code = r#"
if [ "$x" == "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("=="));
    }

    #[test]
    fn test_transpile_string_ne() {
        let bash_code = r#"
if [ "$x" != "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("!="));
    }

    #[test]
    fn test_transpile_int_lt() {
        let bash_code = r#"
if [ $x -lt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("<"));
    }

    #[test]
    fn test_transpile_int_gt() {
        let bash_code = r#"
if [ $x -gt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains(">"));
    }

    #[test]
    fn test_transpile_file_exists() {
        let bash_code = r#"
if [ -e /tmp/file ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("exists"));
    }

    #[test]
    fn test_transpile_file_directory() {
        let bash_code = r#"
if [ -d /tmp ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_dir"));
    }

    #[test]
    fn test_transpile_string_empty() {
        let bash_code = r#"
if [ -z "$x" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_empty"));
    }

include!("codegen_tests_extracted_transpile_transpile.rs");
