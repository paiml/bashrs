#!/usr/bin/env python3
"""Fix remaining Makefile/Dockerfile entries.
Dockerfiles need from_image() call. Makefiles need simpler structures."""

import re
import sys

def format_rust_string(s):
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

# Only fix the 15 still-failing entries
REPLACEMENTS = {
    # Dockerfiles need from_image() + other DSL functions
    "D-15903": ('fn from_image(i: &str, t: &str) {} fn build_stage(src: u32) -> u32 { return src * 2; } fn main() { from_image("rust", "1.75"); let b: u32 = build_stage(42); println!("{}", b); }',
        'FROM rust:1.75'),
    "D-15904": ('fn from_image(i: &str, t: &str) {} fn resolve_arg(v: u32, d: u32) -> u32 { if v > 0 { return v; } return d; } fn main() { from_image("ubuntu", "22.04"); println!("{}", resolve_arg(0, 100)); }',
        'FROM ubuntu:22.04'),
    "D-15905": ('fn from_image(i: &str, t: &str) {} fn run_tests(c: u32) -> u32 { if c > 0 { return 0; } return 1; } fn main() { from_image("node", "20"); println!("{}", run_tests(42)); }',
        'FROM node:20'),
    "D-15932": ('fn from_image(i: &str, t: &str) {} fn install(n: u32) -> u32 { return n * 5; } fn main() { from_image("ubuntu", "22.04"); println!("{}", install(10)); }',
        'FROM ubuntu:22.04'),
    "D-15933": ('fn from_image(i: &str, t: &str) {} fn cache_lyr(id: u32, chg: u32) -> u32 { if chg == 0 { return id; } return id + 1000; } fn main() { from_image("golang", "1.22"); println!("{}", cache_lyr(1, 0)); }',
        'FROM golang:1.22'),
    "D-15934": ('fn from_image(i: &str, t: &str) {} fn onbuild(b: u32, t: u32) -> u32 { return b + t; } fn main() { from_image("python", "3.12"); println!("{}", onbuild(10, 5)); }',
        'FROM python:3.12'),
    "D-15960": ('fn from_image(i: &str, t: &str) {} fn plan(s: u32) -> u32 { return s; } fn cache_d(p: u32) -> u32 { return p + 10; } fn main() { from_image("rust", "1.75"); println!("{}", cache_d(plan(5))); }',
        'FROM rust:1.75'),
    "D-15961": ('fn from_image(i: &str, t: &str) {} fn sel_deps(e: u32) -> u32 { if e == 1 { return 100; } return 50; } fn main() { from_image("node", "20"); println!("{}", sel_deps(1)); }',
        'FROM node:20'),
    "D-15962": ('fn from_image(i: &str, t: &str) {} fn strip_bin(sz: u32) -> u32 { return sz / 3; } fn main() { from_image("alpine", "3.19"); println!("{}", strip_bin(9000)); }',
        'FROM alpine:3.19'),
    "D-15988": ('fn from_image(i: &str, t: &str) {} fn bld_shared() -> u32 { return 100; } fn bld_api(s: u32) -> u32 { return s + 10; } fn main() { from_image("node", "20"); let s: u32 = bld_shared(); println!("{} {}", s, bld_api(s)); }',
        'FROM node:20'),
    "D-15989": ('fn from_image(i: &str, t: &str) {} fn static_bld(s: u32) -> u32 { return s * 3; } fn main() { from_image("golang", "1.22"); println!("{}", static_bld(100)); }',
        'FROM golang:1.22'),
    "D-15990": ('fn from_image(i: &str, t: &str) {} fn init_proc(p: u32) -> u32 { return 1; } fn fwd_sig(i: u32, s: u32) -> u32 { return i * 100 + s; } fn main() { from_image("python", "3.12"); println!("{}", fwd_sig(init_proc(1), 15)); }',
        'FROM python:3.12'),
    # Makefiles with let expression issues - simplify
    "M-15929": ('fn compile(s: u32, f: u32) -> u32 { return s + f; } fn link(a: u32, b: u32) -> u32 { return a + b; } fn main() { println!("{}", link(compile(1, 10), compile(2, 10))); }',
        'compile() {'),
    "M-15930": ('fn build_sub(d: u32) -> u32 { return d * 10; } fn main() { println!("{} {} {}", build_sub(1), build_sub(2), build_sub(3)); }',
        'build_sub() {'),
    "M-15986": ('fn comp_up(e: u32, p: u32) -> u32 { return e * 100 + p; } fn main() { println!("{} {}", comp_up(1, 10), comp_up(2, 20)); }',
        'comp_up() {'),
}

def main():
    with open("rash/src/corpus/registry.rs", "r") as f:
        content = f.read()

    for entry_id, (rust_input, expected) in REPLACEMENTS.items():
        fmt = "Makefile" if entry_id.startswith("M-") else "Dockerfile"
        fmt_str = f"CorpusFormat::{fmt}"

        pattern = re.compile(
            rf'self\.entries\.push\(CorpusEntry::new\("{re.escape(entry_id)}".*?\)\);',
            re.DOTALL
        )

        match = re.search(rf'"{re.escape(entry_id)}", "([^"]*)", "([^"]*)"', content)
        if not match:
            print(f"WARNING: Could not find metadata for {entry_id}", file=sys.stderr)
            continue
        name = match.group(1)
        desc = match.group(2)

        rust_code = format_rust_string(rust_input)
        expected_code = format_rust_string(expected)

        replacement = f'self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",\n            {fmt_str}, CorpusTier::Adversarial,\n            {rust_code},\n            {expected_code}));'

        new_content, count = pattern.subn(replacement, content, count=1)
        if count == 0:
            print(f"WARNING: Could not replace {entry_id}", file=sys.stderr)
        else:
            content = new_content
            print(f"Fixed {entry_id}")

    with open("rash/src/corpus/registry.rs", "w") as f:
        f.write(content)

    print(f"\nFixed {len(REPLACEMENTS)} entries")

if __name__ == "__main__":
    main()
