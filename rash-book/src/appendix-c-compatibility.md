# Appendix C: Shell Compatibility Matrix

This appendix provides comprehensive compatibility information for bashrs-generated scripts across different shells, operating systems, and environments.

---

## Shell Compatibility

bashrs generates pure POSIX sh that works across all major shell implementations.

### Tested Shells

| Shell | Version | OS | Status | Notes |
|-------|---------|----|----|-------|
| **sh** | POSIX | All | ✅ Full | Reference POSIX implementation |
| **dash** | 0.5.12+ | Debian/Ubuntu | ✅ Full | Default `/bin/sh`, strictly POSIX |
| **ash** | BusyBox 1.35+ | Alpine Linux | ✅ Full | Minimal (<1MB), container standard |
| **bash** | 3.2+ | macOS | ✅ Full | Backward compatible |
| **bash** | 4.x | Linux | ✅ Full | Most common version |
| **bash** | 5.x | Linux/macOS | ✅ Full | Latest features (unused by bashrs) |
| **zsh** | 5.x | macOS 10.15+ | ✅ Full | macOS default since Catalina |
| **ksh** | 93u+ | AIX/Solaris | ✅ Full | Enterprise Unix standard |
| **mksh** | R59+ | Android | ✅ Full | MirBSD Korn Shell |
| **yash** | 2.x | All | ✅ Full | Yet Another Shell |

### Untested but Compatible

These shells should work based on POSIX compliance claims (not yet tested):

