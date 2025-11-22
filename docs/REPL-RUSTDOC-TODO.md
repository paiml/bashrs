# REPL Rustdoc TODO - Path to 100% Public API Documentation

**Status**: Module-level documentation complete, function/struct/enum documentation needed
**Goal**: 100% public API documentation coverage
**Target**: REPL-017-003

## Completed ✅

- [x] Module-level documentation for `rash/src/repl/mod.rs` (162 lines)
- [x] Comprehensive overview with examples
- [x] Module descriptions
- [x] Feature documentation
- [x] Quick start guide
- [x] API usage examples

## Remaining Work 🚧

### Priority 1: Core Public APIs (Required for 100% coverage)

These modules expose public APIs that need comprehensive documentation:

1. **`config.rs`** - `ReplConfig` struct
   - [ ] Document all public fields
   - [ ] Document all public methods
   - [ ] Add usage examples

2. **`state.rs`** - `ReplState` struct
   - [ ] Document all public fields
   - [ ] Document all public methods
   - [ ] Add state management examples

3. **`modes.rs`** - `ReplMode` enum
   - [ ] Document each mode variant
   - [ ] Add mode switching examples

4. **`errors.rs`** - Error types and functions
   - [ ] Document all public structs (`ErrorMessage`, `SourceContext`, etc.)
   - [ ] Document all public enums (`ErrorType`, `Severity`)
   - [ ] Document all public functions
   - [ ] Add error handling examples

5. **`purifier.rs`** - Purification APIs
   - [ ] Document all public functions (`purify_bash`, `purify_and_lint`, etc.)
   - [ ] Document all public structs (`PurifiedLintResult`, `TransformationExplanation`, etc.)
   - [ ] Document all public enums (`TransformationCategory`, `SafetySeverity`)
   - [ ] Add purification examples

6. **`linter.rs`** - Linting APIs
   - [ ] Document all public functions
   - [ ] Add linting examples

7. **`parser.rs`** - Parsing APIs
   - [ ] Document all public functions
   - [ ] Add parsing examples

8. **`explain.rs`** - Explanation system
   - [ ] Document `Explanation` struct
   - [ ] Document `explain_bash` function
   - [ ] Add explanation examples

9. **`highlighting.rs`** - Syntax highlighting
   - [ ] Document all public structs (`Token`, `TokenType`)
   - [ ] Document all public functions
   - [ ] Add highlighting examples

10. **`debugger.rs`** - Debugging APIs
    - [ ] Document all public structs (`DebugSession`, `StackFrame`, etc.)
    - [ ] Document all public enums (`ContinueResult`, `LineComparison`)
    - [ ] Document all public functions
    - [ ] Add debugging examples

11. **`determinism.rs`** - Determinism checking
    - [ ] Document all public structs
    - [ ] Document all public enums
    - [ ] Document all public functions
    - [ ] Add determinism checking examples

12. **`breakpoint.rs`** - Breakpoint management
    - [ ] Document `Breakpoint` struct
    - [ ] Document `BreakpointManager` struct
    - [ ] Add breakpoint examples

### Priority 2: Internal Modules (Lower priority, but needed for completeness)

13. **`ast_display.rs`** - AST formatting
    - [ ] Document `format_ast` function

14. **`completion.rs`** - Tab completion
    - [ ] Document public completion functions

15. **`executor.rs`** - Command execution
    - [ ] Document public execution functions

16. **`help.rs`** - Help system
    - [ ] Document public help functions

17. **`loader.rs`** - Script loading
    - [ ] Document public loading functions

18. **`multiline.rs`** - Multi-line input
    - [ ] Document public multiline functions

19. **`variables.rs`** - Variable management
    - [ ] Document public variable functions

20. **`diff.rs`** - Diff display
    - [ ] Document `display_diff` function

### Priority 3: Main Entry Point

21. **`loop.rs`** - Main REPL loop
    - [ ] Document `run_repl` function with comprehensive examples

## Documentation Standards

Each public item should have:

1. **Summary**: One-line description
2. **Description**: Detailed explanation of what it does
3. **Examples**: At least one usage example
4. **Parameters**: Document all parameters (for functions)
5. **Returns**: Document return value (for functions)
6. **Errors**: Document possible errors (for fallible functions)
7. **Panics**: Document panic conditions (if any)
8. **Safety**: Document safety requirements (for unsafe code)

### Example Template

```rust
/// Short one-line summary of what this does.
///
/// Longer description explaining:
/// - What the function/struct/enum is for
/// - When you should use it
/// - How it relates to other APIs
///
/// # Examples
///
/// ```
/// use rash::repl::{ReplConfig, run_repl};
///
/// let config = ReplConfig::default();
/// run_repl(config)?;
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The terminal is not compatible
/// - The history file cannot be created
///
/// # Panics
///
/// Panics if the internal state becomes corrupted (should never happen).
pub fn function_name() -> Result<()> {
    // implementation
}
```

## Measurement

Track progress with:

```bash
# Check for missing documentation warnings
cargo doc --no-deps 2>&1 | grep "missing documentation"

# Count documented vs undocumented items
cargo doc --no-deps 2>&1 | grep -c "warning: missing documentation"

# Goal: 0 warnings
```

## Timeline

- **Week 1**: Document Priority 1 modules (core public APIs)
- **Week 2**: Document Priority 2 modules (internal modules)
- **Week 3**: Document Priority 3 (main entry point)
- **Week 4**: Review, polish, and verify 100% coverage

## Current Status

- **Module-level documentation**: ✅ Complete (1/1, 100%)
- **Function/struct/enum documentation**: 🚧 In Progress (0/~150, 0%)

**Estimated Remaining Work**: ~40-60 hours

## Next Steps

1. Start with `config.rs` (ReplConfig) - most commonly used by API users
2. Document `state.rs` (ReplState) - core state management
3. Continue with remaining modules in priority order
4. Run `cargo doc` frequently to verify documentation builds
5. Add examples to documentation tests
6. Verify examples compile with `cargo test --doc`

## Notes

- All examples in documentation must compile and pass
- Use `#` to hide setup code in examples (lines starting with `#`)
- Link to related documentation with `[Type]` markdown links
- Keep examples simple and focused
- Add "See Also" sections linking to related documentation
