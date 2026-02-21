#!/usr/bin/env python3
"""Round 7: 60 Bash entries + 5 Makefile + 5 Dockerfile across categories A-W.
Expected strings are carefully chosen to match current transpiler output format.
"""

def q(s):
    """Format a Rust raw string, using r##"..."## if '"#' present."""
    if '"#' in s:
        return f'r##"{s}"##'
    elif '"' in s or '\\' in s:
        return f'r#"{s}"#'
    else:
        return f'r#"{s}"#'

def entry(eid, name, desc, fmt, tier, code, expected):
    fmt_str = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}[fmt]
    tier_str = {"adversarial": "CorpusTier::Adversarial", "production": "CorpusTier::Production", "standard": "CorpusTier::Standard"}[tier]
    return f'            CorpusEntry::new("{eid}", "{name}", "{desc}",\n                {fmt_str}, {tier_str},\n                {q(code)},\n                {q(expected)}),'

bash_entries = []

# A. Shell redirection, pipes and flow
bash_entries.append(entry("B-1646", "r7-pipe-chain-sim", "Pipeline chain simulation with variable passing",
    "bash", "adversarial",
    r'fn stage1(x: i32) -> i32 { x + 10 } fn stage2(x: i32) -> i32 { x * 2 } fn stage3(x: i32) -> i32 { x - 5 } fn pipeline(v: i32) -> i32 { let a = stage1(v); let b = stage2(a); stage3(b) } fn main() { let r = pipeline(7); }',
    "stage1() {"))
bash_entries.append(entry("B-1647", "r7-redirect-append-sim", "File append simulation with counter",
    "bash", "adversarial",
    r'fn main() { let mut count = 0; for i in 1..=10 { count += i; } println!("total: {}", count); }',
    "rash_println"))
bash_entries.append(entry("B-1648", "r7-flow-control-complex", "Complex flow with nested if and loop",
    "bash", "adversarial",
    r'fn main() { let mut sum = 0; let mut i = 0; while i < 20 { if i % 3 == 0 { sum += i; } else if i % 5 == 0 { sum += i * 2; } i += 1; } }',
    'while [ "$i" -lt 20 ]; do'))

# B. Pathological quoting
bash_entries.append(entry("B-1649", "r7-nested-single-quotes", "Nested variable assignments with single-quoted strings",
    "bash", "standard",
    r'fn main() { let name = "hello world"; let greeting = "good morning"; let farewell = "goodbye"; }',
    "name='hello world'"))
bash_entries.append(entry("B-1650", "r7-empty-string-assign", "Empty string and whitespace assignments",
    "bash", "adversarial",
    r'fn main() { let empty = ""; let space = " "; }',
    "empty=''"))

# C. Pathological one-liners
bash_entries.append(entry("B-1651", "r7-one-liner-ternary-chain", "Ternary-style chain of conditionals",
    "bash", "adversarial",
    r'fn pick(a: i32, b: i32, c: i32) -> i32 { if a > b { if a > c { a } else { c } } else { if b > c { b } else { c } } } fn main() { let m = pick(10, 20, 15); }',
    "pick() {"))
bash_entries.append(entry("B-1652", "r7-one-liner-accumulate", "One-liner style accumulation pattern",
    "bash", "standard",
    r'fn main() { let a = 1; let b = a + 2; let c = b + 3; let d = c + 4; let e = d + 5; }',
    "b=$((a + 2))"))

# D. Pathological glob/wildcards + heredoc
bash_entries.append(entry("B-1653", "r7-glob-sim-patterns", "Glob-style pattern matching simulation",
    "bash", "adversarial",
    r'fn match_ext(ext: i32) -> i32 { match ext { 1 => { return 10; } 2 => { return 20; } 3 => { return 30; } _ => { return 0; } } } fn main() { let r1 = match_ext(1); let r2 = match_ext(2); let r3 = match_ext(99); }',
    'case "$ext" in'))

# E. Pathological environmental variables
bash_entries.append(entry("B-1654", "r7-env-default-values", "Environment variable defaults via function parameters",
    "bash", "adversarial",
    r'fn get_config(key: i32, default_val: i32) -> i32 { if key > 0 { key } else { default_val } } fn main() { let port = get_config(8080, 3000); let timeout = get_config(0, 30); }',
    "get_config() {"))
bash_entries.append(entry("B-1655", "r7-env-path-builder", "Path builder from segments",
    "bash", "standard",
    r'fn main() { let base = "/usr/local"; let bin_dir = "bin"; let lib_dir = "lib"; }',
    "base='/usr/local'"))

