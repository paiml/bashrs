# Makefile Best Practices

Makefiles are critical build infrastructure, but they're often overlooked in code quality efforts. Shell commands embedded in Makefile recipes can harbor the same security, determinism, and idempotency issues as standalone shell scripts. This chapter covers best practices for writing safe, maintainable Makefiles and how bashrs helps enforce quality standards.

## Why Makefiles Need Linting

### The Hidden Shell Problem

Every Makefile recipe is shell code. Consider this common pattern:

```makefile
deploy:
	mkdir $(DEPLOY_DIR)
	rm $(OLD_FILES)
	ln -s $(RELEASE_DIR) $(CURRENT_LINK)
```

This looks innocent, but contains three critical flaws:

1. **Non-idempotent operations**: Re-running fails if directory exists
2. **Unquoted variables**: Shell injection risk if variables contain spaces
3. **Non-deterministic behavior**: Fails unpredictably in different states

### Real-World Impact

**Security**: Unquoted variables in recipes can lead to command injection:
```makefile
clean:
	rm -rf $(BUILD_DIR)  # If BUILD_DIR="/ etc", disaster!
```

**Reliability**: Non-idempotent operations break CI/CD pipelines:
```makefile
setup:
	mkdir build  # Fails on second run
```

**Determinism**: Timestamp-based commands produce unreproducible builds:
```makefile
release:
	echo "Built at $(shell date +%s)" > version.txt
```

### bashrs Makefile Support

bashrs v6.31.0 provides comprehensive Makefile analysis:

- **Parsing**: Full Makefile AST including targets, variables, and recipes
- **Linting**: Apply all security and determinism rules to shell recipes
- **Purification**: Transform recipes into safe, idempotent shell code
- **Validation**: Detect missing .PHONY declarations and anti-patterns

## Common Makefile Anti-Patterns

### 1. Unquoted Shell Variables in Recipes

**Problem**: Variables without quotes can cause word splitting and injection attacks.

**Anti-pattern**:
```makefile
INSTALL_DIR = /opt/myapp
SRC_FILES = $(wildcard src/*.c)

install:
	cp $(SRC_FILES) $(INSTALL_DIR)
	chmod 755 $(INSTALL_DIR)/*
```

**Issue**: If `INSTALL_DIR` contains spaces or special characters, the command breaks or executes unintended operations.

**Best Practice**:
```makefile
INSTALL_DIR = /opt/myapp
SRC_FILES = $(wildcard src/*.c)

install:
	cp "$(SRC_FILES)" "$(INSTALL_DIR)"
	chmod 755 "$(INSTALL_DIR)"/*
```

**bashrs Detection**:
```bash
$ bashrs make lint Makefile

Warning: Unquoted variable expansion in recipe
  --> Makefile:5:6
   |
 5 |     cp $(SRC_FILES) $(INSTALL_DIR)
   |        ^^^^^^^^^^^^ SC2086: Quote to prevent splitting
```

### 2. Non-Idempotent Operations

**Problem**: Operations that fail when run multiple times break build reproducibility.

**Anti-pattern**:
```makefile
setup:
	mkdir build
	mkdir dist
	ln -s build/output dist/latest
```

**Issue**: Second invocation fails because directories already exist.

**Best Practice**:
```makefile
setup:
	mkdir -p build
	mkdir -p dist
	rm -f dist/latest
	ln -s build/output dist/latest
```

**bashrs Detection**:
```bash
$ bashrs make lint Makefile

Warning: Non-idempotent operation
  --> Makefile:2:2
   |
 2 |     mkdir build
   |     ^^^^^^^^^^^ Use 'mkdir -p' for idempotent directory creation
```

### 3. Non-Deterministic Commands

**Problem**: Commands that produce different output on each run break reproducible builds.

**Anti-pattern**:
```makefile
VERSION = $(shell date +%Y%m%d%H%M%S)

release:
	echo "Release ID: $(RANDOM)" > release.txt
	echo "Built: $(shell date)" >> release.txt
	tar czf myapp-$(VERSION).tar.gz dist/
```

**Issue**: Every build creates a different artifact, making debugging and rollbacks impossible.

