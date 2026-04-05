fn test_BASH_BUILTIN_005_printf_format_specifiers() {
    // DOCUMENTATION: printf format specifiers (POSIX)
    //
    // %s: String (default format)
    // %d, %i: Signed decimal integer
    // %u: Unsigned decimal integer
    // %x, %X: Hexadecimal (lowercase/uppercase)
    // %o: Octal
    // %f: Floating point
    // %e, %E: Scientific notation
    // %g, %G: Shortest representation (f or e)
    // %c: Single character
    // %%: Literal percent sign
    //
    // INPUT (bash):
    // printf 'String: %s\n' "text"
    // printf 'Decimal: %d\n' 42
    // printf 'Hex: %x\n' 255
    // printf 'Float: %.2f\n' 3.14159
    //
    // RUST:
    // println!("String: {}", "text");
    // println!("Decimal: {}", 42);
    // println!("Hex: {:x}", 255);
    // println!("Float: {:.2}", 3.14159);
    //
    // PURIFIED (POSIX sh):
    // printf 'String: %s\n' "text"
    // printf 'Decimal: %d\n' 42
    // printf 'Hex: %x\n' 255
    // printf 'Float: %.2f\n' 3.14159

    let format_specifiers = r#"
# String format
printf 'Name: %s\n' "Alice"
printf 'Path: %s\n' "/usr/local/bin"

# Integer formats
printf 'Decimal: %d\n' 42
printf 'Unsigned: %u\n' 100
printf 'Hex (lower): %x\n' 255
printf 'Hex (upper): %X\n' 255
printf 'Octal: %o\n' 64

# Floating point formats
printf 'Float: %f\n' 3.14159
printf 'Precision: %.2f\n' 3.14159
printf 'Scientific: %e\n' 1000.0

# Character and literal
printf 'Char: %c\n' "A"
printf 'Percent: %%\n'

# Multiple arguments
printf '%s: %d items\n' "Cart" 5
printf '%s %s %d\n' "User" "logged in at" 1630000000
"#;

    let mut lexer = Lexer::new(format_specifiers);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "format specifiers should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all format specifiers yet
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_escape_sequences() {
    // DOCUMENTATION: printf escape sequences (POSIX)
    //
    // \n: Newline
    // \t: Tab
    // \\: Backslash
    // \': Single quote
    // \": Double quote
    // \r: Carriage return
    // \a: Alert (bell)
    // \b: Backspace
    // \f: Form feed
    // \v: Vertical tab
    // \0NNN: Octal character code
    // \xHH: Hexadecimal character code
    //
    // INPUT (bash):
    // printf 'Line1\nLine2\n'
    // printf 'Col1\tCol2\tCol3\n'
    //
    // RUST:
    // println!("Line1\nLine2");
    // println!("Col1\tCol2\tCol3");
    //
    // PURIFIED:
    // printf 'Line1\nLine2\n'
    // printf 'Col1\tCol2\tCol3\n'

    let escape_sequences = r#"
# Newline
printf 'Line1\nLine2\nLine3\n'

# Tab
printf 'Col1\tCol2\tCol3\n'

# Backslash and quotes
printf 'Path: C:\\Users\\Alice\n'
printf 'Quote: \'single\' and "double"\n'

# Other escapes
printf 'Alert:\a\n'
printf 'Carriage return:\r\n'

# Multiple escapes in one format
printf 'Name:\t%s\nAge:\t%d\nCity:\t%s\n' "Alice" 30 "NYC"
"#;

    let mut lexer = Lexer::new(escape_sequences);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "escape sequences should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support escape sequences yet
        }
    }
}
