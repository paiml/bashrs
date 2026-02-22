#!/usr/bin/env python3
"""Round 9: EXTREME+ pathological entries — deeper recursion, more complex control flow.
Categories A-X with maximum difficulty.
Expansion function: 187
START_ID: 16192
"""

START_ID = 16192

def format_rust_string(s):
    if '"' in s and '\\' in s:
        return f'r##"{s}"##'
    elif '"' in s:
        return f'r#"{s}"#'
    else:
        return f'r#"{s}"#'

def entry(bid, name, desc, fmt, code, expected):
    tier = "CorpusTier::Adversarial"
    code_s = format_rust_string(code)
    exp_s = format_rust_string(expected)
    return f'        self.entries.push(CorpusEntry::new("{bid}", "{name}", "{desc}",\n            CorpusFormat::{fmt}, {tier},\n            {code_s},\n            {exp_s}));'

entries_bash = []
entries_make = []
entries_docker = []
n = START_ID

# ============================================
# A. EXTREME Pipeline compositions
# ============================================

entries_bash.append(entry(f"B-{n}", "pipeline-diamond", "Diamond-shaped pipeline: split+merge",
    "Bash",
    "fn upper(x: u32) -> u32 { return x * 3 + 1; } fn lower(x: u32) -> u32 { return x * 2 + 5; } fn merge(a: u32, b: u32) -> u32 { return a + b; } fn diamond(x: u32) -> u32 { return merge(upper(x), lower(x)); } fn main() { let mut i: u32 = 0; while i < 5 { println!(\"{}\", diamond(i)); i = i + 1; } }",
    "6"))
n += 1

entries_bash.append(entry(f"B-{n}", "pipeline-cascade-err", "Pipeline with error propagation simulation",
    "Bash",
    "fn step1(x: u32) -> u32 { if x == 0 { return 99999; } return x * 2; } fn step2(x: u32) -> u32 { if x == 99999 { return 99999; } return x + 10; } fn step3(x: u32) -> u32 { if x == 99999 { return 99999; } return x / 3; } fn main() { println!(\"{} {} {}\", step3(step2(step1(5))), step3(step2(step1(0))), step3(step2(step1(100)))); }",
    "6 99999 70"))
n += 1

# ============================================
# B. EXTREME Quoting
# ============================================

entries_bash.append(entry(f"B-{n}", "escape-tower-8", "Eight-level escape tower simulation",
    "Bash",
    "fn tower(level: u32) -> u32 { if level == 0 { return 1; } let sub: u32 = tower(level - 1); return sub * 2 + level; } fn main() { println!(\"{} {} {} {}\", tower(0), tower(3), tower(5), tower(8)); }",
    "1 18 62 502"))
n += 1

# ============================================
# C. EXTREME One-liners
# ============================================

entries_bash.append(entry(f"B-{n}", "oneliner-20op", "Twenty operations single expression",
    "Bash",
    "fn main() { let mut a: u32 = 1; a = a + 1; a = a * 2; a = a + 3; a = a * 4; a = a + 5; a = a * 6; a = a + 7; a = a * 8; a = a + 9; a = a * 2; a = a + 1; a = a * 3; a = a + 2; a = a * 4; a = a + 3; a = a * 5; a = a + 4; a = a * 6; a = a + 5; println!(\"{}\", a); }",
    "218824685"))
n += 1

# ============================================
# D. EXTREME Glob
# ============================================

entries_bash.append(entry(f"B-{n}", "glob-nfa-multi", "Glob NFA with multiple wildcards simulation",
    "Bash",
    "fn nfa_states(wildcards: u32, literals: u32) -> u32 { return wildcards * 2 + literals + 1; } fn nfa_transitions(states: u32) -> u32 { return states * states; } fn main() { println!(\"{} {} {} {}\", nfa_states(1, 5), nfa_transitions(nfa_states(1, 5)), nfa_states(3, 10), nfa_transitions(nfa_states(3, 10))); }",
    "8 64 17 289"))
n += 1

# ============================================
# E. EXTREME Heredoc
# ============================================