**Best Practice**:
```makefile
# Use explicit version from git or environment
VERSION ?= $(shell git describe --tags --always)
BUILD_ID ?= $(shell git rev-parse --short HEAD)

release:
	echo "Release ID: $(BUILD_ID)" > release.txt
	echo "Version: $(VERSION)" >> release.txt
	tar czf myapp-$(VERSION).tar.gz dist/
```

**bashrs Detection**:
```bash
$ bashrs make lint Makefile

Error: Non-deterministic command
  --> Makefile:4:2
   |
 4 |     echo "Release ID: $(RANDOM)" > release.txt
   |                       ^^^^^^^^^ DET003: Avoid $RANDOM
```

### 4. Missing .PHONY Declarations

**Problem**: Targets without .PHONY can be confused with actual files, causing unexpected behavior.

**Anti-pattern**:
```makefile
clean:
	rm -rf build/

test:
	cargo test

deploy:
	./deploy.sh
```

**Issue**: If a file named "clean", "test", or "deploy" exists, Make won't run the recipe.

**Best Practice**:
```makefile
.PHONY: clean test deploy

clean:
	rm -rf build/

test:
	cargo test

deploy:
	./deploy.sh
```

**bashrs Detection**:
```bash
$ bashrs make lint Makefile

Warning: Missing .PHONY declaration
  --> Makefile:1:1
   |
 1 | clean:
   | ^^^^^ Target 'clean' should be marked .PHONY
```

### 5. Hardcoded Paths

**Problem**: Hardcoded paths reduce portability and flexibility.

**Anti-pattern**:
```makefile
install:
	cp binary /usr/local/bin/myapp
	cp config.toml /etc/myapp/config.toml
	chmod 755 /usr/local/bin/myapp
```

**Issue**: Assumes specific system layout, breaks on different systems.

**Best Practice**:
```makefile
PREFIX ?= /usr/local
SYSCONFDIR ?= /etc
BINDIR = $(PREFIX)/bin
CONFDIR = $(SYSCONFDIR)/myapp

install:
	install -D -m 755 binary "$(BINDIR)/myapp"
	install -D -m 644 config.toml "$(CONFDIR)/config.toml"
```

### 6. Unsafe Command Chaining

**Problem**: Using `&&` without proper error handling can hide failures.

**Anti-pattern**:
```makefile
deploy:
	cd /var/www && rm -rf * && cp -r dist/* .
```

**Issue**: If `cd` fails, subsequent commands execute in the wrong directory (potentially catastrophic with `rm -rf *`).

**Best Practice**:
```makefile
DEPLOY_DIR = /var/www/myapp

deploy:
	test -d "$(DEPLOY_DIR)" || exit 1
	rm -rf "$(DEPLOY_DIR)"/*
	cp -r dist/* "$(DEPLOY_DIR)"/
```

## Best Practices with bashrs

### 1. Quote All Variables in Shell Recipes

**Rule**: Always quote Make variables when used in shell commands.

**Before**:
```makefile
SRC_DIR = src
BUILD_DIR = build

compile:
	gcc $(SRC_DIR)/*.c -o $(BUILD_DIR)/program
```

**After**:
```makefile
SRC_DIR = src
BUILD_DIR = build

compile:
	gcc "$(SRC_DIR)"/*.c -o "$(BUILD_DIR)/program"
```

**bashrs Verification**:
```bash
$ bashrs make purify Makefile
✓ All variables properly quoted
✓ No shell injection vulnerabilities
```

### 2. Use Idempotent Operations

**Rule**: All recipes should be safe to run multiple times.

**Before**:
```makefile
setup:
	mkdir build
	mkdir dist
	ln -s ../build dist/build
```

**After**:
```makefile
.PHONY: setup

setup:
	mkdir -p build
	mkdir -p dist
	ln -sf ../build dist/build
```

**Key Idempotent Patterns**:
- `mkdir -p` instead of `mkdir`
- `rm -f` instead of `rm`
- `ln -sf` instead of `ln -s`
- `install -D` for creating parent directories

### 3. Avoid Non-Deterministic Commands

**Rule**: Builds should be reproducible - same input = same output.

**Prohibited Patterns**:
```makefile
# DON'T: Non-deterministic ID generation
release:
	echo $(RANDOM) > release-id.txt

# DON'T: Timestamp-based versioning
VERSION = $(shell date +%s)

# DON'T: Process ID usage
lockfile:
	echo $$ > app.pid
```

