#!/usr/bin/env python3
"""Fix Makefile/Dockerfile entries to use u32 types and single-line format (matching working entries)."""

import re
import sys

def format_rust_string(s):
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

# Rewrite all M/D entries to use simple u32 single-line format like the working M-700..M-717 entries
REPLACEMENTS = {
    # Round 1 Makefiles
    "M-15902": ('fn select_flags(debug: u32, plat: u32) -> u32 { if debug == 1 { return 0; } if plat == 1 { return 10; } return 20; } fn main() { println!("{}", select_flags(1, 0)); }',
        'select_flags() {'),
    # Round 1 Dockerfiles
    "D-15903": ('fn build_stage(src: u32) -> u32 { return src * 2; } fn runtime_stage(a: u32) -> u32 { return a + 1; } fn main() { let b: u32 = build_stage(42); println!("{}", runtime_stage(b)); }',
        'build_stage() {'),
    "D-15904": ('fn resolve_arg(v: u32, d: u32) -> u32 { if v > 0 { return v; } return d; } fn main() { println!("{} {}", resolve_arg(0, 100), resolve_arg(3000, 8080)); }',
        'resolve_arg() {'),
    "D-15905": ('fn run_tests(c: u32) -> u32 { if c > 0 { return 0; } return 1; } fn build_prod(t: u32) -> u32 { if t == 0 { return 1; } return 0; } fn main() { println!("{}", build_prod(run_tests(42))); }',
        'run_tests() {'),
    # Round 2 Makefiles
    "M-15929": ('fn compile(s: u32, f: u32) -> u32 { return s + f; } fn link(a: u32, b: u32) -> u32 { return a + b; } fn main() { let o1: u32 = compile(1, 10); let o2: u32 = compile(2, 10); println!("{}", link(o1, o2)); }',
        'compile() {'),
    "M-15930": ('fn build_sub(d: u32) -> u32 { return d * 10; } fn main() { let r1: u32 = build_sub(1); let r2: u32 = build_sub(2); let r3: u32 = build_sub(3); println!("{} {} {}", r1, r2, r3); }',
        'build_sub() {'),
    "M-15931": ('fn gen_deps(s: u32) -> u32 { return s * 100 + 1; } fn compile_dep(s: u32, d: u32) -> u32 { return s + d; } fn main() { println!("{}", compile_dep(1, gen_deps(1))); }',
        'gen_deps() {'),
    # Round 2 Dockerfiles
    "D-15932": ('fn install(n: u32) -> u32 { return n * 5; } fn cleanup(i: u32) -> u32 { return i - 2; } fn main() { println!("{}", cleanup(install(10))); }',
        'install() {'),
    "D-15933": ('fn cache_lyr(id: u32, chg: u32) -> u32 { if chg == 0 { return id; } return id + 1000; } fn main() { println!("{} {}", cache_lyr(1, 0), cache_lyr(2, 1)); }',
        'cache_lyr() {'),
    "D-15934": ('fn onbuild(b: u32, t: u32) -> u32 { return b + t; } fn set_env(k: u32, v: u32) -> u32 { return k * 100 + v; } fn main() { println!("{} {}", onbuild(10, 5), set_env(1, 42)); }',
        'onbuild() {'),
    # Round 3 Makefiles
    "M-15957": ('fn embed_ver(ma: u32, mi: u32, p: u32) -> u32 { return ma * 10000 + mi * 100 + p; } fn main() { println!("{}", embed_ver(1, 2, 3)); }',
        'embed_ver() {'),
    "M-15958": ('fn gen_tgt(s: u32, x: u32) -> u32 { return s * 10 + x; } fn main() { println!("{} {} {}", gen_tgt(1, 1), gen_tgt(2, 2), gen_tgt(3, 1)); }',
        'gen_tgt() {'),
    "M-15959": ('fn dbg_flags() -> u32 { return 1; } fn rel_flags() -> u32 { return 2; } fn build(f: u32, s: u32) -> u32 { return f * 1000 + s; } fn main() { println!("{} {}", build(dbg_flags(), 42), build(rel_flags(), 42)); }',
        'dbg_flags() {'),
    # Round 3 Dockerfiles
    "D-15960": ('fn plan(s: u32) -> u32 { return s; } fn cache_d(p: u32) -> u32 { return p + 10; } fn bld(c: u32) -> u32 { return c * 2; } fn pkg(b: u32) -> u32 { return b + 1; } fn main() { println!("{}", pkg(bld(cache_d(plan(5))))); }',
        'plan() {'),
    "D-15961": ('fn sel_deps(e: u32) -> u32 { if e == 1 { return 100; } return 50; } fn bld_env(d: u32, e: u32) -> u32 { return d + e * 10; } fn main() { println!("{} {}", bld_env(sel_deps(1), 1), bld_env(sel_deps(0), 0)); }',
        'sel_deps() {'),
    "D-15962": ('fn strip_bin(sz: u32) -> u32 { return sz / 3; } fn set_user(uid: u32) -> u32 { if uid == 0 { return 65534; } return uid; } fn main() { println!("{} {}", strip_bin(9000), set_user(0)); }',
        'strip_bin() {'),
    # Round 4 Makefiles
    "M-15985": ('fn cross_bld(os: u32, arch: u32, s: u32) -> u32 { return os * 10000 + arch * 100 + s; } fn main() { println!("{} {}", cross_bld(1, 1, 42), cross_bld(2, 2, 42)); }',
        'cross_bld() {'),
    "M-15986": ('fn comp_up(e: u32, p: u32) -> u32 { return e * 100 + p; } fn comp_dn(h: u32) -> u32 { return h; } fn main() { let d: u32 = comp_up(1, 10); println!("{} {}", d, comp_dn(d)); }',
        'comp_up() {'),
    "M-15987": ('fn bld_static(a: u32, b: u32) -> u32 { return a + b; } fn bld_shared(a: u32, b: u32) -> u32 { return (a + b) * 10; } fn main() { println!("{} {}", bld_static(3, 7), bld_shared(3, 7)); }',
        'bld_static() {'),
    # Round 4 Dockerfiles
    "D-15988": ('fn bld_shared() -> u32 { return 100; } fn bld_api(s: u32) -> u32 { return s + 10; } fn bld_web(s: u32) -> u32 { return s + 20; } fn main() { let s: u32 = bld_shared(); println!("{} {} {}", s, bld_api(s), bld_web(s)); }',
        'bld_shared() {'),
    "D-15989": ('fn static_bld(s: u32) -> u32 { return s * 3; } fn strip_dbg(b: u32) -> u32 { return b / 2; } fn main() { println!("{}", strip_dbg(static_bld(100))); }',
        'static_bld() {'),
    "D-15990": ('fn init_proc(p: u32) -> u32 { return 1; } fn fwd_sig(i: u32, s: u32) -> u32 { return i * 100 + s; } fn health(port: u32) -> u32 { if port > 0 { return 0; } return 1; } fn main() { println!("{} {}", fwd_sig(init_proc(1), 15), health(8000)); }',
        'init_proc() {'),
}

def main():
    with open("rash/src/corpus/registry.rs", "r") as f:
        content = f.read()

    for entry_id, (rust_input, expected) in REPLACEMENTS.items():
        # Determine format from prefix
        fmt = "Makefile" if entry_id.startswith("M-") else "Dockerfile"
        fmt_str = f"CorpusFormat::{fmt}"

        pattern = re.compile(
            rf'self\.entries\.push\(CorpusEntry::new\("{re.escape(entry_id)}".*?\)\);',
            re.DOTALL
        )

        # Get existing name and desc
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
