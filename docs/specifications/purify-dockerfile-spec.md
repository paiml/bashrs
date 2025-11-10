# Specification: Dockerfile Purification

## Overview

Extend bashrs purification capabilities to support Dockerfiles, enabling automatic fixes for security issues, best practices violations, and optimization opportunities.

**Status**: DRAFT (v1.0)
**Target Version**: v6.36.0
**Created**: 2025-11-10
**Author**: EXTREME TDD Specification

---

## Motivation

### Problem Statement

Currently, bashrs can:
- ✅ **Lint** Dockerfiles (detect issues via DOCKER001-DOCKER010+ rules)
- ✅ **Purify** bash scripts (fix determinism, idempotency, safety issues)
- ✅ **Purify** Makefiles (fix formatting, preserve structure)

However, it **cannot**:
- ❌ **Auto-fix** Dockerfile issues (no purification pipeline)
- ❌ **Transform** RUN commands to be safer/more efficient
- ❌ **Optimize** layer structure automatically

### Use Cases

1. **Security Hardening**: Auto-add USER directives, pin base images
2. **Image Optimization**: Combine RUN commands, add cleanup steps
3. **Best Practices**: Convert ADD to COPY, add --no-install-recommends
4. **Standardization**: Enforce consistent formatting across team Dockerfiles
5. **CI/CD Integration**: Auto-fix Dockerfiles in pipelines

---

## Scope

### In Scope

1. **Security Fixes** (based on existing lint rules):
   - Add missing USER directive (DOCKER001)
   - Pin unpinned base images (DOCKER002)
   - Add apt/apk cleanup commands (DOCKER003)
   - Fix invalid COPY --from syntax (DOCKER004)
   - Add --no-install-recommends flags (DOCKER005)
   - Convert ADD to COPY where appropriate (DOCKER006)

2. **Bash Purification in RUN Commands**:
   - Apply bash purification to shell scripts in RUN instructions
   - Maintain determinism and idempotency in RUN commands
   - Quote variables, add safety flags

3. **Optimization** (optional):
   - Combine consecutive RUN commands (reduces layers)
   - Reorder instructions for better caching
   - Add multi-stage build suggestions