**Approved Patterns**:
```makefile
# DO: Use git for versioning
VERSION = $(shell git describe --tags --always)

# DO: Use explicit version numbers
RELEASE_VERSION = 1.0.0

# DO: Use deterministic hashing
BUILD_HASH = $(shell git rev-parse --short HEAD)
```

### 4. Declare .PHONY Targets

**Rule**: All non-file targets must be marked .PHONY.

**Complete Example**:
```makefile
.PHONY: all clean build test install deploy help

all: build test

clean:
	rm -rf build/ dist/

build:
	cargo build --release

test:
	cargo test

install: build
	install -D -m 755 target/release/myapp "$(BINDIR)/myapp"

deploy: build test
	./scripts/deploy.sh

help:
	@echo "Available targets:"
	@echo "  all     - Build and test"
	@echo "  clean   - Remove build artifacts"
	@echo "  test    - Run test suite"
	@echo "  install - Install to system"
	@echo "  deploy  - Deploy to production"
```

### 5. Use bashrs make lint in Development

**Integrate into Workflow**:

```makefile
.PHONY: lint lint-make lint-scripts

lint: lint-make lint-scripts

lint-make:
	bashrs make lint Makefile

lint-scripts:
	bashrs lint scripts/*.sh
```

**Pre-commit Hook** (`.git/hooks/pre-commit`):
```bash
#!/bin/sh
set -e

echo "Linting Makefile..."
bashrs make lint Makefile

echo "Linting shell scripts..."
find . -name "*.sh" -exec bashrs lint {} \;
```

### 6. Handle Errors Properly

**Rule**: Use `.ONESHELL` and proper error handling for multi-line recipes.

**Before**:
```makefile
deploy:
	cd /var/www
	rm -rf old/
	cp -r dist/ .
```

**After**:
```makefile
.ONESHELL:
.SHELLFLAGS = -euo pipefail -c

DEPLOY_DIR = /var/www/myapp

deploy:
	cd "$(DEPLOY_DIR)" || exit 1
	rm -rf old/
	cp -r dist/ .
```

**Key Flags**:
- `-e`: Exit on error
- `-u`: Error on undefined variables
- `-o pipefail`: Catch errors in pipelines

## Examples: Problematic vs Clean Makefiles

### Example 1: Build System

**Problematic**:
```makefile
# Bad Makefile - DO NOT USE

SRC_DIR=src
BUILD_DIR=build
VERSION=$(shell date +%Y%m%d)

build:
	mkdir $(BUILD_DIR)
	gcc $(SRC_DIR)/*.c -o $(BUILD_DIR)/program
	echo "Built at: $(shell date)" > $(BUILD_DIR)/build-info.txt

clean:
	rm -r $(BUILD_DIR)

install:
	cp $(BUILD_DIR)/program /usr/local/bin
```

**Issues Found by bashrs**:
```bash
$ bashrs make lint Makefile

Error: Non-deterministic command (DET001)
  --> Makefile:3:9
   |
 3 | VERSION=$(shell date +%Y%m%d)
   |         ^^^^^^^^^^^^^^^^^^^^^^

Error: Non-idempotent operation (IDEM001)
  --> Makefile:6:2
   |
 6 |     mkdir $(BUILD_DIR)
   |     ^^^^^^^^^^^^^^^^^^ Use 'mkdir -p'

Warning: Unquoted variable (SC2086)
  --> Makefile:7:6
   |
 7 |     gcc $(SRC_DIR)/*.c -o $(BUILD_DIR)/program
   |         ^^^^^^^^^

Error: Non-deterministic command (DET002)
  --> Makefile:8:18
   |
 8 |     echo "Built at: $(shell date)" > $(BUILD_DIR)/build-info.txt
   |                     ^^^^^^^^^^^^^

Error: Missing .PHONY declarations
  --> Makefile:1:1
   | Targets should be .PHONY: build, clean, install

5 errors, 1 warning
```

