/// Summary of gate execution
#[derive(Debug, Clone)]
pub struct GateSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration: Duration,
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "gates_tests_ml_001.rs"]
// FIXME(PMAT-238): mod tests_extracted;
