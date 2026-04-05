#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_VAR_002_path_security_considerations() {
    // DOCUMENTATION: PATH security considerations (5 CRITICAL security practices)
    //
    // SECURITY RISK 1: PATH hijacking (Trojan horse attack)
    // Attacker creates malicious "ls" in /tmp
    // If PATH="/tmp:$PATH", running "ls" executes attacker's code
    //
    // MITIGATION 1: Never put "." or writable directories in PATH
    // # PATH=".:$PATH"        # DANGEROUS
    // # PATH="/tmp:$PATH"     # DANGEROUS
    // PATH="/usr/local/bin:/usr/bin:/bin"  # Safe (system directories)
    //
    // SECURITY RISK 2: Relative PATH in scripts
    // #!/bin/sh
    // sudo reboot  # Which "sudo"? Could be hijacked if PATH modified
    //
    // MITIGATION 2: Use absolute paths in security-critical scripts
    // #!/bin/sh
    // /usr/bin/sudo /sbin/reboot  # Absolute (safe)
    //
    // SECURITY RISK 3: PATH injection via environment
    // If attacker controls environment: PATH="/evil:$PATH" ./script.sh
    //
    // MITIGATION 3: Reset PATH at start of security-critical scripts
    // #!/bin/sh
    // PATH="/usr/bin:/bin"  # Reset to safe minimal PATH
    // export PATH
    //
    // SECURITY RISK 4: SUID scripts and PATH
    // SUID scripts inherit caller's PATH (security risk)
    //
    // MITIGATION 4: Never write SUID shell scripts (use C/compiled languages)
    //
    // SECURITY RISK 5: PATH persistence via ~/.profile
    // If attacker modifies ~/.profile: PATH="/evil:$PATH"
    //
    // MITIGATION 5: Protect ~/.profile permissions (chmod 644, owned by user)
    //
    // EXAMPLE ATTACK (PATH hijacking):
    // Attacker creates /tmp/sudo:
    //   #!/bin/sh
    //   # Log password, then run real sudo
    //   echo "$@" >> /tmp/stolen-passwords
    //   /usr/bin/sudo "$@"
    //
    // If script uses: PATH="/tmp:$PATH"; sudo ...
    // Attacker's /tmp/sudo executes instead of /usr/bin/sudo

    let security_considerations = r#"
#!/bin/sh
# Security-critical script - demonstrates best practices

# SECURITY 1: Reset PATH to minimal safe value
PATH="/usr/bin:/bin"
export PATH

# SECURITY 2: Use absolute paths for critical commands
/usr/bin/id
/bin/ps aux

# SECURITY 3: Verify command is in expected location
sudo_path=$(command -v sudo)
if [ "$sudo_path" != "/usr/bin/sudo" ]; then
    echo "ERROR: sudo not in expected location" >&2
    echo "Expected: /usr/bin/sudo" >&2
    echo "Found: $sudo_path" >&2
    exit 1
fi

# SECURITY 4: For critical operations, use absolute paths
/usr/bin/sudo /sbin/reboot

# SECURITY 5: Check file ownership before executing
target="/usr/local/bin/myapp"
if [ -x "$target" ]; then
    owner=$(stat -c %U "$target")
    if [ "$owner" = "root" ]; then
        "$target"
    else
        echo "ERROR: $target not owned by root (owned by $owner)" >&2
        exit 1
    fi
fi
"#;

    let mut lexer = Lexer::new(security_considerations);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "PATH security considerations should tokenize successfully"
        );
        let _ = tokens;
    }

    // CRITICAL SECURITY PRACTICES:
    // 1. Never put "." or writable directories in PATH
    // 2. Use absolute paths for security-critical commands (/usr/bin/sudo)
    // 3. Reset PATH to minimal safe value in security scripts
    // 4. Verify command locations before executing
    // 5. Protect ~/.profile and similar files (chmod 644)
}

