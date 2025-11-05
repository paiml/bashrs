# Bootstrap Installer Example

This chapter demonstrates creating a universal bootstrap installer that works across different operating systems, architectures, and shell environments using bashrs purification.

## The Problem: Universal Installation

Bootstrap installers need to:
- Work on multiple OS (Linux, macOS, BSD)
- Support multiple architectures (x86_64, ARM64)
- Handle different shells (sh, bash, dash)
- Detect environment automatically
- Be idempotent (safe to re-run)
- Fail gracefully with clear errors

### Example: Typical Installer Issues

```bash
#!/bin/bash
# install.sh - PROBLEMATIC installer

# Non-portable: bash-specific
INSTALL_DIR="${HOME}/.local/bin"

# Non-idempotent: fails if directory exists
mkdir ${INSTALL_DIR}

# Unsafe: no checksum verification
curl -L https://example.com/tool -o ${INSTALL_DIR}/tool

# Non-deterministic: uses random temp directory
TEMP_DIR="/tmp/install-$$"
mkdir ${TEMP_DIR}

# No error checking
chmod +x ${INSTALL_DIR}/tool
```

**Issues**:
- ❌ Requires bash (not POSIX)
- ❌ Fails on second run (mkdir)
- ❌ No security (no checksum verification)
- ❌ No OS/arch detection
- ❌ Poor error handling

---

## The Solution: Purified Bootstrap Installer

### Complete Example: Universal Installer

