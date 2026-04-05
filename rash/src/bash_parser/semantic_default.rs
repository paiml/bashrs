impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub scope_info: ScopeInfo,
    pub effects: EffectTracker,
    pub warnings: Vec<String>,
}

#[cfg(test)]
#[path = "semantic_tests_extracted.rs"]
mod tests_extracted;
