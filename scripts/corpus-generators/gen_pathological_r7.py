#!/usr/bin/env python3
"""Round 7: Extreme pathological entries — pushing transpiler hard.
Focus on: recursive algorithms, complex control flow, deep state machines."""

START_ID = 16119  # After round 6's ~50 entries
EXPANSION_NUM = 185

def fmt(s):
    if '"' in s and '#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

def eid(n):
    return f"B-{START_ID + n}"

entries = []
n = 0

# A: Extreme pipeline patterns
entries.append((eid(n), "pipeline-fork-join", "Fork-join pipeline with parallel stages",
    'fn fork_a(x: u32) -> u32 { return x * 3; } fn fork_b(x: u32) -> u32 { return x + 7; } fn join(a: u32, b: u32) -> u32 { return a + b; } fn pipeline(x: u32) -> u32 { return join(fork_a(x), fork_b(x)); } fn main() { println!("{} {} {}", pipeline(5), pipeline(10), pipeline(0)); }',
    '27 47 7'))
n += 1

entries.append((eid(n), "pipeline-feedback", "Pipeline with feedback loop simulation",
    'fn process(x: u32) -> u32 { return x * 2 + 1; } fn feedback(x: u32, rounds: u32) -> u32 { let mut v: u32 = x; let mut i: u32 = 0; while i < rounds { v = process(v); i = i + 1; } return v; } fn main() { println!("{} {} {}", feedback(1, 3), feedback(0, 5), feedback(2, 4)); }',
    '15 31 47'))
n += 1

# B: Extreme quoting
entries.append((eid(n), "backslash-counting", "Backslash counting for escape levels",
    'fn escape_level(raw: u32) -> u32 { let mut level: u32 = 0; let mut n: u32 = raw; while n > 1 { n = n / 2; level = level + 1; } return level; } fn main() { println!("{} {} {} {} {}", escape_level(1), escape_level(2), escape_level(4), escape_level(8), escape_level(16)); }',
    '0 1 2 3 4'))
n += 1

# C: Extreme one-liners
entries.append((eid(n), "ackermann-small", "Ackermann function for small inputs",
    'fn ack(m: u32, n: u32) -> u32 { if m == 0 { return n + 1; } if n == 0 { return ack(m - 1, 1); } return ack(m - 1, ack(m, n - 1)); } fn main() { println!("{} {} {} {} {}", ack(0, 0), ack(1, 1), ack(2, 2), ack(3, 2), ack(3, 3)); }',
    '1 3 7 29 61'))
n += 1

entries.append((eid(n), "tower-of-hanoi-count", "Tower of Hanoi move counter",
    'fn hanoi_moves(n: u32) -> u32 { if n == 0 { return 0; } let sub: u32 = hanoi_moves(n - 1); return 2 * sub + 1; } fn main() { println!("{} {} {} {} {}", hanoi_moves(0), hanoi_moves(1), hanoi_moves(3), hanoi_moves(5), hanoi_moves(10)); }',
    '0 1 7 31 1023'))
n += 1

# D: Glob patterns
entries.append((eid(n), "fnmatch-score", "Fnmatch-like complexity scorer",
    'fn complexity(stars: u32, ranges: u32, literals: u32) -> u32 { return stars * stars * 10 + ranges * 5 + literals; } fn main() { println!("{} {} {}", complexity(3, 2, 5), complexity(0, 0, 20), complexity(1, 5, 0)); }',
    '105 20 35'))
n += 1

# E: Heredoc extremes
entries.append((eid(n), "heredoc-nested-sim", "Nested heredoc depth simulation",
    'fn nest(depth: u32, base: u32) -> u32 { if depth == 0 { return base; } return nest(depth - 1, base * 2 + 1); } fn main() { println!("{} {} {} {}", nest(0, 1), nest(1, 1), nest(3, 1), nest(5, 1)); }',
    '1 3 15 63'))
n += 1

# F: Environment extremes
entries.append((eid(n), "env-scope-shadow", "Variable shadowing across scopes",
    'fn outer(x: u32) -> u32 { let x: u32 = x + 10; return inner(x); } fn inner(x: u32) -> u32 { let x: u32 = x * 2; return x; } fn main() { println!("{} {} {}", outer(1), outer(5), outer(0)); }',
    '22 30 20'))
n += 1

# H: Extreme text processing
entries.append((eid(n), "grep-count-sim", "Grep match counting simulation",
    'fn grep_c(haystack_len: u32, needle_len: u32, overlap: u32) -> u32 { if needle_len == 0 { return 0; } if overlap != 0 { return haystack_len - needle_len + 1; } return haystack_len / needle_len; } fn main() { println!("{} {} {} {}", grep_c(100, 5, 0), grep_c(100, 5, 1), grep_c(10, 3, 0), grep_c(10, 3, 1)); }',
    '20 96 3 8'))
