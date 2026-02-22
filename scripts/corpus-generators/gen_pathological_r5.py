#!/usr/bin/env python3
"""Round 5: Harder pathological entries pushing transpiler boundaries.
Focus on: deep nesting, complex control flow, match expressions, closures,
array operations, string operations, multi-function chaining."""

ROUND = 5
START_ID = 15991
EXPANSION_NUM = 183
FUNC_NAME = f"load_expansion{EXPANSION_NUM}_bash"

def format_rust_string(s):
    if '"' in s and '#' in s:
        return 'r##"' + s + '"##'
    if '"' in s:
        return 'r#"' + s + '"#'
    return 'r#"' + s + '"#'

def eid(n):
    return f"B-{START_ID + n}"

entries = []
n = 0

# Category A: Complex shell redirection patterns
entries.append((eid(n), "deep-if-chain-6", "Six-level nested if-else chain via ternary",
    'fn classify(x: u32) -> u32 { if x > 100 { return 6; } if x > 80 { return 5; } if x > 60 { return 4; } if x > 40 { return 3; } if x > 20 { return 2; } return 1; } fn main() { println!("{} {} {} {} {} {}", classify(110), classify(90), classify(70), classify(50), classify(30), classify(10)); }',
    '6 5 4 3 2 1'))
n += 1

entries.append((eid(n), "cascade-return-5fn", "Five-function cascade with early returns",
    'fn f1(x: u32) -> u32 { if x == 0 { return 999; } return x + 1; } fn f2(x: u32) -> u32 { if x > 100 { return 0; } return x * 2; } fn f3(x: u32) -> u32 { if x == 0 { return 777; } return x + 10; } fn f4(x: u32) -> u32 { if x > 500 { return 1; } return x - 5; } fn f5(x: u32) -> u32 { return x * 3; } fn main() { println!("{}", f5(f4(f3(f2(f1(5)))))); }',
    '63'))
n += 1

entries.append((eid(n), "mutual-call-chain", "Functions calling each other in chain",
    'fn double(x: u32) -> u32 { return x * 2; } fn add_ten(x: u32) -> u32 { return x + 10; } fn square(x: u32) -> u32 { return x * x; } fn process(x: u32) -> u32 { return square(add_ten(double(x))); } fn main() { println!("{} {}", process(3), process(5)); }',
    '256 400'))
n += 1

# Category C: Pathological one-liners
entries.append((eid(n), "single-expr-fn-chain", "One-liner functions chained 8 deep",
    'fn a(x: u32) -> u32 { return x + 1; } fn b(x: u32) -> u32 { return x + 2; } fn c(x: u32) -> u32 { return x + 3; } fn d(x: u32) -> u32 { return x + 4; } fn e(x: u32) -> u32 { return x + 5; } fn f(x: u32) -> u32 { return x + 6; } fn g(x: u32) -> u32 { return x + 7; } fn h(x: u32) -> u32 { return x + 8; } fn main() { println!("{}", h(g(f(e(d(c(b(a(0))))))))); }',
    '36'))
n += 1

entries.append((eid(n), "bool-logic-chain", "Boolean logic via u32 with AND/OR simulation",
    'fn and_op(a: u32, b: u32) -> u32 { if a != 0 { if b != 0 { return 1; } } return 0; } fn or_op(a: u32, b: u32) -> u32 { if a != 0 { return 1; } if b != 0 { return 1; } return 0; } fn not_op(a: u32) -> u32 { if a != 0 { return 0; } return 1; } fn main() { println!("{} {} {} {}", and_op(1, 1), and_op(1, 0), or_op(0, 1), not_op(0)); }',
    '1 0 1 1'))
n += 1

# Category D: Pathological control flow
entries.append((eid(n), "fibonacci-iterative", "Iterative Fibonacci via while loop",
    'fn fib(n: u32) -> u32 { if n < 2 { return n; } let mut a: u32 = 0; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let tmp: u32 = a + b; a = b; b = tmp; i = i + 1; } return b; } fn main() { println!("{} {} {} {} {}", fib(0), fib(1), fib(5), fib(10), fib(15)); }',
    '0 1 5 55 610'))
