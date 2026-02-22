#!/usr/bin/env python3
"""Split large load_expansion functions in registry.rs into sub-functions.

Strategy: Find CorpusEntry::new( boundaries and split by line ranges.
No brace counting needed - just line-range splitting.

Two-level splitting:
- Sub-functions get MAX_ENTRIES entries each
- Dispatchers with >MAX_DISPATCHER_CALLS calls get hierarchical sub-dispatchers
"""

import re
import sys

MAX_ENTRIES = 2
MAX_DISPATCHER_CALLS = 50


def main():
    with open('rash/src/corpus/registry.rs', 'r') as f:
        lines = f.readlines()

    # Find all load_expansion function starts
    func_pattern = re.compile(
        r'^(\s+)(pub\s+)?fn (load_expansion\d+_(?:bash|makefile|dockerfile))\(&mut self\)\s*\{'
    )
    func_starts = []
    for i, line in enumerate(lines):
        m = func_pattern.match(line)
        if m:
            func_starts.append((i, m.group(3), m.group(1), 'pub ' if m.group(2) else ''))

    # Process functions
    new_lines = []
    last_end = 0
    split_count = 0
    subfunc_count = 0

    for idx, (func_start, func_name, indent, pub_kw) in enumerate(func_starts):
        # Determine function end
        if idx + 1 < len(func_starts):
            func_end = func_starts[idx + 1][0]
        else:
            func_end = len(lines) - 1
            for j in range(func_start + 2, len(lines)):
                stripped = lines[j].strip()
                ind_len = len(lines[j]) - len(lines[j].lstrip())
                if stripped == '}' and ind_len <= len(indent):
                    func_end = j + 1
                    break

        # Find all CorpusEntry::new( start lines in this function
        entry_lines = []
        for j in range(func_start, func_end):
            if 'CorpusEntry::new(' in lines[j]:
                entry_lines.append(j)

        if len(entry_lines) <= MAX_ENTRIES:
            continue

        # Detect pattern
        has_vec = any('vec![' in lines[i] for i in range(func_start, min(func_start + 8, func_end)))

        # Find the function's actual closing brace
        func_close = func_end - 1
        for j in range(func_end - 1, entry_lines[-1], -1):
            stripped = lines[j].strip()
            if stripped == '}':
                ind_len = len(lines[j]) - len(lines[j].lstrip())
                if ind_len <= len(indent):
                    func_close = j
                    break

        num_chunks = (len(entry_lines) + MAX_ENTRIES - 1) // MAX_ENTRIES

        # Copy lines before this function
        new_lines.extend(lines[last_end:func_start])

        # Write dispatcher function (with hierarchical splitting if needed)
        if num_chunks > MAX_DISPATCHER_CALLS:
            # Two-level dispatcher: fn -> groups -> sub-functions
            num_groups = (num_chunks + MAX_DISPATCHER_CALLS - 1) // MAX_DISPATCHER_CALLS
            new_lines.append(f'{indent}{pub_kw}fn {func_name}(&mut self) {{\n')
            for gi in range(num_groups):
                new_lines.append(f'{indent}    self.{func_name}_g{gi}();\n')
            new_lines.append(f'{indent}}}\n')
            new_lines.append(f'\n')

            # Write group dispatchers
            for gi in range(num_groups):
                group_start = gi * MAX_DISPATCHER_CALLS
                group_end = min((gi + 1) * MAX_DISPATCHER_CALLS, num_chunks)
                new_lines.append(f'{indent}fn {func_name}_g{gi}(&mut self) {{\n')
                for ci in range(group_start, group_end):
                    new_lines.append(f'{indent}    self.{func_name}_p{ci}();\n')
                new_lines.append(f'{indent}}}\n')
                new_lines.append(f'\n')
        else:
            # Single-level dispatcher
            new_lines.append(f'{indent}{pub_kw}fn {func_name}(&mut self) {{\n')
            for ci in range(num_chunks):
                new_lines.append(f'{indent}    self.{func_name}_p{ci}();\n')
            new_lines.append(f'{indent}}}\n')
            new_lines.append(f'\n')

        # Write sub-functions
        for ci in range(num_chunks):
            chunk_start_idx = ci * MAX_ENTRIES
            chunk_end_idx = min((ci + 1) * MAX_ENTRIES, len(entry_lines))

            # Determine line range for this chunk
            first_entry_line = entry_lines[chunk_start_idx]
            if chunk_end_idx < len(entry_lines):
                last_line = entry_lines[chunk_end_idx]  # Start of next chunk's first entry
            else:
                # Last chunk: go up to the function close (but not including the
                # vec ]; or for loop or closing brace)
                last_line = func_close
                # Find the line just before the closing structure
                for j in range(func_close - 1, entry_lines[-1], -1):
                    stripped = lines[j].strip()
                    if stripped in ('};', '}', '];', ''):
                        continue
                    if 'for ' in stripped and 'in entries' in stripped:
                        continue
                    if 'self.entries.push(' in stripped and 'CorpusEntry' not in stripped:
                        continue
                    if 'self.entries.extend(' in stripped:
                        continue
                    last_line = j + 1
                    break

            new_lines.append(f'{indent}fn {func_name}_p{ci}(&mut self) {{\n')

            if has_vec:
                new_lines.append(f'{indent}    let entries = vec![\n')
                for k in range(first_entry_line, last_line):
                    new_lines.append(lines[k])
                new_lines.append(f'{indent}    ];\n')
                new_lines.append(f'{indent}    for entry in entries {{\n')
                new_lines.append(f'{indent}        self.entries.push(entry);\n')
                new_lines.append(f'{indent}    }}\n')
            else:
                # Push pattern - just copy the push statements
                for k in range(first_entry_line, last_line):
                    new_lines.append(lines[k])

            new_lines.append(f'{indent}}}\n')
            new_lines.append(f'\n')

            subfunc_count += 1

        last_end = func_close + 1
        split_count += 1

    # Copy remaining lines
    new_lines.extend(lines[last_end:])

    print(f"Split {split_count} functions into {subfunc_count} sub-functions", file=sys.stderr)

    with open('rash/src/corpus/registry.rs', 'w') as f:
        f.writelines(new_lines)

    print(f"Written to rash/src/corpus/registry.rs", file=sys.stderr)


if __name__ == '__main__':
    main()