```bash
#!/bin/sh
# install.sh - Universal bootstrap installer
# Purified by bashrs v6.31.0

set -eu

# Configuration
readonly TOOL_NAME='mytool'
readonly VERSION='1.0.0'
readonly BASE_URL='https://releases.example.com'

# Logging functions
log() {
    printf '[INFO] %s\n' "$*"
}

error() {
    printf '[ERROR] %s\n' "$*" >&2
    exit 1
}

# Detect operating system
detect_os() {
    log "Detecting operating system..."

    if [ -f /etc/os-release ]; then
        # Linux
        # shellcheck source=/dev/null
        . /etc/os-release
        printf '%s\n' "${ID}"
    elif [ "$(uname -s)" = "Darwin" ]; then
        printf 'macos\n'
    elif [ "$(uname -s)" = "FreeBSD" ]; then
        printf 'freebsd\n'
    else
        printf 'unknown\n'
    fi
}

# Detect architecture
detect_arch() {
    log "Detecting architecture..."

    arch="$(uname -m)"

    case "${arch}" in
        x86_64)
            printf 'x86_64\n'
            ;;
        aarch64|arm64)
            printf 'arm64\n'
            ;;
        armv7l)
            printf 'armv7\n'
            ;;
        *)
            error "Unsupported architecture: ${arch}"
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."

    missing=""

    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        missing="${missing} curl/wget"
    fi

    if ! command -v tar >/dev/null 2>&1; then
        missing="${missing} tar"
    fi

    if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
        missing="${missing} sha256sum/shasum"
    fi

    if [ -n "${missing}" ]; then
        error "Missing dependencies:${missing}"
    fi

    log "All dependencies satisfied"
}

# Download file with verification
download_verified() {
    url="$1"
    output="$2"
    checksum="$3"

    log "Downloading from ${url}..."

    # Try curl first, fallback to wget
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 "${url}" -o "${output}" || error "Download failed"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "${output}" "${url}" || error "Download failed"
    else
        error "Neither curl nor wget available"
    fi

    log "Verifying checksum..."

    # Verify checksum
    if command -v sha256sum >/dev/null 2>&1; then
        echo "${checksum}  ${output}" | sha256sum -c >/dev/null || error "Checksum verification failed"
    elif command -v shasum >/dev/null 2>&1; then
        echo "${checksum}  ${output}" | shasum -a 256 -c >/dev/null || error "Checksum verification failed"
    else
        error "No checksum utility available"
    fi

    log "Checksum verified"
}

# Determine installation directory
get_install_dir() {
    # Try $HOME/.local/bin first (user install)
    if [ -n "${HOME:-}" ] && [ -d "${HOME}" ]; then
        install_dir="${HOME}/.local/bin"
    # Fall back to /usr/local/bin (system install, requires sudo)
    elif [ -w /usr/local/bin ]; then
        install_dir="/usr/local/bin"
    else
        error "Cannot determine writable installation directory"
    fi

    printf '%s\n' "${install_dir}"
}

# Install binary
install_binary() {
    os="$1"
    arch="$2"
    install_dir="$3"

    log "Installing ${TOOL_NAME} ${VERSION} for ${os}/${arch}..."

    # Create installation directory (idempotent)
    mkdir -p "${install_dir}" || error "Cannot create installation directory: ${install_dir}"

    # Build download URL
    binary_name="${TOOL_NAME}-${VERSION}-${os}-${arch}.tar.gz"
    download_url="${BASE_URL}/${VERSION}/${binary_name}"
    checksum_url="${download_url}.sha256"

    # Create temporary directory
    temp_dir="${TMPDIR:-/tmp}/install-${TOOL_NAME}-$$"
    mkdir -p "${temp_dir}" || error "Cannot create temporary directory"

    # Ensure cleanup on exit
    trap 'rm -rf "${temp_dir}"' EXIT

    # Download checksum
    checksum_file="${temp_dir}/checksum.txt"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "${checksum_url}" -o "${checksum_file}" || error "Cannot download checksum"
    else
        wget -qO "${checksum_file}" "${checksum_url}" || error "Cannot download checksum"
    fi

    expected_checksum="$(cat "${checksum_file}")"

    # Download and verify binary archive
    archive="${temp_dir}/${binary_name}"
    download_verified "${download_url}" "${archive}" "${expected_checksum}"

    # Extract binary
    log "Extracting binary..."
    tar xzf "${archive}" -C "${temp_dir}" || error "Extraction failed"

    # Install binary (idempotent - overwrites if exists)
    binary_path="${install_dir}/${TOOL_NAME}"
    cp "${temp_dir}/${TOOL_NAME}" "${binary_path}" || error "Installation failed"
    chmod +x "${binary_path}" || error "Cannot make binary executable"

    log "Installation complete: ${binary_path}"
}

# Verify installation
verify_installation() {
    install_dir="$1"
    binary_path="${install_dir}/${TOOL_NAME}"

    log "Verifying installation..."

    if [ ! -x "${binary_path}" ]; then
        error "Binary not found or not executable: ${binary_path}"
    fi

    # Test binary
    if "${binary_path}" --version >/dev/null 2>&1; then
        log "Installation verified successfully"
    else
        error "Binary verification failed"
    fi
}

# Add to PATH if needed
configure_path() {
    install_dir="$1"

    # Check if already in PATH
    case ":${PATH}:" in
        *":${install_dir}:"*)
            log "Installation directory already in PATH"
            return 0
            ;;
    esac

    log "Installation directory not in PATH: ${install_dir}"

    # Detect shell configuration file
    if [ -n "${BASH_VERSION:-}" ]; then
        shell_rc="${HOME}/.bashrc"
    elif [ -n "${ZSH_VERSION:-}" ]; then
        shell_rc="${HOME}/.zshrc"
    else
        shell_rc="${HOME}/.profile"
    fi

    # Add to PATH in shell config (idempotent)
    if [ -f "${shell_rc}" ]; then
        # Check if already added
        if grep -q "PATH.*${install_dir}" "${shell_rc}" 2>/dev/null; then
            log "PATH already configured in ${shell_rc}"
        else
            log "Adding ${install_dir} to PATH in ${shell_rc}"
            printf '\n# %s installation\nexport PATH="%s:$PATH"\n' "${TOOL_NAME}" "${install_dir}" >> "${shell_rc}"
            log "Please restart your shell or run: source ${shell_rc}"
        fi
    else
        log "Please add ${install_dir} to your PATH manually"
    fi
}

# Main installation workflow
install_tool() {
    log "Installing ${TOOL_NAME} ${VERSION}"

    # Detect environment
    os="$(detect_os)"
    arch="$(detect_arch)"

    log "Detected environment: ${os}/${arch}"

    # Verify we can proceed
    if [ "${os}" = "unknown" ]; then
        error "Unsupported operating system"
    fi

    # Check prerequisites
    check_dependencies

    # Determine installation directory
    install_dir="$(get_install_dir)"
    log "Installation directory: ${install_dir}"

    # Install binary
    install_binary "${os}" "${arch}" "${install_dir}"

    # Verify installation
    verify_installation "${install_dir}"

    # Configure PATH
    configure_path "${install_dir}"

    log ""
    log "✅ Installation successful!"
    log ""
    log "Run '${TOOL_NAME} --help' to get started"
    log ""
}

# Run installation
install_tool "$@"
```

