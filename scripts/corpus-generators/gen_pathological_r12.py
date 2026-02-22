#!/usr/bin/env python3
"""Round 12: BOUNDARY-PUSHING entries — target remaining transpiler weaknesses.
Tests: return if-expr (now fixed), deeply nested expressions, complex struct-like patterns,
multi-return-path functions, interleaved recursion.
Expansion function: 190
START_ID: 16289
"""

START_ID = 16289

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
# TRANSPILER STRESS TESTS
# ============================================

# Return if-expr with 4-way chain (uses the new fix)
entries_bash.append(entry(f"B-{n}", "return-if-4chain", "Return with 4-level if-else chain",
    "Bash",
    "fn grade(score: u32) -> u32 { return if score >= 90 { 4 } else { if score >= 80 { 3 } else { if score >= 70 { 2 } else { 1 } } }; } fn main() { println!(\"{} {} {} {}\", grade(95), grade(85), grade(75), grade(65)); }",
    "4 3 2 1"))
n += 1

# Return if-expr in recursive function
entries_bash.append(entry(f"B-{n}", "return-if-recursive", "Return if-expr in recursive function",
    "Bash",
    "fn search(target: u32, lo: u32, hi: u32) -> u32 { if lo >= hi { return 0; } let mid: u32 = (lo + hi) / 2; return if target == mid { 1 } else { if target < mid { search(target, lo, mid) } else { search(target, mid + 1, hi) } }; } fn main() { println!(\"{} {} {} {}\", search(5, 0, 10), search(0, 0, 10), search(15, 0, 10), search(3, 0, 100)); }",
    "1 1 0 1"))
n += 1

# Deeply nested arithmetic (20 levels of parens)
entries_bash.append(entry(f"B-{n}", "deep-paren-20", "Twenty levels of nested parentheses",
    "Bash",
    "fn main() { let r: u32 = ((((((((((((((((((((1 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1); println!(\"{}\", r); }",
    "20"))
n += 1

# Function with 8 parameters
entries_bash.append(entry(f"B-{n}", "fn-8params", "Function with eight parameters",
    "Bash",
    "fn sum8(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 { return a + b + c + d + e + f + g + h; } fn main() { println!(\"{}\", sum8(1, 2, 3, 4, 5, 6, 7, 8)); }",
    "36"))
n += 1

# 10 functions each calling the next
entries_bash.append(entry(f"B-{n}", "fn-chain-linear-10", "Linear chain of 10 function calls",
    "Bash",
    "fn a(x: u32) -> u32 { return b(x + 1); } fn b(x: u32) -> u32 { return c(x + 2); } fn c(x: u32) -> u32 { return d(x + 3); } fn d(x: u32) -> u32 { return e(x + 4); } fn e(x: u32) -> u32 { return f2(x + 5); } fn f2(x: u32) -> u32 { return g(x + 6); } fn g(x: u32) -> u32 { return h(x + 7); } fn h(x: u32) -> u32 { return i(x + 8); } fn i(x: u32) -> u32 { return j(x + 9); } fn j(x: u32) -> u32 { return x + 10; } fn main() { println!(\"{}\", a(0)); }",
    "55"))
n += 1

# Triple nested while loops with complex body
entries_bash.append(entry(f"B-{n}", "triple-while-complex", "Triple nested while with complex body",
    "Bash",
    "fn main() { let mut total: u32 = 0; let mut i: u32 = 1; while i <= 5 { let mut j: u32 = 1; while j <= i { let mut k: u32 = 1; while k <= j { total = total + i * j * k; k = k + 1; } j = j + 1; } i = i + 1; } println!(\"{}\", total); }",
    "225"))
n += 1

# if-else assignment followed by use in arithmetic
entries_bash.append(entry(f"B-{n}", "if-expr-then-arith", "If-expr assignment then arithmetic use",
    "Bash",
    "fn main() { let x: u32 = 7; let y: u32 = if x > 5 { x * 2 } else { x + 10 }; let z: u32 = y + 100; println!(\"{} {}\", y, z); }",
    "14 114"))
n += 1

# Return if-expr with arithmetic in branches
entries_bash.append(entry(f"B-{n}", "return-if-arith-branches", "Return if-expr with arithmetic in each branch",
    "Bash",
    "fn compute(x: u32, mode: u32) -> u32 { return if mode == 0 { x * x } else { if mode == 1 { x + x } else { x } }; } fn main() { println!(\"{} {} {} {} {} {}\", compute(5, 0), compute(5, 1), compute(5, 2), compute(10, 0), compute(10, 1), compute(10, 2)); }",
    "25 10 5 100 20 10"))
n += 1

