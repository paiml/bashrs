/// Result of continue execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinueResult {
    /// Stopped at a breakpoint on the specified line (1-indexed)
    BreakpointHit(usize),
    /// Execution completed without hitting breakpoint
    Finished,
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "debugger_tests_ext.rs"]
// FIXME(PMAT-238): mod tests_ext;
