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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "semantic_tests_make_ast.rs"]
// FIXME(PMAT-238): mod tests_extracted;