# Fibonacci with accumulator (tail-recursive style)
entries_bash.append(entry(f"B-{n}", "fib-accum-tail", "Fibonacci with accumulator (tail-recursive style)",
    "Bash",
    "fn fib_acc(n: u32, a: u32, b: u32) -> u32 { if n == 0 { return a; } return fib_acc(n - 1, b, a + b); } fn fib(n: u32) -> u32 { return fib_acc(n, 0, 1); } fn main() { println!(\"{} {} {} {} {}\", fib(0), fib(1), fib(5), fib(10), fib(15)); }",
    "0 1 5 55 610"))
n += 1

# Multiple variable shadowing in sequence
entries_bash.append(entry(f"B-{n}", "shadow-chain-5level", "Five sequential variable shadowings",
    "Bash",
    "fn main() { let x: u32 = 1; let x: u32 = x + x; let x: u32 = x * x; let x: u32 = x + 1; let x: u32 = x * 3; println!(\"{}\", x); }",
    "15"))
n += 1

# Recursive power with modulus
entries_bash.append(entry(f"B-{n}", "power-mod-recursive", "Recursive power with modulus",
    "Bash",
    "fn powmod(base: u32, exp: u32, m: u32) -> u32 { if exp == 0 { return 1; } if exp % 2 == 0 { let half: u32 = powmod(base, exp / 2, m); return (half * half) % m; } return (base * powmod(base, exp - 1, m)) % m; } fn main() { println!(\"{} {} {}\", powmod(2, 10, 1000), powmod(3, 7, 100), powmod(7, 3, 50)); }",
    "24 87 43"))
n += 1

# Complex multi-function interaction
entries_bash.append(entry(f"B-{n}", "multi-fn-interact-5", "Five functions interacting complexly",
    "Bash",
    "fn step1(x: u32) -> u32 { return x + step2(x / 2); } fn step2(x: u32) -> u32 { if x < 2 { return x; } return step3(x); } fn step3(x: u32) -> u32 { return x * 2 + step4(x - 1); } fn step4(x: u32) -> u32 { if x == 0 { return 1; } return x; } fn step5(x: u32) -> u32 { return step1(x) + step2(x); } fn main() { println!(\"{} {}\", step5(4), step5(8)); }",
    "22 44"))
n += 1

# Interleaved mutable variable updates
entries_bash.append(entry(f"B-{n}", "interleaved-3var", "Three variables with interleaved updates",
    "Bash",
    "fn main() { let mut a: u32 = 1; let mut b: u32 = 2; let mut c: u32 = 3; let mut i: u32 = 0; while i < 5 { let ta: u32 = b + c; let tb: u32 = a + c; let tc: u32 = a + b; a = ta; b = tb; c = tc; i = i + 1; } println!(\"{} {} {}\", a, b, c); }",
    "86 82 78"))
n += 1

# Boolean function with multiple && and || simulation
entries_bash.append(entry(f"B-{n}", "complex-bool-5var", "Complex boolean with 5 variables",
    "Bash",
    "fn check(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 { if a != 0 { if b != 0 { return 1; } if c != 0 { if d != 0 { return 1; } } } if e != 0 { if c != 0 { return 1; } } return 0; } fn main() { println!(\"{} {} {} {} {}\", check(1, 1, 0, 0, 0), check(1, 0, 1, 1, 0), check(0, 0, 1, 0, 1), check(0, 0, 0, 0, 0), check(1, 0, 0, 1, 0)); }",
    "1 1 1 0 0"))
n += 1

# Tower function (rapid growth)
entries_bash.append(entry(f"B-{n}", "tower-fn-bounded", "Tower function with bounded growth",
    "Bash",
    "fn tower(n: u32, max_val: u32) -> u32 { if n == 0 { return 1; } let sub: u32 = tower(n - 1, max_val); if sub > max_val / 2 { return max_val; } return sub * 2; } fn main() { println!(\"{} {} {} {} {}\", tower(0, 1000), tower(1, 1000), tower(5, 1000), tower(10, 1000), tower(15, 1000)); }",
    "1 2 32 1000 1000"))
n += 1

# Euclidean algorithm with step counting
entries_bash.append(entry(f"B-{n}", "euclidean-step-count", "Euclidean GCD with step counting",
    "Bash",
    "fn gcd_steps(a: u32, b: u32, steps: u32) -> u32 { if b == 0 { return steps; } return gcd_steps(b, a % b, steps + 1); } fn main() { println!(\"{} {} {} {}\", gcd_steps(48, 18, 0), gcd_steps(100, 1, 0), gcd_steps(fib(10), fib(9), 0), gcd_steps(1, 1, 0)); } fn fib(n: u32) -> u32 { if n <= 1 { return n; } let mut a: u32 = 0; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let t: u32 = a + b; a = b; b = t; i = i + 1; } return b; }",
    "4 1 9 1"))
n += 1

# Selection sort step count
entries_bash.append(entry(f"B-{n}", "selection-sort-comparisons", "Selection sort comparison count",
    "Bash",
    "fn sort_comparisons(n: u32) -> u32 { return n * (n - 1) / 2; } fn main() { println!(\"{} {} {} {} {}\", sort_comparisons(1), sort_comparisons(5), sort_comparisons(10), sort_comparisons(50), sort_comparisons(100)); }",
    "0 10 45 1225 4950"))