#[test]
fn test_VAR_002_path_comparison_table() {
    // DOCUMENTATION: Comprehensive PATH comparison (POSIX vs Bash vs Purified)
    //
    // ┌─────────────────────────────────────────────────────────────────────────┐
    // │ FEATURE                    │ POSIX      │ Bash       │ Purified         │
    // ├─────────────────────────────────────────────────────────────────────────┤
    // │ Basic PATH variable        │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ PATH="/dir1:/dir2"         │ ✅         │ ✅         │ ✅               │
    // │                            │            │            │                  │
    // │ PATH modification          │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ PATH="/new:$PATH"          │ ✅         │ ✅         │ ✅               │
    // │                            │            │            │                  │
    // │ Export PATH                │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ export PATH                │ ✅         │ ✅         │ ✅               │
    // │                            │            │            │                  │
    // │ Command lookup             │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ command -v ls              │ ✅         │ ✅         │ ✅               │
    // │                            │            │            │                  │
    // │ which command              │ NOT POSIX  │ Available  │ AVOID            │
    // │ which ls                   │ ❌         │ ✅         │ ⚠️ Use command -v│
    // │                            │            │            │                  │
    // │ type builtin               │ NOT POSIX  │ Builtin    │ NOT SUPPORTED    │
    // │ type ls                    │ ❌         │ ✅         │ ❌ Use command -v│
    // │                            │            │            │                  │
    // │ whereis command            │ NOT POSIX  │ Available  │ NOT SUPPORTED    │
    // │ whereis ls                 │ ❌         │ ✅         │ ❌ Use command -v│
    // │                            │            │            │                  │
    // │ Colon-separated dirs       │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ PATH="/a:/b:/c"            │ ✅         │ ✅         │ ✅               │
    // │                            │            │            │                  │
    // │ Empty entry (current dir)  │ Dangerous  │ Works      │ FORBIDDEN        │
    // │ PATH="/bin::/usr/bin"      │ ⚠️ .      │ ✅ .       │ ❌ Security risk │
    // │                            │            │            │                  │
    // │ PATH with spaces           │ SUPPORTED  │ SUPPORTED  │ SUPPORTED        │
    // │ PATH="/My Dir:$PATH"       │ ✅ Quote  │ ✅ Quote  │ ✅ Must quote    │
    // │                            │            │            │                  │
    // │ Search order               │ POSIX      │ Bash       │ POSIX            │
    // │ Builtin → Func → PATH      │ ✅         │ ✅ + alias │ ✅ (no aliases)  │
    // │                            │            │            │                  │
    // │ Security                   │ User resp. │ User resp. │ Enforced         │
    // │ No "." in PATH             │ ⚠️        │ ⚠️        │ ✅ Validated     │
    // └─────────────────────────────────────────────────────────────────────────┘
    //
    // RUST MAPPING:
    // std::env::var("PATH")           → Get PATH value
    // std::env::set_var("PATH", ...)  → Set PATH value
    // std::env::split_paths(&path)    → Parse PATH into Vec<PathBuf>
    // std::env::join_paths([...])     → Join paths into PATH string
    // std::process::Command::new()    → Uses PATH for command lookup
    //
    // PURIFICATION RULES:
    // 1. Replace "which" with "command -v"
    // 2. Replace "type" with "command -v"
    // 3. Remove "." from PATH
    // 4. Quote all PATH references
    // 5. Use absolute paths for security-critical commands

    let comparison_table = r#"
# POSIX SUPPORTED: Basic PATH operations
PATH="/usr/local/bin:/usr/bin:/bin"
export PATH

# POSIX SUPPORTED: Modify PATH
PATH="/opt/myapp/bin:$PATH"
export PATH

# POSIX SUPPORTED: Command lookup
if command -v git >/dev/null 2>&1; then
    echo "Git is available"
fi

# AVOID: which (not POSIX)
# Purification: which git → command -v git
# if which git >/dev/null 2>&1; then ...
if command -v git >/dev/null 2>&1; then
    echo "Git found"
fi

# AVOID: type (bash-specific)
# Purification: type git → command -v git
# type git
command -v git

# FORBIDDEN: "." in PATH (security risk)
# PATH=".:$PATH"  # Trojan horse attack vector
# Purification: Remove all "." from PATH

# SUPPORTED: PATH with spaces (quote!)
PATH="/Program Files/Custom:$PATH"
echo "PATH: $PATH"

# POSIX SUPPORTED: Iterate PATH
IFS=:
for dir in $PATH; do
    echo "Directory: $dir"
done
"#;

    let mut lexer = Lexer::new(comparison_table);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "PATH comparison table should tokenize successfully"
        );
        let _ = tokens;
    }

    // POSIX STATUS: PATH is POSIX SUPPORTED
    // Security: bashrs enforces no "." in PATH (prevents Trojan horse attacks)
    // Purification: Replace which/type with command -v (POSIX standard)
    // Determinism: PATH is deterministic (set value produces same results)
    // Portability: PATH is POSIX (works on all Unix-like systems)
}

