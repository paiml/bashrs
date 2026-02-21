#!/usr/bin/env python3
"""Round 11: TRANSPILER BOUNDARY entries — these target known edge cases.
Specifically exercises: shadowing, if-else as expr, nested match-like patterns,
deeply recursive calls, mixed mut/immut, complex while conditions.
Expansion function: 189
START_ID: 16263
"""

START_ID = 16263

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
# TRANSPILER EDGE CASES
# ============================================

# Variable shadowing in nested scopes
entries_bash.append(entry(f"B-{n}", "shadow-nested-3level", "Three-level variable shadowing",
    "Bash",
    "fn main() { let x: u32 = 1; let x: u32 = x + 10; let x: u32 = x * 2; println!(\"{}\", x); }",
    "22"))
n += 1

# Multiple let bindings with same name
entries_bash.append(entry(f"B-{n}", "shadow-rebind-chain", "Chain of shadowed rebindings",
    "Bash",
    "fn main() { let a: u32 = 5; let b: u32 = a + 1; let a: u32 = b * 2; let b: u32 = a + 3; let a: u32 = b - 1; println!(\"{} {}\", a, b); }",
    "14 15"))
n += 1

# If-else as expression in assignment
entries_bash.append(entry(f"B-{n}", "if-expr-assign-basic", "If-else expression in let binding",
    "Bash",
    "fn main() { let x: u32 = 10; let y: u32 = if x > 5 { 100 } else { 200 }; println!(\"{}\", y); }",
    "100"))
n += 1

# Nested if-else expressions
entries_bash.append(entry(f"B-{n}", "if-expr-nested", "Nested if-else expressions in assignment",
    "Bash",
    "fn main() { let x: u32 = 15; let y: u32 = if x > 20 { 1 } else { if x > 10 { 2 } else { 3 } }; println!(\"{}\", y); }",
    "2"))
n += 1

# if-else expr in function return
entries_bash.append(entry(f"B-{n}", "if-expr-in-return", "If expression used as function return value",
    "Bash",
    "fn classify(x: u32) -> u32 { return if x < 10 { 1 } else { if x < 100 { 2 } else { 3 } }; } fn main() { println!(\"{} {} {}\", classify(5), classify(50), classify(500)); }",
    "1 2 3"))
n += 1

# Complex while loop with multiple conditions (using sequential ifs)
entries_bash.append(entry(f"B-{n}", "while-complex-body", "While loop with complex body",
    "Bash",
    "fn main() { let mut x: u32 = 100; let mut count: u32 = 0; while x > 1 { if x % 2 == 0 { x = x / 2; } else { x = x + 1; } count = count + 1; } println!(\"{}\", count); }",
    "8"))
n += 1

# Function with many parameters
entries_bash.append(entry(f"B-{n}", "fn-6params", "Function with six parameters",
    "Bash",
    "fn combine(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 { return a + b * 2 + c * 3 + d * 4 + e * 5 + f * 6; } fn main() { println!(\"{}\", combine(1, 2, 3, 4, 5, 6)); }",
    "91"))
n += 1

# Mutual recursion
entries_bash.append(entry(f"B-{n}", "mutual-recursion-even-odd", "Mutual recursion: is_even/is_odd",
    "Bash",
    "fn is_even(n: u32) -> u32 { if n == 0 { return 1; } return is_odd(n - 1); } fn is_odd(n: u32) -> u32 { if n == 0 { return 0; } return is_even(n - 1); } fn main() { println!(\"{} {} {} {} {}\", is_even(0), is_even(1), is_even(4), is_odd(3), is_odd(6)); }",
    "1 0 1 1 0"))
n += 1

# Deep recursion stack test
entries_bash.append(entry(f"B-{n}", "deep-recursion-100", "Recursion depth 100",
    "Bash",
    "fn deep(n: u32) -> u32 { if n == 0 { return 42; } return deep(n - 1); } fn main() { println!(\"{}\", deep(100)); }",
    "42"))
n += 1

# Multiple return paths in function
entries_bash.append(entry(f"B-{n}", "fn-multi-return", "Function with multiple return paths",
    "Bash",
    "fn multi_ret(x: u32, y: u32) -> u32 { if x == 0 { return y; } if y == 0 { return x; } if x > y { return x - y; } return y - x; } fn main() { println!(\"{} {} {} {}\", multi_ret(0, 5), multi_ret(5, 0), multi_ret(10, 3), multi_ret(3, 10)); }",
    "5 5 7 7"))
n += 1

