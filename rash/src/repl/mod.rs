// REPL module for bashrs - Interactive REPL with integrated debugger
//
// Architecture: Debugger-as-REPL (matklad pattern)
// Integration: Symbiotic embedding (RuchyRuchy pattern)
//
// Sprint: REPL-003 (Basic REPL Loop)
// Status: Phase 1 - RED-GREEN-REFACTOR-PROPERTY-MUTATION

pub mod config;

pub use config::ReplConfig;