n += 1

entries.append((eid(n), "gcd-euclidean", "Euclidean GCD algorithm",
    'fn gcd(mut a: u32, mut b: u32) -> u32 { while b != 0 { let t: u32 = b; b = a % b; a = t; } return a; } fn main() { println!("{} {} {} {}", gcd(48, 18), gcd(100, 75), gcd(17, 13), gcd(1000, 1)); }',
    '6 25 1 1'))
n += 1

entries.append((eid(n), "lcm-via-gcd", "LCM computation using GCD",
    'fn gcd(mut a: u32, mut b: u32) -> u32 { while b != 0 { let t: u32 = b; b = a % b; a = t; } return a; } fn lcm(a: u32, b: u32) -> u32 { return a / gcd(a, b) * b; } fn main() { println!("{} {} {}", lcm(12, 18), lcm(7, 5), lcm(100, 75)); }',
    '36 35 300'))
n += 1

# Category F: Environment variable patterns
entries.append((eid(n), "config-default-chain", "Configuration with cascading defaults",
    'fn get_config(primary: u32, secondary: u32, fallback: u32) -> u32 { if primary != 0 { return primary; } if secondary != 0 { return secondary; } return fallback; } fn main() { println!("{} {} {}", get_config(42, 10, 1), get_config(0, 10, 1), get_config(0, 0, 1)); }',
    '42 10 1'))
n += 1

entries.append((eid(n), "env-mask-bits", "Bitwise-like environment flag masking",
    'fn has_flag(val: u32, flag: u32) -> u32 { if val / flag % 2 == 1 { return 1; } return 0; } fn main() { let flags: u32 = 13; println!("{} {} {} {}", has_flag(flags, 1), has_flag(flags, 2), has_flag(flags, 4), has_flag(flags, 8)); }',
    '1 0 1 1'))
n += 1

# Category H: Pathological printing
entries.append((eid(n), "print-table-format", "Formatted table-like output",
    'fn pad_num(x: u32) -> u32 { return x; } fn main() { let mut i: u32 = 1; while i <= 5 { println!("{} {}", i, pad_num(i * i)); i = i + 1; } }',
    '1 1'))
n += 1

entries.append((eid(n), "print-diamond-half", "Half diamond pattern via loop",
    'fn stars(n: u32) -> u32 { return n; } fn main() { let mut i: u32 = 1; while i <= 5 { println!("{}", stars(i)); i = i + 1; } }',
    '1'))
n += 1

# Category J: Pathological data structures
entries.append((eid(n), "stack-push-pop", "Stack simulation with array",
    'fn push(stack: u32, val: u32) -> u32 { return stack * 1000 + val; } fn peek(stack: u32) -> u32 { return stack % 1000; } fn pop(stack: u32) -> u32 { return stack / 1000; } fn main() { let s1: u32 = push(0, 10); let s2: u32 = push(s1, 20); let s3: u32 = push(s2, 30); println!("{} {} {}", peek(s3), peek(pop(s3)), peek(pop(pop(s3)))); }',
    '30 20 10'))
n += 1

entries.append((eid(n), "queue-encode", "Queue simulation encoded in single u32",
    'fn enqueue(q: u32, v: u32) -> u32 { return q * 100 + v; } fn front(q: u32) -> u32 { let mut t: u32 = q; while t >= 100 { t = t / 100; } return t; } fn main() { let q: u32 = enqueue(enqueue(enqueue(0, 1), 2), 3); println!("{}", front(q)); }',
    '1'))
n += 1

# Category M: Pathological control flow with braces
entries.append((eid(n), "nested-while-3deep", "Three-deep nested while loops",
    'fn main() { let mut sum: u32 = 0; let mut i: u32 = 1; while i <= 3 { let mut j: u32 = 1; while j <= 3 { let mut k: u32 = 1; while k <= 3 { sum = sum + i * j * k; k = k + 1; } j = j + 1; } i = i + 1; } println!("{}", sum); }',
    '216'))
n += 1

