#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // RED Phase: Failing Tests First (EXTREME TDD)
    // Test naming: test_<TASK_ID>_<feature>_<scenario>
    // TASK_ID: CHECKPOINT_106
    // =========================================================================

    #[test]
    fn test_CHECKPOINT_106_create_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = CheckpointStore::new(temp_dir.path()).unwrap();
        assert!(store.current_run_id().is_none());
    }

    #[test]
    fn test_CHECKPOINT_106_start_run() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        let run_id = store.start_run("my-installer", "1.0.0").unwrap();
        assert!(run_id.starts_with("run-"));
        assert!(store.current_run_id().is_some());
    }

    #[test]
    fn test_CHECKPOINT_106_add_step() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Pending);
    }

    #[test]
    fn test_CHECKPOINT_106_step_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        // Start
        store.start_step("step-1").unwrap();
        assert_eq!(
            store.get_step("step-1").unwrap().status,
            StepStatus::Running
        );

        // Complete
        store
            .complete_step("step-1", Some("output".to_string()))
            .unwrap();
        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Completed);
        assert_eq!(step.output_log, Some("output".to_string()));
    }

    #[test]
    fn test_CHECKPOINT_106_step_failure() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();
        store.start_step("step-1").unwrap();
        store.fail_step("step-1", "Something went wrong").unwrap();

        let step = store.get_step("step-1").unwrap();
        assert_eq!(step.status, StepStatus::Failed);
        assert_eq!(step.error_message, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_CHECKPOINT_106_last_successful_step() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();

        store.add_step("step-1").unwrap();
        store.start_step("step-1").unwrap();
        store.complete_step("step-1", None).unwrap();

        store.add_step("step-2").unwrap();
        store.start_step("step-2").unwrap();
        store.complete_step("step-2", None).unwrap();

        store.add_step("step-3").unwrap();
        store.start_step("step-3").unwrap();
        store.fail_step("step-3", "error").unwrap();

        let last = store.last_successful_step().unwrap();
        assert_eq!(last.step_id, "step-2");
    }

    #[test]
    fn test_CHECKPOINT_106_hermetic_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store
            .start_hermetic_run("my-installer", "1.0.0", "abc123")
            .unwrap();
        assert!(store.is_hermetic());

        // Verify consistency with same hash
        store.verify_hermetic_consistency("abc123").unwrap();

        // Verify fails with different hash
        let result = store.verify_hermetic_consistency("different");
        assert!(result.is_err());
    }

    #[test]
    fn test_CHECKPOINT_106_track_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CheckpointStore::new(temp_dir.path()).unwrap();

        store.start_run("my-installer", "1.0.0").unwrap();
        store.add_step("step-1").unwrap();

        store
            .track_file("step-1", Path::new("/etc/config.txt"), "sha256:abc")
            .unwrap();

        let files = store.state_files_for_step("step-1");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content_hash, "sha256:abc");
    }

    #[test]
    fn test_CHECKPOINT_106_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and populate store
        {
            let mut store = CheckpointStore::new(temp_dir.path()).unwrap();
            store.start_run("my-installer", "1.0.0").unwrap();
            store.add_step("step-1").unwrap();
            store.start_step("step-1").unwrap();
            store
                .complete_step("step-1", Some("done".to_string()))
                .unwrap();
        }

        // Load from disk
        {
            let store = CheckpointStore::load(temp_dir.path()).unwrap();
            assert!(store.current_run_id().is_some());
            let step = store.get_step("step-1").unwrap();
            assert_eq!(step.status, StepStatus::Completed);
        }
    }

    #[test]
    fn test_CHECKPOINT_106_run_status_roundtrip() {
        for status in [
            RunStatus::Running,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Aborted,
        ] {
            let s = status.as_str();
            assert_eq!(RunStatus::parse(s), Some(status));
        }
    }

    #[test]
    fn test_CHECKPOINT_106_step_status_roundtrip() {
        for status in [
            StepStatus::Pending,
            StepStatus::Running,
            StepStatus::Completed,
            StepStatus::Failed,
            StepStatus::Skipped,
        ] {
            let s = status.as_str();
            assert_eq!(StepStatus::parse(s), Some(status));
        }
    }
}