n += 1

# Complex pattern: function returning different things based on 5 conditions
entries_bash.append(entry(f"B-{n}", "multi-dispatch-5way", "Five-way dispatch function",
    "Bash",
    "fn dispatch5(cmd: u32, arg: u32) -> u32 { if cmd == 0 { return arg + 1; } if cmd == 1 { return arg * 2; } if cmd == 2 { return arg * arg; } if cmd == 3 { if arg > 0 { return arg - 1; } return 0; } if cmd == 4 { return arg % 10; } return 0; } fn main() { println!(\"{} {} {} {} {}\", dispatch5(0, 5), dispatch5(1, 5), dispatch5(2, 5), dispatch5(3, 5), dispatch5(4, 123)); }",
    "6 10 25 4 3"))
n += 1

# Mixed println with calculations
entries_bash.append(entry(f"B-{n}", "println-mixed-calc", "Multiple println with inline calculations",
    "Bash",
    "fn double(x: u32) -> u32 { return x * 2; } fn square(x: u32) -> u32 { return x * x; } fn main() { println!(\"{}\", double(5)); println!(\"{}\", square(4)); println!(\"{}\", double(square(3))); println!(\"{}\", square(double(2))); }",
    "10"))
n += 1

# Recursive digit sum
entries_bash.append(entry(f"B-{n}", "recursive-digit-sum", "Recursive digital root computation",
    "Bash",
    "fn dsum(n: u32) -> u32 { if n < 10 { return n; } let mut s: u32 = 0; let mut v: u32 = n; while v > 0 { s = s + v % 10; v = v / 10; } return dsum(s); } fn main() { println!(\"{} {} {} {} {}\", dsum(0), dsum(9), dsum(123), dsum(9999), dsum(12345)); }",
    "0 9 6 9 6"))
n += 1

bash_count = len(entries_bash)

# Makefile entries
entries_make.append(entry(f"M-{n}", "make-submake-parallel", "Makefile: parallel submake simulation",
    "Makefile",
    "fn make_j(targets: u32, jobs: u32) -> u32 { if jobs == 0 { return targets; } return (targets + jobs - 1) / jobs; } fn main() { println!(\"{} {}\", make_j(10, 4), make_j(100, 8)); }",
    "make_j() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-grouped-target", "Makefile: grouped target simulation",
    "Makefile",
    "fn group_targets(primary: u32, secondaries: u32) -> u32 { return primary * 10 + secondaries; } fn main() { println!(\"{} {}\", group_targets(1, 3), group_targets(5, 2)); }",
    "group_targets() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-conditional-assign", "Makefile: conditional assignment simulation",
    "Makefile",
    "fn cond_assign(current: u32, new_val: u32) -> u32 { if current == 0 { return new_val; } return current; } fn main() { println!(\"{} {}\", cond_assign(0, 42), cond_assign(10, 42)); }",
    "cond_assign() {"))
n += 1

make_count = len(entries_make)

# Dockerfile entries
entries_docker.append(entry(f"D-{n}", "docker-squash-layers", "Dockerfile: layer squashing simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn squash(layers: u32) -> u32 { return 1; } fn total_size(base: u32, added: u32, squashed: u32) -> u32 { return base + added - squashed; } fn main() { from_image("ubuntu", "24.04"); println!("{} {}", squash(5), total_size(100, 50, 20)); }',
    "FROM ubuntu:24.04"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-volume-mount", "Dockerfile: VOLUME mount simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn volume_count(mounts: u32) -> u32 { return mounts; } fn total_space(count: u32, avg: u32) -> u32 { return count * avg; } fn main() { from_image("postgres", "16"); println!("{} {}", volume_count(3), total_space(3, 1000)); }',
    "FROM postgres:16"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-expose-ports", "Dockerfile: EXPOSE port mapping simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn expose(port: u32, proto: u32) -> u32 { return port * 10 + proto; } fn main() { from_image("nginx", "1.25"); println!("{} {} {}", expose(80, 1), expose(443, 1), expose(8080, 2)); }',
    "FROM nginx:1.25"))
n += 1

docker_count = len(entries_docker)
total = bash_count + make_count + docker_count

print(f"// Round 12: {bash_count} bash + {make_count} makefile + {docker_count} dockerfile = {total} entries")
print(f"// IDs: B-{START_ID}..D-{n-1}")
print(f"// Expansion function: 190")
print()

print("    fn load_expansion190_bash(&mut self) {")
for e in entries_bash:
    print(e)
print("    }")
print()
print("    fn load_expansion190_makefile(&mut self) {")
for e in entries_make:
    print(e)
print("    }")
print()
print("    fn load_expansion190_dockerfile(&mut self) {")
for e in entries_docker:
    print(e)
print("    }")
