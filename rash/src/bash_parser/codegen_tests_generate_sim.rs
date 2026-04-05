#[cfg(test)]
mod codegen_tests {
    use super::*;
    use crate::bash_parser::BashParser;

    // ============================================================================
    // Statement Generation Tests
    // ============================================================================

    #[test]
    fn test_generate_simple_command() {
        let input = "echo hello world";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello world") || output.contains("echo 'hello' 'world'"));
    }

    #[test]
    fn test_generate_command_with_quotes() {
        let input = r#"echo "hello world""#;
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_generate_assignment() {
        let input = "x=42";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("x=42"));
    }

    #[test]
    fn test_generate_exported_assignment() {
        let input = "export PATH=/usr/bin";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("export") && output.contains("PATH"));
    }

    #[test]
    fn test_generate_comment() {
        let input = "# This is a comment\necho hello";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        // Comment should be preserved (may have different formatting)
        assert!(output.contains("#") && output.contains("comment"));
    }

    #[test]
    fn test_generate_function() {
        let input = "hello() { echo hi; }";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("hello()") && output.contains("echo"));
    }

    #[test]
    fn test_generate_if_statement() {
        let input = "if [ -f file ]; then echo exists; fi";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if") && output.contains("then") && output.contains("fi"));
    }

    #[test]
    fn test_generate_if_else_statement() {
        let input = "if [ -f file ]; then echo yes; else echo no; fi";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if") && output.contains("else") && output.contains("fi"));
    }

    #[test]
    fn test_generate_for_loop() {
        let input = "for i in 1 2 3; do echo $i; done";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for") && output.contains("do") && output.contains("done"));
    }

    #[test]
    fn test_generate_while_loop() {
        let input = "while [ $x -lt 10 ]; do echo $x; done";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("while") && output.contains("do") && output.contains("done"));
    }

    #[test]
    fn test_generate_case_statement() {
        let input = "case $x in a) echo a;; b) echo b;; esac";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("case") && output.contains("esac"));
    }

    #[test]
    fn test_generate_pipeline() {
        let input = "ls | grep foo";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("|"));
    }

    #[test]
    fn test_generate_and_list() {
        let input = "test -f file && echo exists";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_or_list() {
        let input = "test -f file || echo missing";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_redirect() {
        let input = "echo hello > output.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains(">"));
    }

    #[test]
    fn test_generate_append_redirect() {
        let input = "echo hello >> output.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains(">>"));
    }

    #[test]
    fn test_generate_input_redirect() {
        let input = "cat < input.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("<"));
    }

    #[test]
    fn test_generate_variable_expansion() {
        let input = r#"echo "$HOME""#;
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("HOME"));
    }

    #[test]
    fn test_generate_arithmetic() {
        let input = "x=$((1 + 2))";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("$((") || output.contains("x="));
    }

    #[test]
    fn test_generate_command_substitution() {
        let input = "x=$(pwd)";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("$(") || output.contains("pwd"));
    }

    #[test]
    fn test_generate_return_statement() {
        let input = "return 0";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return"));
    }

    #[test]
    fn test_generate_shebang_replaced() {
        let input = "#!/bin/bash\necho hello";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        // Shebang should be replaced with #!/bin/sh
        assert!(output.starts_with("#!/bin/sh"));
        // Should not have duplicate shebangs
        assert_eq!(output.matches("#!/bin/sh").count(), 1);
    }
}

#[cfg(test)]
mod codegen_tests_ext {
    use super::*;
    use crate::bash_parser::BashParser;
    include!("codegen_tests_ext_generate.rs");
}
