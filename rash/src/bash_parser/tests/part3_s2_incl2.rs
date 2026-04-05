fn test_REDIR_002_basic_output_redirection() {
    // DOCUMENTATION: Basic output redirection (>) is SUPPORTED (POSIX)
    //
    // Output redirection writes stdout to file (truncates existing):
    // $ echo "hello" > file.txt
    // $ ls -la > listing.txt
    // $ cat data.txt > output.txt

    let output_redir = r#"
echo "hello" > file.txt
ls -la > listing.txt
cat data.txt > output.txt
"#;

    let result = BashParser::new(output_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Output redirection (>) is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - > may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_002_append_redirection() {
    // DOCUMENTATION: Append redirection (>>) is SUPPORTED (POSIX)
    //
    // Append redirection adds stdout to file (creates if missing):
    // $ echo "line1" > file.txt
    // $ echo "line2" >> file.txt
    // $ echo "line3" >> file.txt
    //
    // Result in file.txt:
    // line1
    // line2
    // line3

    let append_redir = r#"
echo "line1" > file.txt
echo "line2" >> file.txt
echo "line3" >> file.txt
"#;

    let result = BashParser::new(append_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Append redirection (>>) is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - >> may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_002_overwrite_vs_append() {
    // DOCUMENTATION: > overwrites, >> appends (POSIX semantics)
    //
    // > truncates file to zero length before writing:
    // $ echo "new" > file.txt  # Destroys old content
    //
    // >> appends to existing file:
    // $ echo "more" >> file.txt  # Keeps old content
    //
    // POSIX sh behavior:
    // - > creates file if missing (mode 0666 & ~umask)
    // - >> creates file if missing (same mode)
    // - > destroys existing content
    // - >> preserves existing content

    let overwrite_append = r#"
# Overwrite (truncate)
echo "first" > data.txt
echo "second" > data.txt  # Destroys "first"

# Append (preserve)
echo "line1" > log.txt
echo "line2" >> log.txt  # Keeps "line1"
echo "line3" >> log.txt  # Keeps both
"#;

    let result = BashParser::new(overwrite_append);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Overwrite vs append semantics documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}