**Clean Version**:
```makefile
# Clean Makefile - Best Practices Applied

.PHONY: all build clean install

# Use git for deterministic versioning
VERSION := $(shell git describe --tags --always --dirty)
BUILD_HASH := $(shell git rev-parse --short HEAD)

# Configurable directories
SRC_DIR := src
BUILD_DIR := build
INSTALL_PREFIX ?= /usr/local
BINDIR := $(INSTALL_PREFIX)/bin

all: build

build:
	mkdir -p "$(BUILD_DIR)"
	gcc "$(SRC_DIR)"/*.c -o "$(BUILD_DIR)/program"
	echo "Version: $(VERSION)" > "$(BUILD_DIR)/build-info.txt"
	echo "Commit: $(BUILD_HASH)" >> "$(BUILD_DIR)/build-info.txt"

clean:
	rm -rf "$(BUILD_DIR)"

install: build
	install -D -m 755 "$(BUILD_DIR)/program" "$(BINDIR)/program"
```

**Verification**:
```bash
$ bashrs make lint Makefile
✓ No issues found
✓ All variables quoted
✓ All operations idempotent
✓ All targets use .PHONY
✓ Deterministic build
```

### Example 2: Deployment Pipeline

**Problematic**:
```makefile
# Bad deployment Makefile

SERVER=prod-01
DEPLOY_PATH=/var/www/app
SESSION_ID=$(RANDOM)

deploy:
	ssh $(SERVER) "mkdir $(DEPLOY_PATH)/releases/$(SESSION_ID)"
	scp -r dist/* $(SERVER):$(DEPLOY_PATH)/releases/$(SESSION_ID)/
	ssh $(SERVER) "rm $(DEPLOY_PATH)/current"
	ssh $(SERVER) "ln -s $(DEPLOY_PATH)/releases/$(SESSION_ID) $(DEPLOY_PATH)/current"
	ssh $(SERVER) "systemctl restart myapp"

rollback:
	ssh $(SERVER) "rm $(DEPLOY_PATH)/current"
	ssh $(SERVER) "ln -s $(DEPLOY_PATH)/releases/previous $(DEPLOY_PATH)/current"
	ssh $(SERVER) "systemctl restart myapp"
```

**Issues**:
- Non-deterministic `$(RANDOM)` for session IDs
- Unquoted variables everywhere
- Non-idempotent operations (`mkdir`, `rm`, `ln`)
- No error handling
- Missing .PHONY declarations

**Clean Version**:
```makefile
# Clean deployment Makefile

.PHONY: deploy rollback status

# Configuration
SERVER := prod-01
DEPLOY_PATH := /var/www/app
RELEASE_DIR := $(DEPLOY_PATH)/releases

# Use git commit hash for deterministic release IDs
RELEASE_ID := $(shell git rev-parse --short HEAD)
RELEASE_PATH := $(RELEASE_DIR)/$(RELEASE_ID)

# Error handling
.ONESHELL:
.SHELLFLAGS := -euo pipefail -c

deploy:
	@echo "Deploying release $(RELEASE_ID) to $(SERVER)..."
	ssh "$(SERVER)" 'mkdir -p "$(RELEASE_PATH)"'
	rsync -avz --delete dist/ "$(SERVER):$(RELEASE_PATH)/"
	ssh "$(SERVER)" 'ln -sfn "$(RELEASE_PATH)" "$(DEPLOY_PATH)/current"'
	ssh "$(SERVER)" 'systemctl reload myapp'
	@echo "Deployment complete: $(RELEASE_ID)"

rollback:
	@echo "Rolling back on $(SERVER)..."
	$(eval PREVIOUS := $(shell ssh "$(SERVER)" 'readlink "$(DEPLOY_PATH)/previous"'))
	@test -n "$(PREVIOUS)" || (echo "No previous release found" && exit 1)
	ssh "$(SERVER)" 'ln -sfn "$(PREVIOUS)" "$(DEPLOY_PATH)/current"'
	ssh "$(SERVER)" 'systemctl reload myapp'
	@echo "Rolled back to: $(PREVIOUS)"

status:
	@echo "Current deployment status:"
	@ssh "$(SERVER)" 'readlink "$(DEPLOY_PATH)/current"'
```