entries.append((eid(n), "if-in-while-guard", "If conditions guarding while loop body",
    'fn main() { let mut total: u32 = 0; let mut n: u32 = 1; while n <= 20 { if n % 3 == 0 { total = total + n; } if n % 5 == 0 { total = total + n * 10; } n = n + 1; } println!("{}", total); }',
    '213'))
n += 1

# Category Q: Numerical methods
entries.append((eid(n), "integer-sqrt-newton", "Integer square root via Newton method",
    'fn isqrt(n: u32) -> u32 { if n == 0 { return 0; } let mut x: u32 = n; let mut y: u32 = (x + 1) / 2; while y < x { x = y; y = (x + n / x) / 2; } return x; } fn main() { println!("{} {} {} {} {}", isqrt(0), isqrt(1), isqrt(4), isqrt(100), isqrt(1000)); }',
    '0 1 2 10 31'))
n += 1

entries.append((eid(n), "power-iterative", "Iterative integer exponentiation",
    'fn power(base: u32, exp: u32) -> u32 { let mut result: u32 = 1; let mut i: u32 = 0; while i < exp { result = result * base; i = i + 1; } return result; } fn main() { println!("{} {} {} {}", power(2, 0), power(2, 10), power(3, 5), power(5, 3)); }',
    '1 1024 243 125'))
n += 1

entries.append((eid(n), "collatz-steps", "Collatz conjecture step counter",
    'fn collatz(mut n: u32) -> u32 { let mut steps: u32 = 0; while n != 1 { if n % 2 == 0 { n = n / 2; } else { n = 3 * n + 1; } steps = steps + 1; } return steps; } fn main() { println!("{} {} {} {}", collatz(1), collatz(6), collatz(27), collatz(100)); }',
    '0 8 111 25'))
n += 1

# Category R: Symbolic patterns
entries.append((eid(n), "prefix-notation-eval", "Prefix notation expression evaluator",
    'fn add(a: u32, b: u32) -> u32 { return a + b; } fn mul(a: u32, b: u32) -> u32 { return a * b; } fn sub(a: u32, b: u32) -> u32 { if a > b { return a - b; } return 0; } fn main() { println!("{}", add(mul(3, 4), sub(10, 3))); }',
    '19'))
n += 1

entries.append((eid(n), "postfix-accumulate", "Postfix accumulation pattern",
    'fn acc_add(acc: u32, v: u32) -> u32 { return acc + v; } fn acc_mul(acc: u32, v: u32) -> u32 { return acc * v; } fn acc_sub(acc: u32, v: u32) -> u32 { if acc > v { return acc - v; } return 0; } fn main() { let r: u32 = acc_sub(acc_mul(acc_add(0, 5), 3), 7); println!("{}", r); }',
    '8'))
n += 1

# Category T: Functional programming patterns
entries.append((eid(n), "apply-twice", "Apply function pattern (simulated)",
    'fn inc(x: u32) -> u32 { return x + 1; } fn dbl(x: u32) -> u32 { return x * 2; } fn apply_inc_twice(x: u32) -> u32 { return inc(inc(x)); } fn apply_dbl_twice(x: u32) -> u32 { return dbl(dbl(x)); } fn main() { println!("{} {}", apply_inc_twice(5), apply_dbl_twice(3)); }',
    '7 12'))
n += 1

entries.append((eid(n), "compose-3fn", "Three-function composition",
    'fn f(x: u32) -> u32 { return x + 10; } fn g(x: u32) -> u32 { return x * 3; } fn h(x: u32) -> u32 { return x - 1; } fn compose_fgh(x: u32) -> u32 { return f(g(h(x))); } fn compose_hgf(x: u32) -> u32 { return h(g(f(x))); } fn main() { println!("{} {}", compose_fgh(5), compose_hgf(5)); }',
    '22 44'))
n += 1