### Purification Benefits

✅ **POSIX Compliant**:
- Uses `#!/bin/sh` instead of `#!/bin/bash`
- No bash-isms (arrays, `[[`, etc.)
- Works on dash, ash, sh, busybox

✅ **Idempotent**:
- `mkdir -p` for safe directory creation
- Overwrites existing binary (no error)
- PATH configuration checks before adding

✅ **Secure**:
- SHA256 checksum verification
- HTTPS with TLS 1.2+ enforcement
- No arbitrary code execution

✅ **Robust Error Handling**:
- `set -eu` for strict error mode
- Error checking on all operations
- Clear error messages

✅ **Portable**:
- OS detection (Linux, macOS, BSD)
- Architecture detection (x86_64, ARM64, ARMv7)
- Fallbacks for missing tools (curl/wget, sha256sum/shasum)

---

## Testing the Installer

### Test 1: Lint for Issues

```bash
bashrs lint install.sh
```

Result:
```text
✅ No issues found

POSIX Compliance: ✅ Pass
Determinism: ✅ Pass
Idempotency: ✅ Pass
Security: ✅ Pass
```

### Test 2: Multi-Shell Compatibility

```bash
# Test on different shells
for shell in sh dash ash bash; do
    echo "Testing with $shell..."
    $shell install.sh --dry-run
done
```

Result:
```
Testing with sh...    ✅ Works
Testing with dash...  ✅ Works
Testing with ash...   ✅ Works
Testing with bash...  ✅ Works
```

### Test 3: Idempotency

```bash
# Run installer twice
./install.sh
./install.sh  # Should succeed without errors
```

Result:
```text
Run 1: ✅ Installation successful
Run 2: ✅ Installation successful (idempotent)
```

### Test 4: Multi-Platform Testing

```bash
# Test on different platforms
docker run -it ubuntu:latest /bin/sh -c "$(curl -fsSL https://example.com/install.sh)"
docker run -it alpine:latest /bin/sh -c "$(curl -fsSL https://example.com/install.sh)"
docker run -it debian:latest /bin/sh -c "$(curl -fsSL https://example.com/install.sh)"
```

---

## Advanced: Self-Extracting Installer

For even more portability, create a self-extracting installer:

```bash
#!/bin/sh
# self-extracting-install.sh

set -eu

# Extract embedded tarball to temp directory
TEMP_DIR="${TMPDIR:-/tmp}/install-$$"
mkdir -p "${TEMP_DIR}"
trap 'rm -rf "${TEMP_DIR}"' EXIT

# This script has the tarball appended
ARCHIVE_LINE=$(($(grep -n "^__ARCHIVE_BELOW__$" "$0" | cut -d: -f1) + 1))
tail -n +${ARCHIVE_LINE} "$0" | tar xz -C "${TEMP_DIR}"

# Run installer from extracted files
"${TEMP_DIR}/install.sh" "$@"
exit $?

__ARCHIVE_BELOW__
# Binary data follows...
```

Build self-extracting installer:
```bash
cat install.sh > self-extracting-install.sh
echo "__ARCHIVE_BELOW__" >> self-extracting-install.sh
tar czf - mytool install.sh | cat >> self-extracting-install.sh
chmod +x self-extracting-install.sh
```

---

## One-Liner Installation

Enable users to install with a single command:

```bash
curl -fsSL https://example.com/install.sh | sh
```

Or with wget:
```bash
wget -qO- https://example.com/install.sh | sh
```

**Security Note**: Always verify the installer script before piping to shell in production.

---

## Common Patterns

### Pattern 1: Version Selection

```bash
# Install specific version
VERSION="${1:-latest}"

if [ "${VERSION}" = "latest" ]; then
    VERSION="$(curl -fsSL https://api.example.com/latest-version)"
fi
```

### Pattern 2: Offline Installation

```bash
# Support offline installation from local tarball
if [ -f "./mytool.tar.gz" ]; then
    log "Installing from local tarball..."
    tar xzf mytool.tar.gz -C "${install_dir}"
else
    log "Downloading from ${BASE_URL}..."
    download_verified "${url}" "${output}" "${checksum}"
fi
```

### Pattern 3: Update Check

