#!/usr/bin/env python3
"""Fix corrupted M/D entries from fix_md_entries3.py.

The previous fix script's regex matched too early, leaving orphaned lines.
Pattern to fix:
  r#"FROM rust:1.75"#)); }"#,     <- needs )); }"# removed to become just "#));
  r#"build_stage() {"#));          <- orphaned line, delete entirely

And similarly for Makefile entries:
  r#"compile() {"#)); }"#,        <- same corruption
  r#"<old_expected>"#));           <- orphaned line
"""

import re

def main():
    with open("rash/src/corpus/registry.rs", "r") as f:
        lines = f.readlines()

    # Lines to delete (orphaned old expected_contains)
    delete_indices = set()

    for i, line in enumerate(lines):
        # Fix corrupted lines: r#"FROM rust:1.75"#)); }"#, -> r#"FROM rust:1.75"#));
        # Also: r#"compile() {"#)); }"#, -> r#"compile() {"#));
        if '"#)); }"#,' in line:
            # Fix: remove the )); }"#, and replace with "#));
            lines[i] = line.replace('"#)); }"#,', '"#));')
            # The next line is the orphaned old expected_contains - mark for deletion
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if next_line.startswith('r#"') and next_line.endswith('"#));'):
                    delete_indices.add(i + 1)

    # Remove orphaned lines (in reverse order to preserve indices)
    new_lines = [line for i, line in enumerate(lines) if i not in delete_indices]

    with open("rash/src/corpus/registry.rs", "w") as f:
        f.writelines(new_lines)

    print(f"Fixed {len(delete_indices)} corrupted entries")
    print(f"Deleted {len(delete_indices)} orphaned lines")

if __name__ == "__main__":
    main()
