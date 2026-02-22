#!/usr/bin/env python3
"""Round 8: EXTREME pathological entries — push transpiler to failure boundaries.
Categories A-X with much harder constructs.
Expansion function: 186
START_ID: 16151
"""

START_ID = 16151

def format_rust_string(s):
    """Format a string for embedding in Rust source as raw string or regular string."""
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
# A. Shell redirection, pipes and flow (HARD)
# ============================================

# A1: 10-stage pipeline with alternating transforms
entries_bash.append(entry(f"B-{n}", "pipeline-10stage-alt", "Ten-stage alternating pipeline",
    "Bash",
    "fn add5(x: u32) -> u32 { return x + 5; } fn mul3(x: u32) -> u32 { return x * 3; } fn sub2(x: u32) -> u32 { if x > 2 { return x - 2; } return 0; } fn main() { let r: u32 = sub2(mul3(add5(sub2(mul3(add5(sub2(mul3(add5(sub2(1))))))))); println!(\"{}\", r); }",
    "1072"))
n += 1

# A2: Simulated pipe with filter and map
entries_bash.append(entry(f"B-{n}", "pipe-filter-map-reduce", "Pipeline: filter then map then reduce",
    "Bash",
    "fn is_odd(x: u32) -> u32 { return x % 2; } fn square(x: u32) -> u32 { return x * x; } fn main() { let mut sum: u32 = 0; let mut i: u32 = 1; while i <= 10 { if is_odd(i) != 0 { sum = sum + square(i); } i = i + 1; } println!(\"{}\", sum); }",
    "165"))
n += 1

# A3: Simulated fd duplication (2>&1)
entries_bash.append(entry(f"B-{n}", "fd-dup-sim", "File descriptor duplication simulation",
    "Bash",
    "fn fd_write(fd: u32, val: u32) -> u32 { return fd * 1000 + val; } fn fd_dup(src: u32, dst: u32) -> u32 { return src; } fn main() { let stdout: u32 = 1; let stderr: u32 = 2; let dup: u32 = fd_dup(stderr, stdout); println!(\"{} {} {}\", fd_write(stdout, 42), fd_write(stderr, 99), fd_write(dup, 77)); }",
    "1042 2099 2077"))
n += 1

# ============================================
# B. Pathological quoting (EXTREME)
# ============================================

# B1: Multiple escaping levels with function composition
entries_bash.append(entry(f"B-{n}", "escape-level-6deep", "Six levels of escape simulation",
    "Bash",
    "fn esc(x: u32, level: u32) -> u32 { if level == 0 { return x; } return esc(x + level * 11, level - 1); } fn main() { println!(\"{} {} {} {}\", esc(0, 1), esc(0, 3), esc(0, 5), esc(0, 6)); }",
    "11 66 165 231"))
n += 1

# B2: Quote depth with nested function calls
entries_bash.append(entry(f"B-{n}", "quote-nest-recursive", "Recursive quote depth calculation",
    "Bash",
    "fn quote_cost(depth: u32) -> u32 { if depth == 0 { return 1; } return 2 * quote_cost(depth - 1) + 1; } fn main() { println!(\"{} {} {} {} {}\", quote_cost(0), quote_cost(1), quote_cost(2), quote_cost(3), quote_cost(5)); }",
    "1 3 7 15 63"))
n += 1

# ============================================
# C. Pathological one-liners
# ============================================

# C1: 15 operations in chain
entries_bash.append(entry(f"B-{n}", "oneliner-15op-chain", "Fifteen chained arithmetic operations",
    "Bash",
    "fn main() { let r: u32 = (((((((((((((1 + 2) * 3 + 4) * 5 + 6) * 7 + 8) * 9 + 10) % 1000 + 11) * 2 + 12) % 500 + 13) * 3 + 14) % 200 + 15) * 4 + 16) % 100 + 17) * 5; println!(\"{}\", r); }",
    "465"))
n += 1