entries_bash.append(entry(f"B-{n}", "heredoc-multiline-sim", "Heredoc multiline content size simulation",
    "Bash",
    "fn heredoc_size(lines: u32, avg_len: u32, indent: u32) -> u32 { return lines * (avg_len + indent + 1); } fn tab_stripped(size: u32, tabs: u32, lines: u32) -> u32 { return size - tabs * lines; } fn main() { let raw: u32 = heredoc_size(10, 40, 2); let stripped: u32 = tab_stripped(raw, 2, 10); println!(\"{} {}\", raw, stripped); }",
    "430 410"))
n += 1

# ============================================
# F. EXTREME Environment
# ============================================

entries_bash.append(entry(f"B-{n}", "env-subst-chain-6", "Six-level environment variable substitution chain",
    "Bash",
    "fn subst(val: u32, fallback: u32) -> u32 { if val != 0 { return val; } return fallback; } fn chain6(v1: u32, v2: u32, v3: u32, v4: u32, v5: u32, v6: u32) -> u32 { return subst(v1, subst(v2, subst(v3, subst(v4, subst(v5, v6))))); } fn main() { println!(\"{} {} {} {}\", chain6(0, 0, 0, 0, 0, 42), chain6(0, 0, 10, 0, 0, 42), chain6(1, 2, 3, 4, 5, 6), chain6(0, 0, 0, 0, 0, 0)); }",
    "42 10 1 0"))
n += 1

# ============================================
# G. EXTREME SSH
# ============================================

entries_bash.append(entry(f"B-{n}", "ssh-keygen-sim", "SSH key generation parameter simulation",
    "Bash",
    "fn key_bits(algo: u32) -> u32 { if algo == 1 { return 2048; } if algo == 2 { return 4096; } if algo == 3 { return 256; } if algo == 4 { return 384; } return 0; } fn key_strength(bits: u32) -> u32 { return bits / 2; } fn main() { let mut i: u32 = 1; while i <= 4 { println!(\"{} {}\", key_bits(i), key_strength(key_bits(i))); i = i + 1; } }",
    "2048 1024"))
n += 1

# ============================================
# H. EXTREME Printing
# ============================================

entries_bash.append(entry(f"B-{n}", "printf-color-sim", "ANSI color code simulation",
    "Bash",
    "fn color_code(fg: u32, bg: u32, bold: u32) -> u32 { return bold * 10000 + fg * 100 + bg; } fn reset() -> u32 { return 0; } fn main() { println!(\"{} {} {} {}\", color_code(31, 40, 1), color_code(32, 0, 0), color_code(37, 44, 1), reset()); }",
    "13140 3200 13744 0"))
n += 1

# ============================================
# I. EXTREME awk/sed
# ============================================

entries_bash.append(entry(f"B-{n}", "awk-assoc-sim", "Awk associative array simulation via hash",
    "Bash",
    "fn hash_key(key: u32) -> u32 { return (key * 2654435761) % 1000; } fn main() { println!(\"{} {} {} {} {}\", hash_key(1), hash_key(2), hash_key(10), hash_key(100), hash_key(1000)); }",
    "761 522 521 209 89"))
n += 1

entries_bash.append(entry(f"B-{n}", "sed-backref-sim", "Sed backreference counting simulation",
    "Bash",
    "fn count_groups(pattern_len: u32) -> u32 { return pattern_len / 3; } fn backref_cost(groups: u32) -> u32 { return groups * groups; } fn main() { println!(\"{} {} {} {}\", count_groups(9), backref_cost(count_groups(9)), count_groups(30), backref_cost(count_groups(30))); }",
    "3 9 10 100"))
n += 1

# ============================================
# J. EXTREME Data Structures
# ============================================

entries_bash.append(entry(f"B-{n}", "hash-table-probe", "Hash table linear probing simulation",
    "Bash",
    "fn hash(key: u32, size: u32) -> u32 { return key % size; } fn probe(key: u32, size: u32, occupied: u32) -> u32 { let h: u32 = hash(key, size); let mut pos: u32 = h; let mut probes: u32 = 0; while probes < size { if (occupied / (pos + 1)) % 2 == 0 { return pos; } pos = (pos + 1) % size; probes = probes + 1; } return size; } fn main() { println!(\"{} {} {}\", hash(42, 10), hash(100, 7), probe(42, 10, 0)); }",
    "2 2 2"))
