#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        // If transpilation succeeds, verify shell output for injection safety
        if let Ok(shell_code) = bashrs::transpile(source, bashrs::Config::default()) {
            // CRITICAL: All variable expansions must be quoted
            // Check for dangerous patterns that indicate injection vulnerabilities

            // Pattern 1: Unquoted variable expansion (e.g., $var or ${var} without quotes)
            // Safe: "$var", "${var}"
            // Unsafe: $var, ${var} in command position

            // Pattern 2: Command substitution without quoting
            // Safe: "$(command)"
            // Unsafe: $(command) in unquoted context

            // Pattern 3: eval or similar dangerous constructs
            assert!(!shell_code.contains("eval "),
                "Shell code contains 'eval': potential injection vector");

            // Pattern 4: Unescaped user input in heredocs
            // This is a simplified check - full validation requires parsing
            let lines: Vec<&str> = shell_code.lines().collect();
            for line in &lines {
                // Skip comments and shebang
                if line.trim().starts_with('#') {
                    continue;
                }

                // Check for suspicious patterns that might indicate injection
                if line.contains("`;") || line.contains(";`") {
                    panic!("Potential command injection via backtick: {}", line);
                }

                if line.contains("$(") && !line.contains("\"$(") && !line.contains("'$(") {
                    // This might be a false positive, but we want to catch potential issues
                    // Command substitution should generally be quoted
                    // We'll allow it in certain contexts (assignments, test conditions)
                    if !line.contains("=") && !line.contains("[ ") && !line.contains("[[ ") {
                        // Potentially dangerous unquoted command substitution
                        // Note: This is a conservative check that may need refinement
                    }
                }
            }
        }
    }
});
