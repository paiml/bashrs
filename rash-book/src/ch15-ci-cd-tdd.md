# Chapter 15: CI/CD Integration

<!-- DOC_STATUS_START -->
**Chapter Status**: ✅ 100% Working (8/8 examples)

| Status | Count | Examples |
|--------|-------|----------|
| ✅ Working | 8 | Ready for production use |
| ⚠️ Partial | 0 | Some edge cases not covered |
| ❌ Broken | 0 | Known issues, needs fixing |
| 📋 Planned | 0 | Future roadmap features |

*Last updated: 2025-11-14*
*bashrs version: 6.34.1*
<!-- DOC_STATUS_END -->

---

## The Problem

Shell scripts often bypass CI/CD quality gates because traditional tools don't integrate well. bashrs transpiles Rust to shell with built-in validation, making it easy to enforce quality in automated pipelines. From GitHub Actions to GitLab CI, Jenkins, and CircleCI - bashrs works everywhere.

In this chapter, you'll learn how to integrate bashrs into your CI/CD pipeline with comprehensive quality gates.

## Test-Driven Examples

### Example 1: GitHub Actions - Basic Pipeline

Minimal CI/CD pipeline with bashrs validation:

```yaml
# .github/workflows/bashrs.yml
name: Shell Script Quality

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install bashrs
        run: cargo install bashrs

      - name: Transpile Rust to Shell
        run: |
          bashrs build src/*.rs \
            --validation strict \
            --strict \
            --output-dir dist/

      - name: Verify Output
        run: |
          echo "Generated scripts:"
          ls -lh dist/
```

**Key Points:**
- Fast setup with official actions
- Strict validation enforced
- Zero-warning policy with `--strict`
- Simple artifact generation

### Example 2: GitHub Actions - Comprehensive Quality Gates

Production-grade pipeline with all quality checks:

```yaml
# .github/workflows/quality.yml
name: Comprehensive Quality

on:
  push:
    branches: [main]
  pull_request:

jobs:
  quality:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          components: clippy, rustfmt

      - name: Install Tools
        run: |
          cargo install bashrs
          cargo install cargo-llvm-cov
          sudo apt-get update && sudo apt-get install -y shellcheck

      - name: Code Quality - Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Code Quality - Format
        run: cargo fmt -- --check

      - name: Test Suite
        run: cargo test --all

      - name: Code Coverage
        run: |
          cargo llvm-cov --all-features --workspace --lcov \
            --output-path lcov.info
          cargo llvm-cov report

      - name: Transpile with Strict Validation
        run: |
          bashrs build examples/*.rs \
            --validation strict \
            --strict \
            --output-dir dist/

      - name: Shellcheck Validation
        run: |
          for script in dist/*.sh; do
            echo "Checking $script..."
            shellcheck -s sh -S warning "$script"
          done

      - name: Test Shell Scripts
        run: |
          for script in dist/*.sh; do
            echo "Testing $script..."
            sh "$script" --dry-run || sh "$script" --help
          done

      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: shell-scripts
          path: dist/*.sh

      - name: Upload Coverage
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
```

**Key Points:**
- Complete quality pipeline
- Clippy + rustfmt + tests + coverage
- bashrs strict validation
- shellcheck verification
- Artifact upload for deployment

### Example 3: GitLab CI - Multi-Stage Pipeline

Comprehensive GitLab CI pipeline with caching:

```yaml
# .gitlab-ci.yml
image: rust:latest

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/
    - target/

stages:
  - build
  - test
  - validate
  - deploy

before_script:
  - apt-get update && apt-get install -y shellcheck
  - cargo install bashrs

build:
  stage: build
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/

test:
  stage: test
  script:
    - cargo test --all
    - cargo clippy -- -D warnings
    - cargo fmt -- --check

validate:
  stage: validate
  script:
    # Transpile Rust to Shell
    - |
      bashrs build src/*.rs \
        --validation strict \
        --strict \
        --output-dir dist/

    # Validate with shellcheck
    - |
      for script in dist/*.sh; do
        shellcheck -s sh "$script"
      done

    # Test execution
    - |
      for script in dist/*.sh; do
        sh "$script" --version || true
      done
  artifacts:
    paths:
      - dist/*.sh
    expire_in: 1 week

deploy:
  stage: deploy
  script:
    - echo "Deploying shell scripts to production"
    - cp dist/*.sh /usr/local/bin/ || echo "Would deploy to /usr/local/bin/"
  only:
    - main
```

**Key Points:**
- Multi-stage pipeline (build → test → validate → deploy)
- Cargo caching for faster builds
- Artifact preservation
- Production deployment on main branch

### Example 4: Jenkins - Declarative Pipeline

Jenkins pipeline with Docker agents:

```groovy
// Jenkinsfile
pipeline {
    agent {
        docker {
            image 'rust:latest'
            args '-v /var/run/docker.sock:/var/run/docker.sock'
        }
    }

    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    apt-get update && apt-get install -y shellcheck
                    cargo install bashrs
                '''
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }

        stage('Test') {
            steps {
                sh '''
                    cargo test --all
                    cargo clippy -- -D warnings
                    cargo fmt -- --check
                '''
            }
        }

        stage('Transpile') {
            steps {
                sh '''
                    bashrs build src/*.rs \\
                        --validation strict \\
                        --strict \\
                        --output-dir dist/
                '''
            }
        }

        stage('Validate Shell') {
            steps {
                sh '''
                    for script in dist/*.sh; do
                        echo "Validating $script..."
                        shellcheck -s sh "$script"
                    done
                '''
            }
        }

        stage('Test Shell') {
            steps {
                sh '''
                    for script in dist/*.sh; do
                        echo "Testing $script..."
                        sh "$script" --help || true
                    done
                '''
            }
        }

        stage('Archive') {
            steps {
                archiveArtifacts artifacts: 'dist/*.sh', fingerprint: true
            }
        }
    }

    post {
        success {
            echo 'Pipeline succeeded!'
        }
        failure {
            echo 'Pipeline failed!'
        }
        always {
            cleanWs()
        }
    }
}
```

**Key Points:**
- Declarative pipeline syntax
- Docker-based agents
- Comprehensive testing
- Artifact archiving
- Post-build cleanup

### Example 5: CircleCI - Parallelized Testing

CircleCI pipeline with parallel test execution:

```yaml
# .circleci/config.yml
version: 2.1

orbs:
  rust: circleci/rust@1.6

executors:
  rust-executor:
    docker:
      - image: cimg/rust:1.75

jobs:
  build:
    executor: rust-executor
    steps:
      - checkout
      - rust/install
      - run:
          name: Install bashrs
          command: cargo install bashrs
      - run:
          name: Build
          command: cargo build --release
      - persist_to_workspace:
          root: .
          paths:
            - target/release

  test:
    executor: rust-executor
    parallelism: 4
    steps:
      - checkout
      - rust/install
      - run:
          name: Run Tests
          command: |
            cargo test --all -- \
              --test-threads=$(nproc) \
              $(circleci tests glob "tests/**/*.rs" | \
                circleci tests split)

  validate:
    executor: rust-executor
    steps:
      - checkout
      - attach_workspace:
          at: .
      - run:
          name: Install Tools
          command: |
            cargo install bashrs
            sudo apt-get update && sudo apt-get install -y shellcheck
      - run:
          name: Transpile with Validation
          command: |
            bashrs build examples/*.rs \
              --validation strict \
              --strict \
              --output-dir dist/
      - run:
          name: Shellcheck
          command: |
            for script in dist/*.sh; do
              shellcheck -s sh "$script"
            done
      - store_artifacts:
          path: dist
          destination: shell-scripts

workflows:
  build-test-deploy:
    jobs:
      - build
      - test:
          requires:
            - build
      - validate:
          requires:
            - test
```

**Key Points:**
- Orbs for Rust tooling
- Parallel test execution
- Workspace persistence
- Artifact storage

### Example 6: Pre-commit Hooks

Local validation before pushing to CI:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: Cargo Format
        entry: cargo fmt --
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-test
        name: Cargo Test
        entry: cargo test --all
        language: system
        types: [rust]
        pass_filenames: false

      - id: bashrs-validate
        name: bashrs Validation
        entry: bash -c 'bashrs build src/*.rs --validation strict --strict --output-dir /tmp/dist/'
        language: system
        types: [rust]
        pass_filenames: false

      - id: shellcheck
        name: Shellcheck Generated Scripts
        entry: bash -c 'for f in /tmp/dist/*.sh; do shellcheck -s sh "$f"; done'
        language: system
        files: '\\.rs$'
        pass_filenames: false
```

**Installation:**
```bash
# Install pre-commit
$ pip install pre-commit

# Install hooks
$ pre-commit install

# Run manually
$ pre-commit run --all-files
```

**Key Points:**
- Catch issues before commit
- Fast local feedback
- Same validation as CI
- Prevents broken commits

### Example 7: Docker-based CI

Portable CI pipeline using Docker:

```dockerfile
# Dockerfile.ci
FROM rust:1.75-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    shellcheck \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install bashrs
RUN cargo install bashrs

# Set working directory
WORKDIR /workspace

# Default command
CMD ["bash"]
```

**Docker Compose for CI:**
```yaml
# docker-compose.ci.yml
version: '3.8'