# C2: Deeply nested ternary simulation
entries_bash.append(entry(f"B-{n}", "ternary-8deep", "Eight levels of ternary nesting",
    "Bash",
    "fn pick(c: u32, a: u32, b: u32) -> u32 { if c != 0 { return a; } return b; } fn main() { let r: u32 = pick(1, pick(0, 10, pick(1, 20, pick(0, 30, pick(1, 40, pick(0, 50, pick(1, 60, pick(0, 70, 80))))))), 90); println!(\"{}\", r); }",
    "60"))
n += 1

# ============================================
# D. Pathological glob and wildcards
# ============================================

# D1: Glob pattern scoring with exponential cost
entries_bash.append(entry(f"B-{n}", "glob-exponential-cost", "Glob matching with exponential backtracking cost",
    "Bash",
    "fn glob_cost(stars: u32) -> u32 { let mut cost: u32 = 1; let mut i: u32 = 0; while i < stars { cost = cost * 2; i = i + 1; } return cost; } fn total_cost(patterns: u32, avg_stars: u32) -> u32 { return patterns * glob_cost(avg_stars); } fn main() { println!(\"{} {} {} {}\", glob_cost(1), glob_cost(5), glob_cost(10), total_cost(3, 4)); }",
    "2 32 1024 48"))
n += 1

# D2: Brace expansion cardinality
entries_bash.append(entry(f"B-{n}", "brace-expand-card", "Brace expansion cardinality computation",
    "Bash",
    "fn brace_card(sets: u32, size_per_set: u32) -> u32 { let mut card: u32 = 1; let mut i: u32 = 0; while i < sets { card = card * size_per_set; i = i + 1; } return card; } fn main() { println!(\"{} {} {} {}\", brace_card(1, 3), brace_card(2, 3), brace_card(3, 2), brace_card(4, 5)); }",
    "3 9 8 625"))
n += 1

# ============================================
# E. Pathological heredoc
# ============================================

# E1: Heredoc with nested delimiter counting
entries_bash.append(entry(f"B-{n}", "heredoc-delim-count", "Heredoc delimiter nesting count simulation",
    "Bash",
    "fn delim_depth(raw: u32) -> u32 { let mut d: u32 = 0; let mut v: u32 = raw; while v > 0 { if v % 3 == 0 { d = d + 1; } v = v / 3; } return d; } fn main() { println!(\"{} {} {} {} {}\", delim_depth(1), delim_depth(9), delim_depth(27), delim_depth(81), delim_depth(100)); }",
    "0 2 3 4 2"))
n += 1

# ============================================
# F. Pathological environment variables
# ============================================

# F1: Environment variable cascading with 5 levels
entries_bash.append(entry(f"B-{n}", "env-cascade-5level", "Five-level environment variable cascade",
    "Bash",
    "fn env5(k: u32) -> u32 { return 500; } fn env4(k: u32) -> u32 { if k == 4 { return 400; } return env5(k); } fn env3(k: u32) -> u32 { if k == 3 { return 300; } return env4(k); } fn env2(k: u32) -> u32 { if k == 2 { return 200; } return env3(k); } fn env1(k: u32) -> u32 { if k == 1 { return 100; } return env2(k); } fn main() { println!(\"{} {} {} {} {}\", env1(1), env1(2), env1(3), env1(4), env1(5)); }",
    "100 200 300 400 500"))
n += 1

# ============================================
# G. SSH operations
# ============================================

# G1: SSH jump host chain simulation
entries_bash.append(entry(f"B-{n}", "ssh-jump-chain-3hop", "Three-hop SSH jump chain simulation",
    "Bash",
    "fn hop(port: u32, next: u32) -> u32 { return port * 100 + next; } fn jump3(p1: u32, p2: u32, p3: u32, dest: u32) -> u32 { return hop(p1, hop(p2, hop(p3, dest))); } fn main() { println!(\"{}\", jump3(22, 2222, 22222, 80)); }",
    "222222228080"))
n += 1

# ============================================
# H. Pathological printing
# ============================================