# F. SSH operations simulation
bash_entries.append(entry("B-1656", "r7-ssh-port-forward", "SSH port forward config calculation",
    "bash", "adversarial",
    r'fn local_port(base: i32, offset: i32) -> i32 { base + offset } fn remote_port(service: i32) -> i32 { if service == 1 { 22 } else if service == 2 { 80 } else if service == 3 { 443 } else { 8080 } } fn main() { let lp = local_port(10000, 22); let rp = remote_port(2); }',
    "local_port() {"))

# G. Pathological printing
bash_entries.append(entry("B-1657", "r7-println-format-int", "Formatted println with integer values",
    "bash", "standard",
    r'fn main() { let x = 42; let y = 7; let z = x + y; println!("sum: {}", z); }',
    "rash_println"))
bash_entries.append(entry("B-1658", "r7-eprintln-stderr", "Error printing to stderr",
    "bash", "adversarial",
    r'fn main() { let code = 1; if code != 0 { eprintln!("error: code {}", code); } println!("done"); }',
    "rash_eprintln"))

# H. Pathological awk/sed/grep simulation
bash_entries.append(entry("B-1659", "r7-field-extract", "Field extraction from structured data",
    "bash", "adversarial",
    r'fn extract_field(record: i32, field: i32) -> i32 { record / field } fn main() { let f1 = extract_field(100, 10); let f2 = extract_field(200, 5); let f3 = extract_field(350, 7); }',
    "extract_field() {"))

# I. Pathological data structures
bash_entries.append(entry("B-1660", "r7-stack-push-pop", "Stack simulation with push/pop operations",
    "bash", "adversarial",
    r'fn main() { let mut top = 0; let mut s0 = 0; let mut s1 = 0; let mut s2 = 0; s0 = 10; top = 1; s1 = 20; top = 2; s2 = 30; top = 3; top = top - 1; let popped = s2; }',
    "s0='10'"))
bash_entries.append(entry("B-1661", "r7-queue-enqueue", "Queue simulation with circular buffer",
    "bash", "adversarial",
    r'fn main() { let mut head = 0; let mut tail = 0; let mut q0 = 0; let mut q1 = 0; let mut q2 = 0; q0 = 100; tail = 1; q1 = 200; tail = 2; let dequeued = q0; head = 1; }',
    "q0='100'"))

# J. Pathological sourcing
bash_entries.append(entry("B-1662", "r7-config-load-sim", "Configuration loading simulation",
    "bash", "adversarial",
    r'fn load_defaults() -> i32 { 42 } fn load_override(base: i32, override_val: i32) -> i32 { if override_val > 0 { override_val } else { base } } fn main() { let base_val = load_defaults(); let final_val = load_override(base_val, 100); }',
    "load_defaults() {"))

# K. Pathological scripts
bash_entries.append(entry("B-1663", "r7-init-script-sim", "Init script simulation with phases",
    "bash", "adversarial",
    r'fn phase1() -> bool { true } fn phase2() -> bool { true } fn phase3() -> bool { true } fn main() { let p1 = phase1(); if p1 { let p2 = phase2(); if p2 { let p3 = phase3(); } } }',
    "phase1() {"))
bash_entries.append(entry("B-1664", "r7-health-check-loop", "Health check loop with retry",
    "bash", "adversarial",
    r'fn check_health(attempt: i32) -> bool { attempt > 3 } fn main() { let max_retries = 5; let mut attempt = 0; let mut healthy = false; while attempt < max_retries { attempt += 1; healthy = check_health(attempt); if healthy { break; } } }',
    "check_health() {"))

# L. Pathological braces, semicolons, control flow
bash_entries.append(entry("B-1665", "r7-deeply-nested-if", "3-level deep nested if statements",
    "bash", "adversarial",
    r'fn main() { let a = 1; let b = 2; let c = 3; if a > 0 { if b > 0 { if c > 0 { let result = a + b + c; } } } }',
    'if [ "$a" -gt 0 ]; then'))
bash_entries.append(entry("B-1666", "r7-multi-break-continue", "Multiple break and continue in nested loops",
    "bash", "adversarial",
    r'fn main() { let mut found = 0; for i in 0..10 { if i == 3 { continue; } if i == 7 { found = 1; break; } } }',
    "continue"))

# M. Pathological traps
bash_entries.append(entry("B-1667", "r7-cleanup-pattern", "Cleanup pattern with trap-like behavior",
    "bash", "adversarial",
    r'fn cleanup() { let status = 0; } fn do_work() -> i32 { 42 } fn main() { let result = do_work(); cleanup(); }',
    "cleanup() {"))