# Mutable reference simulation via repeated assignment
entries_bash.append(entry(f"B-{n}", "mut-reassign-many", "Many mutable reassignments",
    "Bash",
    "fn main() { let mut x: u32 = 0; x = x + 1; x = x * 3; x = x + 2; x = x * 3; x = x + 1; x = x * 3; x = x + 2; x = x * 3; println!(\"{}\", x); }",
    "422"))
n += 1

# Nested function calls (10 deep)
entries_bash.append(entry(f"B-{n}", "fn-call-10deep", "Ten-deep nested function calls",
    "Bash",
    "fn f1(x: u32) -> u32 { return x + 1; } fn f2(x: u32) -> u32 { return f1(x) + 1; } fn f3(x: u32) -> u32 { return f2(x) + 1; } fn f4(x: u32) -> u32 { return f3(x) + 1; } fn f5(x: u32) -> u32 { return f4(x) + 1; } fn f6(x: u32) -> u32 { return f5(x) + 1; } fn f7(x: u32) -> u32 { return f6(x) + 1; } fn f8(x: u32) -> u32 { return f7(x) + 1; } fn f9(x: u32) -> u32 { return f8(x) + 1; } fn f10(x: u32) -> u32 { return f9(x) + 1; } fn main() { println!(\"{}\", f10(0)); }",
    "10"))
n += 1

# Mixed arithmetic with all operators
entries_bash.append(entry(f"B-{n}", "mixed-arith-all-ops", "Mixed arithmetic: +, -, *, /, %",
    "Bash",
    "fn main() { let a: u32 = 100; let b: u32 = 7; println!(\"{} {} {} {} {}\", a + b, a - b, a * b, a / b, a % b); }",
    "107 93 700 14 2"))
n += 1

# Complex boolean expression simulation
entries_bash.append(entry(f"B-{n}", "complex-bool-expr", "Complex boolean expression evaluation",
    "Bash",
    "fn eval(a: u32, b: u32, c: u32, d: u32) -> u32 { if a != 0 { if b != 0 { return 1; } if c != 0 { if d != 0 { return 1; } } return 0; } if c != 0 { if d != 0 { return 1; } } return 0; } fn main() { println!(\"{} {} {} {} {}\", eval(1, 1, 0, 0), eval(1, 0, 1, 1), eval(0, 0, 1, 1), eval(0, 0, 0, 1), eval(1, 0, 0, 0)); }",
    "1 1 1 0 0"))
n += 1

# While loop as value producer
entries_bash.append(entry(f"B-{n}", "while-value-producer", "While loop producing final value",
    "Bash",
    "fn sum_digits(mut n: u32) -> u32 { let mut sum: u32 = 0; while n > 0 { sum = sum + n % 10; n = n / 10; } return sum; } fn digital_root(n: u32) -> u32 { let mut x: u32 = n; while x >= 10 { x = sum_digits(x); } return x; } fn main() { println!(\"{} {} {} {}\", digital_root(0), digital_root(9), digital_root(123), digital_root(9999)); }",
    "0 9 6 9"))
n += 1

# Function that returns different types based on condition (simulated)
entries_bash.append(entry(f"B-{n}", "dispatch-table-sim", "Dispatch table simulation with function IDs",
    "Bash",
    "fn add(a: u32, b: u32) -> u32 { return a + b; } fn sub(a: u32, b: u32) -> u32 { if a >= b { return a - b; } return 0; } fn mul(a: u32, b: u32) -> u32 { return a * b; } fn dispatch(op: u32, a: u32, b: u32) -> u32 { if op == 0 { return add(a, b); } if op == 1 { return sub(a, b); } if op == 2 { return mul(a, b); } return 0; } fn main() { println!(\"{} {} {} {}\", dispatch(0, 10, 20), dispatch(1, 30, 10), dispatch(2, 5, 6), dispatch(3, 1, 1)); }",
    "30 20 30 0"))
n += 1

# Large number of sequential prints
entries_bash.append(entry(f"B-{n}", "sequential-print-10", "Ten sequential print statements",
    "Bash",
    "fn main() { println!(\"{}\", 1); println!(\"{}\", 2); println!(\"{}\", 3); println!(\"{}\", 4); println!(\"{}\", 5); println!(\"{}\", 6); println!(\"{}\", 7); println!(\"{}\", 8); println!(\"{}\", 9); println!(\"{}\", 10); }",
    "1"))
n += 1

