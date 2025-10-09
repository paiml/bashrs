---
name: Bug Report
about: Report a bug with Five Whys analysis
title: '[BUG] '
labels: bug, needs-triage
assignees: ''
---

# Bug Report - Five Whys Analysis Required

## Problem Statement

**Severity**: [P0 Critical / P1 High / P2 Medium / P3 Low]

### Description
[Clear, concise description of the bug]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Impact
- **Users affected**: [All / Some / Few]
- **Workaround available**: [Yes/No]
- **Data loss risk**: [Yes/No]

## Reproduction Steps

### Minimal Reproducible Example

```rust
// Rust code that demonstrates the bug
fn main() {
    // ...
}
```

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [Third step]

### Frequency
- [ ] Always reproducible
- [ ] Sometimes reproducible (X% of the time)
- [ ] Rare/one-time occurrence

## Environment

### System Information
- **OS**: [Linux/macOS/Windows]
- **OS Version**: [e.g., Ubuntu 22.04, macOS 13.0]
- **Shell**: [sh/bash/dash/ash/busybox]
- **Shell Version**: [e.g., bash 5.1.16]

### Bashrs Information
- **Version**: [e.g., v0.9.2]
- **Installation**: [cargo/binary/source]
- **Rust Version**: [e.g., 1.75.0]

## Output and Logs

### Error Message
```
[Paste complete error message]
```

### Generated Shell Code (if applicable)
```bash
# Problematic generated code
```

### Stack Trace (if panic)
```
[Paste full stack trace]
```

## Analysis (Optional but helpful)

### Five Whys Preliminary Analysis

If you've investigated the issue, provide your Five Whys:

**Why #1**: Why did this happen?
[Your analysis]

**Why #2**: Why did [answer from #1]?
[Your analysis]

**Why #3**: Why did [answer from #2]?
[Your analysis]

Continue if you can...

### Suspected Root Cause
[Your hypothesis about the root cause]

### Suggested Fix
[If you have ideas about how to fix it]

## Testing

### Tests Added (if you're submitting PR)
- [ ] Unit test reproducing the bug
- [ ] Property test covering the scenario
- [ ] Integration test verifying the fix

### Test Code
```rust
#[test]
fn test_bug_reproduction() {
    // Test that currently fails
}
```

## Additional Context

### Related Issues
- Related to #XXX
- Similar to #YYY

### Screenshots
[If applicable, add screenshots]

### Examples
[Link to examples or repositories demonstrating the bug]

## Security Implications

- [ ] This is a security vulnerability
- [ ] Potential for shell injection
- [ ] Data integrity issue
- [ ] None identified

**If security issue**: Please report privately to [security contact] instead of public issue.

## Checklist

- [ ] I have searched existing issues to avoid duplicates
- [ ] I have provided a minimal reproducible example
- [ ] I have included version and environment information
- [ ] I have described expected vs actual behavior
- [ ] I have attempted Five Whys analysis (optional)

---

## For Maintainers

### Triage
- [ ] Severity confirmed
- [ ] Priority assigned
- [ ] Sprint assigned
- [ ] Ticket created (RASH-XXXX)

### Investigation
- [ ] Bug reproduced locally
- [ ] Five Whys analysis completed
- [ ] Root cause identified
- [ ] Fix strategy documented

### Fix
- [ ] Tests written (RED)
- [ ] Fix implemented (GREEN)
- [ ] Code refactored (REFACTOR)
- [ ] Quality gates passed
- [ ] PR submitted

### Documentation
- [ ] Five Whys analysis documented
- [ ] CHANGELOG.md updated
- [ ] ROADMAP.md updated (if needed)
- [ ] Lessons learned captured