# N. Pathological command line parsing
bash_entries.append(entry("B-1668", "r7-arg-parser-sim", "Argument parser simulation",
    "bash", "adversarial",
    r'fn parse_flag(flag: i32) -> bool { flag == 1 } fn parse_value(raw: i32, default_v: i32) -> i32 { if raw > 0 { raw } else { default_v } } fn main() { let verbose = parse_flag(1); let threads = parse_value(4, 1); let timeout = parse_value(0, 30); }',
    "parse_flag() {"))
bash_entries.append(entry("B-1669", "r7-getopt-style", "Getopt-style option processing",
    "bash", "adversarial",
    r'fn is_option(val: i32) -> bool { val < 0 } fn get_option_value(opt: i32) -> i32 { if opt == -1 { 1 } else if opt == -2 { 2 } else { 0 } } fn main() { let a = is_option(-1); let b = get_option_value(-1); let c = get_option_value(-2); let d = get_option_value(5); }',
    "is_option() {"))

# Q. Numerical methods
bash_entries.append(entry("B-1670", "r7-newton-method-int", "Integer Newton method for sqrt",
    "bash", "adversarial",
    r'fn isqrt(n: i32) -> i32 { if n < 2 { return n; } let mut x = n; let mut y = (x + 1) / 2; while y < x { x = y; y = (x + n / x) / 2; } x } fn main() { let s1 = isqrt(144); let s2 = isqrt(200); let s3 = isqrt(0); let s4 = isqrt(1); }',
    "isqrt() {"))
bash_entries.append(entry("B-1671", "r7-bisection-method", "Integer bisection for finding roots",
    "bash", "adversarial",
    r'fn f(x: i32) -> i32 { x * x - 25 } fn bisect(lo: i32, hi: i32) -> i32 { let mut a = lo; let mut b = hi; while b - a > 1 { let mid = (a + b) / 2; if f(mid) <= 0 { a = mid; } else { b = mid; } } a } fn main() { let root = bisect(0, 10); }',
    "bisect() {"))
bash_entries.append(entry("B-1672", "r7-matrix-ops-flat", "Matrix operations via flat variables",
    "bash", "adversarial",
    r'fn mat_add(a: i32, b: i32) -> i32 { a + b } fn mat_scale(a: i32, k: i32) -> i32 { a * k } fn dot2(a1: i32, a2: i32, b1: i32, b2: i32) -> i32 { a1 * b1 + a2 * b2 } fn main() { let s = mat_add(10, 20); let sc = mat_scale(5, 3); let d = dot2(1, 2, 3, 4); }',
    "mat_add() {"))
bash_entries.append(entry("B-1673", "r7-running-stats", "Running mean and variance computation",
    "bash", "adversarial",
    r'fn running_mean(old_mean: i32, new_val: i32, n: i32) -> i32 { (old_mean * (n - 1) + new_val) / n } fn running_max(current_max: i32, new_val: i32) -> i32 { if new_val > current_max { new_val } else { current_max } } fn running_min(current_min: i32, new_val: i32) -> i32 { if new_val < current_min { new_val } else { current_min } } fn main() { let m = running_mean(10, 20, 2); let mx = running_max(50, 30); let mn = running_min(10, 5); }',
    "running_mean() {"))

# R. Pathological symbolic bash
bash_entries.append(entry("B-1674", "r7-bitwise-ops", "Bitwise operation simulation",
    "bash", "adversarial",
    r'fn bit_set(flags: i32, bit: i32) -> i32 { flags + bit } fn bit_clear(flags: i32, bit: i32) -> i32 { flags - bit } fn bit_test(flags: i32, bit: i32) -> bool { flags >= bit } fn main() { let f = 0; let f2 = bit_set(f, 4); let f3 = bit_set(f2, 2); let t = bit_test(f3, 4); }',
    "bit_set() {"))
bash_entries.append(entry("B-1675", "r7-boolean-algebra", "Boolean algebra operations",
    "bash", "adversarial",
    r'fn and_op(a: bool, b: bool) -> bool { a && b } fn or_op(a: bool, b: bool) -> bool { a || b } fn not_op(a: bool) -> bool { !a } fn main() { let r1 = and_op(true, false); let r2 = or_op(true, false); let r3 = not_op(true); }',
    "and_op() {"))