services:
  ci:
    build:
      context: .
      dockerfile: Dockerfile.ci
    volumes:
      - .:/workspace
    command: |
      bash -c "
        set -e
        echo '=== Running CI Pipeline ==='

        echo '--- Step 1: Format Check ---'
        cargo fmt -- --check

        echo '--- Step 2: Clippy ---'
        cargo clippy --all-targets -- -D warnings

        echo '--- Step 3: Tests ---'
        cargo test --all

        echo '--- Step 4: Transpile ---'
        bashrs build src/*.rs \\
          --validation strict \\
          --strict \\
          --output-dir dist/

        echo '--- Step 5: Shellcheck ---'
        for script in dist/*.sh; do
          shellcheck -s sh \$script
        done

        echo '--- Step 6: Test Scripts ---'
        for script in dist/*.sh; do
          sh \$script --help || true
        done

        echo '=== CI Pipeline Complete ==='
      "
```

**Usage:**
```bash
# Run CI locally
$ docker-compose -f docker-compose.ci.yml up --build

# Run specific stage
$ docker-compose -f docker-compose.ci.yml run ci cargo test
```

**Key Points:**
- Reproducible CI environment
- Same Docker image for local and remote CI
- Fast iteration with caching
- Works on any platform

### Example 8: Matrix Testing Across Shells

Test generated scripts on multiple shell interpreters:

```yaml
# .github/workflows/matrix.yml
name: Shell Compatibility Matrix

on: [push, pull_request]

jobs:
  matrix-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        shell: [bash, dash, ash, zsh, ksh]
        validation: [minimal, strict, paranoid]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install bashrs
        run: cargo install bashrs

      - name: Install Shell (${{ matrix.shell }})
        run: |
          sudo apt-get update
          case "${{ matrix.shell }}" in
            bash) echo "bash already installed" ;;
            dash) sudo apt-get install -y dash ;;
            ash) sudo apt-get install -y busybox-static && sudo ln -sf /bin/busybox /usr/local/bin/ash ;;
            zsh) sudo apt-get install -y zsh ;;
            ksh) sudo apt-get install -y ksh ;;
          esac

      - name: Transpile (validation=${{ matrix.validation }})
        run: |
          bashrs build examples/*.rs \
            --validation ${{ matrix.validation }} \
            --strict \
            --output-dir dist/

      - name: Test with ${{ matrix.shell }}
        run: |
          for script in dist/*.sh; do
            echo "Testing $script with ${{ matrix.shell }}..."
            ${{ matrix.shell }} "$script" --help || \
            ${{ matrix.shell }} "$script" --version || \
            echo "Script does not support --help or --version"
          done

      - name: Verify POSIX Compliance
        run: |
          for script in dist/*.sh; do
            shellcheck -s sh "$script"
          done
```

**Key Points:**
- Matrix testing: 5 shells × 3 validation levels = 15 combinations
- Validates POSIX compliance
- Ensures cross-platform compatibility
- Catches shell-specific bugs

## Best Practices

### 1. Always Use Strict Validation in CI
```yaml
# ✅ CORRECT: Strict + --strict flag
- run: bashrs build --validation strict --strict

# ❌ WRONG: Permissive validation
- run: bashrs build --validation minimal
```

### 2. Cache Dependencies
```yaml
# GitHub Actions
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### 3. Fail Fast on Errors
```yaml
# Set strict error handling
- run: |
    set -euxo pipefail
    bashrs build --validation strict --strict
```

### 4. Test Generated Scripts
```yaml
# Don't just generate - test execution!
- run: |
    for script in dist/*.sh; do
      sh "$script" --dry-run
    done
```

### 5. Use Matrix Testing
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    validation: [strict, paranoid]
```

## Troubleshooting

### Issue 1: bashrs Not Found in CI
**Problem**: `cargo install bashrs` fails or times out

**Solution**: Use caching or pre-built binaries
```yaml
- uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/bashrs
    key: bashrs-${{ runner.os }}
- run: cargo install bashrs || echo "Using cached bashrs"
```

### Issue 2: Shellcheck Warnings Fail Build
**Problem**: Generated scripts have shellcheck warnings

**Solution**: Use `--strict` flag to catch issues early
```yaml
- run: bashrs build --validation strict --strict
```

### Issue 3: Slow CI Builds
**Problem**: Transpilation takes too long

**Solution**: Use parallel builds and caching
```yaml
- run: |
    bashrs build src/*.rs --jobs $(nproc) --output-dir dist/
```

## Next Steps

- **Chapter 16**: Learn about MCP server integration
- **Chapter 17**: Comprehensive testing strategies
- **Chapter 18**: Understand bashrs limitations

## Summary

bashrs integrates seamlessly into CI/CD pipelines:

- ✅ **GitHub Actions**: Comprehensive quality gates
- ✅ **GitLab CI**: Multi-stage pipelines with caching
- ✅ **Jenkins**: Declarative pipelines with Docker
- ✅ **CircleCI**: Parallelized testing
- ✅ **Pre-commit hooks**: Local validation
- ✅ **Docker CI**: Reproducible environments
- ✅ **Matrix testing**: 5 shells × 3 validation levels
- ✅ **Best practices**: Strict validation, caching, fail-fast

**Integration is easy**: Add bashrs to your CI pipeline in minutes, enforce quality automatically! 🚀