**Key Improvements**:
- Deterministic release IDs from git
- All variables properly quoted
- Idempotent operations (`mkdir -p`, `ln -sfn`)
- Error handling with `.ONESHELL` and `-euo pipefail`
- .PHONY declarations
- Informative output

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install bashrs
        run: cargo install bashrs --version 6.31.0

      - name: Lint Makefile
        run: bashrs make lint Makefile

      - name: Lint shell scripts
        run: |
          find . -name "*.sh" -print0 | \
            xargs -0 -I {} bashrs lint {}

      - name: Verify idempotency
        run: |
          make clean
          make build
          make build  # Should succeed on second run

  build:
    needs: lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: make build
```

### GitLab CI Example

```yaml
stages:
  - lint
  - build
  - test

lint:makefile:
  stage: lint
  image: rust:latest
  before_script:
    - cargo install bashrs --version 6.31.0
  script:
    - bashrs make lint Makefile
    - make lint-scripts

lint:idempotency:
  stage: lint
  script:
    - make clean
    - make setup
    - make setup  # Verify idempotency

build:
  stage: build
  needs: ["lint:makefile", "lint:idempotency"]
  script:
    - make build
```

### Pre-commit Configuration

`.pre-commit-config.yaml`:
```yaml
repos:
  - repo: local
    hooks:
      - id: bashrs-makefile
        name: bashrs Makefile linting
        entry: bashrs make lint
        language: system
        files: '^Makefile$|\.mk$'

      - id: bashrs-scripts
        name: bashrs shell script linting
        entry: bashrs lint
        language: system
        files: '\.sh$'
```

## Testing Makefiles

### 1. Dry-Run Testing

Verify targets without executing:

```bash
# Check what would be executed
make -n build

# Verify variable expansion
make -n deploy | grep "Release ID"
```

**In Makefile**:
```makefile
.PHONY: test-dry-run

test-dry-run:
	@echo "Testing dry-run for all targets..."
	@make -n build > /dev/null && echo "✓ build dry-run OK"
	@make -n test > /dev/null && echo "✓ test dry-run OK"
	@make -n deploy > /dev/null && echo "✓ deploy dry-run OK"
```

### 2. Idempotency Testing

Ensure targets can run multiple times safely:

```makefile
.PHONY: test-idempotency

test-idempotency:
	@echo "Testing idempotency..."
	@make clean
	@make setup && echo "✓ First setup OK"
	@make setup && echo "✓ Second setup OK (idempotent)"
	@make build && echo "✓ First build OK"
	@make build && echo "✓ Second build OK (idempotent)"
```

**Automated Test Script** (`test-makefile.sh`):
```bash
#!/bin/bash
set -euo pipefail

echo "Testing Makefile idempotency..."

# Test each target twice
for target in setup build test; do
    echo "Testing target: $target"

    make clean
    make "$target" || exit 1
    echo "  ✓ First run succeeded"

    make "$target" || exit 1
    echo "  ✓ Second run succeeded (idempotent)"
done

echo "All idempotency tests passed!"
```

### 3. Determinism Testing

Verify reproducible builds:

```bash
#!/bin/bash
set -euo pipefail

echo "Testing build determinism..."

# Build twice and compare
make clean
make build
HASH1=$(find build -type f -exec sha256sum {} \; | sort | sha256sum)

make clean
make build
HASH2=$(find build -type f -exec sha256sum {} \; | sort | sha256sum)

if [ "$HASH1" = "$HASH2" ]; then
    echo "✓ Build is deterministic"
else
    echo "✗ Build is non-deterministic"
    exit 1
fi
```

### 4. shellcheck Integration

Verify generated shell commands:

```makefile
.PHONY: test-shellcheck

test-shellcheck:
	@echo "Extracting and checking shell recipes..."
	@bashrs make purify Makefile --output /tmp/purified.sh
	@shellcheck /tmp/purified.sh && echo "✓ All recipes pass shellcheck"
```

## Troubleshooting

### Issue: "Target not marked .PHONY"

**Symptom**:
```bash
$ bashrs make lint Makefile
Warning: Target 'clean' should be marked .PHONY
```

**Solution**:
```makefile
.PHONY: clean build test

clean:
	rm -rf build/
```

**Why**: Without .PHONY, if a file named "clean" exists, Make won't run the recipe.

### Issue: "Unquoted variable expansion"

**Symptom**:
```bash
$ bashrs make lint Makefile
Warning: Unquoted variable expansion (SC2086)
  --> Makefile:5:6