# S. Vi/editor simulation
bash_entries.append(entry("B-1676", "r7-editor-mode-sim", "Editor mode state machine",
    "bash", "adversarial",
    r'fn next_mode(current: i32, key: i32) -> i32 { if current == 0 { if key == 105 { 1 } else if key == 58 { 2 } else { 0 } } else if current == 1 { if key == 27 { 0 } else { 1 } } else { 0 } } fn main() { let m0 = 0; let m1 = next_mode(m0, 105); let m2 = next_mode(m1, 27); let m3 = next_mode(m2, 58); }',
    "next_mode() {"))

# T. Functions, closures, functional programming
bash_entries.append(entry("B-1677", "r7-compose-funcs", "Function composition chain",
    "bash", "adversarial",
    r'fn dbl(x: i32) -> i32 { x * 2 } fn inc(x: i32) -> i32 { x + 1 } fn sqr(x: i32) -> i32 { x * x } fn compose3(v: i32) -> i32 { sqr(inc(dbl(v))) } fn main() { let r1 = compose3(3); let r2 = compose3(5); let r3 = compose3(0); }',
    "compose3() {"))
bash_entries.append(entry("B-1678", "r7-higher-order-sim", "Higher-order function simulation via dispatch",
    "bash", "adversarial",
    r'fn apply_op(op: i32, a: i32, b: i32) -> i32 { if op == 1 { a + b } else if op == 2 { a - b } else if op == 3 { a * b } else if op == 4 { if b != 0 { a / b } else { 0 } } else { 0 } } fn main() { let r1 = apply_op(1, 10, 5); let r2 = apply_op(2, 10, 3); let r3 = apply_op(3, 4, 5); let r4 = apply_op(4, 20, 4); }',
    "apply_op() {"))
bash_entries.append(entry("B-1679", "r7-fold-reduce-sim", "Fold/reduce simulation over values",
    "bash", "adversarial",
    r'fn fold_add(acc: i32, val: i32) -> i32 { acc + val } fn fold_mul(acc: i32, val: i32) -> i32 { acc * val } fn fold_max(acc: i32, val: i32) -> i32 { if val > acc { val } else { acc } } fn main() { let mut sum = 0; sum = fold_add(sum, 10); sum = fold_add(sum, 20); sum = fold_add(sum, 30); let mut product = 1; product = fold_mul(product, 2); product = fold_mul(product, 3); product = fold_mul(product, 4); let mut mx = 0; mx = fold_max(mx, 15); mx = fold_max(mx, 42); mx = fold_max(mx, 8); }',
    "fold_add() {"))

# U. Miri-base provable code
bash_entries.append(entry("B-1680", "r7-provable-factorial", "Provably terminating factorial",
    "bash", "adversarial",
    r'fn factorial(n: i32) -> i32 { let mut result = 1; let mut i = 1; while i <= n { result = result * i; i += 1; } result } fn main() { let f5 = factorial(5); let f0 = factorial(0); let f1 = factorial(1); let f7 = factorial(7); }',
    "factorial() {"))
bash_entries.append(entry("B-1681", "r7-provable-gcd", "Provably terminating GCD via Euclidean algorithm",
    "bash", "adversarial",
    r'fn gcd(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x } fn lcm(a: i32, b: i32) -> i32 { a * b / gcd(a, b) } fn main() { let g1 = gcd(48, 18); let g2 = gcd(100, 75); let l1 = lcm(12, 18); }',
    "gcd() {"))
bash_entries.append(entry("B-1682", "r7-provable-fib", "Provably terminating Fibonacci",
    "bash", "adversarial",
    r'fn fib(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let tmp = b; b = a + b; a = tmp; i += 1; } a } fn main() { let f0 = fib(0); let f1 = fib(1); let f5 = fib(5); let f10 = fib(10); }',
    "fib() {"))

# V. Clippy pedantic patterns
bash_entries.append(entry("B-1683", "r7-clippy-no-unwrap", "Clippy-safe: no unwrap, explicit error handling",
    "bash", "standard",
    r'fn safe_div(a: i32, b: i32) -> i32 { if b == 0 { return 0; } a / b } fn safe_mod(a: i32, b: i32) -> i32 { if b == 0 { return 0; } a % b } fn main() { let d = safe_div(100, 7); let m = safe_mod(100, 7); let z = safe_div(42, 0); }',
    "safe_div() {"))
bash_entries.append(entry("B-1684", "r7-clippy-explicit-return", "Clippy-safe: explicit returns everywhere",
    "bash", "standard",
    r'fn min_val(a: i32, b: i32) -> i32 { if a < b { return a; } return b; } fn max_val(a: i32, b: i32) -> i32 { if a > b { return a; } return b; } fn clamp_v(v: i32, lo: i32, hi: i32) -> i32 { return min_val(max_val(v, lo), hi); } fn main() { let c = clamp_v(15, 0, 10); }',
    "min_val() {"))

