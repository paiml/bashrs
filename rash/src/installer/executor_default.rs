impl Default for StepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "executor_tests_113_executor.rs"]
mod tests_extracted;