4. **Formatting**:
   - Preserve or standardize line continuations (`\`)
   - Maintain consistent indentation
   - Preserve comments

### Out of Scope (Future Work)

- Full Dockerfile DSL parsing (use regex/heuristic approach initially)
- Multi-stage build optimization (complex analysis required)
- Base image vulnerability scanning (external tool integration)
- Custom linting rules (extensibility for v2.0)

---

## Purification Transformations

### 1. Security Fixes

#### DOCKER001: Add Missing USER Directive

**Detection**: Dockerfile missing `USER` directive (runs as root)

**Transformation**:
```dockerfile
# Input
FROM debian:12-slim
WORKDIR /app
CMD ["python3", "app.py"]

# Output (purified)
FROM debian:12-slim
WORKDIR /app

# Security: Run as non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

CMD ["python3", "app.py"]
```

**Edge Cases**:
- Skip for `FROM scratch` images (no user support)
- Skip if USER already present
- Insert before CMD/ENTRYPOINT

#### DOCKER002: Pin Unpinned Base Images

**Detection**: Base image without version tag

**Transformation**:
```dockerfile
# Input
FROM ubuntu
FROM debian:latest

# Output (purified)
FROM ubuntu:22.04  # Pinned to LTS
FROM debian:12-slim  # Pinned to stable
```

**Strategy**:
- Use latest stable/LTS version for common images
- Configurable via `.bashrs.toml` (preferred versions)
- Warn if cannot determine version

#### DOCKER003: Add apt/apk Cleanup

**Detection**: `apt-get install` or `apk add` without cleanup

**Transformation**:
```dockerfile
# Input
RUN apt-get update && apt-get install -y curl

# Output (purified)
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    rm -rf /var/lib/apt/lists/*
```

**Package Managers**:
- apt/apt-get (Debian/Ubuntu)
- apk (Alpine)
- yum/dnf (RHEL/Fedora)

#### DOCKER005: Add --no-install-recommends

**Detection**: apt-get install without `--no-install-recommends`

**Transformation**:
```dockerfile
# Input
RUN apt-get install -y python3

# Output (purified)
RUN apt-get install -y --no-install-recommends python3
```

#### DOCKER006: Convert ADD to COPY

**Detection**: `ADD` used for local files (not URLs/tarballs)

**Transformation**:
```dockerfile
# Input
ADD app.py /app/

# Output (purified)
COPY app.py /app/
```

**Keep ADD for**:
- URLs: `ADD https://example.com/file.tar.gz /tmp/`
- Tar extraction: `ADD archive.tar.gz /app/`

### 2. Bash Purification in RUN Commands

Apply existing bash purification to RUN instructions:

```dockerfile
# Input
RUN TEMP_DIR="/tmp/build-$$" && mkdir $TEMP_DIR

# Output (purified)
RUN TEMP_DIR="/tmp/build-deterministic" && mkdir -p "${TEMP_DIR}"
```

**Transformations**:
- Remove `$$`, `$RANDOM` (determinism)
- Add `-p`, `-f` flags (idempotency)
- Quote variables (safety)

### 3. Layer Optimization (Optional)

**Detection**: Consecutive RUN commands

**Transformation**:
```dockerfile
# Input
RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y wget

# Output (purified with --optimize-layers)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        wget && \
    rm -rf /var/lib/apt/lists/*
```

**Enabled via**: `--optimize-layers` flag (off by default)

---

## Command-Line Interface

### Basic Usage

```bash
# Purify Dockerfile to stdout
bashrs dockerfile purify Dockerfile

# Purify to file
bashrs dockerfile purify Dockerfile -o Dockerfile.purified

# In-place purification (creates .bak backup)
bashrs dockerfile purify Dockerfile --fix

# Dry run (show what would change)
bashrs dockerfile purify Dockerfile --dry-run
```

### Options

```bash
bashrs dockerfile purify [OPTIONS] <FILE>

Options:
  -o, --output <FILE>        Output file (default: stdout)
      --fix                  Apply fixes in-place (creates .bak backup)
      --no-backup            Don't create backup with --fix (dangerous!)
      --dry-run              Show changes without applying
      --report               Show detailed transformation report
      --format <FORMAT>      Report format: human|json|markdown [default: human]
      --optimize-layers      Combine consecutive RUN commands (reduces layers)
      --preserve-formatting  Keep original line breaks and indentation
      --pin-versions <FILE>  Config file for pinning base image versions
      --skip-user            Don't add USER directive (for special cases)
      --skip-bash-purify     Don't purify bash in RUN commands
```

### Integration with Existing Commands

```bash
# Lint before purifying (recommended workflow)
bashrs lint Dockerfile
bashrs dockerfile purify Dockerfile --fix

# Lint after purifying (verify fixes)
bashrs dockerfile purify Dockerfile | bashrs lint -
```

---

## Implementation Architecture

### Phase 1: Parser (Simple Regex-Based)

**Goal**: Extract Dockerfile instructions without full DSL parsing

```rust
pub struct DockerfileAst {
    pub instructions: Vec<DockerInstruction>,
    pub metadata: DockerMetadata,
}

pub enum DockerInstruction {
    From { image: String, tag: Option<String>, span: Span },
    Run { commands: Vec<String>, span: Span },
    Copy { source: String, dest: String, from: Option<String>, span: Span },
    Add { source: String, dest: String, span: Span },
    User { username: String, span: Span },
    Workdir { path: String, span: Span },
    Cmd { args: Vec<String>, span: Span },
    Entrypoint { args: Vec<String>, span: Span },
    Comment { text: String, span: Span },
    // ... other instructions
}
```

**Parsing Strategy**:
- Line-by-line regex matching (Dockerfiles are line-oriented)
- Handle line continuations (`\`)
- Preserve comments
- Track source locations (span) for error reporting

### Phase 2: Analyzer

**Goal**: Detect issues that can be auto-fixed

```rust
pub struct DockerfileAnalyzer;

impl DockerfileAnalyzer {
    pub fn analyze(ast: &DockerfileAst) -> AnalysisResult {
        // Detect missing USER directive
        // Detect unpinned base images
        // Detect missing cleanup in RUN commands
        // etc.
    }
}

pub struct AnalysisResult {
    pub issues: Vec<DockerIssue>,
    pub fixable: Vec<DockerFix>,
}
```

### Phase 3: Transformer

**Goal**: Apply transformations to AST

```rust
pub struct DockerfileTransformer;

impl DockerfileTransformer {
    pub fn transform(
        ast: &DockerfileAst,
        fixes: &[DockerFix],
        options: &PurifyOptions,
    ) -> DockerfileAst {
        // Apply each fix
        // Maintain instruction order
        // Preserve non-affected instructions
    }
}
```

### Phase 4: Generator

**Goal**: Generate purified Dockerfile from AST

```rust
pub fn generate_purified_dockerfile(
    ast: &DockerfileAst,
    options: &PurifyOptions,
) -> String {
    // Reconstruct Dockerfile text
    // Apply formatting options
    // Preserve comments
}
```

---

## Configuration

### `.bashrs.toml` Configuration

```toml
[dockerfile]
# Purification settings
add_user_directive = true
pin_base_images = true
cleanup_package_managers = true
optimize_layers = false  # Off by default (can change semantics)
preserve_formatting = true

# Base image version pinning
[dockerfile.pinned_versions]
"ubuntu" = "22.04"
"debian" = "12-slim"
"alpine" = "3.19"
"node" = "20-alpine"
"python" = "3.11-slim"
"rust" = "1.75-alpine"

# Package manager settings
[dockerfile.package_managers]
apt_cleanup = "rm -rf /var/lib/apt/lists/*"
apk_cleanup = "rm -rf /var/cache/apk/*"
yum_cleanup = "yum clean all"

# User creation template
[dockerfile.user_template]
command = "RUN groupadd -r appuser && useradd -r -g appuser appuser"
username = "appuser"
```

---

## Quality Guarantees

### Correctness

1. **Semantic Preservation**: Purified Dockerfile must build identical image functionality
2. **Build Success**: Purified Dockerfile must build without errors
3. **No Regressions**: Existing working Dockerfiles must continue to work

### Testing Strategy (EXTREME TDD)

1. **Unit Tests**: Test each transformation independently
2. **Integration Tests**: Test full purification pipeline
3. **Property Tests**: Generative testing (100+ Dockerfiles)
4. **Mutation Tests**: ≥90% kill rate on purification logic
5. **Real-World Tests**: Test against popular Dockerfiles (nginx, postgres, etc.)

### Validation

```bash
# Before/after comparison
docker build -t test:before -f Dockerfile.original .
docker build -t test:after -f Dockerfile.purified .

# Verify images are functionally equivalent
docker run test:before env | sort > before.txt
docker run test:after env | sort > after.txt
diff before.txt after.txt  # Should be minimal/expected differences
```

---

## Success Metrics

### Coverage Targets

- **Transformation Rules**: 10 rules (DOCKER001-006 + bash purification + optimizations)
- **Test Coverage**: ≥85% line coverage
- **Mutation Score**: ≥90% kill rate
- **Real-World Success**: Works on 90% of Dockerfiles in `rash/examples/dockerfiles/`

### Performance Targets

- **Parse Time**: <50ms for typical Dockerfile (<100 lines)
- **Purify Time**: <200ms end-to-end
- **Memory**: <10MB peak usage

### User Experience

- **Clear Reports**: Show exactly what changed and why
- **Rollback**: Easy to revert with .bak files
- **Dry Run**: Preview changes before applying
- **Documentation**: Complete examples and best practices guide

---

## Rollout Plan

### v6.36.0 (Phase 1 - MVP)

**Target**: 2-3 weeks from spec approval

1. Basic parser (regex-based, handle common instructions)
2. Core transformations (DOCKER001, DOCKER002, DOCKER003)
3. CLI integration (`bashrs dockerfile purify`)
4. 50+ unit tests, 10 integration tests

**Success Criteria**: Can purify simple Dockerfiles (single-stage, common base images)

### v6.37.0 (Phase 2 - Production Ready)

**Target**: +2 weeks

1. All 10 transformation rules implemented
2. Bash purification in RUN commands
3. Property testing (100+ generated Dockerfiles)
4. Real-world validation (nginx, postgres, node, python images)

**Success Criteria**: 90% success rate on real-world Dockerfiles

### v6.38.0 (Phase 3 - Optimization)

**Target**: +2 weeks

1. Layer optimization (combine RUN commands)
2. Multi-stage build support
3. Advanced formatting options
4. Performance optimization (<100ms purify time)

**Success Criteria**: Production-grade performance and features

---

## Risk Assessment

### High Risk

1. **Semantic Changes**: Optimizations might break builds
   - **Mitigation**: Off by default, extensive testing, dry-run mode

2. **Bash Purification Conflicts**: RUN commands might have special quoting
   - **Mitigation**: Conservative bash purification, skip on parse errors

### Medium Risk

1. **Version Pinning Staleness**: Pinned versions become outdated
   - **Mitigation**: Configurable via .bashrs.toml, warn on old versions

2. **Edge Cases**: Complex Dockerfiles with unusual syntax
   - **Mitigation**: Graceful degradation, warn and skip unsupported features

### Low Risk

1. **Performance**: Large Dockerfiles (>1000 lines)
   - **Mitigation**: Streaming parser, benchmark with large files

---

## Alternatives Considered

### 1. Full Dockerfile Parser (like BuildKit)

**Pros**: Correct handling of all Dockerfile syntax
**Cons**: Complex, slow, overkill for purification
**Decision**: Use regex-based approach for MVP, upgrade later if needed

### 2. External Tool Integration (hadolint, dockle)

**Pros**: Mature, well-tested
**Cons**: No auto-fix capability, not integrated with bashrs ecosystem
**Decision**: Implement native purification, learn from hadolint design

### 3. LLM-Based Purification

**Pros**: Can handle complex cases
**Cons**: Non-deterministic, slow, requires API
**Decision**: Rule-based for reliability, consider LLM as optional enhancement

---

## References

- [Dockerfile Best Practices](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
- [hadolint](https://github.com/hadolint/hadolint) - Dockerfile linter
- [dockle](https://github.com/goodwithtech/dockle) - Security linter
- bashrs DOCKER001-010 rules (existing implementation)

---

## Approval

**Status**: DRAFT - Awaiting Review
**Reviewers**: bashrs maintainers
**Approval Date**: TBD

---

**Document Version**: 1.0
**Last Updated**: 2025-11-10
**Next Review**: After Phase 1 implementation
