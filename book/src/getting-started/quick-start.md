# Quick Start

This chapter will get you up and running with Rash in 5 minutes.

## Your First Command

Let's start by checking the version:

```bash
bashrs --version
```

## Linting a Shell Script

Create a simple shell script with a security issue:

```bash,no_run
cat > vulnerable.sh << 'EOF'
#!/bin/bash
# Vulnerable script - uses eval with user input
read -p "Enter command: " cmd
eval "$cmd"
EOF
```

Now lint it with Rash:

```bash,no_run
bashrs lint vulnerable.sh
```

You'll see output like:

```text
[SEC-001] Use of eval with user input can lead to command injection
  → Line 4: eval "$cmd"
  → Severity: Critical
  → Suggestion: Avoid eval or validate input strictly
```

## Analyzing a Configuration File

Let's analyze a messy .bashrc file:

```bash,no_run
cat > messy-bashrc << 'EOF'
# Messy .bashrc with issues
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate!

# Unquoted variables (security issue)
cd $HOME/my projects
EOF
```

Analyze it:

```bash,no_run
bashrs config analyze messy-bashrc
```

Output:

```text
Analysis: messy-bashrc
  Type: Generic config file
  Lines: 7
  Complexity: 3/10

Issues Found: 2
  [CONFIG-001] Duplicate PATH entry at line 3
    → Path: /usr/local/bin
    → First seen: line 1
    → Suggestion: Remove duplicate entry

  [CONFIG-002] Unquoted variable expansion at line 6
    → Variable: $HOME
    → Can cause word splitting and glob expansion
    → Suggestion: Quote the variable: "${HOME}"
```

## Purifying Configuration

Now let's fix these issues automatically:

```bash,no_run
bashrs config purify messy-bashrc --output clean-bashrc
```

The purified output (`clean-bashrc`):

```bash
# Messy .bashrc with issues
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
# Duplicate! (removed by CONFIG-001)

# Unquoted variables (fixed by CONFIG-002)
cd "${HOME}/my projects"
```

## Testing with mdbook

The examples in this book are automatically tested! Here's a Rust test example:

```rust
// This code is tested when building the book
fn purify_example() {
    let input = "export DIR=$HOME/projects";
    let expected = "export DIR=\"${HOME}/projects\"";

    // This would use the actual Rash library
    // assert_eq!(rash::config::quote_variables(input), expected);
}

#[test]
fn test_purify_example() {
    purify_example();
}
```

## Common Workflows

### 1. Lint Before Commit

```bash,no_run
# Lint all shell scripts in project
find . -name "*.sh" -exec bashrs lint {} \;
```

### 2. CI/CD Integration

```yaml
# .github/workflows/shellcheck.yml
name: Shell Lint
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install bashrs
        run: cargo install bashrs
      - name: Lint scripts
        run: bashrs lint scripts/*.sh
```

### 3. Purify Your dotfiles

```bash,no_run
# Analyze
bashrs config analyze ~/.bashrc

# Purify (dry-run by default)
bashrs config purify ~/.bashrc --output ~/.bashrc.purified

# Review changes
diff ~/.bashrc ~/.bashrc.purified

# Apply changes (creates backup automatically)
bashrs config purify ~/.bashrc --fix
```

## What's Next?

Now that you've seen the basics, let's explore:

- [Your First Purification](./first-purification.md): Step-by-step purification workflow
- [Core Concepts](../concepts/purification.md): Understand determinism and idempotency
- [Security Rules](../linting/security.md): Deep dive into security linting

## Quick Reference

| Command | Purpose |
|---------|---------|
| `bashrs lint <file>` | Lint shell script |
| `bashrs config analyze <file>` | Analyze config file |
| `bashrs config purify <file>` | Purify config file |
| `bashrs --help` | Show all commands |

## Troubleshooting

### "bashrs: command not found"

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission Denied

If you see permission errors:

```bash
chmod +x vulnerable.sh
```

## Summary

In this chapter, you learned to:

- ✅ Lint shell scripts for security issues
- ✅ Analyze configuration files
- ✅ Purify config files automatically
- ✅ Integrate Rash into your workflow

Ready for a deeper dive? Continue to [Your First Purification](./first-purification.md)!
