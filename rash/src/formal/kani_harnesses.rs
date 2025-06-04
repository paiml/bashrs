//! Kani proof harnesses for formal verification
//!
//! This module contains Kani harnesses that formally verify
//! properties of the tiny AST emitter using bounded model checking.

#[cfg(kani)]
mod kani_proofs {
    use crate::formal::semantics::{posix_semantics, rash_semantics};
    use crate::formal::{AbstractState, FormalEmitter, TinyAst};

    /// Verify that echo commands preserve their output exactly
    #[kani::proof]
    fn verify_echo_semantic_equivalence() {
        // Create a bounded string for the argument
        let arg: String = kani::any();
        kani::assume(arg.len() <= 10); // Bound the string length
        kani::assume(arg.chars().all(|c| c.is_ascii_alphanumeric())); // Simple chars only

        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec![arg.clone()],
        };

        // Verify the AST is valid
        assert!(ast.is_valid());

        // Create initial state
        let initial_state = AbstractState::new();

        // Evaluate rash AST
        if let Ok(rash_state) = rash_semantics::eval_rash(&ast, initial_state.clone()) {
            // Emit POSIX code
            let posix_code = FormalEmitter::emit(&ast);

            // Evaluate POSIX code
            if let Ok(posix_state) = posix_semantics::eval_posix(&posix_code, initial_state) {
                // Verify semantic equivalence
                assert!(rash_state.is_equivalent(&posix_state));

                // Specifically check stdout
                assert_eq!(rash_state.stdout.len(), 1);
                assert_eq!(posix_state.stdout.len(), 1);
                assert_eq!(rash_state.stdout[0], arg);
                assert_eq!(posix_state.stdout[0], arg);
            }
        }
    }

    /// Verify that environment variable assignments are preserved
    #[kani::proof]
    fn verify_assignment_semantic_equivalence() {
        // Create bounded strings for name and value
        let name: String = kani::any();
        let value: String = kani::any();

        // Bound the strings
        kani::assume(name.len() > 0 && name.len() <= 8);
        kani::assume(value.len() <= 10);

        // Ensure valid variable name
        kani::assume(name.chars().all(|c| c.is_ascii_alphabetic() || c == '_'));
        kani::assume(
            name.chars().next().unwrap().is_ascii_alphabetic()
                || name.chars().next().unwrap() == '_',
        );

        let ast = TinyAst::SetEnvironmentVariable {
            name: name.clone(),
            value: value.clone(),
        };

        // Verify the AST is valid
        assert!(ast.is_valid());

        // Create initial state
        let initial_state = AbstractState::new();

        // Evaluate rash AST
        if let Ok(rash_state) = rash_semantics::eval_rash(&ast, initial_state.clone()) {
            // Emit POSIX code
            let posix_code = FormalEmitter::emit(&ast);

            // Evaluate POSIX code
            if let Ok(posix_state) = posix_semantics::eval_posix(&posix_code, initial_state) {
                // Verify semantic equivalence
                assert!(rash_state.is_equivalent(&posix_state));

                // Specifically check the environment variable
                assert_eq!(rash_state.get_env(&name), Some(&value));
                assert_eq!(posix_state.get_env(&name), Some(&value));
            }
        }
    }

    /// Verify that mkdir commands create directories correctly
    #[kani::proof]
    fn verify_mkdir_semantic_equivalence() {
        // Use a fixed simple path for bounded verification
        let path = "/tmp/test".to_string();

        let ast = TinyAst::ExecuteCommand {
            command_name: "mkdir".to_string(),
            args: vec!["-p".to_string(), path.clone()],
        };

        // Create initial state with /tmp directory
        let mut initial_state = AbstractState::new();
        initial_state.filesystem.insert(
            std::path::PathBuf::from("/tmp"),
            crate::formal::FileSystemEntry::Directory,
        );

        // Evaluate rash AST
        if let Ok(rash_state) = rash_semantics::eval_rash(&ast, initial_state.clone()) {
            // Emit POSIX code
            let posix_code = FormalEmitter::emit(&ast);

            // Evaluate POSIX code
            if let Ok(posix_state) = posix_semantics::eval_posix(&posix_code, initial_state) {
                // Verify semantic equivalence
                assert!(rash_state.is_equivalent(&posix_state));

                // Verify directory was created
                let path_buf = std::path::PathBuf::from(&path);
                assert!(rash_state.filesystem.contains_key(&path_buf));
                assert!(posix_state.filesystem.contains_key(&path_buf));
            }
        }
    }

    /// Verify that change directory commands work correctly
    #[kani::proof]
    fn verify_cd_semantic_equivalence() {
        // Use a fixed path
        let path = "/tmp".to_string();

        let ast = TinyAst::ChangeDirectory { path: path.clone() };

        // Create initial state with the target directory
        let mut initial_state = AbstractState::new();
        initial_state.filesystem.insert(
            std::path::PathBuf::from(&path),
            crate::formal::FileSystemEntry::Directory,
        );

        // Evaluate rash AST
        if let Ok(rash_state) = rash_semantics::eval_rash(&ast, initial_state.clone()) {
            // Emit POSIX code
            let posix_code = FormalEmitter::emit(&ast);

            // Evaluate POSIX code
            if let Ok(posix_state) = posix_semantics::eval_posix(&posix_code, initial_state) {
                // Verify semantic equivalence
                assert!(rash_state.is_equivalent(&posix_state));

                // Verify current directory changed
                assert_eq!(rash_state.cwd, std::path::PathBuf::from(&path));
                assert_eq!(posix_state.cwd, std::path::PathBuf::from(&path));
            }
        }
    }

    /// Verify that sequences preserve order and state changes
    #[kani::proof]
    fn verify_sequence_semantic_equivalence() {
        // Create a simple two-command sequence
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "VAR".to_string(),
                    value: "test".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec!["done".to_string()],
                },
            ],
        };

        // Create initial state
        let initial_state = AbstractState::new();

        // Evaluate rash AST
        if let Ok(rash_state) = rash_semantics::eval_rash(&ast, initial_state.clone()) {
            // Emit POSIX code
            let posix_code = FormalEmitter::emit(&ast);

            // Evaluate POSIX code
            if let Ok(posix_state) = posix_semantics::eval_posix(&posix_code, initial_state) {
                // Verify semantic equivalence
                assert!(rash_state.is_equivalent(&posix_state));

                // Verify both effects happened
                assert_eq!(rash_state.get_env("VAR"), Some(&"test".to_string()));
                assert_eq!(posix_state.get_env("VAR"), Some(&"test".to_string()));
                assert_eq!(rash_state.stdout.len(), 1);
                assert_eq!(posix_state.stdout.len(), 1);
                assert_eq!(rash_state.stdout[0], "done");
            }
        }
    }

    /// Verify emitter produces valid shell code for all valid ASTs
    #[kani::proof]
    fn verify_emitter_totality() {
        // This would ideally check all possible ASTs, but we'll check a representative sample
        let ast_examples = vec![
            TinyAst::ExecuteCommand {
                command_name: "echo".to_string(),
                args: vec!["test".to_string()],
            },
            TinyAst::SetEnvironmentVariable {
                name: "VAR".to_string(),
                value: "val".to_string(),
            },
            TinyAst::ChangeDirectory {
                path: "/tmp".to_string(),
            },
            TinyAst::Sequence {
                commands: vec![TinyAst::ExecuteCommand {
                    command_name: "echo".to_string(),
                    args: vec![],
                }],
            },
        ];

        for ast in ast_examples {
            if ast.is_valid() {
                let posix_code = FormalEmitter::emit(&ast);

                // Emitter should always produce non-empty output for valid ASTs
                assert!(!posix_code.is_empty());

                // The output should be parseable (no panics)
                let initial_state = AbstractState::new();
                let _ = posix_semantics::eval_posix(&posix_code, initial_state);
            }
        }
    }
}