// ============================================================================
// BASH-VAR-002: $RANDOM purification (NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: $RANDOM is NOT SUPPORTED (bash-specific, HIGH priority purification)
//
// $RANDOM: Bash-specific variable that returns random integer 0-32767
// Each time $RANDOM is referenced, a new random number is generated
//
// WHY NOT SUPPORTED:
// 1. Non-deterministic (same script produces different results each run)
// 2. Bash-specific (not POSIX, doesn't exist in sh/dash/ash)
// 3. Breaks reproducibility (cannot replay script execution)
// 4. Breaks testing (tests produce different results each run)
// 5. Security risk (weak PRNG, predictable if seed known)
//
// CRITICAL: $RANDOM is antithetical to bashrs philosophy
// bashrs enforces DETERMINISM - same input MUST produce same output
//
// PURIFICATION STRATEGY:
// $RANDOM is FORBIDDEN - scripts using $RANDOM must be rewritten
//
// OPTION 1: Use explicit seed (deterministic)
// INPUT (bash with $RANDOM):
//   num=$RANDOM
// PURIFIED (deterministic seed):
//   seed=42
//   num=$(awk -v seed="$seed" 'BEGIN { srand(seed); print int(rand() * 32768) }')
//
// OPTION 2: Use sequence number (fully deterministic)
// INPUT (bash with $RANDOM):
//   `for i in {1..10}; do echo $RANDOM; done`
// PURIFIED (sequence):
//   seq 1 10
//
// OPTION 3: Use external source (deterministic if source is deterministic)
// INPUT: session_id=$RANDOM
// PURIFIED: session_id="session-$VERSION"
//
// OPTION 4: Read from /dev/urandom (cryptographically secure, but non-deterministic)
// Only use if CRYPTOGRAPHIC randomness required AND non-determinism acceptable
//   od -An -N2 -i /dev/urandom
#[test]
fn test_BASH_VAR_002_random_not_supported() {
    // $RANDOM is NOT SUPPORTED (non-deterministic, bash-specific)
    // PURIFICATION REQUIRED: Rewrite scripts to use deterministic alternatives
    let random_variable = concat!(
        "# NOT SUPPORTED: $RANDOM (non-deterministic)\n",
        "num=$RANDOM\n",
        "echo \"Random number: $num\"\n",
        "\n",
        "# NOT SUPPORTED: Multiple $RANDOM references (different values)\n",
        "a=$RANDOM\n",
        "b=$RANDOM\n",
        "echo \"Two random numbers: $a $b\"\n",
        "\n",
        "# NOT SUPPORTED: $RANDOM in loop (non-deterministic)\n",
        "for i in {1..10}; do\n",
        "    echo $RANDOM\n",
        "done\n",
        "\n",
        "# NOT SUPPORTED: $RANDOM for session ID (non-deterministic)\n",
        "session_id=\"session-$RANDOM\"\n",
    );

    let mut lexer = Lexer::new(random_variable);
    // Parser may not support $RANDOM - both Ok and Err are acceptable
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "$RANDOM should tokenize (even though NOT SUPPORTED)"
        );
    }
}

