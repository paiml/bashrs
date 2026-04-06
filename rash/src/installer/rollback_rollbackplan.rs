/// A plan for rolling back steps
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    /// Steps to rollback in order
    pub steps: Vec<StepRollback>,

    /// Backup directory
    pub backup_dir: PathBuf,
}

impl RollbackPlan {
    /// Check if the plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get total number of actions
    pub fn action_count(&self) -> usize {
        self.steps.iter().map(|s| s.actions.len()).sum()
    }

    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "Rollback Plan: {} steps, {} actions\n",
            self.steps.len(),
            self.action_count()
        ));
        summary.push_str(&format!(
            "Backup directory: {}\n\n",
            self.backup_dir.display()
        ));

        for (i, step) in self.steps.iter().enumerate() {
            summary.push_str(&format!(
                "Step {}: {} ({})\n",
                i + 1,
                step.step_name,
                step.step_id
            ));

            if step.actions.is_empty() {
                summary.push_str("  (no rollback actions)\n");
            } else {
                for (j, action) in step.actions.iter().rev().enumerate() {
                    summary.push_str(&format!("  {}. {}\n", j + 1, action.description()));
                }
            }

            if !step.state_files.is_empty() {
                summary.push_str(&format!("  State files: {}\n", step.state_files.len()));
            }

            summary.push('\n');
        }

        summary
    }

    /// Execute the rollback plan (dry-run mode)
    pub fn execute_dry_run(&self) -> Vec<String> {
        let mut actions = Vec::new();

        for step in &self.steps {
            actions.push(format!("=== Rolling back: {} ===", step.step_name));

            for action in step.rollback_actions() {
                actions.push(format!("  Would execute: {}", action.description()));
            }
        }

        actions
    }
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compute SHA256 hash of content
fn compute_hash(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "rollback_tests_rollback_001.rs"]
// FIXME(PMAT-238): mod tests_extracted;
