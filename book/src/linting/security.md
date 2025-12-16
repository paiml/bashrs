# Security Rules (SEC001-SEC008)

Rash includes 8 critical security rules designed to detect common shell script vulnerabilities. These rules follow **NASA-level quality standards** with an average 81.2% mutation test kill rate.

## Overview

Security linting in Rash focuses on **critical vulnerabilities** that can lead to:
- Command injection attacks
- Credential leaks
- Privilege escalation
- Remote code execution

All SEC rules are **Error severity** by default and should be addressed immediately.

## Quality Metrics

Our SEC rules undergo rigorous testing:

| Rule | Purpose | Mutation Kill Rate | Tests |
|------|---------|-------------------|-------|
| SEC001 | eval injection | 100% ✅ | 18 |
| SEC002 | Unquoted variables | 75.0% (baseline) | 24 |
| SEC003 | find -exec | 81.8% | 9 |
| SEC004 | TLS verification | 76.9% (baseline) | 13 |
| SEC005 | Hardcoded secrets | 73.1% (baseline) | 27 |
| SEC006 | Unsafe temp files | 85.7% (baseline) | 9 |
| SEC007 | Root operations | 88.9% (baseline) | 9 |
| SEC008 | curl \| sh | 87.0% (baseline) | 25 |

**Average Baseline**: 81.2% (exceeding 80% NASA-level target)

## SEC001: Command Injection via eval

**Severity**: Error (Critical)

### What it Detects

Use of `eval` with potentially user-controlled input.

### Why This Matters

`eval` is the #1 command injection vector in shell scripts. Attackers can execute arbitrary commands by injecting shell metacharacters.

### Examples

❌ **CRITICAL VULNERABILITY**:
```bash
#!/bin/bash
read -p "Enter command: " cmd
eval "$cmd"  # SEC001: Command injection via eval
```

```bash
#!/bin/bash
USER_INPUT="$1"
eval "rm -rf $USER_INPUT"  # SEC001: Dangerous!
```

✅ **SAFE ALTERNATIVE**:
```bash
#!/bin/bash
# Use array and proper quoting instead of eval
USER_INPUT="$1"

# Validate input first
if [[ ! "$USER_INPUT" =~ ^[a-zA-Z0-9_/-]+$ ]]; then
    echo "Invalid input"
    exit 1
fi

# Use explicit command construction
rm -rf "$USER_INPUT"
```

### Auto-fix

Not auto-fixable - requires manual security review.

## SEC002: Unquoted Variables in Commands

**Severity**: Error (Critical)

### What it Detects

Variables used in commands without proper quoting.

### Why This Matters

Unquoted variables can lead to:
- Word splitting attacks
- Glob expansion vulnerabilities
- Command injection via spaces/metacharacters

### Examples

❌ **VULNERABILITY**:
```bash
#!/bin/bash
rm -rf $HOME/my-folder  # SEC002: Word splitting risk
cd $HOME/my projects    # SEC002: Will fail on space
```

✅ **SAFE**:
```bash
#!/bin/bash
rm -rf "${HOME}/my-folder"  # Quoted - safe
cd "${HOME}/my projects"    # Quoted - handles spaces
```

### Auto-fix

Automatically quotes unquoted variables.

## SEC003: Command Injection via find -exec sh -c

**Severity**: Error (Critical)

### What it Detects

`find -exec sh -c` or `find -exec bash -c` with `{}` embedded inside the shell command string.

### Why This Matters

When `{}` appears inside a shell command string (`sh -c '...{}...'`), filenames with special characters can lead to command injection. The `{}` is expanded by `find` BEFORE the shell parses the string, allowing malicious filenames to inject arbitrary commands.

**Important**: Unquoted `{}` as a **separate argument** (`find -exec rm {} \;`) is **SAFE** because `find` passes the filename as a single argument to the command. The shell never interprets it.

### Examples

❌ **VULNERABILITY** (command injection via embedded `{}`):
```bash
#!/bin/bash
find . -exec sh -c 'echo {}' \;          # SEC003: {} embedded in shell string
find . -exec bash -c "rm {}" \;          # SEC003: {} embedded in shell string
```

✅ **SAFE** (use positional parameters):
```bash
#!/bin/bash
find . -exec sh -c 'echo "$1"' _ {} \;   # Safe: use positional params
find . -exec bash -c 'rm "$1"' _ {} \;   # Safe: $1 is properly quoted
```

