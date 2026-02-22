#!/usr/bin/env python3
"""Round 6: Even harder pathological entries.
Categories: A-X with extreme patterns pushing transpiler limits."""

START_ID = 16037
EXPANSION_NUM = 184

def fmt(s):
    if '"' in s and '#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

def eid(n):
    return f"B-{START_ID + n}"

entries = []
n = 0

# A: Complex multi-stage pipeline simulation
entries.append((eid(n), "pipeline-7stage", "Seven-stage data pipeline with transforms",
    'fn s1(x: u32) -> u32 { return x + 1; } fn s2(x: u32) -> u32 { return x * 2; } fn s3(x: u32) -> u32 { return x + 3; } fn s4(x: u32) -> u32 { return x * 4; } fn s5(x: u32) -> u32 { if x > 100 { return x - 50; } return x; } fn s6(x: u32) -> u32 { return x + 7; } fn s7(x: u32) -> u32 { return x % 100; } fn main() { println!("{}", s7(s6(s5(s4(s3(s2(s1(5)))))))); }',
    '62'))
n += 1

entries.append((eid(n), "tee-simulate", "Simulated tee: process input two ways",
    'fn upper(x: u32) -> u32 { return x * 10; } fn lower(x: u32) -> u32 { return x + 1; } fn combine(a: u32, b: u32) -> u32 { return a + b; } fn main() { let v: u32 = 5; println!("{}", combine(upper(v), lower(v))); }',
    '56'))
n += 1

# B: Pathological quoting patterns
entries.append((eid(n), "nested-escape-sim", "Simulated nested escaping levels",
    'fn esc1(x: u32) -> u32 { return x + 92; } fn esc2(x: u32) -> u32 { return esc1(esc1(x)); } fn esc3(x: u32) -> u32 { return esc2(esc2(x)); } fn main() { println!("{} {} {}", esc1(0), esc2(0), esc3(0)); }',
    '92 184 368'))
n += 1

entries.append((eid(n), "quote-depth-chain", "Quote depth tracking simulation",
    'fn depth(level: u32, val: u32) -> u32 { if level == 0 { return val; } return depth(level - 1, val + level); } fn main() { println!("{} {} {} {}", depth(0, 10), depth(1, 10), depth(3, 10), depth(5, 10)); }',
    '10 11 16 25'))
n += 1

# C: Pathological one-liners
entries.append((eid(n), "ternary-chain-10", "Ten-deep ternary-like chain",
    'fn t(c: u32, a: u32, b: u32) -> u32 { if c != 0 { return a; } return b; } fn main() { println!("{}", t(1, t(0, 10, t(1, 20, 30)), t(1, t(0, 40, 50), 60))); }',
    '20'))
n += 1

entries.append((eid(n), "oneliner-math-10op", "Ten arithmetic operations in one expression",
    'fn main() { let r: u32 = ((((((((1 + 2) * 3) + 4) * 5) + 6) * 7) + 8) * 9 + 10); println!("{}", r); }',
    '10000'))
n += 1

# D: Pathological glob/wildcard simulation
entries.append((eid(n), "glob-match-sim", "Glob pattern matching simulation",
    'fn match_star(len: u32) -> u32 { return len; } fn match_q(pos: u32) -> u32 { return 1; } fn glob_score(stars: u32, questions: u32, literals: u32) -> u32 { return stars * 100 + questions * 10 + literals; } fn main() { println!("{} {} {}", glob_score(2, 3, 5), glob_score(0, 0, 10), glob_score(1, 1, 0)); }',
    '235 10 110'))
n += 1

# E: Pathological heredoc simulation
entries.append((eid(n), "heredoc-indent-sim", "Heredoc indentation level simulation",
    'fn indent(level: u32, content: u32) -> u32 { return level * 1000 + content; } fn main() { println!("{} {} {}", indent(0, 42), indent(2, 42), indent(4, 42)); }',
    '42 2042 4042'))
n += 1

entries.append((eid(n), "heredoc-var-expand", "Heredoc variable expansion simulation",
    'fn expand(var: u32, quoted: u32) -> u32 { if quoted != 0 { return 0; } return var; } fn main() { println!("{} {} {} {}", expand(42, 0), expand(42, 1), expand(100, 0), expand(100, 1)); }',
    '42 0 100 0'))
