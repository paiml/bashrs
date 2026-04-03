//! Stdlib runtime writers + selective dispatch. Extracted from posix.rs.
use crate::models::Result;
use std::fmt::Write;
impl super::posix::PosixEmitter {
    pub(crate) fn write_println_function(&self, output: &mut String) -> Result<()> {
        let lines = ["rash_println() {", "    printf '%s\\n' \"$1\"", "}", ""];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_print_function(&self, output: &mut String) -> Result<()> {
        let lines = ["rash_print() {", "    printf '%s' \"$1\"", "}", ""];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_eprintln_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_eprintln() {",
            "    printf '%s\\n' \"$1\" >&2",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_require_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_require() {",
            "    if ! \"$@\"; then",
            "        echo \"FATAL: Requirement failed: $*\" >&2",
            "        exit 1",
            "    fi",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_download_function(&self, output: &mut String) -> Result<()> {
        writeln!(output, "rash_download_verified() {{")?;
        writeln!(output, "    url=\"$1\"; dst=\"$2\"; checksum=\"$3\"")?;
        writeln!(output, "    ")?;
        self.write_download_logic(output)?;
        self.write_checksum_logic(output)?;
        writeln!(output, "}}")?;
        writeln!(output)?;
        Ok(())
    }
    pub(crate) fn write_download_logic(&self, output: &mut String) -> Result<()> {
        let lines = [
            "    if command -v curl >/dev/null 2>&1; then",
            "        curl -fsSL --proto '=https' --tlsv1.2 \"$url\" -o \"$dst\"",
            "    elif command -v wget >/dev/null 2>&1; then",
            "        wget -qO \"$dst\" \"$url\"",
            "    else",
            "        echo \"FATAL: Neither curl nor wget found\" >&2",
            "        return 1",
            "    fi",
            "    ",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_checksum_logic(&self, output: &mut String) -> Result<()> {
        let lines = [
            "    if command -v sha256sum >/dev/null 2>&1; then",
            "        echo \"$checksum  $dst\" | sha256sum -c >/dev/null",
            "    elif command -v shasum >/dev/null 2>&1; then",
            "        echo \"$checksum  $dst\" | shasum -a 256 -c >/dev/null",
            "    else",
            "        echo \"FATAL: No checksum utility found\" >&2",
            "        return 1",
            "    fi",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_shell_lines(&self, output: &mut String, lines: &[&str]) -> Result<()> {
        for line in lines {
            writeln!(output, "{line}")?;
        }
        Ok(())
    }
    // Stdlib function implementations
    pub(crate) fn write_string_trim_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_trim() {",
            "    s=\"$1\"",
            "    # Remove leading whitespace",
            "    s=\"${s#\"${s%%[![:space:]]*}\"}\"",
            "    # Remove trailing whitespace",
            "    s=\"${s%\"${s##*[![:space:]]}\"}\"",
            "    printf '%s' \"$s\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_string_contains_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_contains() {",
            "    haystack=\"$1\"",
            "    needle=\"$2\"",
            "    case \"$haystack\" in",
            "        *\"$needle\"*) return 0 ;;",
            "        *) return 1 ;;",
            "    esac",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_string_starts_with_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_starts_with() {",
            "    haystack=\"$1\"",
            "    prefix=\"$2\"",
            "    case \"$haystack\" in",
            "        \"$prefix\"*) return 0 ;;",
            "        *) return 1 ;;",
            "    esac",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_string_ends_with_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_ends_with() {",
            "    haystack=\"$1\"",
            "    suffix=\"$2\"",
            "    case \"$haystack\" in",
            "        *\"$suffix\") return 0 ;;",
            "        *) return 1 ;;",
            "    esac",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_string_len_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_len() {",
            "    s=\"$1\"",
            "    printf '%s' \"$s\" | wc -c | tr -d ' '",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_fs_exists_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_exists() {",
            "    path=\"$1\"",
            "    test -e \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }
    pub(crate) fn write_fs_read_file_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_read_file() {",
            "    path=\"$1\"",
            "    if [ ! -f \"$path\" ]; then",
            "        echo \"ERROR: File not found: $path\" >&2",
            "        return 1",
            "    fi",
            "    cat \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_fs_write_file_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_write_file() {",
            "    path=\"$1\"",
            "    content=\"$2\"",
            "    printf '%s' \"$content\" > \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_string_replace_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_replace() {",
            "    s=\"$1\"",
            "    old=\"$2\"",
            "    new=\"$3\"",
            "    # POSIX-compliant string replacement using case/sed fallback",
            "    if [ -z \"$old\" ]; then",
            "        printf '%s' \"$s\"",
            "        return",
            "    fi",
            "    # Replace first occurrence using parameter expansion",
            "    printf '%s' \"${s%%\"$old\"*}${new}${s#*\"$old\"}\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_string_to_upper_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_to_upper() {",
            "    s=\"$1\"",
            "    # POSIX-compliant uppercase conversion",
            "    printf '%s' \"$s\" | tr '[:lower:]' '[:upper:]'",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_string_to_lower_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_to_lower() {",
            "    s=\"$1\"",
            "    # POSIX-compliant lowercase conversion",
            "    printf '%s' \"$s\" | tr '[:upper:]' '[:lower:]'",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_fs_copy_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_copy() {",
            "    src=\"$1\"",
            "    dst=\"$2\"",
            "    if [ ! -f \"$src\" ]; then",
            "        echo \"ERROR: Source file not found: $src\" >&2",
            "        return 1",
            "    fi",
            "    cp \"$src\" \"$dst\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_fs_remove_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_remove() {",
            "    path=\"$1\"",
            "    if [ ! -e \"$path\" ]; then",
            "        echo \"ERROR: Path not found: $path\" >&2",
            "        return 1",
            "    fi",
            "    rm -f \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_fs_is_file_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_is_file() {",
            "    path=\"$1\"",
            "    test -f \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_fs_is_dir_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_fs_is_dir() {",
            "    path=\"$1\"",
            "    test -d \"$path\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    // Sprint 28: Complete Missing Stdlib Functions - GREEN PHASE

    pub(crate) fn write_string_split_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_string_split() {",
            "    text=\"$1\"",
            "    delimiter=\"$2\"",
            "    # Use tr to replace delimiter with newline for POSIX compliance",
            "    printf '%s\\n' \"$text\" | tr \"$delimiter\" '\\n'",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_array_len_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_array_len() {",
            "    array=\"$1\"",
            "    # Count non-empty lines",
            "    if [ -z \"$array\" ]; then",
            "        printf '0'",
            "    else",
            "        printf '%s\\n' \"$array\" | wc -l | tr -d ' '",
            "    fi",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn write_array_join_function(&self, output: &mut String) -> Result<()> {
        let lines = [
            "rash_array_join() {",
            "    array=\"$1\"",
            "    separator=\"$2\"",
            "    ",
            "    # Read lines and join with separator",
            "    first=1",
            "    result=\"\"",
            "    while IFS= read -r line; do",
            "        if [ \"$first\" = 1 ]; then",
            "            result=\"$line\"",
            "            first=0",
            "        else",
            "            result=\"${result}${separator}${line}\"",
            "        fi",
            "    done <<EOF",
            "$array",
            "EOF",
            "    printf '%s' \"$result\"",
            "}",
            "",
        ];
        self.write_shell_lines(output, &lines)
    }

    pub(crate) fn needs_runtime(&self, used_functions: &std::collections::HashSet<&str>) -> bool {
        // Check if any rash_* runtime functions are actually used
        used_functions.iter().any(|f| f.starts_with("rash_"))
    }

    pub(crate) fn write_selective_runtime(
        &self,
        output: &mut String,
        used_functions: &std::collections::HashSet<&str>,
    ) -> Result<()> {
        writeln!(output, "# Rash runtime functions")?;

        // Core runtime functions
        if used_functions.contains("rash_println") {
            self.write_println_function(output)?;
        }
        if used_functions.contains("rash_print") {
            self.write_print_function(output)?;
        }
        if used_functions.contains("rash_eprintln") {
            self.write_eprintln_function(output)?;
        }
        if used_functions.contains("rash_require") {
            self.write_require_function(output)?;
        }
        if used_functions.contains("rash_download_verified") {
            self.write_download_function(output)?;
        }

        // String stdlib functions
        let has_string_funcs = used_functions.iter().any(|f| f.starts_with("rash_string_"));
        let has_fs_funcs = used_functions.iter().any(|f| f.starts_with("rash_fs_"));
        let has_array_funcs = used_functions.iter().any(|f| f.starts_with("rash_array_"));

        if has_string_funcs || has_fs_funcs || has_array_funcs {
            writeln!(output, "# Rash stdlib functions")?;
        }

        if used_functions.contains("rash_string_trim") {
            self.write_string_trim_function(output)?;
        }
        if used_functions.contains("rash_string_contains") {
            self.write_string_contains_function(output)?;
        }
        if used_functions.contains("rash_string_len") {
            self.write_string_len_function(output)?;
        }
        if used_functions.contains("rash_string_replace") {
            self.write_string_replace_function(output)?;
        }
        if used_functions.contains("rash_string_to_upper") {
            self.write_string_to_upper_function(output)?;
        }
        if used_functions.contains("rash_string_to_lower") {
            self.write_string_to_lower_function(output)?;
        }

        if used_functions.contains("rash_string_starts_with") {
            self.write_string_starts_with_function(output)?;
        }
        if used_functions.contains("rash_string_ends_with") {
            self.write_string_ends_with_function(output)?;
        }

        // FS stdlib functions
        if used_functions.contains("rash_fs_exists") {
            self.write_fs_exists_function(output)?;
        }
        if used_functions.contains("rash_fs_read_file") {
            self.write_fs_read_file_function(output)?;
        }
        if used_functions.contains("rash_fs_write_file") {
            self.write_fs_write_file_function(output)?;
        }
        if used_functions.contains("rash_fs_copy") {
            self.write_fs_copy_function(output)?;
        }
        if used_functions.contains("rash_fs_remove") {
            self.write_fs_remove_function(output)?;
        }
        if used_functions.contains("rash_fs_is_file") {
            self.write_fs_is_file_function(output)?;
        }
        if used_functions.contains("rash_fs_is_dir") {
            self.write_fs_is_dir_function(output)?;
        }

        // Array stdlib functions
        if used_functions.contains("rash_string_split") {
            self.write_string_split_function(output)?;
        }
        if used_functions.contains("rash_array_len") {
            self.write_array_len_function(output)?;
        }
        if used_functions.contains("rash_array_join") {
            self.write_array_join_function(output)?;
        }

        Ok(())
    }

}