# W. C mixed with Bash simulation
bash_entries.append(entry("B-1685", "r7-c-style-ptr-sim", "C-style pointer arithmetic simulation",
    "bash", "adversarial",
    r'fn ptr_offset(base_addr: i32, offset: i32, elem_size: i32) -> i32 { base_addr + offset * elem_size } fn ptr_diff(a: i32, b: i32, elem_size: i32) -> i32 { (a - b) / elem_size } fn main() { let base_addr = 1000; let p1 = ptr_offset(base_addr, 3, 4); let p2 = ptr_offset(base_addr, 7, 4); let diff = ptr_diff(p2, p1, 4); }',
    "ptr_offset() {"))
bash_entries.append(entry("B-1686", "r7-c-struct-sim", "C struct simulation with flat variables",
    "bash", "adversarial",
    r'fn point_dist_sq(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 { let dx = x2 - x1; let dy = y2 - y1; dx * dx + dy * dy } fn main() { let ax = 3; let ay = 4; let bx = 6; let by = 8; let d2 = point_dist_sq(ax, ay, bx, by); }',
    "point_dist_sq() {"))

# More hard entries across categories
bash_entries.append(entry("B-1687", "r7-pipeline-error-prop", "Pipeline error propagation pattern",
    "bash", "adversarial",
    r'fn step1(x: i32) -> i32 { if x < 0 { return -1; } x + 10 } fn step2(x: i32) -> i32 { if x < 0 { return -1; } x * 2 } fn step3(x: i32) -> i32 { if x < 0 { return -1; } x - 3 } fn main() { let a = step1(5); let b = step2(a); let c = step3(b); let err = step1(-1); let err2 = step2(err); }',
    "step1() {"))

bash_entries.append(entry("B-1688", "r7-state-machine-3", "Three-state machine with transitions",
    "bash", "adversarial",
    r'fn transition(state: i32, input: i32) -> i32 { if state == 0 { if input == 1 { 1 } else { 0 } } else if state == 1 { if input == 2 { 2 } else if input == 0 { 0 } else { 1 } } else { if input == 0 { 0 } else { 2 } } } fn main() { let s0 = 0; let s1 = transition(s0, 1); let s2 = transition(s1, 2); let s3 = transition(s2, 0); }',
    "transition() {"))

bash_entries.append(entry("B-1689", "r7-taylor-exp-int", "Taylor series exp approximation (integer)",
    "bash", "adversarial",
    r'fn int_exp_approx(x: i32, terms: i32) -> i32 { let mut result = 1000; let mut term = 1000; let mut i = 1; while i <= terms { term = term * x / (i * 1000); result = result + term; i += 1; } result } fn main() { let e1 = int_exp_approx(1000, 5); let e2 = int_exp_approx(500, 5); }',
    "int_exp_approx() {"))

bash_entries.append(entry("B-1690", "r7-map-sim", "Map/transform simulation over values",
    "bash", "adversarial",
    r'fn map_double(x: i32) -> i32 { x * 2 } fn map_negate(x: i32) -> i32 { 0 - x } fn map_square(x: i32) -> i32 { x * x } fn main() { let a = 5; let d = map_double(a); let n = map_negate(a); let s = map_square(a); let combo = map_double(map_square(3)); }',
    "map_double() {"))

bash_entries.append(entry("B-1691", "r7-priority-queue-sim", "Priority queue simulation",
    "bash", "adversarial",
    r'fn higher_priority(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn insert_check(current: i32, new_item: i32) -> i32 { higher_priority(current, new_item) } fn main() { let mut top = 999; top = insert_check(top, 50); top = insert_check(top, 10); top = insert_check(top, 30); top = insert_check(top, 5); }',
    "higher_priority() {"))

bash_entries.append(entry("B-1692", "r7-hash-sim", "Simple hash function simulation",
    "bash", "adversarial",
    r'fn hash_combine(h: i32, val: i32) -> i32 { h * 31 + val } fn hash_finalize(h: i32, buckets: i32) -> i32 { let positive = if h < 0 { 0 - h } else { h }; positive % buckets } fn main() { let mut h = 0; h = hash_combine(h, 72); h = hash_combine(h, 101); h = hash_combine(h, 108); let bucket = hash_finalize(h, 16); }',
    "hash_combine() {"))