n += 1

# I: Advanced data structures
entries.append((eid(n), "trie-depth-sim", "Trie depth calculation simulation",
    'fn trie_insert(depth: u32, key_len: u32) -> u32 { if key_len > depth { return key_len; } return depth; } fn main() { let mut d: u32 = 0; d = trie_insert(d, 3); d = trie_insert(d, 5); d = trie_insert(d, 2); d = trie_insert(d, 7); println!("{}", d); }',
    '7'))
n += 1

entries.append((eid(n), "ring-buffer-sim", "Ring buffer with wrap-around",
    'fn ring_push(head: u32, capacity: u32) -> u32 { return (head + 1) % capacity; } fn main() { let cap: u32 = 4; let mut h: u32 = 0; h = ring_push(h, cap); h = ring_push(h, cap); h = ring_push(h, cap); h = ring_push(h, cap); h = ring_push(h, cap); println!("{}", h); }',
    '1'))
n += 1

# L: Complex control flow
entries.append((eid(n), "state-machine-4st", "4-state finite state machine",
    'fn transition(state: u32, input: u32) -> u32 { if state == 0 { if input == 1 { return 1; } return 0; } if state == 1 { if input == 0 { return 2; } return 1; } if state == 2 { if input == 1 { return 3; } return 0; } if state == 3 { return 0; } return 0; } fn run(inputs: u32, count: u32) -> u32 { let mut state: u32 = 0; let mut i: u32 = 0; let mut bits: u32 = inputs; while i < count { let bit: u32 = bits % 2; state = transition(state, bit); bits = bits / 2; i = i + 1; } return state; } fn main() { println!("{} {} {}", run(0, 4), run(5, 4), run(15, 4)); }',
    '0 2 1'))
n += 1

entries.append((eid(n), "brainfk-sim-add", "Brainfuck-like addition simulation",
    'fn bf_inc(cell: u32) -> u32 { return (cell + 1) % 256; } fn bf_dec(cell: u32) -> u32 { if cell > 0 { return cell - 1; } return 255; } fn bf_add(a: u32, b: u32) -> u32 { let mut result: u32 = a; let mut count: u32 = b; while count > 0 { result = bf_inc(result); count = count - 1; } return result; } fn main() { println!("{} {} {}", bf_add(10, 20), bf_add(250, 10), bf_add(0, 0)); }',
    '30 4 0'))
n += 1

# M: Trap/signal handling
entries.append((eid(n), "signal-mask-sim", "Signal mask bit manipulation",
    'fn mask_set(mask: u32, sig: u32) -> u32 { let mut bit: u32 = 1; let mut i: u32 = 0; while i < sig { bit = bit * 2; i = i + 1; } return mask + bit; } fn mask_test(mask: u32, sig: u32) -> u32 { let mut bit: u32 = 1; let mut i: u32 = 0; while i < sig { bit = bit * 2; i = i + 1; } if (mask / bit) % 2 == 1 { return 1; } return 0; } fn main() { let mut m: u32 = 0; m = mask_set(m, 2); m = mask_set(m, 5); println!("{} {} {} {}", mask_test(m, 2), mask_test(m, 3), mask_test(m, 5), m); }',
    '1 0 1 36'))
n += 1

# N: Command parsing extremes
entries.append((eid(n), "arg-tokenizer-sim", "Argument tokenizer simulation",
    'fn token_count(total_len: u32, avg_word: u32) -> u32 { if avg_word == 0 { return 0; } return total_len / avg_word; } fn main() { println!("{} {} {} {}", token_count(50, 5), token_count(100, 10), token_count(0, 3), token_count(7, 3)); }',
    '10 10 0 2'))
n += 1

# Q: Hard numerical
entries.append((eid(n), "modpow-small", "Modular exponentiation",
    'fn modpow(mut base: u32, mut exp: u32, modulus: u32) -> u32 { let mut result: u32 = 1; base = base % modulus; while exp > 0 { if exp % 2 == 1 { result = (result * base) % modulus; } exp = exp / 2; base = (base * base) % modulus; } return result; } fn main() { println!("{} {} {} {}", modpow(2, 10, 1000), modpow(3, 7, 100), modpow(5, 3, 13), modpow(7, 0, 10)); }',
    '24 87 8 1'))
n += 1

