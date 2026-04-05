/// Summary of gate execution
#[derive(Debug, Clone)]
pub struct GateSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration: Duration,
}

#[cfg(test)]
#[path = "gates_tests_extracted.rs"]
mod tests_extracted;