bash_entries.append(entry("B-1693", "r7-bloom-filter-sim", "Bloom filter bit-set simulation",
    "bash", "adversarial",
    r'fn hash1(val: i32, sz: i32) -> i32 { val % sz } fn hash2(val: i32, sz: i32) -> i32 { (val * 7 + 3) % sz } fn check_bit(bits: i32, pos: i32) -> bool { bits >= pos } fn main() { let h1 = hash1(42, 64); let h2 = hash2(42, 64); let ok = check_bit(100, h1); }',
    "hash1() {"))

bash_entries.append(entry("B-1694", "r7-scheduler-sim", "Process scheduler simulation",
    "bash", "adversarial",
    r'fn round_robin_next(current: i32, total: i32) -> i32 { (current + 1) % total } fn priority_compare(p1: i32, p2: i32) -> i32 { if p1 < p2 { p1 } else { p2 } } fn time_slice(priority: i32, base_q: i32) -> i32 { base_q * priority } fn main() { let next = round_robin_next(3, 5); let better = priority_compare(2, 5); let sl = time_slice(3, 10); }',
    "round_robin_next() {"))

bash_entries.append(entry("B-1695", "r7-crypto-xor-sim", "XOR cipher simulation",
    "bash", "adversarial",
    r'fn xor_byte(data: i32, key: i32) -> i32 { let mut result = 0; let mut d = data; let mut k = key; let mut bit = 1; let mut i = 0; while i < 8 { let db = d % 2; let kb = k % 2; if db != kb { result = result + bit; } d = d / 2; k = k / 2; bit = bit * 2; i += 1; } result } fn main() { let encrypted = xor_byte(65, 42); let decrypted = xor_byte(encrypted, 42); }',
    "xor_byte() {"))

bash_entries.append(entry("B-1696", "r7-crc8-sim", "CRC-8 checksum simulation",
    "bash", "adversarial",
    r'fn crc_step(crc: i32, bit: i32) -> i32 { let xor_val = (crc + bit) % 2; if xor_val != 0 { (crc / 2) + 128 } else { crc / 2 } } fn crc_byte(crc: i32, byte: i32) -> i32 { let mut c = crc; let mut b = byte; let mut i = 0; while i < 8 { c = crc_step(c, b % 2); b = b / 2; i += 1; } c } fn main() { let c1 = crc_byte(0, 72); let c2 = crc_byte(c1, 101); }',
    "crc_step() {"))

bash_entries.append(entry("B-1697", "r7-binary-search-val", "Binary search for target value",
    "bash", "adversarial",
    r'fn binary_search(target: i32, sz: i32) -> i32 { let mut lo = 0; let mut hi = sz - 1; let mut result = -1; while lo <= hi { let mid = (lo + hi) / 2; if mid == target { result = mid; break; } else if mid < target { lo = mid + 1; } else { hi = mid - 1; } } result } fn main() { let found = binary_search(42, 100); let not_found = binary_search(101, 100); }',
    "binary_search() {"))

bash_entries.append(entry("B-1698", "r7-merge-sorted", "Merge two sorted sequences",
    "bash", "adversarial",
    r'fn min2(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn max2(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn merge_step(a: i32, b: i32) -> i32 { min2(a, b) } fn main() { let r1 = merge_step(10, 5); let r2 = merge_step(3, 7); let r3 = min2(max2(1, 2), max2(3, 0)); }',
    "min2() {"))

bash_entries.append(entry("B-1699", "r7-insertion-sort-step", "Single insertion sort step",
    "bash", "adversarial",
    r'fn should_swap(a: i32, b: i32) -> bool { a > b } fn swap_first(a: i32, b: i32) -> i32 { b } fn swap_second(a: i32, b: i32) -> i32 { a } fn main() { let x = 30; let y = 10; if should_swap(x, y) { let new_x = swap_first(x, y); let new_y = swap_second(x, y); } }',
    "should_swap() {"))

bash_entries.append(entry("B-1700", "r7-milestone-1700", "Milestone 1700: complex multi-function program",
    "bash", "adversarial",
    r'fn add_nums(a: i32, b: i32) -> i32 { a + b } fn sub_nums(a: i32, b: i32) -> i32 { a - b } fn mul_nums(a: i32, b: i32) -> i32 { a * b } fn safe_div(a: i32, b: i32) -> i32 { if b == 0 { 0 } else { a / b } } fn abs_val(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn clamp_val(v: i32, lo: i32, hi: i32) -> i32 { if v < lo { lo } else if v > hi { hi } else { v } } fn map_range(v: i32, in_lo: i32, in_hi: i32, out_lo: i32, out_hi: i32) -> i32 { let scaled = (v - in_lo) * (out_hi - out_lo); safe_div(scaled, in_hi - in_lo) + out_lo } fn main() { let r1 = map_range(50, 0, 100, 0, 255); let r2 = clamp_val(r1, 0, 255); let d = abs_val(sub_nums(r1, r2)); }',
    "map_range() {"))