n += 1

entries_bash.append(entry(f"B-{n}", "priority-queue-sim", "Priority queue with binary heap simulation",
    "Bash",
    "fn parent(i: u32) -> u32 { if i == 0 { return 0; } return (i - 1) / 2; } fn sift_up_count(idx: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = idx; while i > 0 { i = parent(i); count = count + 1; } return count; } fn main() { println!(\"{} {} {} {} {}\", sift_up_count(0), sift_up_count(1), sift_up_count(3), sift_up_count(7), sift_up_count(15)); }",
    "0 1 2 3 4"))
n += 1

# ============================================
# K. EXTREME Sourcing
# ============================================

entries_bash.append(entry(f"B-{n}", "source-diamond-dep", "Diamond dependency in source resolution",
    "Bash",
    "fn mod_a() -> u32 { return 1; } fn mod_b(a: u32) -> u32 { return a + 10; } fn mod_c(a: u32) -> u32 { return a + 20; } fn mod_d(b: u32, c: u32) -> u32 { return b + c; } fn main() { let a: u32 = mod_a(); let b: u32 = mod_b(a); let c: u32 = mod_c(a); let d: u32 = mod_d(b, c); println!(\"{} {} {} {}\", a, b, c, d); }",
    "1 11 21 32"))
n += 1

# ============================================
# L. EXTREME Control Flow
# ============================================

entries_bash.append(entry(f"B-{n}", "nested-if-while-mix", "Mixed nested if and while control flow",
    "Bash",
    "fn main() { let mut sum: u32 = 0; let mut i: u32 = 0; while i < 10 { if i % 3 == 0 { let mut j: u32 = 0; while j < i { if j % 2 == 0 { sum = sum + j; } j = j + 1; } } else { sum = sum + i; } i = i + 1; } println!(\"{}\", sum); }",
    "59"))
n += 1

entries_bash.append(entry(f"B-{n}", "while-break-sim", "While with break simulation via flag",
    "Bash",
    "fn main() { let mut i: u32 = 0; let mut found: u32 = 0; while i < 100 { if found == 0 { if i * i > 50 { found = i; } } i = i + 1; } println!(\"{}\", found); }",
    "8"))
n += 1

# ============================================
# M. EXTREME Traps
# ============================================

entries_bash.append(entry(f"B-{n}", "trap-cleanup-stack", "Trap cleanup stack (LIFO) simulation",
    "Bash",
    "fn push_cleanup(stack: u32, action: u32) -> u32 { return stack * 100 + action; } fn pop_cleanup(stack: u32) -> u32 { return stack / 100; } fn run_cleanup(stack: u32) -> u32 { if stack == 0 { return 0; } return (stack % 100) + run_cleanup(pop_cleanup(stack)); } fn main() { let mut s: u32 = 0; s = push_cleanup(s, 10); s = push_cleanup(s, 20); s = push_cleanup(s, 30); println!(\"{}\", run_cleanup(s)); }",
    "60"))
n += 1

# ============================================
# N. EXTREME Parsing
# ============================================

entries_bash.append(entry(f"B-{n}", "arg-parser-state-machine", "Argument parser state machine",
    "Bash",
    "fn parse_state(state: u32, ch: u32) -> u32 { if state == 0 { if ch == 45 { return 1; } return 3; } if state == 1 { if ch == 45 { return 2; } return 3; } if state == 2 { return 4; } if state == 3 { return 3; } return 4; } fn count_opts(inputs: u32, n: u32) -> u32 { let mut state: u32 = 0; let mut opts: u32 = 0; let mut v: u32 = inputs; let mut i: u32 = 0; while i < n { let ch: u32 = v % 100; state = parse_state(state, ch); if state == 3 { opts = opts + 1; } v = v / 100; i = i + 1; } return opts; } fn main() { println!(\"{}\", count_opts(454500, 3)); }",
    "0"))
n += 1

# ============================================
# Q. EXTREME Numerical
# ============================================