# H1: Printf simulation with format codes
entries_bash.append(entry(f"B-{n}", "printf-fmt-sim", "Printf format code simulation",
    "Bash",
    "fn fmt_d(val: u32) -> u32 { return val; } fn fmt_x(val: u32) -> u32 { return val; } fn fmt_o(val: u32) -> u32 { return val; } fn fmt_pad(val: u32, width: u32) -> u32 { return width * 100000 + val; } fn main() { println!(\"{} {} {} {}\", fmt_d(255), fmt_x(255), fmt_o(255), fmt_pad(42, 10)); }",
    "255 255 255 1000042"))
n += 1

# ============================================
# I. Pathological awk/sed/grep/tr
# ============================================

# I1: NFA simulation for regex matching
entries_bash.append(entry(f"B-{n}", "nfa-sim-3state", "Three-state NFA regex simulation",
    "Bash",
    "fn nfa_step(state: u32, input: u32) -> u32 { if state == 0 { if input == 1 { return 1; } return 0; } if state == 1 { if input == 2 { return 2; } return 0; } if state == 2 { return 2; } return 0; } fn nfa_run(inputs: u32, count: u32) -> u32 { let mut s: u32 = 0; let mut v: u32 = inputs; let mut i: u32 = 0; while i < count { s = nfa_step(s, v % 10); v = v / 10; i = i + 1; } return s; } fn main() { println!(\"{} {} {}\", nfa_run(12, 2), nfa_run(123, 3), nfa_run(111, 3)); }",
    "2 0 0"))
n += 1

# I2: Awk field split simulation
entries_bash.append(entry(f"B-{n}", "awk-nf-sim", "Awk NF (number of fields) simulation",
    "Bash",
    "fn count_fields(encoded: u32, sep: u32) -> u32 { if encoded == 0 { return 0; } let mut count: u32 = 1; let mut v: u32 = encoded; while v > 0 { if v % 10 == sep { count = count + 1; } v = v / 10; } return count; } fn main() { println!(\"{} {} {} {}\", count_fields(12345, 0), count_fields(10203, 0), count_fields(0, 0), count_fields(11111, 1)); }",
    "1 3 0 6"))
n += 1

# ============================================
# J. Pathological data structures
# ============================================

# J1: Stack simulation with push/pop/peek
entries_bash.append(entry(f"B-{n}", "stack-push-pop-3op", "Stack with 3 operations simulation",
    "Bash",
    "fn push(stack: u32, val: u32) -> u32 { return stack * 100 + val; } fn pop(stack: u32) -> u32 { return stack / 100; } fn peek(stack: u32) -> u32 { return stack % 100; } fn main() { let mut s: u32 = 0; s = push(s, 10); s = push(s, 20); s = push(s, 30); println!(\"{}\", peek(s)); s = pop(s); println!(\"{}\", peek(s)); s = pop(s); println!(\"{}\", peek(s)); }",
    "30"))
n += 1

# J2: Binary tree height calculation
entries_bash.append(entry(f"B-{n}", "btree-height-sim", "Binary tree height calculation",
    "Bash",
    "fn max_val(a: u32, b: u32) -> u32 { if a > b { return a; } return b; } fn tree_height(nodes: u32) -> u32 { if nodes <= 1 { return 0; } let mut h: u32 = 0; let mut n: u32 = nodes; while n > 1 { n = n / 2; h = h + 1; } return h; } fn main() { println!(\"{} {} {} {} {}\", tree_height(1), tree_height(3), tree_height(7), tree_height(15), tree_height(100)); }",
    "0 1 2 3 6"))
n += 1

# ============================================
# K. Pathological sourcing/modules
# ============================================

# K1: Module dependency order resolution
entries_bash.append(entry(f"B-{n}", "module-dep-order", "Module dependency topological order simulation",
    "Bash",
    "fn dep_level(mod_id: u32) -> u32 { if mod_id <= 1 { return 0; } if mod_id <= 3 { return 1; } if mod_id <= 7 { return 2; } return 3; } fn load_order(n_mods: u32) -> u32 { let mut total: u32 = 0; let mut i: u32 = 1; while i <= n_mods { total = total + dep_level(i); i = i + 1; } return total; } fn main() { println!(\"{} {} {} {}\", load_order(1), load_order(3), load_order(7), load_order(10)); }",
    "0 2 10 19"))
n += 1