bash_entries.append(entry("B-1701", "r7-huffman-freq-sim", "Huffman frequency counting simulation",
    "bash", "adversarial",
    r'fn count_bit(byte: i32, pos: i32) -> i32 { let mut val = byte; let mut i = 0; while i < pos { val = val / 2; i += 1; } val % 2 } fn popcount(byte: i32) -> i32 { let mut count = 0; let mut val = byte; while val > 0 { count = count + val % 2; val = val / 2; } count } fn main() { let b = 170; let bit3 = count_bit(b, 3); let ones = popcount(b); }',
    "count_bit() {"))

bash_entries.append(entry("B-1702", "r7-vm-fetch-decode", "Virtual machine fetch-decode cycle",
    "bash", "adversarial",
    r'fn fetch(pc: i32) -> i32 { pc * 100 + 42 } fn decode_op(instr: i32) -> i32 { instr / 100 } fn decode_arg(instr: i32) -> i32 { instr % 100 } fn execute(op: i32, arg: i32, acc: i32) -> i32 { if op == 1 { acc + arg } else if op == 2 { acc - arg } else if op == 3 { acc * arg } else { acc } } fn main() { let instr = fetch(1); let op = decode_op(instr); let arg = decode_arg(instr); let result = execute(op, arg, 0); }',
    "fetch() {"))

bash_entries.append(entry("B-1703", "r7-alloc-sim", "Memory allocator simulation",
    "bash", "adversarial",
    r'fn align_up(sz: i32, align: i32) -> i32 { let mask = align - 1; (sz + mask) / align * align } fn can_alloc(free: i32, request: i32) -> bool { free >= request } fn alloc(free: i32, request: i32) -> i32 { if can_alloc(free, request) { free - request } else { free } } fn main() { let aligned = align_up(13, 8); let remaining = alloc(1024, aligned); let remaining2 = alloc(remaining, align_up(100, 16)); }',
    "align_up() {"))

bash_entries.append(entry("B-1704", "r7-signal-process", "Signal processing: running average filter",
    "bash", "adversarial",
    r'fn filter_step(prev: i32, current: i32, alpha: i32) -> i32 { (alpha * current + (100 - alpha) * prev) / 100 } fn clamp_signal(val: i32, lo: i32, hi: i32) -> i32 { if val < lo { lo } else if val > hi { hi } else { val } } fn main() { let mut filtered = 100; filtered = filter_step(filtered, 150, 30); filtered = filter_step(filtered, 200, 30); filtered = clamp_signal(filtered, 0, 255); }',
    "filter_step() {"))

bash_entries.append(entry("B-1705", "r7-regex-match-sim", "Simple regex-like pattern match simulation",
    "bash", "adversarial",
    r'fn match_char(pattern: i32, input: i32) -> bool { pattern == input || pattern == 0 } fn match_2(p1: i32, p2: i32, i1: i32, i2: i32) -> bool { match_char(p1, i1) && match_char(p2, i2) } fn main() { let exact = match_char(65, 65); let wildcard = match_char(0, 65); let fail = match_char(65, 66); let m2 = match_2(65, 0, 65, 90); }',
    "match_char() {"))

# Makefile entries
makefile_entries = []
makefile_entries.append(entry("M-421", "r7-monorepo-build", "Monorepo build with workspace targets",
    "makefile", "adversarial",
    r'fn main() { exec(".PHONY: all crate-a crate-b crate-c"); exec("all: crate-a crate-b crate-c"); exec("crate-a:"); exec("\tcargo build -p crate-a"); exec("crate-b: crate-a"); exec("\tcargo build -p crate-b"); exec("crate-c: crate-b"); exec("\tcargo build -p crate-c"); }',
    "cargo build -p crate-a"))
makefile_entries.append(entry("M-422", "r7-db-migration", "Database migration targets",
    "makefile", "adversarial",
    r'fn main() { exec(".PHONY: migrate rollback seed"); exec("migrate:"); exec("\tsqlx migrate run"); exec("rollback:"); exec("\tsqlx migrate revert"); exec("seed: migrate"); exec("\tcargo run --bin seed"); }',
    "sqlx migrate run"))