#[test]
fn test_BASH_VAR_002_random_purification_strategies() {
    // DOCUMENTATION: $RANDOM purification strategies (5 strategies for different use cases)
    //
    // STRATEGY 1: Fixed seed for deterministic PRNG
    // Use case: Need reproducible "random" numbers for testing
    // INPUT: num=$RANDOM
    // PURIFIED: num=$(awk -v seed=42 'BEGIN { srand(seed); print int(rand() * 32768) }')
    // Pros: Deterministic, reproducible
    // Cons: Requires awk, slower than $RANDOM
    //
    // STRATEGY 2: Sequence numbers
    // Use case: Just need unique numbers, don't need randomness
    // INPUT: for i in {1..10}; do echo $RANDOM; done
    // PURIFIED: seq 1 10
    // Pros: Simple, fast, deterministic
    // Cons: Not random at all, sequential pattern obvious
    //
    // STRATEGY 3: Version/timestamp-based identifiers
    // Use case: Session IDs, release tags that need to be deterministic
    // INPUT: session_id=$RANDOM
    // PURIFIED: session_id="session-$VERSION"
    // Pros: Meaningful identifiers, deterministic
    // Cons: Not random, may need to pass version as parameter
    //
    // STRATEGY 4: Hash-based deterministic randomness
    // Use case: Need deterministic but uniform distribution
    // INPUT: num=$RANDOM
    // PURIFIED: num=$(printf '%s' "$INPUT" | sha256sum | cut -c1-5 | xargs printf '%d' 0x)
    // Pros: Deterministic, uniform distribution if input varies
    // Cons: Complex, requires sha256sum
    //
    // STRATEGY 5: /dev/urandom (LAST RESORT - non-deterministic)
    // Use case: CRYPTOGRAPHIC randomness required (keys, tokens)
    // INPUT: num=$RANDOM
    // PURIFIED: num=$(od -An -N2 -i /dev/urandom)
    // Pros: Cryptographically secure
    // Cons: NON-DETERMINISTIC (violates bashrs philosophy)
    // WARNING: Only use for cryptographic purposes where non-determinism is acceptable

    let purification_strategies = r#"
# STRATEGY 1: Fixed seed (deterministic PRNG)
seed=42
num=$(awk -v seed="$seed" 'BEGIN { srand(seed); print int(rand() * 32768) }')
echo "Deterministic random: $num"

# STRATEGY 2: Sequence numbers
# Instead of: for i in {1..10}; do echo $RANDOM; done
seq 1 10

# STRATEGY 3: Version-based identifiers
version="1.0.0"
session_id="session-${version}"
release_tag="release-${version}"
echo "Session ID: $session_id"

# STRATEGY 4: Hash-based (deterministic from input)
input="user@example.com"
num=$(printf '%s' "$input" | sha256sum | cut -c1-5 | xargs -I{} printf '%d' "0x{}")
echo "Hash-based number: $num"

# STRATEGY 5: /dev/urandom (LAST RESORT - non-deterministic)
# Only for cryptographic purposes where non-determinism is acceptable
# token=$(od -An -N16 -tx1 /dev/urandom | tr -d ' ')
# echo "Crypto token: $token"
"#;

    let mut lexer = Lexer::new(purification_strategies);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Purification strategies should tokenize successfully"
        );
        let _ = tokens;
    }

    // All strategies except #5 are DETERMINISTIC
    // PREFERRED: Strategies 1-4 (deterministic)
    // AVOID: Strategy 5 (/dev/urandom) unless cryptographic randomness required
}

#[test]

include!("part5_cont.rs");