n += 1

# F: Environment variable cascading
entries.append((eid(n), "env-inherit-3level", "Three-level environment inheritance",
    'fn global(k: u32) -> u32 { if k == 1 { return 100; } if k == 2 { return 200; } return 0; } fn local1(k: u32) -> u32 { if k == 2 { return 999; } return global(k); } fn local2(k: u32) -> u32 { if k == 1 { return 888; } return local1(k); } fn main() { println!("{} {} {}", local2(1), local2(2), local2(3)); }',
    '888 999 0'))
n += 1

entries.append((eid(n), "env-path-join", "PATH-like variable joining simulation",
    'fn join2(a: u32, b: u32) -> u32 { return a * 1000 + b; } fn join3(a: u32, b: u32, c: u32) -> u32 { return join2(join2(a, b), c); } fn main() { println!("{} {}", join2(1, 2), join3(1, 2, 3)); }',
    '1002 1002003'))
n += 1

# G: SSH operation simulation
entries.append((eid(n), "ssh-tunnel-sim", "SSH tunnel port forwarding simulation",
    'fn tunnel(local: u32, remote: u32) -> u32 { return local * 10000 + remote; } fn hop(t1: u32, t2: u32) -> u32 { return t1 + t2; } fn main() { println!("{} {}", tunnel(8080, 80), hop(tunnel(2222, 22), tunnel(3306, 3306))); }',
    '80800080 55483306'))
n += 1

# H: Pathological printing
entries.append((eid(n), "printf-width-sim", "Printf field width simulation",
    'fn fmt_width(val: u32, width: u32) -> u32 { return width * 10000 + val; } fn main() { println!("{} {} {}", fmt_width(42, 5), fmt_width(1, 10), fmt_width(999, 3)); }',
    '50042 100001 30999'))
n += 1

entries.append((eid(n), "print-matrix-2x3", "Print 2x3 matrix values",
    'fn cell(r: u32, c: u32) -> u32 { return r * 10 + c; } fn main() { let mut r: u32 = 0; while r < 2 { let mut c: u32 = 0; while c < 3 { println!("{}", cell(r, c)); c = c + 1; } r = r + 1; } }',
    '0'))
n += 1

# I: Pathological text processing simulation
entries.append((eid(n), "field-extract-sim", "Awk-like field extraction simulation",
    'fn field(record: u32, idx: u32) -> u32 { let mut r: u32 = record; let mut i: u32 = 0; while i < idx { r = r / 100; i = i + 1; } return r % 100; } fn main() { let rec: u32 = 10203; println!("{} {} {}", field(rec, 0), field(rec, 1), field(rec, 2)); }',
    '3 2 1'))
n += 1

entries.append((eid(n), "sed-replace-sim", "Sed-like replacement counting",
    'fn count_matches(text_len: u32, pattern_len: u32) -> u32 { if pattern_len == 0 { return 0; } return text_len / pattern_len; } fn main() { println!("{} {} {} {}", count_matches(100, 5), count_matches(10, 3), count_matches(0, 1), count_matches(50, 50)); }',
    '20 3 0 1'))
n += 1

entries.append((eid(n), "tr-rotate-sim", "Tr-like character rotation simulation",
    'fn rotate(val: u32, shift: u32, modulus: u32) -> u32 { return (val + shift) % modulus; } fn main() { println!("{} {} {} {}", rotate(0, 13, 26), rotate(25, 1, 26), rotate(13, 13, 26), rotate(0, 26, 26)); }',
    '13 0 0 0'))
n += 1

# J: Pathological data structures
entries.append((eid(n), "linked-list-sim", "Linked list simulation via encoded u32",
    'fn cons(val: u32, next: u32) -> u32 { return val * 10000 + next; } fn head(list: u32) -> u32 { return list / 10000; } fn tail(list: u32) -> u32 { return list % 10000; } fn length(list: u32) -> u32 { if list == 0 { return 0; } return 1 + length(tail(list)); } fn main() { let l: u32 = cons(1, cons(2, cons(3, 0))); println!("{} {} {}", head(l), head(tail(l)), length(l)); }',
    '1 2 3'))
n += 1