makefile_entries.append(entry("M-423", "r7-wasm-build", "WebAssembly build pipeline",
    "makefile", "adversarial",
    r'fn main() { exec(".PHONY: wasm wasm-opt serve"); exec("wasm:"); exec("\twasm-pack build --target web"); exec("wasm-opt: wasm"); exec("\twasm-opt -O3 pkg/app_bg.wasm -o pkg/app_bg.wasm"); exec("serve: wasm-opt"); exec("\tpython3 -m http.server 8000"); }',
    "wasm-pack build --target web"))
makefile_entries.append(entry("M-424", "r7-bench-suite", "Benchmark suite with comparison",
    "makefile", "adversarial",
    r'fn main() { exec(".PHONY: bench bench-save bench-compare"); exec("bench:"); exec("\tcargo bench"); exec("bench-save:"); exec("\tcargo bench -- --save-baseline main"); exec("bench-compare:"); exec("\tcargo bench -- --baseline main"); }',
    "cargo bench"))
makefile_entries.append(entry("M-425", "r7-container-lifecycle", "Container lifecycle management",
    "makefile", "adversarial",
    r'fn main() { exec(".PHONY: build run stop clean"); exec("build:"); exec("\tdocker build -t myapp ."); exec("run: build"); exec("\tdocker run -d --name myapp -p 8080:8080 myapp"); exec("stop:"); exec("\tdocker stop myapp"); exec("clean: stop"); exec("\tdocker rm myapp"); }',
    "docker build -t myapp ."))

# Dockerfile entries
dockerfile_entries = []
dockerfile_entries.append(entry("D-381", "r7-elixir-phoenix", "Elixir Phoenix framework build",
    "dockerfile", "adversarial",
    r'fn main() { from_image("elixir:1.16-alpine"); workdir("/app"); copy("mix.exs mix.lock", "./"); run("mix deps.get"); copy(".", "."); run("mix compile"); expose(4000); cmd("mix phx.server"); }',
    "FROM elixir:1.16-alpine"))
dockerfile_entries.append(entry("D-382", "r7-dotnet-runtime", ".NET runtime container",
    "dockerfile", "adversarial",
    r'fn main() { from_image("mcr.microsoft.com/dotnet/sdk:8.0"); workdir("/src"); copy("*.csproj", "./"); run("dotnet restore"); copy(".", "."); run("dotnet publish -c Release -o /app"); }',
    "FROM mcr.microsoft.com/dotnet/sdk:8.0"))
dockerfile_entries.append(entry("D-383", "r7-python-uvicorn", "Python FastAPI with uvicorn",
    "dockerfile", "adversarial",
    r'fn main() { from_image("python:3.12-slim"); workdir("/app"); copy("requirements.txt", "."); run("pip install --no-cache-dir -r requirements.txt"); copy(".", "."); expose(8000); cmd("uvicorn main:app --host 0.0.0.0 --port 8000"); }',
    "FROM python:3.12-slim"))
dockerfile_entries.append(entry("D-384", "r7-redis-custom2", "Redis with custom config",
    "dockerfile", "adversarial",
    r'fn main() { from_image("redis:7-alpine"); copy("redis.conf", "/usr/local/etc/redis/redis.conf"); expose(6379); cmd("redis-server /usr/local/etc/redis/redis.conf"); }',
    "FROM redis:7-alpine"))
dockerfile_entries.append(entry("D-385", "r7-nginx-proxy", "Nginx reverse proxy config",
    "dockerfile", "adversarial",
    r'fn main() { from_image("nginx:alpine"); copy("nginx.conf", "/etc/nginx/nginx.conf"); expose(80); expose(443); cmd("nginx -g daemon off;"); }',
    "FROM nginx:alpine"))

# Generate output
print("    /// Round 7 Bash: B-1646..B-1705 — harder entries across categories A-W")
print("    fn load_expansion43_bash(&mut self) {")
print("        let entries = vec![")
for e in bash_entries:
    print(e)
print("        ];")
print("        for entry in entries {")
print("            self.entries.push(entry);")
print("        }")
print("    }")
print()
print("    /// Round 7 Makefile: M-421..M-425")
print("    fn load_expansion32_makefile(&mut self) {")
print("        let entries = vec![")
for e in makefile_entries:
    print(e)
print("        ];")
print("        for entry in entries {")
print("            self.entries.push(entry);")
print("        }")
print("    }")
print()
print("    /// Round 7 Dockerfile: D-381..D-385")
print("    fn load_expansion32_dockerfile(&mut self) {")
print("        let entries = vec![")
for e in dockerfile_entries:
    print(e)
print("        ];")
print("        for entry in entries {")
print("            self.entries.push(entry);")
print("        }")
print("    }")
