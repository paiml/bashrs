#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_root() {
        let state = EnhancedState::new();
        assert_eq!(state.euid, 0);
        assert_eq!(state.egid, 0);
        assert_eq!(state.cwd, PathBuf::from("/"));
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_permission_aware_mkdir_success() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Add /tmp as world-writable
        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        // ✅ User can create directory in /tmp
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());
        assert_eq!(state.exit_code, 0);

        // Verify ownership
        let entry = state.filesystem.get(&PathBuf::from("/tmp/user")).unwrap();
        assert_eq!(entry.uid(), 1000);
        assert_eq!(entry.gid(), 1000);
    }

    #[test]
    fn test_permission_aware_mkdir_denied() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Add /opt as root-only
        state.filesystem.insert(
            PathBuf::from("/opt"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755, // No write for others
                uid: 0,
                gid: 0,
            },
        );

        // ❌ User cannot create directory in /opt
        assert!(state
            .create_directory_safe(PathBuf::from("/opt/app"), 0o755)
            .is_err());
        assert_eq!(state.exit_code, 1);
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_idempotent_mkdir_existing_directory() {
        let mut state = EnhancedState::new_user(1000, 1000);

        state.filesystem.insert(
            PathBuf::from("/tmp"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o1777,
                uid: 0,
                gid: 0,
            },
        );

        // First creation
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());

        // ✅ Second creation is idempotent (no error)
        assert!(state
            .create_directory_safe(PathBuf::from("/tmp/user"), 0o755)
            .is_ok());
        assert_eq!(state.exit_code, 0);
    }

    #[test]
    fn test_can_read_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // Owner-readable file
        state.filesystem.insert(
            PathBuf::from("/home/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "data".to_string(),
                mode: 0o600, // rw-------
                uid: 1000,
                gid: 1000,
                mtime: None,
            },
        );

        assert!(state.can_read(&PathBuf::from("/home/file.txt")));

        // Other user's private file
        state.filesystem.insert(
            PathBuf::from("/root/secret.txt"),
            EnhancedFileSystemEntry::File {
                content: "secret".to_string(),
                mode: 0o600, // rw-------
                uid: 0,      // root
                gid: 0,
                mtime: None,
            },
        );

        assert!(!state.can_read(&PathBuf::from("/root/secret.txt")));
    }

    #[test]
    fn test_can_write_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        // User's writable directory
        state.filesystem.insert(
            PathBuf::from("/home/user"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o755,
                uid: 1000,
                gid: 1000,
            },
        );

        assert!(state.can_write(&PathBuf::from("/home/user")));

        // Root-owned directory
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700,
                uid: 0,
                gid: 0,
            },
        );

        assert!(!state.can_write(&PathBuf::from("/root")));
    }

    #[test]
    fn test_root_bypasses_permission_checks() {
        let mut state = EnhancedState::new(); // Root user (UID 0)

        // Create root-only directory
        state.filesystem.insert(
            PathBuf::from("/root"),
            EnhancedFileSystemEntry::Directory {
                mode: 0o700, // rwx------
                uid: 0,
                gid: 0,
            },
        );

        // ✅ Root can read/write/execute despite restrictive permissions
        assert!(state.can_read(&PathBuf::from("/root")));
        assert!(state.can_write(&PathBuf::from("/root")));
        assert!(state.can_execute(&PathBuf::from("/root")));
    }

    #[test]
    fn test_read_file_with_permission_check() {
        let mut state = EnhancedState::new_user(1000, 1000);

        state.filesystem.insert(
            PathBuf::from("/tmp/file.txt"),
            EnhancedFileSystemEntry::File {
                content: "hello".to_string(),
                mode: 0o644, // rw-r--r--
                uid: 1000,
                gid: 1000,
                mtime: None,
            },
        );

        // ✅ User can read their own file
        assert_eq!(
            state.read_file(&PathBuf::from("/tmp/file.txt")).unwrap(),
            "hello"
        );

        // Create unreadable file
        state.filesystem.insert(
            PathBuf::from("/tmp/private.txt"),
            EnhancedFileSystemEntry::File {
                content: "secret".to_string(),
                mode: 0o000, // ---------
                uid: 0,
                gid: 0,
                mtime: None,
            },
        );

        // ❌ User cannot read unreadable file
        assert!(state.read_file(&PathBuf::from("/tmp/private.txt")).is_err());
        assert!(state.stderr.last().unwrap().contains("Permission denied"));
    }
}

#[cfg(test)]
mod enhanced_state_tests_extracted_state {
    use super::*;
    include!("enhanced_state_tests_extracted_state.rs");
}
