# CLAUDE.md - Rash (bashrs) Development Guidelines

Rash is a shell safety tool: it purifies messy bash (and Makefiles) into deterministic, idempotent POSIX sh.
Rust → shell transpile is PLANNED (v3.0+), not production-ready.
Git/PR workflow, `pmat query` code search, no tarpaulin: see the global `~/.claude/CLAUDE.md`.
Long reference (full checklists, templates, WASM, measurements): `docs/agent-rules-history.md` (anchors per section).

## Contract-First (CRITICAL)
- NEVER write code before a provable contract (YAML in ../provable-contracts/contracts/<project>/ or .pmat-work/<TICKET>/contract.json); enforced by `pmat comply` CB-1400.
- Run `pmat comply check`; minimum level L1 (recommended L3+).
- Read docs/agent-instructions/provable-contract-first-agents.md before starting.

## Invariants
- Purified bash behaves like the original; no $RANDOM/timestamps/PIDs; idempotent (mkdir -p, rm -f, ln -sf); all variables quoted.
- Every generated/purified script passes `shellcheck -s sh`; same input gives byte-identical output.
- Coverage >85% on parser/transformer/generator modules; target >95%; complexity <10.
- Roadmaps MUST be YAML (`ROADMAP.yaml`, `docs/BASH-INGESTION-ROADMAP.yaml`), not markdown.

## EXTREME TDD
- TDD (RED→GREEN→REFACTOR) + proptest (100+ cases) + cargo-mutants (≥90% kill on touched code) + fuzz where applicable + PMAT gates (complexity <10, score ≥9.0) + `cargo run --example` for relevant examples.
- Focused mutation runs: `cargo mutants --file <path>`; config `.cargo/mutants.toml`.

## STOP THE LINE (Andon)
- On any bug found (missing impl, wrong transform, non-determinism, non-idempotency, failing test, shellcheck violation): HALT, document, open a P0 ticket, fix with EXTREME TDD, verify, only then resume.
- Fix steps: RED test, GREEN fix, REFACTOR, `bashrs repl` check, property test, mutation ≥90%, pmat verify, example run, integration test, regression test + roadmap + CHANGELOG.
- Read the P0 template and the full 10-step checklist in docs/agent-rules-history.md#stop-the-line-andon-cord before filing a P0.

## CLI testing (MANDATORY)
- All CLI tests MUST use `assert_cmd`; `std::process::Command` for CLI testing is a quality defect.
- Test names: `test_<TASK_ID>_<feature>_<scenario>` (e.g. `test_PARAM_POS_001_positional_params_basic`).
- Validate each CLI feature with parse, purify, transpile, lint, check (and ast/analyze for production features); cover success, error and edge cases.

## unwrap() policy
- Production lib code: BANNED, use `expect()` with a message. Tests/examples: allowed (`#![allow(clippy::unwrap_used)]`).
- Why: Cloudflare outage 2025-11-18. Workspace lint is still `allow` (not enforced); only Makefile `lint-check` denies it. Run `make lint-check` before adding calls.

## Quality gates
- Pre-commit hooks are MANDATORY (`pmat hooks install`, `pmat hooks status`); complexity max cyclomatic 30 / cognitive 25, SATD checked.
- Quality bar: 100% shellcheck/determinism/idempotency tests pass, zero defects, mutation >90%.
- Coverage: use the global rule (`pmat query --coverage-gaps`); never tarpaulin.

## Release (MANDATORY)
- Every release MUST be published to GitHub AND crates.io; it is not done until both are updated.
- Gate-driven, any day: `gate` green on the merge commit, full checklist passing, no calendar restriction.
- `bashrs corpus run` MUST report >= 17,942 entries; record the score in CHANGELOG (an empty corpus reads as a clean run, see #284).
- Book MUST be updated: `./scripts/check-book-updated.sh` and `mdbook test book`; release notes state what was measured.
- Rollback: `cargo yank --vers <version>`, then fix forward (also subject to the global yank rules).
- Read docs/agent-rules-history.md#release-protocol (5-phase checklist, commands) before releasing.

## WASM
- NEVER use Python (use `ruchy serve --port 8000 --watch-wasm` or bash).
- Run `make wasm-canary` before commit; `make wasm-canary-all`, `make wasm-mutation`, `make wasm-bench` before release; every anomaly (OOM, storage full, network failure, malformed input) needs a test.
- Performance baselines are MANDATORY (load <5s, 1KB analysis <100ms, memory <10MB); cross-browser matrix required.
- Read docs/agent-rules-history.md#webassembly-wasm before WASM work.

## Stack search
- Look up cross-stack patterns with `batuta oracle --rag "<question>"` (index: `batuta oracle --rag-index`).
