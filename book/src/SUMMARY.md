# The Rash Book

[Introduction](./introduction.md)

## Getting Started

- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)
- [Your First Purification](./getting-started/first-purification.md)
- [Interactive REPL](./getting-started/repl.md)

## Core Concepts

- [What is Purification?](./concepts/purification.md)
- [Determinism](./concepts/determinism.md)
- [Idempotency](./concepts/idempotency.md)
- [POSIX Compliance](./concepts/posix.md)

## Shell Script Linting

- [Security Rules (SEC001-SEC008)](./linting/security.md)
- [Determinism Rules (DET001-DET003)](./linting/determinism.md)
- [Idempotency Rules (IDEM001-IDEM003)](./linting/idempotency.md)
- [Writing Custom Rules](./linting/custom-rules.md)

## Shell Configuration Management

- [Overview](./config/overview.md)
- [Analyzing Config Files](./config/analyzing.md)
- [Purifying .bashrc and .zshrc](./config/purifying.md)
- [CONFIG-001: PATH Deduplication](./config/rules/config-001.md)
- [CONFIG-002: Quote Variables](./config/rules/config-002.md)
- [CONFIG-003: Consolidate Aliases](./config/rules/config-003.md)

## Makefile Linting

- [Makefile Overview](./makefile/overview.md)
- [Makefile Security](./makefile/security.md)
- [Makefile Best Practices](./makefile/best-practices.md)

## Examples

- [Bootstrap Installer](./examples/bootstrap-installer.md)
- [Deployment Script](./examples/deployment-script.md)
- [Configuration Management](./examples/config-management.md)
- [CI/CD Pipeline](./examples/cicd-pipeline.md)
- [Complete Quality Workflow: Real .zshrc](./example_zshrc_workflow.md)

## Advanced Topics

- [AST-Level Transformation](./advanced/ast-transformation.md)
- [Property Testing](./advanced/property-testing.md)
- [Mutation Testing](./advanced/mutation-testing.md)
- [Performance Optimization](./advanced/performance.md)

## Reference

- [CLI Commands](./reference/cli.md)
- [REPL Commands](./reference/repl-commands.md)
- [Configuration](./reference/configuration.md)
- [Exit Codes](./reference/exit-codes.md)
- [Linter Rules Reference](./reference/rules.md)

## Contributing

- [Development Setup](./contributing/setup.md)
- [EXTREME TDD](./contributing/extreme-tdd.md)
- [Toyota Way Principles](./contributing/toyota-way.md)
- [Release Process](./contributing/release.md)