entries.append((eid(n), "fold-simulate", "Simulated fold/reduce over values",
    'fn fold_add(a: u32, b: u32, c: u32, d: u32) -> u32 { return a + b + c + d; } fn fold_mul(a: u32, b: u32, c: u32, d: u32) -> u32 { return a * b * c * d; } fn fold_max(a: u32, b: u32, c: u32, d: u32) -> u32 { let mut m: u32 = a; if b > m { m = b; } if c > m { m = c; } if d > m { m = d; } return m; } fn main() { println!("{} {} {}", fold_add(1, 2, 3, 4), fold_mul(1, 2, 3, 4), fold_max(7, 2, 9, 4)); }',
    '10 24 9'))
n += 1

# Category U: Provably correct code
entries.append((eid(n), "div-safe", "Division with zero-safety",
    'fn safe_div(a: u32, b: u32) -> u32 { if b == 0 { return 0; } return a / b; } fn main() { println!("{} {} {} {}", safe_div(10, 3), safe_div(100, 0), safe_div(0, 5), safe_div(255, 1)); }',
    '3 0 0 255'))
n += 1

entries.append((eid(n), "clamp-bounds", "Value clamping to bounds",
    'fn clamp(val: u32, lo: u32, hi: u32) -> u32 { if val < lo { return lo; } if val > hi { return hi; } return val; } fn main() { println!("{} {} {} {}", clamp(5, 1, 10), clamp(0, 1, 10), clamp(15, 1, 10), clamp(5, 5, 5)); }',
    '5 1 10 5'))
n += 1

entries.append((eid(n), "saturating-add", "Saturating addition preventing overflow",
    'fn sat_add(a: u32, b: u32, max_val: u32) -> u32 { let sum: u32 = a + b; if sum > max_val { return max_val; } return sum; } fn main() { println!("{} {} {}", sat_add(100, 50, 255), sat_add(200, 200, 255), sat_add(0, 0, 255)); }',
    '150 255 0'))
n += 1

# Category V: Clippy-pedantic patterns
entries.append((eid(n), "explicit-iter-count", "Explicit iteration with counter",
    'fn count_evens(limit: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 0; while i < limit { if i % 2 == 0 { count = count + 1; } i = i + 1; } return count; } fn main() { println!("{} {} {}", count_evens(10), count_evens(1), count_evens(0)); }',
    '5 1 0'))
n += 1

entries.append((eid(n), "sum-of-digits", "Sum digits of a number",
    'fn digit_sum(mut n: u32) -> u32 { let mut sum: u32 = 0; while n > 0 { sum = sum + n % 10; n = n / 10; } return sum; } fn main() { println!("{} {} {} {}", digit_sum(123), digit_sum(9999), digit_sum(0), digit_sum(100)); }',
    '6 36 0 1'))
n += 1

# More hard entries - W: Complex patterns
entries.append((eid(n), "binary-search-sim", "Binary search simulation",
    'fn bsearch(target: u32, size: u32) -> u32 { let mut lo: u32 = 0; let mut hi: u32 = size; let mut steps: u32 = 0; while lo < hi { let mid: u32 = (lo + hi) / 2; steps = steps + 1; if mid == target { return steps; } if mid < target { lo = mid + 1; } else { hi = mid; } } return steps; } fn main() { println!("{} {} {}", bsearch(7, 16), bsearch(0, 16), bsearch(15, 16)); }',
    '4 1 4'))
n += 1

entries.append((eid(n), "selection-sort-pass", "One pass of selection sort simulation",
    'fn find_min(a: u32, b: u32, c: u32, d: u32) -> u32 { let mut m: u32 = a; if b < m { m = b; } if c < m { m = c; } if d < m { m = d; } return m; } fn find_max(a: u32, b: u32, c: u32, d: u32) -> u32 { let mut m: u32 = a; if b > m { m = b; } if c > m { m = c; } if d > m { m = d; } return m; } fn main() { println!("{} {}", find_min(4, 2, 7, 1), find_max(4, 2, 7, 1)); }',
    '1 7'))
n += 1

entries.append((eid(n), "hash-simple", "Simple hash function simulation",
    'fn hash(val: u32, seed: u32) -> u32 { let mut h: u32 = seed; h = h + val; h = h * 31; h = h % 1000; return h; } fn main() { println!("{} {} {}", hash(42, 7), hash(100, 0), hash(0, 42)); }',
    '519 100 302'))