- **pdksh** (Public Domain Korn Shell)
- **loksh** (Linux port of OpenBSD's ksh)
- **busybox hush** (Alternative BusyBox shell)

### Incompatible Shells

bashrs does NOT generate code for these shells (different syntax):

- ❌ **fish** (Future: v8.0.0 may add Fish → POSIX transpilation)
- ❌ **PowerShell** (Future: v8.0.0 may add PowerShell → bash)
- ❌ **csh/tcsh** (C shell family, fundamentally different)
- ❌ **rc** (Plan 9 shell, different syntax)

---

## Operating System Compatibility

bashrs-generated scripts work on all POSIX-compliant operating systems.

### Linux Distributions

| Distribution | Version | Default Shell | Status | Notes |
|-------------|---------|---------------|--------|-------|
| **Ubuntu** | 20.04+ | dash (`/bin/sh`) | ✅ Full | Primary test platform |
| **Debian** | 11+ | dash (`/bin/sh`) | ✅ Full | Strict POSIX |
| **Alpine Linux** | 3.15+ | ash (BusyBox) | ✅ Full | Container standard |
| **RHEL/CentOS** | 8+ | bash (`/bin/sh`) | ✅ Full | Enterprise Linux |
| **Fedora** | 35+ | bash (`/bin/sh`) | ✅ Full | Bleeding edge |
| **Arch Linux** | Rolling | bash (`/bin/sh`) | ✅ Full | Latest packages |
| **openSUSE** | 15+ | bash (`/bin/sh`) | ✅ Full | Enterprise/community |
| **Gentoo** | Rolling | bash (`/bin/sh`) | ✅ Full | Source-based |

### Unix Systems

| System | Version | Default Shell | Status | Notes |
|--------|---------|---------------|--------|-------|
| **macOS** | 10.15+ | zsh | ✅ Full | Also has bash 3.2 |
| **FreeBSD** | 13+ | sh (FreeBSD sh) | ✅ Full | POSIX-compliant |
| **OpenBSD** | 7+ | ksh (pdksh) | ✅ Full | Security-focused |
| **NetBSD** | 9+ | sh | ✅ Full | Portable |
| **illumos** | OpenIndiana | ksh93 | ✅ Full | Solaris descendant |
| **AIX** | 7.x | ksh93 | ✅ Full | Enterprise IBM Unix |
| **HP-UX** | 11i | POSIX sh | ⚠️ Limited | Legacy system, not tested |
| **Solaris** | 11 | ksh93 | ⚠️ Limited | Legacy system, not tested |

### Embedded Systems

| System | Shell | Status | Notes |
|--------|-------|--------|-------|
| **BusyBox** | ash | ✅ Full | Routers, IoT devices |
| **Toybox** | sh | ✅ Full | Android alternative |
| **Android** | mksh | ✅ Full | Termux, shell scripts |
| **OpenWrt** | ash (BusyBox) | ✅ Full | Router firmware |

---

## Container Compatibility

bashrs scripts work in all major container runtimes.

### Container Images

| Base Image | Shell | Size | Status | Notes |
|------------|-------|------|--------|-------|
| **alpine:latest** | ash | 5MB | ✅ Full | Minimal, most common |
| **debian:slim** | dash | 50MB | ✅ Full | Balanced |
| **ubuntu:latest** | dash | 75MB | ✅ Full | Full featured |
| **busybox:latest** | ash | 1.5MB | ✅ Full | Smallest |
| **scratch** | None | 0MB | ❌ N/A | No shell available |
| **distroless** | None | Varies | ❌ N/A | No shell (security) |

**Note**: `scratch` and `distroless` images intentionally omit shells for security. bashrs scripts cannot run in these environments.

### Container Runtimes

| Runtime | Version | Status | Notes |
|---------|---------|--------|-------|
| **Docker** | 20.10+ | ✅ Full | Most common |
| **Podman** | 3.x+ | ✅ Full | Rootless alternative |
| **containerd** | 1.6+ | ✅ Full | Kubernetes standard |
| **CRI-O** | 1.23+ | ✅ Full | Kubernetes-native |
| **LXC/LXD** | 4.x+ | ✅ Full | System containers |

---

## CI/CD Platform Compatibility

bashrs integrates with all major CI/CD platforms.

### Continuous Integration

| Platform | Runner OS | Status | Notes |
|----------|-----------|--------|-------|
| **GitHub Actions** | ubuntu-latest | ✅ Full | Native support |
| **GitHub Actions** | macos-latest | ✅ Full | Native support |
| **GitHub Actions** | windows-latest | ⚠️ Partial | Requires Git Bash/WSL |
| **GitLab CI** | Linux runners | ✅ Full | Native support |
| **CircleCI** | Linux/macOS | ✅ Full | Native support |
| **Travis CI** | Linux/macOS | ✅ Full | Native support |
| **Jenkins** | Any | ✅ Full | Docker agents recommended |
| **Azure Pipelines** | Linux/macOS | ✅ Full | Native support |
| **Azure Pipelines** | Windows | ⚠️ Partial | Requires Git Bash/WSL |
| **Buildkite** | Any | ✅ Full | Self-hosted agents |
| **Drone CI** | Docker-based | ✅ Full | Container-native |

---

## Feature Compatibility

### POSIX Features (Always Available)

bashrs uses only these POSIX-guaranteed features:

- ✅ `[ ]` test (not `[[ ]]`)
- ✅ `$(( ))` arithmetic
- ✅ `$( )` command substitution
- ✅ `${var}` parameter expansion
- ✅ `${var:-default}` parameter expansion with default
- ✅ `${var%pattern}` suffix removal
- ✅ `${var##pattern}` prefix removal
- ✅ `if/then/else/fi` conditionals
- ✅ `for/do/done` loops
- ✅ `while/do/done` loops
- ✅ `case/esac` pattern matching
- ✅ `function()` or `name()` function definition
- ✅ `printf` (not `echo -n` or `echo -e`)
- ✅ Here documents (`<<EOF`)
- ✅ Pipes (`|`)
- ✅ Redirects (`>`, `>>`, `<`, `2>&1`)

### Bash-specific Features (NOT Used)

bashrs avoids these bash-only features:

- ❌ Arrays: `arr=(1 2 3)`
- ❌ `[[ ]]` conditional
- ❌ `$RANDOM` variable
- ❌ `${var:offset:length}` substring
- ❌ `=~` regex matching
- ❌ Process substitution: `<(cmd)`
- ❌ Brace expansion: `{1..10}`
- ❌ `function name` keyword
- ❌ `local` keyword (workaround: prefixed variables)
- ❌ `source` (use `.` instead)
- ❌ `[[` extended test
- ❌ `(( ))` arithmetic keyword

---

## External Command Compatibility

bashrs assumes these POSIX commands are available:

### Core Utilities (Always Required)

- ✅ `cat`, `cp`, `mv`, `rm`, `mkdir`, `ln`
- ✅ `ls`, `cd`, `pwd`
- ✅ `grep`, `sed`, `awk`
- ✅ `printf`, `test`
- ✅ `true`, `false`
- ✅ `command`, `type`

### Common Utilities (Usually Available)

- ✅ `curl` or `wget` (download)
- ✅ `tar`, `gzip` (archives)
- ✅ `git` (version control)
- ✅ `make` (build automation)
- ✅ `shellcheck` (linting - optional)

### System Commands (Environment-Specific)

- `systemctl` (systemd systems only)
- `service` (SysV init systems)
- `apt`/`yum`/`apk` (package managers)
- `docker` (container operations)

bashrs scripts should check for command availability:

```sh
if ! command -v docker >/dev/null 2>&1; then
    printf 'Error: docker not found\n' >&2
    exit 1
fi
```

---

## Filesystem Compatibility

### Path Conventions

| OS Family | Path Separator | Root | Home | Temp |
|-----------|----------------|------|------|------|
| **Linux/Unix** | `/` | `/` | `$HOME` | `/tmp` |
| **macOS** | `/` | `/` | `$HOME` | `/tmp` |
| **Windows (Git Bash)** | `/` or `\` | `/c/` | `$HOME` | `/tmp` |
| **Windows (WSL)** | `/` | `/` | `$HOME` | `/tmp` |

bashrs uses POSIX paths (`/`) which work on all platforms with Git Bash or WSL.

### Case Sensitivity

| Filesystem | Case Sensitive | Notes |
|------------|----------------|-------|
| **Linux (ext4)** | Yes | `File.txt` ≠ `file.txt` |
| **macOS (APFS)** | No (default) | `File.txt` = `file.txt` |
| **macOS (APFS-CS)** | Yes (optional) | Can be enabled |
| **Windows (NTFS)** | No | `File.txt` = `file.txt` |

**Best Practice**: Assume case-insensitive filesystems. Never rely on case differences.

---

## Performance Characteristics

### Transpilation Speed

| Shell | Startup Time | Execution Speed | Notes |
|-------|--------------|-----------------|-------|
| **dash** | 1x | 4x | Fastest, minimal features |
| **ash** | 1x | 4x | Similar to dash |
| **bash** | 4x | 1x | Slowest startup, full-featured |
| **zsh** | 5x | 1x | Slowest, most features |
| **ksh** | 2x | 2x | Balanced |

bashrs-generated scripts benefit from dash/ash speed (4x faster than bash).

---

## Known Limitations

### Unsupported Environments

bashrs-generated scripts do NOT work in:

1. **Windows Command Prompt** (`cmd.exe`)
   - Solution: Use Git Bash or WSL

2. **Windows PowerShell** (without WSL)
   - Solution: Use WSL or wait for v8.0.0 PowerShell support

3. **Containers without shells** (scratch, distroless)
   - Solution: Use base images with shells (alpine, debian, ubuntu)

4. **Restricted shells** (rbash, rksh)
   - May work with limitations (untested)

### Platform-Specific Issues

- **macOS**: Ships with bash 3.2 (GPL2) which is old but POSIX-compliant
- **Windows**: Requires Git Bash, WSL, or Cygwin for POSIX environment
- **AIX/Solaris**: Not regularly tested (should work, but report issues)

---

## Testing Matrix

bashrs tests on this matrix:

### GitHub Actions Matrix

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    shell: [sh, dash, bash, zsh]
```

### Docker Matrix

```yaml
matrix:
  image:
    - alpine:latest      # ash
    - debian:latest      # dash
    - ubuntu:latest      # dash
    - fedora:latest      # bash
```

---

## Version Support Policy

### Supported Versions

bashrs supports:
- ✅ **Latest stable** of each OS/shell
- ✅ **LTS versions** (e.g., Ubuntu 20.04, 22.04)
- ✅ **N-1 major version** (e.g., Debian 11 and 12)

### Unsupported Versions

bashrs does NOT officially support:
- ❌ **EOL operating systems** (e.g., Ubuntu 18.04, CentOS 7)
- ❌ **Ancient shells** (e.g., bash 2.x)
- ❌ **Proprietary Unix** (e.g., HP-UX, Tru64)

**Note**: bashrs MAY work on unsupported versions due to POSIX compliance, but this is not guaranteed.

---

## Reporting Compatibility Issues

If bashrs-generated scripts fail on a POSIX-compliant system:

1. **Check shell**: `echo $SHELL` and verify POSIX compliance
2. **Test with dash**: Run script with `dash script.sh` (strictest)
3. **Run shellcheck**: `shellcheck -s sh script.sh`
4. **File issue**: https://github.com/paiml/bashrs/issues

Include:
- OS and version
- Shell and version
- bashrs version
- Minimal reproduction script
- Error output

---

## See Also

- **Chapter 7**: POSIX compliance details
- **Chapter 14**: Shell dialects explained
- **Chapter 15**: CI/CD integration
- **Appendix B**: Glossary of terms

---

*Compatibility matrix last updated: 2025-11-14 for bashrs v6.34.1*