entries.append((eid(n), "extended-gcd", "Extended GCD for Bezout coefficients (abs)",
    'fn gcd(mut a: u32, mut b: u32) -> u32 { while b != 0 { let t: u32 = b; b = a % b; a = t; } return a; } fn modinv(a: u32, m: u32) -> u32 { let mut t: u32 = 0; let mut new_t: u32 = 1; let mut r: u32 = m; let mut new_r: u32 = a; while new_r != 0 { let q: u32 = r / new_r; let tmp_t: u32 = new_t; let tmp_r: u32 = new_r; new_r = r - q * new_r; r = tmp_r; new_t = t; t = tmp_t; } return t % m; } fn main() { println!("{}", gcd(48, 18)); }',
    '6'))
n += 1

entries.append((eid(n), "digit-product", "Product of digits of a number",
    'fn digit_prod(mut n: u32) -> u32 { if n == 0 { return 0; } let mut prod: u32 = 1; while n > 0 { let d: u32 = n % 10; if d == 0 { return 0; } prod = prod * d; n = n / 10; } return prod; } fn main() { println!("{} {} {} {} {}", digit_prod(123), digit_prod(999), digit_prod(100), digit_prod(0), digit_prod(234)); }',
    '6 729 0 0 24'))
n += 1

entries.append((eid(n), "harmonic-approx", "Harmonic number approximation via integer sum",
    'fn harmonic_x1000(n: u32) -> u32 { let mut sum: u32 = 0; let mut i: u32 = 1; while i <= n { sum = sum + 1000 / i; i = i + 1; } return sum; } fn main() { println!("{} {} {}", harmonic_x1000(1), harmonic_x1000(5), harmonic_x1000(10)); }',
    '1000 2283 2928'))
n += 1

# R: Symbolic extremes
entries.append((eid(n), "symbolic-diff-sim", "Symbolic differentiation simulation (polynomial)",
    'fn diff_const() -> u32 { return 0; } fn diff_x() -> u32 { return 1; } fn diff_xn(n: u32) -> u32 { return n; } fn main() { println!("{} {} {} {} {}", diff_const(), diff_x(), diff_xn(2), diff_xn(3), diff_xn(10)); }',
    '0 1 2 3 10'))
n += 1

# S: Editor simulation extremes
entries.append((eid(n), "vi-yank-put-sim", "Vi yank/put register simulation",
    'fn yank(reg: u32, content: u32) -> u32 { return content; } fn put(reg: u32, buffer_lines: u32) -> u32 { return buffer_lines + 1; } fn main() { let r: u32 = yank(0, 42); let lines: u32 = put(0, 10); let lines2: u32 = put(0, lines); println!("{} {} {}", r, lines, lines2); }',
    '42 11 12'))
n += 1

# T: Functional extremes
entries.append((eid(n), "church-numeral-sim", "Church numeral simulation",
    'fn zero() -> u32 { return 0; } fn succ(n: u32) -> u32 { return n + 1; } fn church_add(a: u32, b: u32) -> u32 { let mut result: u32 = a; let mut i: u32 = 0; while i < b { result = succ(result); i = i + 1; } return result; } fn church_mul(a: u32, b: u32) -> u32 { let mut result: u32 = 0; let mut i: u32 = 0; while i < b { result = church_add(result, a); i = i + 1; } return result; } fn main() { println!("{} {} {}", church_add(3, 4), church_mul(3, 4), church_mul(0, 5)); }',
    '7 12 0'))
n += 1

entries.append((eid(n), "fixed-point-sim", "Fixed-point iteration simulation",
    'fn f(x: u32) -> u32 { return (x + 10) / 2; } fn fixed_point(start: u32, iters: u32) -> u32 { let mut x: u32 = start; let mut i: u32 = 0; while i < iters { x = f(x); i = i + 1; } return x; } fn main() { println!("{} {} {}", fixed_point(0, 10), fixed_point(100, 10), fixed_point(10, 1)); }',
    '9 9 10'))
n += 1

# U: Provably correct extremes
entries.append((eid(n), "binary-gcd", "Binary GCD (Stein algorithm)",
    'fn bgcd(mut a: u32, mut b: u32) -> u32 { if a == 0 { return b; } if b == 0 { return a; } let mut shift: u32 = 0; while (a + b) % 2 == 0 { a = a / 2; b = b / 2; shift = shift + 1; } while a % 2 == 0 { a = a / 2; } while b != 0 { while b % 2 == 0 { b = b / 2; } if a > b { let t: u32 = a; a = b; b = t; } b = b - a; } let mut result: u32 = a; let mut i: u32 = 0; while i < shift { result = result * 2; i = i + 1; } return result; } fn main() { println!("{} {} {}", bgcd(48, 18), bgcd(100, 75), bgcd(17, 13)); }',
    '6 25 1'))
n += 1

