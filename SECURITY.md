# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 6.x     | Yes                |
| < 6.0   | No                 |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public issue
2. Email security concerns to the maintainers
3. Include steps to reproduce the vulnerability
4. Allow reasonable time for a fix before disclosure

## Security Practices

- All dependencies are audited weekly via `cargo audit`
- License compliance checked via `cargo deny`
- No unsafe code (`#![forbid(unsafe_code)]` enforced via workspace lints)
- `unwrap()` banned in production code via clippy configuration
- Input validation on all shell script parsing paths
- Fuzzing via `cargo fuzz` for parser hardening
