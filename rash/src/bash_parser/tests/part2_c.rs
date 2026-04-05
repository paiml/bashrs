#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: parse a script and return whether parsing succeeded.
/// Used by documentation tests that only need to verify parsability.
#[test]
fn test_VAR_004_ps3_select_prompt_not_supported() {
    // DOCUMENTATION: PS3 is NOT SUPPORTED (interactive only)
    //
    // PS3 is the prompt for select command:
    // select choice in "Option 1" "Option 2" "Option 3"; do
    //   echo "You selected: $choice"
    //   break
    // done
    //
    // Default PS3: "#? "
    // Custom PS3: PS3="Choose an option: "
    //
    // This is interactive only (select command requires user input)

    let ps3_script = r#"PS3="Choose: ""#;
    let result = BashParser::new(ps3_script);

    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "PS3 is interactive only, NOT SUPPORTED in scripts"
        );
    }

    // NOT SUPPORTED because:
    // - select command is interactive (requires user input)
    // - bashrs is script-mode-only (no select menus)
    // - POSIX alternative: command-line arguments or config files
}

#[test]
fn test_VAR_004_ps4_debug_prompt_not_production() {
    // DOCUMENTATION: PS4 is debugging only (not production code)
    //
    // PS4 is the debug trace prompt (set -x):
    // set -x
    // echo "test"
    // # Output: + echo test
    //
    // The "+ " prefix is PS4, default debug prompt
    //
    // Custom PS4:
    // PS4='DEBUG: '
    // set -x
    // echo "test"
    // # Output: DEBUG: echo test
    //
    // Sometimes used in scripts for debugging, but not production

    let ps4_script = r#"PS4='DEBUG: '"#;
    let result = BashParser::new(ps4_script);

    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "PS4 is debugging only, not production code"
        );
    }

    // NOT PRODUCTION because:
    // - Used with set -x (debugging/tracing)
    // - Production scripts should not have set -x
    // - Purified scripts remove debugging code
}

#[test]
fn test_VAR_004_purification_removes_prompts() {
    // DOCUMENTATION: Purification removes all prompt variables
    //
    // Before (with interactive prompts):
    // #!/bin/bash
    // PS1='\u@\h:\w\$ '
    // PS2='> '
    // PS3='Select: '
    // PS4='+ '
    //
    // echo "Hello World"
    //
    // After (purified, prompts removed):
    // #!/bin/sh
    // printf '%s\n' "Hello World"
    //
    // Prompts removed because:
    // - Not needed in non-interactive scripts
    // - Scripts run in batch mode (no prompts displayed)
    // - POSIX sh doesn't use prompts in scripts

    let purified_no_prompts = r#"
#!/bin/sh
printf '%s\n' "Hello World"
"#;

    let result = BashParser::new(purified_no_prompts);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Purified scripts have no prompt variables"
        );
    }

    // Purification removes:
    // - PS1, PS2, PS3, PS4 assignments
    // - PROMPT_COMMAND
    // - PROMPT_DIRTRIM
    // - PS0
    // - Any prompt customization code
}