# ============================================
# L. Pathological braces/semicolons/control flow
# ============================================

# L1: Deeply nested while loops (5 deep)
entries_bash.append(entry(f"B-{n}", "while-nest-5deep", "Five nested while loops",
    "Bash",
    "fn main() { let mut total: u32 = 0; let mut a: u32 = 0; while a < 3 { let mut b: u32 = 0; while b < 3 { let mut c: u32 = 0; while c < 3 { total = total + 1; c = c + 1; } b = b + 1; } a = a + 1; } println!(\"{}\", total); }",
    "27"))
n += 1

# L2: If-else chain 10 deep
entries_bash.append(entry(f"B-{n}", "if-chain-10deep", "Ten-deep if-else chain",
    "Bash",
    "fn classify(x: u32) -> u32 { if x < 10 { return 1; } if x < 20 { return 2; } if x < 30 { return 3; } if x < 40 { return 4; } if x < 50 { return 5; } if x < 60 { return 6; } if x < 70 { return 7; } if x < 80 { return 8; } if x < 90 { return 9; } return 10; } fn main() { println!(\"{} {} {} {} {}\", classify(5), classify(25), classify(55), classify(85), classify(99)); }",
    "1 3 6 9 10"))
n += 1

# ============================================
# M. Pathological traps
# ============================================

# M1: Signal handler priority queue simulation
entries_bash.append(entry(f"B-{n}", "trap-priority-queue", "Signal priority queue simulation",
    "Bash",
    "fn priority(sig: u32) -> u32 { if sig == 9 { return 100; } if sig == 15 { return 90; } if sig == 2 { return 80; } if sig == 1 { return 70; } if sig == 3 { return 60; } return 0; } fn dispatch_first(s1: u32, s2: u32, s3: u32) -> u32 { let p1: u32 = priority(s1); let p2: u32 = priority(s2); let p3: u32 = priority(s3); if p1 >= p2 { if p1 >= p3 { return s1; } return s3; } if p2 >= p3 { return s2; } return s3; } fn main() { println!(\"{} {} {}\", dispatch_first(2, 15, 1), dispatch_first(9, 15, 2), dispatch_first(1, 3, 2)); }",
    "15 9 2"))
n += 1

# ============================================
# N. Pathological command line parsing
# ============================================

# N1: Getopt long option parser simulation
entries_bash.append(entry(f"B-{n}", "getopt-long-sim", "Getopt long option parser simulation",
    "Bash",
    "fn parse_long(opt_hash: u32, has_value: u32, value: u32) -> u32 { if has_value != 0 { return opt_hash * 10000 + value; } return opt_hash * 10000; } fn opt_verbose() -> u32 { return 1; } fn opt_output() -> u32 { return 2; } fn opt_debug() -> u32 { return 3; } fn main() { println!(\"{} {} {}\", parse_long(opt_verbose(), 0, 0), parse_long(opt_output(), 1, 42), parse_long(opt_debug(), 0, 0)); }",
    "10000 20042 30000"))
n += 1

# ============================================
# O. Pathological nested Makefiles
# ============================================
# (handled in makefile entries below)

# ============================================
# P. Pathological Dockerfiles with stages
# ============================================
# (handled in dockerfile entries below)

# ============================================
# Q. Numerical methods (EXTREME)
# ============================================

# Q1: Newton's method integer approximation for square root
entries_bash.append(entry(f"B-{n}", "newton-sqrt-int", "Newton's method integer square root",
    "Bash",
    "fn isqrt(n: u32) -> u32 { if n == 0 { return 0; } let mut x: u32 = n; let mut y: u32 = (x + 1) / 2; while y < x { x = y; y = (x + n / x) / 2; } return x; } fn main() { println!(\"{} {} {} {} {}\", isqrt(0), isqrt(1), isqrt(4), isqrt(100), isqrt(1000)); }",
    "0 1 2 10 31"))
n += 1