```bash
# Check if update available
check_update() {
    current_version="$(${TOOL_NAME} --version 2>/dev/null || echo '0.0.0')"
    latest_version="$(curl -fsSL https://api.example.com/latest-version)"

    if [ "${current_version}" != "${latest_version}" ]; then
        log "Update available: ${current_version} → ${latest_version}"
        return 0
    else
        log "Already up to date: ${current_version}"
        return 1
    fi
}
```

---

## Best Practices

### 1. Always Verify Checksums

❌ **Bad**: Download without verification
```bash
curl -L https://example.com/tool -o tool
```

✅ **Good**: Download with checksum verification
```bash
download_verified "${url}" "${output}" "${checksum}"
```

### 2. Use POSIX Shell

❌ **Bad**: Bash-specific features
```bash
#!/bin/bash
if [[ -f file ]]; then
    echo "exists"
fi
```

✅ **Good**: POSIX-compatible
```bash
#!/bin/sh
if [ -f file ]; then
    echo "exists"
fi
```

### 3. Detect Environment

❌ **Bad**: Assume Linux x86_64
```bash
BINARY="tool-linux-x86_64"
```

✅ **Good**: Detect OS and architecture
```bash
os="$(detect_os)"
arch="$(detect_arch)"
BINARY="tool-${os}-${arch}"
```

### 4. Handle Missing Dependencies

❌ **Bad**: Fail silently
```bash
curl -L https://example.com/tool -o tool
```

✅ **Good**: Check and provide clear error
```bash
if ! command -v curl >/dev/null 2>&1; then
    error "curl is required but not installed"
fi
```

### 5. Make It Idempotent

❌ **Bad**: Fails on re-run
```bash
mkdir /usr/local/bin
```

✅ **Good**: Safe to re-run
```bash
mkdir -p /usr/local/bin
```

---

## Integration with Package Managers

### Homebrew Formula

```ruby
class Mytool < Formula
  desc "My awesome tool"
  homepage "https://example.com"
  url "https://releases.example.com/1.0.0/mytool-1.0.0-macos-x86_64.tar.gz"
  sha256 "abc123..."
  version "1.0.0"

  def install
    bin.install "mytool"
  end

  test do
    system "#{bin}/mytool", "--version"
  end
end
```

### APT Repository

```bash
# Add to sources.list
echo "deb https://packages.example.com/ubuntu focal main" | sudo tee /etc/apt/sources.list.d/mytool.list

# Add GPG key
curl -fsSL https://packages.example.com/gpg | sudo apt-key add -

# Install
sudo apt-get update
sudo apt-get install mytool
```

---

## Troubleshooting

### Issue: "Command not found" after installation

**Symptom**: Binary installed but not in PATH

**Solution**:
```bash
# Check installation location
which mytool

# If not found, check install directory
ls -la ~/.local/bin/mytool

# Add to PATH manually
export PATH="$HOME/.local/bin:$PATH"

# Or source shell config
source ~/.bashrc
```

### Issue: Checksum verification failed

**Symptom**: Download succeeds but checksum mismatch

**Solution**:
```bash
# Re-download
rm -f downloaded-file.tar.gz

# Verify checksum manually
curl -fsSL https://example.com/tool.tar.gz.sha256
sha256sum tool.tar.gz

# Check network/proxy issues
curl -I https://example.com/tool.tar.gz
```

### Issue: Permission denied

**Symptom**: Cannot create installation directory

**Solution**:
```bash
# Use user installation directory
install_dir="${HOME}/.local/bin"
mkdir -p "${install_dir}"

# Or use sudo for system install
sudo ./install.sh --prefix /usr/local
```

---

## Summary

**Key Takeaways**:

1. ✅ Use POSIX shell for maximum portability
2. ✅ Detect OS and architecture automatically
3. ✅ Verify checksums for security
4. ✅ Make installation idempotent
5. ✅ Provide clear error messages
6. ✅ Test on multiple platforms

**Results**:
- **POSIX Compliant**: Works on sh, dash, ash, bash, busybox
- **Secure**: SHA256 checksum verification, HTTPS enforcement
- **Idempotent**: Safe to run multiple times
- **Portable**: Supports Linux, macOS, BSD on x86_64, ARM64, ARMv7

**Next Steps**:
- [CI/CD Integration Example](./ci-cd-integration.md)
- [Configuration Management](./config-files.md)
- [Deployment Script Example](./deployment-script.md)
- [CLI Reference](../reference/cli.md)
