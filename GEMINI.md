# Gemini Code Assistant Guide for `bashrs`

This document provides a guide for the Gemini Code Assistant to effectively understand and contribute to the `bashrs` project.

## Project Overview

`bashrs` (also known as Rash) is a bidirectional shell safety tool written in Rust. Its primary goal is to enable developers to write safe and reliable shell scripts using the Rust programming language. It offers two main workflows:

1.  **Rust to Shell (Primary):** Developers can write scripts in a subset of Rust, which are then transpiled into safe, POSIX-compliant shell scripts. This allows for compile-time checks, use of Rust's testing and linting tools, and eliminates common shell scripting vulnerabilities like command injection.

2.  **Bash to Rust to Purified Bash (Secondary):** `bashrs` can ingest existing "messy" bash scripts, convert them into Rust code with automatically generated tests, and then transpile them back into a "purified," safer version of the original shell script.

The project is mature, with a stable v1.1.0 release, extensive documentation, and a strong focus on quality, with high test coverage and mutation testing.

## Key Concepts and Terminology

*   **Rash:** The name of the tool.
*   **Transpilation:** The process of converting Rust code into shell script code.
*   **Purification:** The process of ingesting a bash script, converting it to Rust, and then transpiling it back to a safer, cleaner bash script.
*   **Safety:** A core principle of the project. `bashrs` aims to eliminate common shell scripting vulnerabilities.
*   **POSIX-compliant:** The generated shell scripts are designed to be compatible with the POSIX standard, ensuring they run on a wide range of shells.
*   **Native Linter:** `bashrs` includes its own linter for shell scripts, which is written in Rust and has no external dependencies.

## Core Technologies

*   **Rust:** The entire project is written in Rust.
*   **POSIX Shell:** The primary output of the transpilation process.

## Development Workflow

The project uses a standard Rust development workflow. Key commands are managed via a `Makefile`.

*   `make test`: Run the test suite.
*   `make validate`: Run all checks, including tests, linting, and formatting.
*   `make release`: Build a release version of the binary.

Contributions should follow the guidelines in `CONTRIBUTING.md`.

## How to Contribute as an LLM

1.  **Understand the Goal:** Before making any changes, understand whether the goal is to improve the Rust-to-Shell workflow, the Bash-to-Rust workflow, the native linter, or the documentation.

2.  **Follow Existing Patterns:** The codebase is well-structured. When adding new features or fixing bugs, follow the existing code style and architectural patterns.

3.  **Prioritize Safety:** All contributions must maintain or improve the safety guarantees of the tool. This means ensuring that generated shell scripts are properly quoted and escaped.

4.  **Write Tests:** All new features and bug fixes must be accompanied by tests. The project has a high standard for test coverage.

5.  **Use the Makefile:** Use the `Makefile` commands to run tests and validate your changes before submitting them.

6.  **Focus on the Rust Subset:** When working on the Rust-to-Shell transpiler, be mindful of the supported subset of the Rust language. The goal is to support Rust features that map cleanly to shell script concepts.

7.  **Consult the Documentation:** The `docs` directory contains a wealth of information about the project's design, features, and roadmap. Use this information to guide your contributions.

By following these guidelines, you can help to improve the `bashrs` project and make shell scripting safer and more reliable for everyone.