# Q2: Simpson's rule for integration approximation (integer)
entries_bash.append(entry(f"B-{n}", "simpson-approx-int", "Simpson's rule integer approximation",
    "Bash",
    "fn f(x: u32) -> u32 { return x * x; } fn simpson(a: u32, b: u32, n: u32) -> u32 { let h: u32 = (b - a) / n; let mut sum: u32 = f(a) + f(b); let mut i: u32 = 1; while i < n { let x: u32 = a + i * h; if i % 2 == 0 { sum = sum + 2 * f(x); } else { sum = sum + 4 * f(x); } i = i + 1; } return sum * h / 3; } fn main() { println!(\"{}\", simpson(0, 10, 10)); }",
    "334"))
n += 1

# ============================================
# R. Pathological symbolic bash (&&, ||, !, $)
# ============================================

# R1: Boolean logic simulation with truth table
entries_bash.append(entry(f"B-{n}", "bool-truth-table-3var", "Three-variable truth table evaluation",
    "Bash",
    "fn eval_expr(a: u32, b: u32, c: u32) -> u32 { if a != 0 { if b != 0 { return 1; } if c != 0 { return 1; } return 0; } return 0; } fn main() { println!(\"{} {} {} {} {} {} {} {}\", eval_expr(0, 0, 0), eval_expr(0, 0, 1), eval_expr(0, 1, 0), eval_expr(0, 1, 1), eval_expr(1, 0, 0), eval_expr(1, 0, 1), eval_expr(1, 1, 0), eval_expr(1, 1, 1)); }",
    "0 0 0 0 0 1 1 1"))
n += 1

# ============================================
# S. Editor commands (vi/vim/ed)
# ============================================

# S1: Ed-like line addressing simulation
entries_bash.append(entry(f"B-{n}", "ed-line-addr-sim", "Ed-style line address resolution",
    "Bash",
    "fn addr_absolute(n: u32) -> u32 { return n; } fn addr_relative(cur: u32, offset: u32) -> u32 { return cur + offset; } fn addr_end(total: u32) -> u32 { return total; } fn addr_range(start: u32, end: u32) -> u32 { if end >= start { return end - start + 1; } return 0; } fn main() { let total: u32 = 100; let cur: u32 = 50; println!(\"{} {} {} {}\", addr_absolute(10), addr_relative(cur, 5), addr_end(total), addr_range(10, 20)); }",
    "10 55 100 11"))
n += 1

# ============================================
# T. Functions, closures, functional programming
# ============================================

# T1: Compose simulation
entries_bash.append(entry(f"B-{n}", "compose-3fn", "Composition of three functions",
    "Bash",
    "fn inc(x: u32) -> u32 { return x + 1; } fn dbl(x: u32) -> u32 { return x * 2; } fn sqr(x: u32) -> u32 { return x * x; } fn compose_idc(x: u32) -> u32 { return sqr(dbl(inc(x))); } fn compose_dis(x: u32) -> u32 { return inc(dbl(sqr(x))); } fn main() { println!(\"{} {} {} {}\", compose_idc(2), compose_idc(5), compose_dis(2), compose_dis(3)); }",
    "36 144 9 19"))
n += 1

# T2: Y-combinator-like fixed point via iteration
entries_bash.append(entry(f"B-{n}", "fixpoint-iter-sim", "Fixed point iteration simulation",
    "Bash",
    "fn step(x: u32) -> u32 { return (x + 100 / x) / 2; } fn fixpoint(start: u32, iters: u32) -> u32 { let mut x: u32 = start; let mut i: u32 = 0; while i < iters { let next: u32 = step(x); if next == x { return x; } x = next; i = i + 1; } return x; } fn main() { println!(\"{} {}\", fixpoint(50, 20), fixpoint(1, 20)); }",
    "10 10"))
n += 1

# ============================================
# U. Provable code (Miri-safe)
# ============================================

# U1: Provably terminating Collatz bounded
entries_bash.append(entry(f"B-{n}", "collatz-bounded-proof", "Collatz with bounded iteration proof",
    "Bash",
    "fn collatz_steps(mut n: u32, max_steps: u32) -> u32 { let mut steps: u32 = 0; while n != 1 { if steps >= max_steps { return max_steps; } if n % 2 == 0 { n = n / 2; } else { n = 3 * n + 1; } steps = steps + 1; } return steps; } fn main() { println!(\"{} {} {} {}\", collatz_steps(1, 100), collatz_steps(6, 100), collatz_steps(27, 200), collatz_steps(1000, 200)); }",
    "0 8 111 111"))