entries.append((eid(n), "heap-insert-sim", "Binary heap insertion simulation",
    'fn parent(i: u32) -> u32 { if i == 0 { return 0; } return (i - 1) / 2; } fn left(i: u32) -> u32 { return 2 * i + 1; } fn right(i: u32) -> u32 { return 2 * i + 2; } fn main() { println!("{} {} {} {} {}", parent(5), parent(6), left(0), right(0), left(2)); }',
    '2 2 1 2 5'))
n += 1

# K: Pathological sourcing patterns
entries.append((eid(n), "module-load-sim", "Module loading order simulation",
    'fn load_a() -> u32 { return 1; } fn load_b(a: u32) -> u32 { return a + 10; } fn load_c(a: u32, b: u32) -> u32 { return a + b + 100; } fn main() { let a: u32 = load_a(); let b: u32 = load_b(a); let c: u32 = load_c(a, b); println!("{} {} {}", a, b, c); }',
    '1 11 112'))
n += 1

# L: Pathological scripts with braces/semicolons
entries.append((eid(n), "nested-scope-4deep", "Four-deep nested scope simulation",
    'fn scope1(x: u32) -> u32 { let y: u32 = x + 1; return y; } fn scope2(x: u32) -> u32 { let y: u32 = scope1(x) + 2; return y; } fn scope3(x: u32) -> u32 { let y: u32 = scope2(x) + 3; return y; } fn scope4(x: u32) -> u32 { let y: u32 = scope3(x) + 4; return y; } fn main() { println!("{}", scope4(0)); }',
    '10'))
n += 1

entries.append((eid(n), "semicolon-heavy", "Many sequential operations",
    'fn main() { let mut a: u32 = 1; a = a + 1; a = a * 2; a = a + 3; a = a * 4; a = a + 5; a = a * 6; a = a + 7; a = a * 8; a = a + 9; println!("{}", a); }',
    '3753'))
n += 1

# M: Pathological traps / signal handling
entries.append((eid(n), "trap-handler-sim", "Signal handler priority simulation",
    'fn handle_int(state: u32) -> u32 { return state + 100; } fn handle_term(state: u32) -> u32 { return state + 200; } fn handle_hup(state: u32) -> u32 { return state + 300; } fn dispatch(sig: u32, state: u32) -> u32 { if sig == 2 { return handle_int(state); } if sig == 15 { return handle_term(state); } if sig == 1 { return handle_hup(state); } return state; } fn main() { let s0: u32 = 0; let s1: u32 = dispatch(2, s0); let s2: u32 = dispatch(15, s1); let s3: u32 = dispatch(1, s2); println!("{}", s3); }',
    '600'))
n += 1

# N: Command line parsing
entries.append((eid(n), "getopt-sim", "Getopt-style option parsing simulation",
    'fn parse_opt(opt: u32, has_arg: u32, arg_val: u32) -> u32 { if has_arg != 0 { return opt * 1000 + arg_val; } return opt * 1000; } fn main() { println!("{} {} {}", parse_opt(1, 1, 42), parse_opt(2, 0, 0), parse_opt(3, 1, 100)); }',
    '1042 2000 3100'))
n += 1

entries.append((eid(n), "flag-bitmap", "Command flag bitmap construction",
    'fn set_flag(bitmap: u32, bit: u32) -> u32 { let mask: u32 = 1; let mut shifted: u32 = mask; let mut i: u32 = 0; while i < bit { shifted = shifted * 2; i = i + 1; } return bitmap + shifted; } fn main() { let mut flags: u32 = 0; flags = set_flag(flags, 0); flags = set_flag(flags, 2); flags = set_flag(flags, 4); println!("{}", flags); }',
    '21'))
n += 1

# Q: Numerical methods - harder
entries.append((eid(n), "matrix-det-2x2", "2x2 matrix determinant",
    'fn det2(a: u32, b: u32, c: u32, d: u32) -> u32 { if a * d >= b * c { return a * d - b * c; } return b * c - a * d; } fn main() { println!("{} {} {}", det2(3, 1, 2, 4), det2(5, 3, 2, 4), det2(1, 0, 0, 1)); }',
    '10 14 1'))
n += 1

entries.append((eid(n), "factorial-iter", "Iterative factorial",
    'fn fact(n: u32) -> u32 { let mut r: u32 = 1; let mut i: u32 = 2; while i <= n { r = r * i; i = i + 1; } return r; } fn main() { println!("{} {} {} {} {}", fact(0), fact(1), fact(5), fact(8), fact(10)); }',
    '1 1 120 40320 3628800'))
