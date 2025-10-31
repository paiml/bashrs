# Makefile Purification with Rash

Rash provides **Makefile purification** - automatically detecting and fixing non-deterministic patterns in GNU Makefiles to ensure reproducible, deterministic builds.

## Why Purify Makefiles?

### The Problem

Makefiles often contain non-deterministic constructs that lead to unreproducible builds:

```makefile
# ❌ Non-deterministic - file order depends on filesystem
SOURCES := $(wildcard src/*.c)
HEADERS := $(wildcard include/*.h)

# ❌ Non-deterministic - find output order varies
ALL_FILES := $(shell find . -name '*.c')

# ❌ Parallel build races - multiple targets write same file
build/config.h: generate-config
	./gen-config > build/config.h

build/defaults.h: generate-defaults
	./gen-defaults > build/config.h  # ❌ Race condition!
```

**Result**: Different build outputs on different machines, flaky parallel builds, hard-to-reproduce bugs.

### The Solution

Rash automatically transforms Makefiles to be **deterministic** and **safe for parallel builds**:

```makefile
# ✅ Deterministic - sorted file order
SOURCES := $(sort $(wildcard src/*.c))
HEADERS := $(sort $(wildcard include/*.h))

# ✅ Deterministic - sorted find output
ALL_FILES := $(sort $(shell find . -name '*.c'))

# ✅ Parallel-safe - targets write different files
build/config.h: generate-config
	./gen-config > build/config.h

build/defaults.h: generate-defaults
	./gen-defaults > build/defaults.h  # ✅ No race
```

**Result**: Reproducible builds, reliable parallel execution, consistent behavior across machines.

## Features

Rash Makefile purification provides:

### 1. **Wildcard Sorting** (MAKE001)

```bash
$ rash lint Makefile
MAKE001: Non-deterministic wildcard expansion
  --> Makefile:10
   |
10 | SOURCES := $(wildcard src/*.c)
   |            ^^^^^^^^^^^^^^^^^^^ filesystem order is non-deterministic
   |
   = help: Wrap with $(sort ...) for determinism
   = fix: SOURCES := $(sort $(wildcard src/*.c))
```

### 2. **Shell Command Sorting** (MAKE002)

```bash
$ rash lint Makefile
MAKE002: Non-deterministic shell command
  --> Makefile:15
   |
15 | FILES := $(shell find . -name '*.c')
   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^ find output order varies
   |
   = help: Wrap with $(sort ...) for determinism
   = fix: FILES := $(sort $(shell find . -name '*.c'))
```

### 3. **Parallel Build Safety** (MAKE010-MAKE017)

- **MAKE010**: Detect shared file write races
- **MAKE011**: Recommend .NOTPARALLEL for unsafe patterns
- **MAKE012**: Detect missing dependencies
- **MAKE013**: Suggest order-only prerequisites
- **MAKE014**: Detect directory creation races
- **MAKE015**: Handle recursive make calls
- **MAKE016**: Detect output file conflicts
- **MAKE017**: Timestamp reproducibility

### 4. **Auto-Fix**

```bash
# Automatically fix all issues
$ rash lint --fix Makefile

Fixed 3 issues:
  ✅ MAKE001: Wrapped wildcard with sort (line 10)
  ✅ MAKE001: Wrapped wildcard with sort (line 11)
  ✅ MAKE002: Wrapped shell find with sort (line 15)

Makefile is now deterministic and reproducible!
```

## Quick Start

### Analyze a Makefile

```bash
# Check for issues
$ rash lint Makefile

# Auto-fix all issues
$ rash lint --fix Makefile

# Output purified Makefile
$ rash purify Makefile > Makefile.purified
```

### Example: Before and After

**Before** (`Makefile`):
```makefile
# Compiler settings
CC := gcc
CFLAGS := -O2 -Wall

# ❌ Non-deterministic wildcards
SOURCES := $(wildcard src/*.c)
HEADERS := $(wildcard include/*.h)
OBJECTS := $(SOURCES:.c=.o)

# Build rule
all: build/myapp

build/myapp: $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $(OBJECTS)
```