# Power function via repeated squaring
entries_bash.append(entry(f"B-{n}", "fast-power", "Fast power via repeated squaring",
    "Bash",
    "fn fast_pow(mut base: u32, mut exp: u32) -> u32 { let mut result: u32 = 1; while exp > 0 { if exp % 2 == 1 { result = result * base; } base = base * base; exp = exp / 2; } return result; } fn main() { println!(\"{} {} {} {} {}\", fast_pow(2, 0), fast_pow(2, 1), fast_pow(2, 10), fast_pow(3, 5), fast_pow(5, 3)); }",
    "1 2 1024 243 125"))
n += 1

# String length simulation via digit counting
entries_bash.append(entry(f"B-{n}", "digit-count-sim", "Digit count as string length simulation",
    "Bash",
    "fn num_digits(n: u32) -> u32 { if n == 0 { return 1; } let mut count: u32 = 0; let mut v: u32 = n; while v > 0 { count = count + 1; v = v / 10; } return count; } fn main() { println!(\"{} {} {} {} {}\", num_digits(0), num_digits(9), num_digits(99), num_digits(1000), num_digits(99999)); }",
    "1 1 2 4 5"))
n += 1

# Interleaved mutation of two variables
entries_bash.append(entry(f"B-{n}", "interleaved-mutation", "Interleaved mutation of two variables",
    "Bash",
    "fn main() { let mut a: u32 = 1; let mut b: u32 = 1; let mut i: u32 = 0; while i < 10 { let tmp: u32 = a + b; a = b; b = tmp; i = i + 1; } println!(\"{} {}\", a, b); }",
    "89 144"))
n += 1

bash_count = len(entries_bash)

# Makefile entries
entries_make.append(entry(f"M-{n}", "make-shell-override", "Makefile: SHELL override simulation",
    "Makefile",
    "fn shell_exec(shell_id: u32, cmd: u32) -> u32 { return shell_id * 1000 + cmd; } fn main() { println!(\"{} {}\", shell_exec(1, 42), shell_exec(2, 42)); }",
    "shell_exec() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-include-guard", "Makefile: include guard simulation",
    "Makefile",
    "fn guard_check(included: u32) -> u32 { if included != 0 { return 0; } return 1; } fn main() { println!(\"{} {}\", guard_check(0), guard_check(1)); }",
    "guard_check() {"))
n += 1

entries_make.append(entry(f"M-{n}", "make-canned-recipe", "Makefile: canned recipe simulation",
    "Makefile",
    "fn canned_build(target: u32, flags: u32) -> u32 { return target * 100 + flags; } fn canned_test(target: u32) -> u32 { return target * 10 + 1; } fn main() { println!(\"{} {}\", canned_build(1, 42), canned_test(5)); }",
    "canned_build() {"))
n += 1

make_count = len(entries_make)

# Dockerfile entries
entries_docker.append(entry(f"D-{n}", "docker-user-switch", "Dockerfile: USER directive switching",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn run_as(uid: u32, cmd: u32) -> u32 { return uid * 1000 + cmd; } fn main() { from_image("ubuntu", "22.04"); println!("{} {}", run_as(0, 1), run_as(1000, 2)); }',
    "FROM ubuntu:22.04"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-workdir-chain", "Dockerfile: WORKDIR chain resolution",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn workdir_resolve(base: u32, sub: u32) -> u32 { return base * 100 + sub; } fn main() { from_image("node", "20-alpine"); println!("{} {}", workdir_resolve(1, 10), workdir_resolve(1, 20)); }',
    "FROM node:20-alpine"))
n += 1

entries_docker.append(entry(f"D-{n}", "docker-label-metadata", "Dockerfile: LABEL metadata simulation",
    "Dockerfile",
    'fn from_image(i: &str, t: &str) {} fn label_count(labels: u32) -> u32 { return labels; } fn total_metadata(labels: u32, avg_len: u32) -> u32 { return labels * avg_len; } fn main() { from_image("alpine", "3.20"); println!("{} {}", label_count(5), total_metadata(5, 20)); }',
    "FROM alpine:3.20"))
n += 1

docker_count = len(entries_docker)
total = bash_count + make_count + docker_count

print(f"// Round 11: {bash_count} bash + {make_count} makefile + {docker_count} dockerfile = {total} entries")
print(f"// IDs: B-{START_ID}..D-{n-1}")
print(f"// Expansion function: 189")
print()

print("    fn load_expansion189_bash(&mut self) {")
for e in entries_bash:
    print(e)
print("    }")
print()
print("    fn load_expansion189_makefile(&mut self) {")
for e in entries_make:
    print(e)
print("    }")
print()
print("    fn load_expansion189_dockerfile(&mut self) {")
for e in entries_docker:
    print(e)
print("    }")