n += 1

entries.append((eid(n), "combinations", "Combinations C(n,k) computation",
    'fn fact(n: u32) -> u32 { let mut r: u32 = 1; let mut i: u32 = 2; while i <= n { r = r * i; i = i + 1; } return r; } fn comb(n: u32, k: u32) -> u32 { return fact(n) / (fact(k) * fact(n - k)); } fn main() { println!("{} {} {} {}", comb(5, 2), comb(10, 3), comb(6, 0), comb(4, 4)); }',
    '10 120 1 1'))
n += 1

entries.append((eid(n), "euler-totient", "Euler totient function",
    'fn gcd(mut a: u32, mut b: u32) -> u32 { while b != 0 { let t: u32 = b; b = a % b; a = t; } return a; } fn totient(n: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 1; while i <= n { if gcd(i, n) == 1 { count = count + 1; } i = i + 1; } return count; } fn main() { println!("{} {} {} {}", totient(1), totient(10), totient(12), totient(17)); }',
    '1 4 4 16'))
n += 1

# R: Symbolic patterns
entries.append((eid(n), "expr-tree-eval", "Expression tree evaluation simulation",
    'fn add_node(l: u32, r: u32) -> u32 { return l + r; } fn mul_node(l: u32, r: u32) -> u32 { return l * r; } fn sub_node(l: u32, r: u32) -> u32 { if l > r { return l - r; } return 0; } fn main() { let tree: u32 = add_node(mul_node(3, 4), sub_node(10, mul_node(2, 3))); println!("{}", tree); }',
    '16'))
n += 1

entries.append((eid(n), "rpn-calc-5op", "Reverse Polish Notation calculator",
    'fn rpn_add(stack: u32, val: u32) -> u32 { return stack + val; } fn rpn_mul(a: u32, b: u32) -> u32 { return a * b; } fn rpn_sub(a: u32, b: u32) -> u32 { if a > b { return a - b; } return 0; } fn main() { let r: u32 = rpn_sub(rpn_mul(rpn_add(3, 4), 5), 10); println!("{}", r); }',
    '25'))
n += 1

# S: Editor command simulation
entries.append((eid(n), "cursor-motion-sim", "Vi cursor motion simulation",
    'fn move_right(col: u32, max_col: u32) -> u32 { if col < max_col { return col + 1; } return col; } fn move_left(col: u32) -> u32 { if col > 0 { return col - 1; } return 0; } fn move_word(col: u32, word_len: u32, max_col: u32) -> u32 { let new: u32 = col + word_len; if new > max_col { return max_col; } return new; } fn main() { let mut pos: u32 = 0; pos = move_right(pos, 80); pos = move_right(pos, 80); pos = move_word(pos, 5, 80); pos = move_left(pos); println!("{}", pos); }',
    '6'))
n += 1

entries.append((eid(n), "editor-buffer-sim", "Text buffer line operations",
    'fn insert_line(count: u32) -> u32 { return count + 1; } fn delete_line(count: u32) -> u32 { if count > 0 { return count - 1; } return 0; } fn main() { let mut lines: u32 = 0; lines = insert_line(lines); lines = insert_line(lines); lines = insert_line(lines); lines = delete_line(lines); lines = insert_line(lines); println!("{}", lines); }',
    '3'))
n += 1

# T: Functional programming
entries.append((eid(n), "church-bool-sim", "Church boolean simulation",
    'fn true_fn(a: u32, b: u32) -> u32 { return a; } fn false_fn(a: u32, b: u32) -> u32 { return b; } fn and_fn(p: u32, q: u32) -> u32 { if p == 1 { return q; } return 0; } fn or_fn(p: u32, q: u32) -> u32 { if p == 1 { return 1; } return q; } fn not_fn(p: u32) -> u32 { if p == 1 { return 0; } return 1; } fn main() { println!("{} {} {} {} {}", true_fn(1, 0), false_fn(1, 0), and_fn(1, 0), or_fn(0, 1), not_fn(1)); }',
    '1 0 0 1 0'))
n += 1