**After** (`rash lint --fix Makefile`):
```makefile
# Compiler settings
CC := gcc
CFLAGS := -O2 -Wall

# ✅ Deterministic - sorted wildcards
SOURCES := $(sort $(wildcard src/*.c))
HEADERS := $(sort $(wildcard include/*.h))
OBJECTS := $(SOURCES:.c=.o)

# Build rule
all: build/myapp

build/myapp: $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $(OBJECTS)
```

**Verification**:
```bash
# Build twice - should be identical
$ make clean && make
$ md5sum build/myapp > checksum1.txt

$ make clean && make
$ md5sum build/myapp > checksum2.txt

$ diff checksum1.txt checksum2.txt
# ✅ No differences - build is reproducible!
```

## Use Cases

### 1. Reproducible Builds

Ensure the same source code always produces the same binary:

```bash
# Purify Makefile
$ rash lint --fix Makefile

# Build on machine A
$ make clean && make
$ md5sum build/app
abc123...

# Build on machine B (same source)
$ make clean && make
$ md5sum build/app
abc123...  # ✅ Identical
```

### 2. Parallel Build Safety

Detect and fix race conditions in parallel builds:

```bash
$ rash lint Makefile
MAKE010: Parallel build race detected
  --> Makefile:25
   |
25 | build/config.h: generate-config
26 |     ./gen-config > build/config.h
   |
30 | build/defaults.h: generate-defaults
31 |     ./gen-defaults > build/config.h
   |                      ^^^^^^^^^^^^^^^^ multiple targets write same file
   |
   = warning: Running make -j may produce corrupted output
   = fix: Ensure each target writes unique output files
```

### 3. CI/CD Reliability

Eliminate flaky builds in continuous integration:

```yaml
# .github/workflows/build.yml
- name: Lint Makefile
  run: rash lint Makefile

- name: Build (parallel)
  run: make -j$(nproc)
  # ✅ No races, deterministic output
```

### 4. Cross-Platform Consistency

Same build results on Linux, macOS, BSD:

```bash
# purify Makefile to use sorted wildcards
$ rash lint --fix Makefile

# Build on any platform - identical results
```

## How It Works

Rash Makefile purification follows these steps:

1. **Parse** Makefile to AST
2. **Analyze** for non-deterministic patterns
3. **Transform** AST to fix issues
4. **Generate** purified Makefile

```text
┌─────────────┐
│   Makefile  │
│  (original) │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Parse AST  │  ← Lexer + Parser
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Analyze   │  ← Semantic analysis (297 tests)
│   Issues    │    - Wildcards, shell commands
│             │    - Parallel safety, timestamps
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Transform  │  ← Purification engine
│     AST     │    - Wrap with $(sort ...)
│             │    - Fix race conditions
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Generate   │  ← Code generation
│  Purified   │
│  Makefile   │
└─────────────┘
```

## Quality Assurance

Rash Makefile support has **NASA-level testing**:

- **297 unit tests** covering all transformations
- **Property-based testing** with 100+ random Makefiles
- **EXTREME TDD** methodology (RED-GREEN-REFACTOR)
- **Zero tolerance** for regressions

```rust
#[test]
fn test_MAKE001_wildcard_basic() {
    let makefile = "SOURCES := $(wildcard *.c)";
    let result = purify_makefile(makefile).unwrap();

    assert_eq!(
        result,
        "SOURCES := $(sort $(wildcard *.c))"
    );
}
```

**Test Coverage**: 100% of purification logic tested

## Next Steps

- [Makefile Security](./security.md) - Detect injection vulnerabilities
- [Makefile Best Practices](./best-practices.md) - Recommended patterns

## Resources

- GNU Make Manual: https://www.gnu.org/software/make/manual/
- Reproducible Builds: https://reproducible-builds.org/
- SOURCE_DATE_EPOCH: https://reproducible-builds.org/specs/source-date-epoch/

---

**Pro Tip**: Use `rash lint --fix` as a pre-commit hook to ensure all Makefiles remain deterministic:

```bash
# .git/hooks/pre-commit
#!/bin/bash
rash lint --fix Makefile
git add Makefile
```