n += 1

# ============================================
# V. Extreme clippy (pedantic patterns)
# ============================================

# V1: Manual impl of checked arithmetic
entries_bash.append(entry(f"B-{n}", "checked-arith-manual", "Manual checked arithmetic (clippy-safe)",
    "Bash",
    "fn safe_add(a: u32, b: u32) -> u32 { if a > 4294967295 - b { return 4294967295; } return a + b; } fn safe_mul(a: u32, b: u32) -> u32 { if b == 0 { return 0; } if a > 4294967295 / b { return 4294967295; } return a * b; } fn main() { println!(\"{} {} {}\", safe_add(10, 20), safe_mul(100, 200), safe_add(0, 0)); }",
    "30 20000 0"))
n += 1

# ============================================
# W. C mixed with Bash
# ============================================

# W1: C sizeof simulation
entries_bash.append(entry(f"B-{n}", "c-sizeof-sim", "C sizeof type simulation",
    "Bash",
    "fn sizeof_char() -> u32 { return 1; } fn sizeof_short() -> u32 { return 2; } fn sizeof_int() -> u32 { return 4; } fn sizeof_long() -> u32 { return 8; } fn sizeof_ptr() -> u32 { return 8; } fn sizeof_array(elem_size: u32, count: u32) -> u32 { return elem_size * count; } fn main() { println!(\"{} {} {} {} {} {}\", sizeof_char(), sizeof_short(), sizeof_int(), sizeof_long(), sizeof_ptr(), sizeof_array(sizeof_int(), 10)); }",
    "1 2 4 8 8 40"))
n += 1

# W2: C struct padding simulation
entries_bash.append(entry(f"B-{n}", "c-struct-padding", "C struct alignment padding simulation",
    "Bash",
    "fn align_up(size: u32, align: u32) -> u32 { return ((size + align - 1) / align) * align; } fn struct_size(f1: u32, a1: u32, f2: u32, a2: u32, f3: u32, a3: u32) -> u32 { let mut off: u32 = f1; off = align_up(off, a2) + f2; off = align_up(off, a3) + f3; let max_align: u32 = if a1 > a2 { if a1 > a3 { a1 } else { a3 } } else { if a2 > a3 { a2 } else { a3 } }; return align_up(off, max_align); } fn main() { println!(\"{} {}\", struct_size(1, 1, 4, 4, 1, 1), struct_size(1, 1, 8, 8, 1, 1)); }",
    "12 24"))
n += 1

# ============================================
# X. Lua mixed with Bash
# ============================================

# X1: Lua-style string.rep simulation
entries_bash.append(entry(f"B-{n}", "lua-string-rep-sim", "Lua string.rep length simulation",
    "Bash",
    "fn string_rep_len(base_len: u32, count: u32) -> u32 { return base_len * count; } fn string_rep_cat(len1: u32, len2: u32) -> u32 { return len1 + len2; } fn main() { println!(\"{} {} {}\", string_rep_len(5, 3), string_rep_cat(string_rep_len(3, 4), string_rep_len(2, 5)), string_rep_len(0, 100)); }",
    "15 22 0"))
n += 1

# X2: Lua-style table.sort partition simulation
entries_bash.append(entry(f"B-{n}", "lua-qsort-partition", "Lua quicksort partition count simulation",
    "Bash",
    "fn partition_count(n: u32) -> u32 { if n <= 1 { return 0; } return 1 + partition_count(n / 2) + partition_count(n - n / 2 - 1); } fn main() { println!(\"{} {} {} {}\", partition_count(1), partition_count(4), partition_count(8), partition_count(16)); }",
    "0 3 7 15"))
n += 1