✅ **SAFE** (`{}` as separate argument - NOT in shell string):
```bash
#!/bin/bash
find . -name "*.sh" -exec chmod +x {} \; # Safe: {} handled by find
find . -type f -exec rm {} +             # Safe: batch mode
find . -execdir mv {} {}.bak \;          # Safe: {} is separate argument
```

### Auto-fix

Suggests using positional parameters: `sh -c 'cmd "$1"' _ {}`

## SEC004: TLS Verification Disabled

**Severity**: Error (Critical)

### What it Detects

Commands that disable TLS/SSL certificate verification:
- `wget --no-check-certificate`
- `curl -k` or `curl --insecure`

### Why This Matters

Disabling TLS verification enables man-in-the-middle attacks where attackers can:
- Intercept sensitive data
- Inject malicious payloads
- Steal credentials

### Examples

❌ **VULNERABILITY**:
```bash
#!/bin/bash
wget --no-check-certificate https://example.com/install.sh  # SEC004
curl -k https://api.example.com/data                         # SEC004
curl --insecure https://api.example.com/data                 # SEC004
```

✅ **SAFE**:
```bash
#!/bin/bash
wget https://example.com/install.sh      # Verifies certificate
curl https://api.example.com/data         # Verifies certificate

# If you MUST skip verification (not recommended):
# Document WHY and use environment variable
if [ "$DISABLE_TLS_VERIFICATION" = "true" ]; then
    curl -k https://api.example.com/data
fi
```

### Auto-fix

Not auto-fixable - requires manual security review.

## SEC005: Hardcoded Secrets

**Severity**: Error (Critical)

### What it Detects

Hardcoded secrets in shell scripts:
- API keys
- Passwords
- Tokens
- AWS credentials

### Why This Matters

Hardcoded secrets lead to:
- Credential leaks in version control
- Unauthorized access
- Compliance violations (SOC2, PCI-DSS)

### Examples

❌ **CRITICAL VULNERABILITY**:
```bash
#!/bin/bash
API_KEY="sk-1234567890abcdef"           # SEC005: Hardcoded secret
PASSWORD="SuperSecret123"                 # SEC005: Hardcoded password
export AWS_SECRET_KEY="AKIA..."          # SEC005: Hardcoded AWS key
```

✅ **SAFE ALTERNATIVE**:
```bash
#!/bin/bash
# Load from environment
API_KEY="${API_KEY:-}"
if [ -z "$API_KEY" ]; then
    echo "ERROR: API_KEY not set"
    exit 1
fi

# Or load from secure secret manager
PASSWORD=$(aws secretsmanager get-secret-value --secret-id my-password --query SecretString --output text)

# Or load from encrypted file
PASSWORD=$(gpg --decrypt ~/.secrets/password.gpg)
```

### Auto-fix

Not auto-fixable - requires migration to secure secret management.

## SEC006: Unsafe Temporary Files

**Severity**: Error (Critical)

### What it Detects

Predictable temporary file creation:
- `/tmp/fixed_name.txt`
- `$TMPDIR/myapp.tmp`

### Why This Matters

Predictable temp files enable:
- Race condition attacks (TOCTOU)
- Symlink attacks
- Information disclosure

### Examples

❌ **VULNERABILITY**:
```bash
#!/bin/bash
TMP_FILE="/tmp/myapp.txt"      # SEC006: Predictable name
echo "data" > $TMP_FILE         # Race condition risk
```

✅ **SAFE**:
```bash
#!/bin/bash
# Use mktemp for secure temp file creation
TMP_FILE=$(mktemp)              # Random name, mode 0600
trap "rm -f $TMP_FILE" EXIT     # Clean up on exit

echo "data" > "$TMP_FILE"

# Or use mktemp with template
TMP_FILE=$(mktemp /tmp/myapp.XXXXXX)
```

### Auto-fix

Not auto-fixable - requires refactoring to use `mktemp`.

## SEC007: Root Operations Without Validation

**Severity**: Error (Critical)

### What it Detects

Operations run as root (`sudo`, `su`) without input validation:
- `sudo rm -rf $VAR`
- `su -c "$CMD"`

### Why This Matters

Root operations without validation can lead to:
- Complete system compromise
- Data destruction
- Privilege escalation

### Examples

❌ **CRITICAL VULNERABILITY**:
```bash
#!/bin/bash
USER_PATH="$1"
sudo rm -rf $USER_PATH  # SEC007: No validation!
```