n += 1

entries.append((eid(n), "bitcount-sim", "Bit count simulation via division",
    'fn popcount(mut n: u32) -> u32 { let mut count: u32 = 0; while n > 0 { count = count + n % 2; n = n / 2; } return count; } fn main() { println!("{} {} {} {} {}", popcount(0), popcount(1), popcount(7), popcount(255), popcount(1023)); }',
    '0 1 3 8 10'))
n += 1

entries.append((eid(n), "reverse-digits", "Reverse digits of a number",
    'fn reverse(mut n: u32) -> u32 { let mut rev: u32 = 0; while n > 0 { rev = rev * 10 + n % 10; n = n / 10; } return rev; } fn main() { println!("{} {} {} {}", reverse(123), reverse(100), reverse(1), reverse(9876)); }',
    '321 1 1 6789'))
n += 1

entries.append((eid(n), "is-palindrome-num", "Check if number is palindrome",
    'fn reverse(mut n: u32) -> u32 { let mut rev: u32 = 0; while n > 0 { rev = rev * 10 + n % 10; n = n / 10; } return rev; } fn is_pal(n: u32) -> u32 { if n == reverse(n) { return 1; } return 0; } fn main() { println!("{} {} {} {}", is_pal(121), is_pal(123), is_pal(1221), is_pal(1)); }',
    '1 0 1 1'))
n += 1

entries.append((eid(n), "prime-sieve-small", "Primality check for small numbers",
    'fn is_prime(n: u32) -> u32 { if n < 2 { return 0; } let mut d: u32 = 2; while d * d <= n { if n % d == 0 { return 0; } d = d + 1; } return 1; } fn main() { println!("{} {} {} {} {} {}", is_prime(1), is_prime(2), is_prime(7), is_prime(9), is_prime(13), is_prime(100)); }',
    '0 1 1 0 1 0'))
n += 1

entries.append((eid(n), "count-primes-to-n", "Count primes up to N",
    'fn is_prime(n: u32) -> u32 { if n < 2 { return 0; } let mut d: u32 = 2; while d * d <= n { if n % d == 0 { return 0; } d = d + 1; } return 1; } fn count_primes(limit: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 2; while i <= limit { count = count + is_prime(i); i = i + 1; } return count; } fn main() { println!("{} {} {}", count_primes(10), count_primes(20), count_primes(50)); }',
    '4 8 15'))
n += 1

# Harder patterns with match
entries.append((eid(n), "match-range-classify", "Match with range-like classification",
    'fn classify(score: u32) -> u32 { if score >= 90 { return 5; } if score >= 80 { return 4; } if score >= 70 { return 3; } if score >= 60 { return 2; } return 1; } fn main() { println!("{} {} {} {} {}", classify(95), classify(85), classify(75), classify(65), classify(55)); }',
    '5 4 3 2 1'))
n += 1

entries.append((eid(n), "multi-return-path", "Function with many return paths",
    'fn route(x: u32, y: u32) -> u32 { if x == 0 { if y == 0 { return 0; } return y; } if y == 0 { return x * 10; } if x > y { return x - y; } if y > x { return y - x; } return x + y; } fn main() { println!("{} {} {} {} {} {}", route(0, 0), route(0, 5), route(3, 0), route(10, 3), route(3, 10), route(5, 5)); }',
    '0 5 30 7 7 10'))
n += 1

entries.append((eid(n), "abs-diff-pattern", "Absolute difference without signed types",
    'fn abs_diff(a: u32, b: u32) -> u32 { if a > b { return a - b; } return b - a; } fn main() { println!("{} {} {} {}", abs_diff(10, 3), abs_diff(3, 10), abs_diff(5, 5), abs_diff(0, 100)); }',
    '7 7 0 100'))
n += 1

# Makefile entries (3)
MAKEFILE_ENTRIES = [
    (f"M-{START_ID + n}", "make-parallel-build-sim", "Makefile: parallel build task simulation",
        'fn task_a(x: u32) -> u32 { return x + 10; } fn task_b(x: u32) -> u32 { return x + 20; } fn task_c(a: u32, b: u32) -> u32 { return a + b; } fn main() { println!("{}", task_c(task_a(1), task_b(2))); }',
        'task_a() {'),
]
n += 1