entries.append((eid(n), "map-sim-4elem", "Simulated map over 4 elements",
    'fn double(x: u32) -> u32 { return x * 2; } fn main() { println!("{} {} {} {}", double(1), double(2), double(3), double(4)); }',
    '2 4 6 8'))
n += 1

entries.append((eid(n), "filter-sim", "Simulated filter over values",
    'fn is_even(x: u32) -> u32 { if x % 2 == 0 { return 1; } return 0; } fn main() { let mut i: u32 = 1; while i <= 10 { if is_even(i) != 0 { println!("{}", i); } i = i + 1; } }',
    '2'))
n += 1

entries.append((eid(n), "reduce-sum-prod", "Simulated reduce for sum and product",
    'fn main() { let mut sum: u32 = 0; let mut prod: u32 = 1; let mut i: u32 = 1; while i <= 5 { sum = sum + i; prod = prod * i; i = i + 1; } println!("{} {}", sum, prod); }',
    '15 120'))
n += 1

# U: Provably correct
entries.append((eid(n), "bounded-loop-proof", "Loop with provable termination",
    'fn bounded_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 0; while i < n { s = s + i; i = i + 1; } return s; } fn main() { println!("{} {} {} {}", bounded_sum(0), bounded_sum(1), bounded_sum(5), bounded_sum(100)); }',
    '0 0 10 4950'))
n += 1

entries.append((eid(n), "invariant-maintain", "Loop invariant maintenance proof",
    'fn sum_invariant(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 1; while i <= n { s = s + i; i = i + 1; } return s; } fn verify(n: u32) -> u32 { let expected: u32 = n * (n + 1) / 2; let actual: u32 = sum_invariant(n); if expected == actual { return 1; } return 0; } fn main() { println!("{} {} {} {}", verify(0), verify(10), verify(50), verify(100)); }',
    '1 1 1 1'))
n += 1

# V: Clippy pedantic patterns
entries.append((eid(n), "explicit-overflow-check", "Explicit overflow checking",
    'fn checked_add(a: u32, b: u32, max_val: u32) -> u32 { if a > max_val - b { return max_val; } return a + b; } fn checked_mul(a: u32, b: u32, max_val: u32) -> u32 { if b != 0 { if a > max_val / b { return max_val; } } return a * b; } fn main() { println!("{} {} {}", checked_add(100, 200, 255), checked_mul(20, 20, 255), checked_add(0, 0, 255)); }',
    '255 255 0'))
n += 1

# W: C-mixed-with-Bash patterns
entries.append((eid(n), "c-struct-sim", "C struct simulation with encoded fields",
    'fn make_point(x: u32, y: u32) -> u32 { return x * 10000 + y; } fn get_x(p: u32) -> u32 { return p / 10000; } fn get_y(p: u32) -> u32 { return p % 10000; } fn dist_sq(p1: u32, p2: u32) -> u32 { let dx: u32 = if get_x(p1) > get_x(p2) { get_x(p1) - get_x(p2) } else { get_x(p2) - get_x(p1) }; let dy: u32 = if get_y(p1) > get_y(p2) { get_y(p1) - get_y(p2) } else { get_y(p2) - get_y(p1) }; return dx * dx + dy * dy; } fn main() { let p1: u32 = make_point(3, 4); let p2: u32 = make_point(0, 0); println!("{} {} {}", get_x(p1), get_y(p1), dist_sq(p1, p2)); }',
    '3 4 25'))
n += 1

entries.append((eid(n), "c-malloc-sim", "C malloc/free simulation",
    'fn alloc(pool: u32, size: u32) -> u32 { return pool + size; } fn dealloc(pool: u32, size: u32) -> u32 { if pool >= size { return pool - size; } return 0; } fn main() { let mut pool: u32 = 0; pool = alloc(pool, 100); pool = alloc(pool, 50); pool = dealloc(pool, 30); println!("{}", pool); }',
    '120'))
n += 1

# X: Lua-mixed-with-Bash patterns (NEW)
entries.append((eid(n), "lua-table-sim", "Lua table simulation with index mapping",
    'fn tbl_set(tbl: u32, key: u32, val: u32) -> u32 { return tbl + key * 100 + val; } fn tbl_get(tbl: u32, key: u32) -> u32 { return (tbl / (key * 100 + 1)) % 100; } fn main() { let t: u32 = tbl_set(0, 1, 42); println!("{}", t); }',
    '142'))
