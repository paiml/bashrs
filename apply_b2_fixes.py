#!/usr/bin/env python3
"""Apply B2 fixes to registry.rs by replacing expected_contains values.

Strategy: for each fix, find the entry by ID, then find its expected_contains
(the LAST string literal before the closing `)` of CorpusEntry::new), and replace
it with the new value in the SAME string format (raw or regular).

SAFETY: We search for the LAST occurrence of the old string within the entry region
to avoid corrupting the input field (which comes before expected_contains).
"""
import json
import sys


def escape_for_rust_string(s):
    """Escape a string for use inside a Rust regular string literal (between quotes)."""
    return s.replace('\\', '\\\\').replace('"', '\\"')


def main():
    fixes_path = sys.argv[1] if len(sys.argv) > 1 else '/tmp/b2_fixes_round2_clean.json'
    with open(fixes_path) as f:
        fixes = json.load(f)

    registry_path = 'rash/src/corpus/registry.rs'
    with open(registry_path) as f:
        content = f.read()

    applied = 0
    skipped = 0

    for fix in fixes:
        entry_id = fix['id']
        old_expected = fix['old']
        new_expected = fix['new']

        if old_expected == new_expected:
            skipped += 1
            continue

        # Find the entry by its ID string in the file
        id_pattern = f'"{entry_id}"'
        id_pos = content.find(id_pattern)
        if id_pos == -1:
            print(f"WARNING: {entry_id} not found in registry", file=sys.stderr)
            skipped += 1
            continue

        # Search for the old expected string within 2000 chars after the ID
        search_start = id_pos
        search_end = min(id_pos + 2000, len(content))
        region = content[search_start:search_end]

        old_escaped = escape_for_rust_string(old_expected)

        found = False
        for old_rust_candidate in [
            f'"{old_escaped}"',           # regular string with escapes
            f'r#"{old_expected}"#',        # raw string
            f'r#"{old_escaped}"#',         # raw string (shouldn't have escapes but just in case)
        ]:
            # SAFETY: use rfind to get the LAST occurrence in the region
            # This avoids corrupting the input field which comes before expected_contains
            pos = region.rfind(old_rust_candidate)
            if pos != -1:
                # Determine the format used for this entry
                if old_rust_candidate.startswith('r#"'):
                    new_rust = f'r#"{new_expected}"#'
                else:
                    new_escaped = escape_for_rust_string(new_expected)
                    new_rust = f'"{new_escaped}"'

                abs_pos = search_start + pos
                content = content[:abs_pos] + new_rust + content[abs_pos + len(old_rust_candidate):]
                applied += 1
                found = True
                break

        if not found:
            print(f"WARNING: {entry_id} expected string not found: {old_expected[:50]}", file=sys.stderr)
            skipped += 1

    with open(registry_path, 'w') as f:
        f.write(content)

    print(f"Applied: {applied}, Skipped: {skipped}", file=sys.stderr)


if __name__ == '__main__':
    main()