# X3: Lua-style gsub count simulation
entries_bash.append(entry(f"B-{n}", "lua-gsub-count", "Lua gsub replacement count simulation",
    "Bash",
    "fn gsub_count(text_len: u32, pattern_len: u32) -> u32 { if pattern_len == 0 { return text_len + 1; } return text_len / pattern_len; } fn gsub_result_len(text_len: u32, pattern_len: u32, repl_len: u32) -> u32 { let matches: u32 = gsub_count(text_len, pattern_len); return text_len - matches * pattern_len + matches * repl_len; } fn main() { println!(\"{} {} {}\", gsub_count(100, 5), gsub_result_len(100, 5, 10), gsub_result_len(50, 2, 4)); }",
    "20 200 100"))
n += 1

bash_count = len(entries_bash)
last_bash_id = n - 1

# ============================================
# Makefile entries (O. Nested Makefiles)
# ============================================

entries_make.append(entry(f"M-{n}", "make-recursive-3level", "Makefile: 3-level recursive make simulation",
    "Makefile",
    "fn make_sub(dir: u32, target: u32) -> u32 { return dir * 100 + target; } fn make_recursive(depth: u32, base: u32) -> u32 { if depth == 0 { return base; } return make_recursive(depth - 1, make_sub(depth, base)); } fn main() { println!(\"{} {}\", make_recursive(1, 42), make_recursive(3, 1)); }",
    "make_sub() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-vpath-resolve", "Makefile: VPATH source file resolution",
    "Makefile",
    "fn vpath_search(dirs: u32, file: u32) -> u32 { return dirs * 1000 + file; } fn main() { println!(\"{} {}\", vpath_search(3, 42), vpath_search(1, 100)); }",
    "vpath_search() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-pattern-rule-sim", "Makefile: pattern rule matching simulation",
    "Makefile",
    "fn pattern_match(stem: u32, suffix_in: u32, suffix_out: u32) -> u32 { return stem * 100 + suffix_out; } fn main() { println!(\"{} {}\", pattern_match(42, 1, 2), pattern_match(99, 3, 4)); }",
    "pattern_match() {"))
n += 1

make_count = len(entries_make)

# ============================================
# Dockerfile entries (P. Multi-stage)
# ============================================

entries_docker.append(entry(f"D-{n}", "docker-3stage-build", "Dockerfile: 3-stage build with size tracking",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn stage_size(base: u32, added: u32) -> u32 { return base + added; } fn copy_from(src_stage: u32, size: u32) -> u32 { return size; } fn main() { from_image("rust", "1.75"); let build: u32 = stage_size(1000, 500); let test: u32 = stage_size(build, 100); let prod: u32 = copy_from(2, 50); println!("{} {} {}", build, test, prod); }',
    "FROM rust:1.75"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-cache-bust-sim", "Dockerfile: cache busting strategy simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn layer_hash(content: u32, prev: u32) -> u32 { return (content + prev) % 10000; } fn main() { from_image("ubuntu", "22.04"); let l1: u32 = layer_hash(100, 0); let l2: u32 = layer_hash(200, l1); let l3: u32 = layer_hash(300, l2); println!("{} {} {}", l1, l2, l3); }',
    "FROM ubuntu:22.04"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-healthcheck-sim", "Dockerfile: healthcheck interval simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn health_check(interval: u32, timeout: u32, retries: u32) -> u32 { return interval * retries + timeout; } fn main() { from_image("nginx", "alpine"); println!("{} {}", health_check(30, 5, 3), health_check(10, 2, 5)); }',
    "FROM nginx:alpine"))
n += 1

docker_count = len(entries_docker)
total = bash_count + make_count + docker_count

print(f"// Round 8: {bash_count} bash + {make_count} makefile + {docker_count} dockerfile = {total} entries")
print(f"// IDs: B-{START_ID}..D-{n-1}")
print(f"// Expansion function: 186")
print()

# Print bash function
print("    fn load_expansion186_bash(&mut self) {")
for e in entries_bash:
    print(e)
print("    }")
print()

# Print makefile function
print("    fn load_expansion186_makefile(&mut self) {")
for e in entries_make:
    print(e)
print("    }")
print()

# Print dockerfile function
print("    fn load_expansion186_dockerfile(&mut self) {")
for e in entries_docker:
    print(e)
print("    }")