n += 1

entries.append((eid(n), "lua-coroutine-sim", "Lua coroutine yield/resume simulation",
    'fn yield_val(state: u32, val: u32) -> u32 { return state * 100 + val; } fn resume(co: u32) -> u32 { return co % 100; } fn main() { let co1: u32 = yield_val(0, 10); let co2: u32 = yield_val(co1, 20); let co3: u32 = yield_val(co2, 30); println!("{} {} {}", resume(co1), resume(co2), resume(co3)); }',
    '10 20 30'))
n += 1

entries.append((eid(n), "lua-pcall-sim", "Lua pcall error handling simulation",
    'fn pcall(fn_id: u32, arg: u32) -> u32 { if fn_id == 0 { return 0; } if arg == 0 { return 1; } return arg * fn_id; } fn main() { println!("{} {} {} {}", pcall(0, 5), pcall(1, 0), pcall(2, 3), pcall(3, 4)); }',
    '0 1 6 12'))
n += 1

# Makefile entries (3)
MK = []
mk_n = n
MK.append((f"M-{START_ID + mk_n}", "make-pkg-config-sim", "Makefile: pkg-config flag simulation",
    'fn pkg_cflags(lib: u32) -> u32 { return lib * 10 + 1; } fn pkg_libs(lib: u32) -> u32 { return lib * 10 + 2; } fn main() { println!("{} {} {} {}", pkg_cflags(1), pkg_libs(1), pkg_cflags(2), pkg_libs(2)); }',
    'pkg_cflags() {'))
mk_n += 1
MK.append((f"M-{START_ID + mk_n}", "make-install-dirs", "Makefile: install directory creation",
    'fn mkpath(base: u32, sub: u32) -> u32 { return base * 100 + sub; } fn main() { println!("{} {} {}", mkpath(1, 10), mkpath(1, 20), mkpath(2, 10)); }',
    'mkpath() {'))
mk_n += 1
MK.append((f"M-{START_ID + mk_n}", "make-version-embed", "Makefile: version string embedding",
    'fn ver_str(ma: u32, mi: u32, p: u32) -> u32 { return ma * 10000 + mi * 100 + p; } fn main() { println!("{} {}", ver_str(1, 2, 3), ver_str(2, 0, 0)); }',
    'ver_str() {'))
mk_n += 1

# Dockerfile entries (3)
DK = []
dk_n = mk_n
DK.append((f"D-{START_ID + dk_n}", "docker-arg-resolve", "Dockerfile: ARG with default resolution",
    'fn from_image(i: &str, t: &str) {} fn resolve_arg(val: u32, def: u32) -> u32 { if val != 0 { return val; } return def; } fn main() { from_image("rust", "1.75"); println!("{} {}", resolve_arg(0, 8080), resolve_arg(3000, 8080)); }',
    'FROM rust:1.75'))
dk_n += 1
DK.append((f"D-{START_ID + dk_n}", "docker-layer-cache", "Dockerfile: layer caching simulation",
    'fn from_image(i: &str, t: &str) {} fn cache_hit(layer: u32, changed: u32) -> u32 { if changed == 0 { return 1; } return 0; } fn main() { from_image("node", "20"); println!("{} {} {}", cache_hit(1, 0), cache_hit(2, 1), cache_hit(3, 0)); }',
    'FROM node:20'))
dk_n += 1
DK.append((f"D-{START_ID + dk_n}", "docker-entrypoint-sim", "Dockerfile: entrypoint signal forwarding",
    'fn from_image(i: &str, t: &str) {} fn entrypoint(cmd: u32, sig: u32) -> u32 { return cmd * 100 + sig; } fn main() { from_image("alpine", "3.19"); println!("{} {}", entrypoint(1, 15), entrypoint(2, 9)); }',
    'FROM alpine:3.19'))
dk_n += 1

# Generate output
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

total = len(entries) + len(MK) + len(DK)
import sys
print(f"\n// Round 6: {len(entries)} bash + {len(MK)} makefile + {len(DK)} dockerfile = {total} entries", file=sys.stderr)
print(f"// IDs: B-{START_ID}..D-{START_ID + dk_n - 1}", file=sys.stderr)