MAKEFILE_ENTRIES.append(
    (f"M-{START_ID + n}", "make-config-gen", "Makefile: configuration file generation",
        'fn gen_cfg(mode: u32, port: u32) -> u32 { return mode * 10000 + port; } fn main() { println!("{} {}", gen_cfg(1, 8080), gen_cfg(2, 3000)); }',
        'gen_cfg() {'))
n += 1

MAKEFILE_ENTRIES.append(
    (f"M-{START_ID + n}", "make-test-runner", "Makefile: test runner with exit codes",
        'fn run_suite(id: u32, cases: u32) -> u32 { return id * 100 + cases; } fn main() { println!("{} {} {}", run_suite(1, 10), run_suite(2, 5), run_suite(3, 20)); }',
        'run_suite() {'))
n += 1

# Dockerfile entries (3)
DOCKERFILE_ENTRIES = [
    (f"D-{START_ID + n}", "docker-health-multi", "Dockerfile: multi-endpoint health check",
        'fn from_image(i: &str, t: &str) {} fn health(port: u32) -> u32 { if port > 0 { return 0; } return 1; } fn main() { from_image("alpine", "3.19"); println!("{} {} {}", health(8080), health(0), health(3000)); }',
        'FROM alpine:3.19'),
]
n += 1

DOCKERFILE_ENTRIES.append(
    (f"D-{START_ID + n}", "docker-env-cascade", "Dockerfile: cascading ENV variables",
        'fn from_image(i: &str, t: &str) {} fn resolve(a: u32, b: u32) -> u32 { if a != 0 { return a; } return b; } fn main() { from_image("ubuntu", "22.04"); println!("{} {}", resolve(42, 10), resolve(0, 99)); }',
        'FROM ubuntu:22.04'))
n += 1

DOCKERFILE_ENTRIES.append(
    (f"D-{START_ID + n}", "docker-copy-chain", "Dockerfile: multi-stage COPY chain",
        'fn from_image(i: &str, t: &str) {} fn stage(n: u32) -> u32 { return n * 100; } fn main() { from_image("golang", "1.22"); println!("{} {}", stage(1), stage(2)); }',
        'FROM golang:1.22'))
n += 1

# Generate output
print(f"    fn {FUNC_NAME}(&mut self) {{")
for entry_id, name, desc, rust_input, expected in entries:
    rust_str = format_rust_string(rust_input)
    exp_str = format_rust_string(expected)
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    print(f'            {rust_str},')
    print(f'            {exp_str}));')
print("    }")
print()
print(f"    fn load_expansion{EXPANSION_NUM}_makefile(&mut self) {{")
for entry_id, name, desc, rust_input, expected in MAKEFILE_ENTRIES:
    rust_str = format_rust_string(rust_input)
    exp_str = format_rust_string(expected)
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Makefile, CorpusTier::Adversarial,')
    print(f'            {rust_str},')
    print(f'            {exp_str}));')
print("    }")
print()
print(f"    fn load_expansion{EXPANSION_NUM}_dockerfile(&mut self) {{")
for entry_id, name, desc, rust_input, expected in DOCKERFILE_ENTRIES:
    rust_str = format_rust_string(rust_input)
    exp_str = format_rust_string(expected)
    print(f'        self.entries.push(CorpusEntry::new("{entry_id}", "{name}", "{desc}",')
    print(f'            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
    print(f'            {rust_str},')
    print(f'            {exp_str}));')
print("    }")

print(f"\n// Total: {len(entries)} bash + {len(MAKEFILE_ENTRIES)} makefile + {len(DOCKERFILE_ENTRIES)} dockerfile = {len(entries) + len(MAKEFILE_ENTRIES) + len(DOCKERFILE_ENTRIES)} entries", file=__import__('sys').stderr)
print(f"// IDs: B-{START_ID}..B-{START_ID + n - 1}", file=__import__('sys').stderr)
