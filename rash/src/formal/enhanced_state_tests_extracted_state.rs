
    #[test]
    fn test_state_equivalence() {
        let mut state1 = EnhancedState::test_state();
        let mut state2 = EnhancedState::test_state();

        assert!(state1.is_equivalent(&state2));

        // Different environment variable
        state1.set_env("VAR".to_string(), "value".to_string());
        assert!(!state1.is_equivalent(&state2));

        state2.set_env("VAR".to_string(), "value".to_string());
        assert!(state1.is_equivalent(&state2));

        // Different EUID
        state1.euid = 0;
        assert!(!state1.is_equivalent(&state2));
    }

    #[test]
    fn test_group_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.groups = vec![1000, 1001]; // User in multiple groups

        // File owned by group 1001
        state.filesystem.insert(
            PathBuf::from("/shared/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o640, // rw-r-----
                uid: 500,    // Different owner
                gid: 1001,   // Group 1001
                mtime: None,
            },
        );

        // ✅ User can read because they're in group 1001
        assert!(state.can_read(&PathBuf::from("/shared/file.txt")));
    }

    // ===== EnhancedFileSystemEntry Tests =====

    #[test]
    fn test_entry_mode() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 100,
            gid: 100,
        };
        assert_eq!(dir.mode(), 0o755);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 100,
            gid: 100,
            mtime: None,
        };
        assert_eq!(file.mode(), 0o644);
    }

    #[test]
    fn test_entry_uid() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 500,
            gid: 100,
        };
        assert_eq!(dir.uid(), 500);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 1000,
            gid: 100,
            mtime: Some(12345),
        };
        assert_eq!(file.uid(), 1000);
    }

    #[test]
    fn test_entry_gid() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 100,
            gid: 500,
        };
        assert_eq!(dir.gid(), 500);

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 100,
            gid: 1000,
            mtime: None,
        };
        assert_eq!(file.gid(), 1000);
    }

    #[test]
    fn test_entry_is_directory() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 0,
            gid: 0,
        };
        assert!(dir.is_directory());
        assert!(!dir.is_file());

        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 0,
            gid: 0,
            mtime: None,
        };
        assert!(!file.is_directory());
        assert!(file.is_file());
    }

    // ===== EnhancedState Environment Tests =====

    #[test]
    fn test_set_and_get_env() {
        let mut state = EnhancedState::new();
        state.set_env("HOME".to_string(), "/root".to_string());

        assert_eq!(state.get_env("HOME"), Some(&"/root".to_string()));
        assert_eq!(state.get_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_set_env_overwrite() {
        let mut state = EnhancedState::new();
        state.set_env("PATH".to_string(), "/bin".to_string());
        state.set_env("PATH".to_string(), "/usr/bin".to_string());

        assert_eq!(state.get_env("PATH"), Some(&"/usr/bin".to_string()));
    }

    // ===== EnhancedState Directory Operations =====

    #[test]
    fn test_change_directory_success() {
        let mut state = EnhancedState::new();
        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        assert!(state.change_directory(PathBuf::from("/tmp")).is_ok());
        assert_eq!(state.cwd, PathBuf::from("/tmp"));
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_change_directory_not_found() {
        let mut state = EnhancedState::new();
        let result = state.change_directory(PathBuf::from("/nonexistent"));

        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("No such file"));
    }

    #[test]
    fn test_change_directory_not_a_directory() {
        let mut state = EnhancedState::new();
        state.filesystem.insert(
            PathBuf::from("/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o644,
                uid: 0,
                gid: 0,
                mtime: None,
            },
        );

        let result = state.change_directory(PathBuf::from("/file.txt"));
        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Not a directory"));
    }

    #[test]
    fn test_change_directory_permission_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/restricted"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        let result = state.change_directory(PathBuf::from("/restricted"));
        assert!(result.is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    // ===== EnhancedState Write File Tests =====

    #[test]
    fn test_write_file_success() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/home/user"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 1000,
                gid: 1000,
            },
        );

        let result = state.write_file(
            PathBuf::from("/home/user/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_ok());

        let entry = state.filesystem.get(&PathBuf::from("/home/user/file.txt"));
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().uid(), 1000);
    }

    #[test]
    fn test_write_file_permission_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        let result = state.write_file(
            PathBuf::from("/root/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_write_file_parent_no_permission() {
        // Non-root user cannot write to directory they don't have permission for
        let mut state = EnhancedState::new_user(1000, 1000);
        // Create parent directory owned by root with no write permission for others
        state.filesystem.insert(
            PathBuf::from("/restricted"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755, // rwxr-xr-x - no write for others
                uid: 0,
                gid: 0,
            },
        );
        let result = state.write_file(
            PathBuf::from("/restricted/file.txt"),
            "content".to_string(),
            0o644,
        );
        assert!(result.is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    // ===== EnhancedState stdout/stderr Tests =====

    #[test]
    fn test_write_stdout() {
        let mut state = EnhancedState::new();
        state.write_stdout("line1".to_string());
        state.write_stdout("line2".to_string());

        assert_eq!(state.stdout.len(), 2);
        assert_eq!(state.stdout[0], "line1");
        assert_eq!(state.stdout[1], "line2");
    }

    #[test]
    fn test_write_stderr() {
        let mut state = EnhancedState::new();
        state.write_stderr("error1".to_string());
        state.write_stderr("error2".to_string());

        assert_eq!(state.stderr.len(), 2);
        assert_eq!(state.stderr[0], "error1");
        assert_eq!(state.stderr[1], "error2");
    }

    // ===== EnhancedState Test State =====

    #[test]
    fn test_test_state_initialization() {
        let state = EnhancedState::test_state();
        assert_eq!(state.euid, 1000);
        assert_eq!(state.egid, 1000);
        assert!(state.filesystem.contains_key(&PathBuf::from("/tmp")));
    }

    // ===== EnhancedState Permission Edge Cases =====

    #[test]
    fn test_can_read_nonexistent_path_as_non_root() {
        // Non-root user cannot read nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_read(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_can_write_nonexistent_path_as_non_root() {
        // Non-root user cannot write nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_write(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_can_execute_nonexistent_path_as_non_root() {
        // Non-root user cannot execute nonexistent path
        let state = EnhancedState::new_user(1000, 1000);
        assert!(!state.can_execute(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_root_can_access_nonexistent_path() {
        // Root user (UID 0) bypasses permission checks
        let state = EnhancedState::new();
        // Root returns true even for nonexistent paths (bypasses all checks)
        assert!(state.can_read(&PathBuf::from("/nonexistent")));
        assert!(state.can_write(&PathBuf::from("/nonexistent")));
        assert!(state.can_execute(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_other_permissions() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // File with other-readable permission
        state.filesystem.insert(
            PathBuf::from("/public/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "public".to_string(),
                mode: 0o004, // -------r--
                uid: 500,
                gid: 500,
                mtime: None,
            },
        );

        // User can read via "other" permissions
        assert!(state.can_read(&PathBuf::from("/public/file.txt")));
        assert!(!state.can_write(&PathBuf::from("/public/file.txt")));
    }

    // ===== Clone and Debug Tests =====

    #[test]
    fn test_enhanced_state_clone() {
        let state = EnhancedState::new();
        let cloned = state.clone();
        assert_eq!(state.euid, cloned.euid);
        assert_eq!(state.cwd, cloned.cwd);
    }

    #[test]
    fn test_enhanced_state_debug() {
        let state = EnhancedState::new();
        let debug = format!("{:?}", state);
        assert!(debug.contains("euid"));
        assert!(debug.contains("cwd"));
    }

    #[test]
    fn test_entry_clone() {
        let dir = EnhancedFileSystemEntry::Directory {
            mode: 0o755,
            uid: 0,
            gid: 0,
        };
        let cloned = dir.clone();
        assert_eq!(dir, cloned);
    }

    #[test]
    fn test_entry_debug() {
        let file = EnhancedFileSystemEntry::File {
            content: "test".to_string(),
            mode: 0o644,
            uid: 0,
            gid: 0,
            mtime: Some(1234567890),
        };
        let debug = format!("{:?}", file);
        assert!(debug.contains("File"));
        assert!(debug.contains("content"));
    }