✅ **SAFE**:
```bash
#!/bin/bash
USER_PATH="$1"

# Validate input strictly
if [[ ! "$USER_PATH" =~ ^/home/[a-z]+/[a-zA-Z0-9_/-]+$ ]]; then
    echo "Invalid path"
    exit 1
fi

# Verify path exists and is expected
if [ ! -d "$USER_PATH" ]; then
    echo "Path does not exist"
    exit 1
fi

# Use absolute path to avoid PATH attacks
/usr/bin/sudo /bin/rm -rf "$USER_PATH"
```

### Auto-fix

Not auto-fixable - requires manual security review.

## SEC008: curl | sh Pattern

**Severity**: Error (Critical)

### What it Detects

Piping remote content directly to shell execution:
- `curl https://example.com/install.sh | sh`
- `wget -qO- https://example.com/install.sh | bash`

### Why This Matters

This pattern enables:
- Remote code execution without review
- Man-in-the-middle injection
- Supply chain attacks

### Examples

❌ **CRITICAL VULNERABILITY**:
```bash
#!/bin/bash
curl https://example.com/install.sh | sh         # SEC008: Dangerous!
wget -qO- https://get.example.com | bash         # SEC008: No verification
curl -fsSL https://install.example.com | sudo sh # SEC008: Even worse!
```

✅ **SAFE ALTERNATIVE**:
```bash
#!/bin/bash
# Download, verify, then execute
INSTALL_SCRIPT="/tmp/install-$(date +%s).sh"
curl -fsSL https://example.com/install.sh > "$INSTALL_SCRIPT"

# Verify checksum
EXPECTED_SHA256="abc123..."
ACTUAL_SHA256=$(sha256sum "$INSTALL_SCRIPT" | awk '{print $1}')

if [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
    echo "Checksum mismatch!"
    rm "$INSTALL_SCRIPT"
    exit 1
fi

# Review the script manually
less "$INSTALL_SCRIPT"

# Execute after review
bash "$INSTALL_SCRIPT"
rm "$INSTALL_SCRIPT"
```

### Auto-fix

Not auto-fixable - requires manual security review.

## Running Security Linting

### Lint a Single File

```bash
bashrs lint script.sh
```

### Lint All Scripts in Project

```bash
find . -name "*.sh" -exec bashrs lint {} \;
```

### Lint with JSON Output (CI/CD)

```bash
bashrs lint --format json script.sh
```

### Filter Only Security Rules

```bash
bashrs lint --rules SEC script.sh
```

## Common Patterns

### Pattern 1: User Input Validation

Always validate user input before use:

```bash
#!/bin/bash
USER_INPUT="$1"

# Allowlist validation (preferred)
if [[ ! "$USER_INPUT" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    echo "Invalid input: only alphanumeric, underscore, hyphen allowed"
    exit 1
fi

# Now safe to use
echo "Processing: $USER_INPUT"
```

### Pattern 2: Secret Management

Use environment variables or secret managers:

```bash
#!/bin/bash
# Load from environment
API_KEY="${API_KEY:-}"
if [ -z "$API_KEY" ]; then
    echo "ERROR: API_KEY environment variable not set"
    echo "Set it with: export API_KEY=your-key"
    exit 1
fi

# Use the secret
curl -H "Authorization: Bearer $API_KEY" https://api.example.com
```

### Pattern 3: Safe Temporary Files

Always use `mktemp`:

```bash
#!/bin/bash
# Create temp file securely
TMPFILE=$(mktemp) || exit 1
trap "rm -f $TMPFILE" EXIT

# Use temp file
echo "data" > "$TMPFILE"
process_file "$TMPFILE"

# Cleanup happens automatically via trap
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Security Lint
on: [push, pull_request]
jobs:
  security-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install bashrs
        run: cargo install bashrs
      - name: Run security linting
        run: |
          find . -name "*.sh" -exec bashrs lint {} \; || exit 1
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Lint all staged shell scripts
git diff --cached --name-only --diff-filter=ACM | grep '\.sh$' | while read file; do
    bashrs lint "$file" || exit 1
done
```

## Testing Security Rules

All SEC rules are tested to NASA-level standards:

```bash
# Run SEC rule tests
cargo test --lib sec00

# Run mutation tests (requires cargo-mutants)
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib
```

Expected results: 80-100% mutation kill rate.

## Further Reading

- [OWASP Shell Injection](https://owasp.org/www-community/attacks/Command_Injection)
- [CWE-78: OS Command Injection](https://cwe.mitre.org/data/definitions/78.html)
- [NIST SP 800-218: Secure Software Development Framework](https://csrc.nist.gov/pubs/sp/800/218/final)

---

**Quality Guarantee**: All SEC rules undergo mutation testing with 81.2% average baseline kill rate, ensuring high-quality vulnerability detection.