entries_bash.append(entry(f"B-{n}", "sieve-count-primes", "Sieve-based prime counting (small range)",
    "Bash",
    "fn is_prime(n: u32) -> u32 { if n < 2 { return 0; } let mut i: u32 = 2; while i * i <= n { if n % i == 0 { return 0; } i = i + 1; } return 1; } fn count_primes(limit: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 2; while i <= limit { if is_prime(i) != 0 { count = count + 1; } i = i + 1; } return count; } fn main() { println!(\"{} {} {} {}\", count_primes(10), count_primes(30), count_primes(50), count_primes(100)); }",
    "4 10 15 25"))
n += 1

entries_bash.append(entry(f"B-{n}", "matrix-mul-2x2", "2x2 matrix multiplication simulation",
    "Bash",
    "fn mat_mul_00(a00: u32, a01: u32, b00: u32, b10: u32) -> u32 { return a00 * b00 + a01 * b10; } fn mat_mul_01(a00: u32, a01: u32, b01: u32, b11: u32) -> u32 { return a00 * b01 + a01 * b11; } fn mat_mul_10(a10: u32, a11: u32, b00: u32, b10: u32) -> u32 { return a10 * b00 + a11 * b10; } fn mat_mul_11(a10: u32, a11: u32, b01: u32, b11: u32) -> u32 { return a10 * b01 + a11 * b11; } fn main() { println!(\"{} {} {} {}\", mat_mul_00(1, 2, 5, 7), mat_mul_01(1, 2, 6, 8), mat_mul_10(3, 4, 5, 7), mat_mul_11(3, 4, 6, 8)); }",
    "19 22 43 50"))
n += 1

# ============================================
# R. EXTREME Symbolic
# ============================================

entries_bash.append(entry(f"B-{n}", "demorgan-verify", "De Morgan's law verification simulation",
    "Bash",
    "fn not_fn(x: u32) -> u32 { if x != 0 { return 0; } return 1; } fn and_fn(a: u32, b: u32) -> u32 { if a != 0 { if b != 0 { return 1; } } return 0; } fn or_fn(a: u32, b: u32) -> u32 { if a != 0 { return 1; } if b != 0 { return 1; } return 0; } fn verify_dm1(a: u32, b: u32) -> u32 { let lhs: u32 = not_fn(and_fn(a, b)); let rhs: u32 = or_fn(not_fn(a), not_fn(b)); if lhs == rhs { return 1; } return 0; } fn main() { println!(\"{} {} {} {}\", verify_dm1(0, 0), verify_dm1(0, 1), verify_dm1(1, 0), verify_dm1(1, 1)); }",
    "1 1 1 1"))
n += 1

# ============================================
# S. EXTREME Editor
# ============================================

entries_bash.append(entry(f"B-{n}", "vim-macro-sim", "Vim macro repeat simulation",
    "Bash",
    "fn macro_step(pos: u32, action: u32) -> u32 { if action == 1 { return pos + 1; } if action == 2 { return pos + 5; } if action == 3 { if pos > 0 { return pos - 1; } return 0; } return pos; } fn replay(start: u32, actions: u32, n_actions: u32, repeats: u32) -> u32 { let mut pos: u32 = start; let mut r: u32 = 0; while r < repeats { let mut v: u32 = actions; let mut i: u32 = 0; while i < n_actions { pos = macro_step(pos, v % 10); v = v / 10; i = i + 1; } r = r + 1; } return pos; } fn main() { println!(\"{}\", replay(0, 121, 3, 5)); }",
    "25"))
n += 1

# ============================================
# T. EXTREME Functional
# ============================================

entries_bash.append(entry(f"B-{n}", "church-power", "Church numeral exponentiation simulation",
    "Bash",
    "fn church_pow(base: u32, exp: u32) -> u32 { let mut result: u32 = 1; let mut i: u32 = 0; while i < exp { result = result * base; i = i + 1; } return result; } fn main() { println!(\"{} {} {} {} {}\", church_pow(2, 0), church_pow(2, 1), church_pow(2, 10), church_pow(3, 5), church_pow(1, 100)); }",
    "1 2 1024 243 1"))
n += 1