```

**Solution**:
```makefile
# Before
install:
	cp $(FILES) $(DEST)

# After
install:
	cp "$(FILES)" "$(DEST)"
```

**Why**: Prevents word splitting and glob expansion vulnerabilities.

### Issue: "Non-idempotent operation"

**Symptom**:
```bash
$ bashrs make lint Makefile
Error: Non-idempotent operation (IDEM001)
  --> Makefile:3:2
   |
 3 |     mkdir build
```

**Solution**:
```makefile
# Before
setup:
	mkdir build

# After
setup:
	mkdir -p build
```

**Why**: `mkdir -p` succeeds even if directory exists.

### Issue: "Non-deterministic command"

**Symptom**:
```bash
$ bashrs make lint Makefile
Error: Non-deterministic command (DET003)
  --> Makefile:6:2
   |
 6 |     echo "Build: $(RANDOM)" > version.txt
```

**Solution**:
```makefile
# Before
VERSION = $(shell date +%s)

release:
	echo "Build: $(RANDOM)" > version.txt

# After
VERSION = $(shell git describe --tags --always)
BUILD_HASH = $(shell git rev-parse --short HEAD)

release:
	echo "Version: $(VERSION)" > version.txt
	echo "Commit: $(BUILD_HASH)" >> version.txt
```

**Why**: Use git for deterministic, traceable versioning.

### Issue: Make variable vs. Shell variable confusion

**Symptom**:
```makefile
deploy:
	for file in *.txt; do
		echo "Processing $$file"  # Why $$?
	done
```

**Explanation**:
- **Make variable**: `$(VAR)` or `${VAR}` - expanded by Make
- **Shell variable**: `$$VAR` - `$$` escapes to single `$` in shell

**Correct Usage**:
```makefile
# Make variable (expanded by Make before shell sees it)
FILES = $(wildcard *.txt)

deploy:
	echo "Files: $(FILES)"  # Make expansion

	# Shell variable (expanded by shell)
	for file in *.txt; do
		echo "Processing $$file"  # Shell expansion ($$→$)
	done
```

### Issue: Recipe failing silently

**Symptom**: Multi-line recipe stops executing after error, but Make reports success.

**Solution**: Use `.ONESHELL` and proper error flags:

```makefile
.ONESHELL:
.SHELLFLAGS = -euo pipefail -c

deploy:
	cd /var/www
	rm -rf old/
	cp -r dist/ .
	# If any command fails, recipe stops with error
```

**Flags**:
- `-e`: Exit immediately on error
- `-u`: Error on undefined variables
- `-o pipefail`: Pipeline fails if any command fails

## Summary Checklist

Before committing Makefiles, verify:

- [ ] All non-file targets marked `.PHONY`
- [ ] All shell variables quoted in recipes
- [ ] All operations idempotent (use `-p`, `-f`, `-n` flags)
- [ ] No non-deterministic commands (`$RANDOM`, `date`, `$$`)
- [ ] Paths configurable with variables (not hardcoded)
- [ ] Error handling with `.ONESHELL` and proper flags
- [ ] Runs `bashrs make lint Makefile` without errors
- [ ] Tested for idempotency (runs twice successfully)
- [ ] Integrated into CI/CD linting pipeline

## Additional Resources

- **bashrs Makefile Documentation**: See `bashrs make --help`
- **GNU Make Manual**: https://www.gnu.org/software/make/manual/
- **ShellCheck Wiki**: https://www.shellcheck.net/wiki/
- **Reproducible Builds**: https://reproducible-builds.org/
- **POSIX Make**: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/make.html

## Conclusion

Makefiles are executable infrastructure code and deserve the same quality standards as application code. By applying these best practices and leveraging bashrs for automated validation, you can create Makefiles that are:

- **Safe**: No injection vulnerabilities
- **Reliable**: Idempotent operations that always work
- **Reproducible**: Deterministic builds for debugging and compliance
- **Maintainable**: Clear, documented, and testable

Run `bashrs make lint` on every Makefile change, integrate it into your CI/CD pipeline, and enforce these standards through pre-commit hooks. Your future self (and your teammates) will thank you.