# W: C patterns
entries.append((eid(n), "c-bitfield-sim", "C bitfield extraction simulation",
    'fn extract_bits(val: u32, start: u32, width: u32) -> u32 { let mut shifted: u32 = val; let mut i: u32 = 0; while i < start { shifted = shifted / 2; i = i + 1; } let mut mask: u32 = 1; i = 1; while i < width { mask = mask * 2 + 1; i = i + 1; } return shifted % (mask + 1); } fn main() { println!("{} {} {}", extract_bits(255, 0, 4), extract_bits(255, 4, 4), extract_bits(170, 1, 3)); }',
    '15 15 5'))
n += 1

# X: Lua patterns
entries.append((eid(n), "lua-metatbl-sim", "Lua metatable __index simulation",
    'fn raw_get(tbl: u32, key: u32) -> u32 { if tbl == 0 { return 0; } return (tbl / key) % 10; } fn meta_get(tbl: u32, meta: u32, key: u32) -> u32 { let v: u32 = raw_get(tbl, key); if v != 0 { return v; } return raw_get(meta, key); } fn main() { println!("{} {}", meta_get(0, 42, 1), meta_get(100, 42, 1)); }',
    '2 0'))
n += 1

# MK entries (3)
MK = []
mk_n = n
MK.append((f"M-{START_ID + mk_n}", "make-dep-graph-sim", "Makefile: dependency graph resolution",
    'fn dep_order(a: u32, b: u32, c: u32) -> u32 { return a * 100 + b * 10 + c; } fn main() { println!("{} {}", dep_order(1, 2, 3), dep_order(3, 1, 2)); }',
    'dep_order() {'))
mk_n += 1
MK.append((f"M-{START_ID + mk_n}", "make-clean-target", "Makefile: clean target with force removal",
    'fn clean(artifacts: u32) -> u32 { return 0; } fn distclean(artifacts: u32, configs: u32) -> u32 { return 0; } fn main() { println!("{} {}", clean(42), distclean(42, 10)); }',
    'clean() {'))
mk_n += 1
MK.append((f"M-{START_ID + mk_n}", "make-phony-deps", "Makefile: phony target dependency chain",
    'fn lint(files: u32) -> u32 { return files; } fn test_run(files: u32) -> u32 { return files * 2; } fn check(files: u32) -> u32 { return lint(files) + test_run(files); } fn main() { println!("{}", check(10)); }',
    'lint() {'))
mk_n += 1

# DK entries (3)
DK = []
dk_n = mk_n
DK.append((f"D-{START_ID + dk_n}", "docker-scratch-copy", "Dockerfile: scratch image with COPY",
    'fn from_image(i: &str, t: &str) {} fn binary_size(opt_level: u32) -> u32 { return 1000 / (opt_level + 1); } fn main() { from_image("scratch", "latest"); println!("{} {}", binary_size(0), binary_size(2)); }',
    'FROM scratch:latest'))
dk_n += 1
DK.append((f"D-{START_ID + dk_n}", "docker-build-arg-chain", "Dockerfile: chained build arguments",
    'fn from_image(i: &str, t: &str) {} fn chain(a: u32, b: u32, c: u32) -> u32 { return a * 100 + b * 10 + c; } fn main() { from_image("python", "3.12"); println!("{}", chain(1, 2, 3)); }',
    'FROM python:3.12'))
dk_n += 1
DK.append((f"D-{START_ID + dk_n}", "docker-multi-from", "Dockerfile: multiple FROM stages",
    'fn from_image(i: &str, t: &str) {} fn stage_out(id: u32) -> u32 { return id * 10; } fn main() { from_image("rust", "1.75"); println!("{} {} {}", stage_out(1), stage_out(2), stage_out(3)); }',
    'FROM rust:1.75'))
dk_n += 1

# Output
print(f"    fn load_expansion{EXPANSION_NUM}_bash(&mut self) {{")
for entry_id, name, desc, rust_input, expected in entries:
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    print(f'            {fmt(rust_input)},')
    print(f'            {fmt(expected)}));')
print("    }")
print()
print(f"    fn load_expansion{EXPANSION_NUM}_makefile(&mut self) {{")
for entry_id, name, desc, rust_input, expected in MK:
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Makefile, CorpusTier::Adversarial,')
    print(f'            {fmt(rust_input)},')
    print(f'            {fmt(expected)}));')
print("    }")
print()
print(f"    fn load_expansion{EXPANSION_NUM}_dockerfile(&mut self) {{")
for entry_id, name, desc, rust_input, expected in DK:
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
    print(f'            {fmt(rust_input)},')
    print(f'            {fmt(expected)}));')
print("    }")

import sys
total = len(entries) + len(MK) + len(DK)
print(f"\n// Round 7: {len(entries)} bash + {len(MK)} makefile + {len(DK)} dockerfile = {total} entries", file=sys.stderr)