entries_bash.append(entry(f"B-{n}", "fold-left-sim", "Fold left simulation over sequence",
    "Bash",
    "fn fold_add(acc: u32, x: u32) -> u32 { return acc + x; } fn fold_mul(acc: u32, x: u32) -> u32 { return acc * x; } fn fold_max(acc: u32, x: u32) -> u32 { if x > acc { return x; } return acc; } fn apply_fold(n: u32, init: u32, op: u32) -> u32 { let mut acc: u32 = init; let mut i: u32 = 1; while i <= n { if op == 0 { acc = fold_add(acc, i); } if op == 1 { acc = fold_mul(acc, i); } if op == 2 { acc = fold_max(acc, i); } i = i + 1; } return acc; } fn main() { println!(\"{} {} {}\", apply_fold(10, 0, 0), apply_fold(6, 1, 1), apply_fold(5, 0, 2)); }",
    "55 720 5"))
n += 1

# ============================================
# U. EXTREME Provable
# ============================================

entries_bash.append(entry(f"B-{n}", "total-function-verify", "Total function: all inputs produce output",
    "Bash",
    "fn total_fn(x: u32) -> u32 { if x == 0 { return 1; } if x % 2 == 0 { return x / 2; } return x + 1; } fn verify_total(limit: u32) -> u32 { let mut i: u32 = 0; let mut all_ok: u32 = 1; while i <= limit { let r: u32 = total_fn(i); if r > limit + 1 { all_ok = 0; } i = i + 1; } return all_ok; } fn main() { println!(\"{} {}\", verify_total(100), verify_total(1000)); }",
    "1 1"))
n += 1

# ============================================
# V. EXTREME Clippy
# ============================================

entries_bash.append(entry(f"B-{n}", "clippy-needless-return", "No needless return: early exit pattern",
    "Bash",
    "fn classify_temp(t: u32) -> u32 { if t < 32 { return 1; } if t < 50 { return 2; } if t < 70 { return 3; } if t < 90 { return 4; } return 5; } fn main() { println!(\"{} {} {} {} {}\", classify_temp(0), classify_temp(40), classify_temp(60), classify_temp(80), classify_temp(100)); }",
    "1 2 3 4 5"))
n += 1

# ============================================
# W. EXTREME C+Bash
# ============================================

entries_bash.append(entry(f"B-{n}", "c-union-sim", "C union type simulation",
    "Bash",
    "fn as_int(bytes: u32) -> u32 { return bytes; } fn as_float_approx(bytes: u32) -> u32 { return bytes / 1000; } fn main() { let data: u32 = 1078530000; println!(\"{} {}\", as_int(data), as_float_approx(data)); }",
    "1078530000 1078530"))
n += 1

entries_bash.append(entry(f"B-{n}", "c-endian-swap", "C endian byte swap simulation",
    "Bash",
    "fn swap16(x: u32) -> u32 { let lo: u32 = x % 256; let hi: u32 = (x / 256) % 256; return lo * 256 + hi; } fn swap32(x: u32) -> u32 { let b0: u32 = x % 256; let b1: u32 = (x / 256) % 256; let b2: u32 = (x / 65536) % 256; let b3: u32 = (x / 16777216) % 256; return b0 * 16777216 + b1 * 65536 + b2 * 256 + b3; } fn main() { println!(\"{} {} {}\", swap16(256), swap16(1), swap32(1)); }",
    "1 256 16777216"))
n += 1

# ============================================
# X. EXTREME Lua+Bash
# ============================================

entries_bash.append(entry(f"B-{n}", "lua-closure-upval", "Lua closure upvalue capture simulation",
    "Bash",
    "fn make_counter(start: u32) -> u32 { return start; } fn counter_next(state: u32) -> u32 { return state + 1; } fn counter_get(state: u32) -> u32 { return state; } fn main() { let mut c1: u32 = make_counter(0); let mut c2: u32 = make_counter(10); c1 = counter_next(c1); c1 = counter_next(c1); c1 = counter_next(c1); c2 = counter_next(c2); println!(\"{} {}\", counter_get(c1), counter_get(c2)); }",
    "3 11"))
n += 1

entries_bash.append(entry(f"B-{n}", "lua-pattern-sim", "Lua string pattern class simulation",
    "Bash",
    "fn is_alpha(c: u32) -> u32 { if c >= 65 { if c <= 122 { return 1; } } return 0; } fn is_digit(c: u32) -> u32 { if c >= 48 { if c <= 57 { return 1; } } return 0; } fn is_alnum(c: u32) -> u32 { if is_alpha(c) != 0 { return 1; } return is_digit(c); } fn main() { println!(\"{} {} {} {} {}\", is_alpha(65), is_alpha(48), is_digit(57), is_alnum(65), is_alnum(32)); }",
    "1 0 1 1 0"))
n += 1

entries_bash.append(entry(f"B-{n}", "lua-loadstring-sim", "Lua loadstring bytecode simulation",
    "Bash",
    "fn bytecode_size(src_len: u32) -> u32 { return src_len * 3 / 2 + 16; } fn exec_cost(bytecode_len: u32) -> u32 { return bytecode_len * 2; } fn main() { println!(\"{} {} {}\", bytecode_size(10), bytecode_size(100), exec_cost(bytecode_size(50))); }",
    "31 166 182"))
n += 1

bash_count = len(entries_bash)
last_bash_id = n - 1

# Makefile entries
entries_make.append(entry(f"M-{n}", "make-auto-dep-gen", "Makefile: automatic dependency generation simulation",
    "Makefile",
    "fn gen_deps(source: u32, includes: u32) -> u32 { return source * 100 + includes; } fn main() { println!(\"{} {}\", gen_deps(10, 3), gen_deps(5, 1)); }",
    "gen_deps() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-stamp-file", "Makefile: stamp file tracking simulation",
    "Makefile",
    "fn stamp(target: u32, time: u32) -> u32 { return target * 1000 + time; } fn is_stale(stamp_time: u32, source_time: u32) -> u32 { if stamp_time < source_time { return 1; } return 0; } fn main() { println!(\"{} {}\", stamp(1, 500), is_stale(500, 600)); }",
    "stamp() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-cross-compile", "Makefile: cross compilation target simulation",
    "Makefile",
    "fn target_arch(id: u32) -> u32 { if id == 1 { return 86; } if id == 2 { return 64; } if id == 3 { return 32; } return 0; } fn cross_flags(host: u32, target: u32) -> u32 { return host * 100 + target; } fn main() { println!(\"{} {}\", cross_flags(86, 64), cross_flags(64, 32)); }",
    "target_arch() {"))
n += 1

make_count = len(entries_make)

# Dockerfile entries
entries_docker.append(entry(f"D-{n}", "docker-multistage-opt", "Dockerfile: multi-stage optimization simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn stage_layers(cmds: u32) -> u32 { return cmds; } fn optimized_layers(cmds: u32) -> u32 { return (cmds + 1) / 2; } fn main() { from_image("golang", "1.22"); println!("{} {}", stage_layers(10), optimized_layers(10)); }',
    "FROM golang:1.22"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-secret-mount", "Dockerfile: secret mount simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn mount_secret(id: u32, target: u32) -> u32 { return id * 1000 + target; } fn main() { from_image("node", "20-slim"); println!("{} {}", mount_secret(1, 100), mount_secret(2, 200)); }',
    "FROM node:20-slim"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-platform-multi", "Dockerfile: multi-platform build simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn platform_count(arches: u32) -> u32 { return arches; } fn build_time(platforms: u32, base_time: u32) -> u32 { return platforms * base_time; } fn main() { from_image("alpine", "3.19"); println!("{} {}", platform_count(3), build_time(3, 60)); }',
    "FROM alpine:3.19"))
n += 1

docker_count = len(entries_docker)
total = bash_count + make_count + docker_count

print(f"// Round 9: {bash_count} bash + {make_count} makefile + {docker_count} dockerfile = {total} entries")
print(f"// IDs: B-{START_ID}..D-{n-1}")
print(f"// Expansion function: 187")
print()

print("    fn load_expansion187_bash(&mut self) {")
for e in entries_bash:
    print(e)
print("    }")
print()
print("    fn load_expansion187_makefile(&mut self) {")
for e in entries_make:
    print(e)
print("    }")
print()
print("    fn load_expansion187_dockerfile(&mut self) {")
for e in entries_docker:
    print(e)
print("    }")
