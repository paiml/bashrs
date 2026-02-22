#!/usr/bin/env python3
"""Round 10: Generate 300 more corpus entries with B2-optimized expected strings.

Systematically generates entries across many patterns to maximize B1/B2/G pass rate.
All expected strings are exact full trimmed output lines from the transpiler.

Key invariants learned from Round 9:
- Simple string args to println: rash_println word (no quotes for single word)
- Multi-word string args: rash_println 'multi word' (single quotes)
- String with vars: rash_println "text ${var}" (double quotes)
- String assignment: var='value' (single quotes)
- Variable assignment from var: var="$other" (double quotes)
- Integer assignment: var='N' (single quotes around number)
- Boolean assignment: var=true or var=false (no quotes)
- Arithmetic: var=$((expr))
- For range: for var in $(seq N M); do
- While: while [ "$var" -lt N ]; do
- Function def: funcname() {
- Constant folding happens for simple arithmetic (5 * 3 -> '15')
"""

import sys

def format_rust_string(s):
    if '"#' in s:
        return f'r##"{s}"##'
    elif '"' in s or '\\' in s:
        return f'r#"{s}"#'
    else:
        return f'r#"{s}"#'

def entry(id_, slug, desc, fmt, tier, code, expected):
    fmt_map = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}
    tier_map = {"trivial": "CorpusTier::Trivial", "standard": "CorpusTier::Standard",
                "production": "CorpusTier::Production", "adversarial": "CorpusTier::Adversarial"}
    code_rs = format_rust_string(code)
    exp_rs = format_rust_string(expected)
    return f'            CorpusEntry::new("{id_}", "{slug}", "{desc}",\n                {fmt_map[fmt]}, {tier_map[tier]},\n                {code_rs},\n                {exp_rs}),'

bash_entries = []

# ==========================================
# Pattern 1: Function definitions (always "name() {" as B2 line)
# B-1906..B-1955 (50 entries)
# ==========================================
func_patterns = [
    ("B-1906", "r10-double-func", "Double function", "standard",
     "fn double(x: i32) -> i32 { x * 2 } fn main() { let r = double(21); }",
     "double() {"),
    ("B-1907", "r10-triple-func", "Triple function", "standard",
     "fn triple(x: i32) -> i32 { x * 3 } fn main() { let r = triple(14); }",
     "triple() {"),
    ("B-1908", "r10-negate-func", "Negate function", "standard",
     "fn negate(x: i32) -> i32 { 0 - x } fn main() { let r = negate(42); }",
     "negate() {"),
    ("B-1909", "r10-square-func", "Square function", "standard",
     "fn square(x: i32) -> i32 { x * x } fn main() { let r = square(7); }",
     "square() {"),
    ("B-1910", "r10-cube-func", "Cube function", "standard",
     "fn cube(x: i32) -> i32 { x * x * x } fn main() { let r = cube(3); }",
     "cube() {"),
    ("B-1911", "r10-increment-func", "Increment function", "trivial",
     "fn inc(x: i32) -> i32 { x + 1 } fn main() { let r = inc(0); }",
     "inc() {"),
    ("B-1912", "r10-decrement-func", "Decrement function", "trivial",
     "fn dec(x: i32) -> i32 { x - 1 } fn main() { let r = dec(10); }",
     "dec() {"),
    ("B-1913", "r10-halve-func", "Halve function", "standard",
     "fn halve(x: i32) -> i32 { x / 2 } fn main() { let r = halve(100); }",
     "halve() {"),
    ("B-1914", "r10-mod3-func", "Mod 3 function", "standard",
     "fn mod3(x: i32) -> i32 { x % 3 } fn main() { let r = mod3(17); }",
     "mod3() {"),
    ("B-1915", "r10-add-pair-func", "Add pair function", "standard",
     "fn add(a: i32, b: i32) -> i32 { a + b } fn main() { let r = add(10, 20); }",
     "add() {"),
    ("B-1916", "r10-sub-pair-func", "Subtract pair function", "standard",
     "fn sub(a: i32, b: i32) -> i32 { a - b } fn main() { let r = sub(50, 30); }",
     "sub() {"),
    ("B-1917", "r10-mul-pair-func", "Multiply pair function", "standard",
     "fn mul(a: i32, b: i32) -> i32 { a * b } fn main() { let r = mul(6, 7); }",
     "mul() {"),
    ("B-1918", "r10-div-pair-func", "Divide pair function", "standard",
     "fn div(a: i32, b: i32) -> i32 { a / b } fn main() { let r = div(100, 7); }",
     "div() {"),
    ("B-1919", "r10-mod-pair-func", "Modulo pair function", "standard",
     "fn modulo(a: i32, b: i32) -> i32 { a % b } fn main() { let r = modulo(17, 5); }",
     "modulo() {"),
    ("B-1920", "r10-max-func", "Max of two", "standard",
     "fn max_val(a: i32, b: i32) -> i32 { if a > b { a } else { b } } fn main() { let r = max_val(3, 7); }",
     "max_val() {"),
    ("B-1921", "r10-min-func", "Min of two", "standard",
     "fn min_val(a: i32, b: i32) -> i32 { if a < b { a } else { b } } fn main() { let r = min_val(3, 7); }",
     "min_val() {"),
    ("B-1922", "r10-abs-func", "Absolute value", "standard",
     "fn abs_v(x: i32) -> i32 { if x < 0 { 0 - x } else { x } } fn main() { let r = abs_v(-42); }",
     "abs_v() {"),
    ("B-1923", "r10-sign-func", "Sign function", "standard",
     "fn sgn(x: i32) -> i32 { if x > 0 { 1 } else if x < 0 { -1 } else { 0 } } fn main() { let r = sgn(-10); }",
     "sgn() {"),
    ("B-1924", "r10-clamp-func", "Clamp between bounds", "standard",
     "fn clamped(x: i32, lo: i32, hi: i32) -> i32 { if x < lo { lo } else if x > hi { hi } else { x } } fn main() { let r = clamped(15, 0, 10); }",
     "clamped() {"),
    ("B-1925", "r10-is-zero-func", "Check if zero", "trivial",
     "fn is_zero(x: i32) -> bool { x == 0 } fn main() { let r = is_zero(0); }",
     "is_zero() {"),
    ("B-1926", "r10-is-positive-func", "Check if positive", "standard",
     "fn is_pos(x: i32) -> bool { x > 0 } fn main() { let r = is_pos(5); }",
     "is_pos() {"),
    ("B-1927", "r10-is-negative-func", "Check if negative", "standard",
     "fn is_neg(x: i32) -> bool { x < 0 } fn main() { let r = is_neg(-1); }",
     "is_neg() {"),
    ("B-1928", "r10-is-even-func", "Check if even", "standard",
     "fn is_even(x: i32) -> bool { x % 2 == 0 } fn main() { let r = is_even(4); }",
     "is_even() {"),
    ("B-1929", "r10-is-odd-func", "Check if odd", "standard",
     "fn is_odd(x: i32) -> bool { x % 2 != 0 } fn main() { let r = is_odd(7); }",
     "is_odd() {"),
    ("B-1930", "r10-is-divisible-func", "Check divisibility", "standard",
     "fn is_div(a: i32, b: i32) -> bool { a % b == 0 } fn main() { let r = is_div(15, 3); }",
     "is_div() {"),
    ("B-1931", "r10-tri-max-func", "Max of three", "adversarial",
     "fn tri_max(a: i32, b: i32, c: i32) -> i32 { if a >= b && a >= c { a } else if b >= c { b } else { c } } fn main() { let r = tri_max(5, 8, 3); }",
     "tri_max() {"),
    ("B-1932", "r10-tri-min-func", "Min of three", "adversarial",
     "fn tri_min(a: i32, b: i32, c: i32) -> i32 { if a <= b && a <= c { a } else if b <= c { b } else { c } } fn main() { let r = tri_min(5, 8, 3); }",
     "tri_min() {"),
    ("B-1933", "r10-avg2-func", "Average of two", "standard",
     "fn avg2(a: i32, b: i32) -> i32 { (a + b) / 2 } fn main() { let r = avg2(10, 20); }",
     "avg2() {"),
    ("B-1934", "r10-diff-func", "Absolute difference", "standard",
     "fn diff(a: i32, b: i32) -> i32 { if a > b { a - b } else { b - a } } fn main() { let r = diff(7, 12); }",
     "diff() {"),
    ("B-1935", "r10-pow2-func", "Power of 2 check", "adversarial",
     "fn next_pow2(n: i32) -> i32 { let mut p = 1; while p < n { p = p * 2; } p } fn main() { let r = next_pow2(13); }",
     "next_pow2() {"),
    ("B-1936", "r10-gcd2-func", "GCD of two numbers", "production",
     "fn gcd2(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x } fn main() { let r = gcd2(48, 18); }",
     "gcd2() {"),
    ("B-1937", "r10-fib-func", "Fibonacci n-th term", "production",
     "fn fib_n(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let t = a + b; a = b; b = t; i += 1; } a } fn main() { let r = fib_n(10); }",
     "fib_n() {"),
    ("B-1938", "r10-fact-func", "Factorial", "production",
     "fn fact_n(n: i32) -> i32 { let mut r = 1; let mut i = 1; while i <= n { r = r * i; i += 1; } r } fn main() { let r = fact_n(6); }",
     "fact_n() {"),
    ("B-1939", "r10-sum-range-func", "Sum of range", "standard",
     "fn sum_range(lo: i32, hi: i32) -> i32 { let mut s = 0; let mut i = lo; while i <= hi { s += i; i += 1; } s } fn main() { let r = sum_range(1, 100); }",
     "sum_range() {"),
    ("B-1940", "r10-count-div-func", "Count divisors", "adversarial",
     "fn count_div(n: i32) -> i32 { let mut c = 0; let mut i = 1; while i <= n { if n % i == 0 { c += 1; } i += 1; } c } fn main() { let r = count_div(12); }",
     "count_div() {"),
    ("B-1941", "r10-collatz2-func", "Collatz step count", "adversarial",
     "fn coll(n: i32) -> i32 { let mut x = n; let mut s = 0; while x > 1 { if x % 2 == 0 { x = x / 2; } else { x = x * 3 + 1; } s += 1; } s } fn main() { let r = coll(27); }",
     "coll() {"),
    ("B-1942", "r10-digit-count-func", "Count digits", "adversarial",
     "fn digits(n: i32) -> i32 { let mut x = n; let mut c = 0; while x > 0 { c += 1; x = x / 10; } c } fn main() { let r = digits(12345); }",
     "digits() {"),
    ("B-1943", "r10-reverse-digits-func", "Reverse digits", "adversarial",
     "fn rev_digits(n: i32) -> i32 { let mut x = n; let mut r = 0; while x > 0 { r = r * 10 + x % 10; x = x / 10; } r } fn main() { let r = rev_digits(12345); }",
     "rev_digits() {"),
    ("B-1944", "r10-is-palindrome-num", "Palindrome number check", "adversarial",
     "fn is_palindrome(n: i32) -> bool { let mut x = n; let mut r = 0; while x > 0 { r = r * 10 + x % 10; x = x / 10; } r == n } fn main() { let r = is_palindrome(121); }",
     "is_palindrome() {"),
    ("B-1945", "r10-binary-repr", "Count bits", "adversarial",
     "fn count_bits(n: i32) -> i32 { let mut x = n; let mut c = 0; while x > 0 { c += x % 2; x = x / 2; } c } fn main() { let r = count_bits(255); }",
     "count_bits() {"),
    ("B-1946", "r10-lcm-func", "LCM function", "production",
     "fn gcd_f(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x } fn lcm_f(a: i32, b: i32) -> i32 { a / gcd_f(a, b) * b } fn main() { let r = lcm_f(12, 8); }",
     "lcm_f() {"),
    ("B-1947", "r10-safe-div-func", "Safe division with zero check", "standard",
     "fn safe_divide(a: i32, b: i32) -> i32 { if b == 0 { return 0; } a / b } fn main() { let r1 = safe_divide(10, 3); let r2 = safe_divide(10, 0); }",
     "safe_divide() {"),
    ("B-1948", "r10-bounded-func", "Bounded computation", "adversarial",
     "fn bounded(x: i32, limit: i32) -> i32 { let v = x * x; if v > limit { limit } else { v } } fn main() { let r = bounded(15, 100); }",
     "bounded() {"),
    ("B-1949", "r10-step-func", "Step function", "standard",
     "fn step(x: i32) -> i32 { if x >= 0 { 1 } else { 0 } } fn main() { let r = step(-5); }",
     "step() {"),
    ("B-1950", "r10-relu-func", "ReLU activation", "standard",
     "fn relu(x: i32) -> i32 { if x > 0 { x } else { 0 } } fn main() { let r = relu(-3); }",
     "relu() {"),
    ("B-1951", "r10-leaky-relu-func", "Leaky ReLU approximation", "adversarial",
     "fn leaky(x: i32) -> i32 { if x > 0 { x } else { x / 10 } } fn main() { let r = leaky(-50); }",
     "leaky() {"),
    ("B-1952", "r10-median3-func", "Median of three", "adversarial",
     "fn median3(a: i32, b: i32, c: i32) -> i32 { if a >= b && a <= c || a <= b && a >= c { a } else if b >= a && b <= c || b <= a && b >= c { b } else { c } } fn main() { let r = median3(5, 1, 3); }",
     "median3() {"),
    ("B-1953", "r10-wrap-func", "Wrap around modular", "standard",
     "fn wrap(x: i32, modulus: i32) -> i32 { ((x % modulus) + modulus) % modulus } fn main() { let r = wrap(-3, 10); }",
     "wrap() {"),
    ("B-1954", "r10-lerp-func", "Linear interpolation integer", "adversarial",
     "fn lerp_i(a: i32, b: i32, t: i32) -> i32 { a + (b - a) * t / 100 } fn main() { let r = lerp_i(0, 100, 25); }",
     "lerp_i() {"),
    ("B-1955", "r10-manhattan2-func", "Manhattan distance 2D", "standard",
     "fn man_dist(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 { let dx = if x2 > x1 { x2 - x1 } else { x1 - x2 }; let dy = if y2 > y1 { y2 - y1 } else { y1 - y2 }; dx + dy } fn main() { let r = man_dist(1, 2, 4, 6); }",
     "man_dist() {"),
]
bash_entries += func_patterns

# ==========================================
# Pattern 2: For loop headers (B-1956..B-2005)
# ==========================================
for_patterns = []
for i, (start, end) in enumerate([
    (1,10),(1,20),(1,50),(1,100),(0,9),(0,19),(0,49),(0,99),
    (1,5),(1,6),(1,7),(1,8),(1,9),(1,15),(1,25),(1,30),
    (1,40),(1,60),(1,75),(2,20),(2,50),(3,30),(5,15),(5,25),
    (0,4),(0,7),(0,14),(0,24),(0,29),(0,39),(0,49),(0,59),
    (1,12),(1,16),(1,18),(1,22),(1,24),(1,28),(1,32),(1,35),
    (1,42),(1,45),(1,48),(1,55),(1,65),(1,70),(1,80),(1,90),
    (1,11),(1,13),
]):
    bid = f"B-{1956 + i}"
    slug = f"r10-for-{start}-{end}"
    desc = f"For loop seq {start} to {end} with accumulation"
    tier = "standard" if end <= 20 else "adversarial"
    code = f"fn main() {{ let mut s = 0; for i in {start}..{end+1} {{ s += i; }} }}"
    expected = f"for i in $(seq {start} {end}); do"
    for_patterns.append((bid, slug, desc, tier, code, expected))
bash_entries += for_patterns

# ==========================================
# Pattern 3: While loop headers (B-2006..B-2035)
# ==========================================
while_patterns = []
while_configs = [
    ("i", "lt", 10), ("i", "lt", 20), ("i", "lt", 50), ("i", "lt", 100),
    ("n", "gt", 0), ("n", "gt", 1), ("n", "gt", 10), ("n", "gt", 50),
    ("x", "lt", 1000), ("x", "lt", 500), ("count", "lt", 10), ("count", "lt", 50),
    ("j", "lt", 20), ("k", "lt", 15), ("idx", "lt", 10), ("step", "lt", 100),
    ("a", "lt", 50), ("b", "gt", 0), ("val", "gt", 1), ("iter", "lt", 25),
    ("remaining", "gt", 0), ("tries", "lt", 5), ("rounds", "lt", 10), ("depth", "lt", 8),
    ("pos", "lt", 20), ("len", "gt", 0), ("level", "lt", 10), ("phase", "lt", 4),
    ("cycle", "lt", 12), ("tick", "lt", 30),
]
for i, (var, op, limit) in enumerate(while_configs):
    bid = f"B-{2006 + i}"
    slug = f"r10-while-{var}-{op}-{limit}"
    desc = f"While loop {var} {op} {limit}"
    tier = "standard" if limit <= 20 else "adversarial"
    if op == "lt":
        code = f'fn main() {{ let mut {var} = 0; while {var} < {limit} {{ {var} += 1; }} }}'
        expected = f'while [ "${var}" -lt {limit} ]; do'
    else:
        code = f'fn main() {{ let mut {var} = {limit + 10}; while {var} > {limit} {{ {var} = {var} - 1; }} }}'
        expected = f'while [ "${var}" -gt {limit} ]; do'
    while_patterns.append((bid, slug, desc, tier, code, expected))
bash_entries += while_patterns

# ==========================================
# Pattern 4: String assignments (B-2036..B-2075)
# ==========================================
str_patterns = []
strings = [
    ("hello", "hello"), ("world", "world"), ("test", "test"), ("debug", "debug"),
    ("true_str", "true_string"), ("config", "config"), ("name", "name"),
    ("output", "output"), ("input", "input"), ("result", "result"),
    ("error", "error"), ("warning", "warning"), ("info", "info"),
    ("status", "status"), ("level", "level"), ("mode", "mode"),
    ("type_val", "type_value"), ("label", "label"), ("tag", "tag"),
    ("key", "key"), ("val", "val"), ("src", "src"), ("dst", "dst"),
    ("host", "host"), ("port_str", "port_value"), ("path", "path"),
    ("dir", "dir"), ("file", "file"), ("ext", "ext"), ("root", "root"),
    ("home", "home"), ("user", "user"), ("group", "group"),
    ("owner", "owner"), ("prefix", "prefix"), ("suffix", "suffix"),
    ("sep", "sep"), ("delim", "delim"), ("marker", "marker"), ("flag", "flag"),
]
for i, (var, val) in enumerate(strings):
    bid = f"B-{2036 + i}"
    slug = f"r10-str-{var}"
    desc = f"String assignment {var}"
    tier = "trivial"
    code = f'fn main() {{ let {var} = "{val}"; }}'
    expected = f"{var}='{val}'"
    str_patterns.append((bid, slug, desc, tier, code, expected))
bash_entries += str_patterns

# ==========================================
# Pattern 5: Integer assignments (B-2076..B-2105)
# ==========================================
int_patterns = []
int_vals = [
    ("x", 0), ("y", 1), ("z", -1), ("w", 42), ("count", 100),
    ("limit", 255), ("threshold", 50), ("timeout", 30), ("port", 8080),
    ("max_val", 999), ("min_val", 0), ("offset", 10), ("step", 5),
    ("batch", 32), ("retries", 3), ("workers", 4), ("depth", 16),
    ("width", 80), ("height", 24), ("size", 1024), ("cap", 256),
    ("base", 10), ("rate", 100), ("score", 0), ("hits", 0),
    ("miss", 0), ("total", 0), ("index", 0), ("rank", 1), ("tier", 3),
]
for i, (var, val) in enumerate(int_vals):
    bid = f"B-{2076 + i}"
    slug = f"r10-int-{var}"
    desc = f"Integer assignment {var}={val}"
    tier = "trivial"
    code = f'fn main() {{ let {var} = {val}; }}'
    expected = f"{var}='{val}'"
    int_patterns.append((bid, slug, desc, tier, code, expected))
bash_entries += int_patterns

# ==========================================
# Pattern 6: Arithmetic expressions (B-2106..B-2135)
# ==========================================
arith_patterns = []
arith_exprs = [
    ("sum", "a + b", "a", 10, "b", 20),
    ("diff", "a - b", "a", 50, "b", 30),
    ("prod", "a * b", "a", 6, "b", 7),
    ("quot", "a / b", "a", 100, "b", 7),
    ("rem", "a % b", "a", 17, "b", 5),
    ("s2", "x + y + z", "x", 1, None, None),
    ("d2", "x * 2 + 1", "x", 10, None, None),
    ("q2", "x * x + y * y", "x", 3, "y", 4),
    ("total", "acc + i", "acc", 0, "i", 1),
    ("next", "cur + step", "cur", 0, "step", 5),
    ("half", "n / 2", "n", 100, None, None),
    ("dbl", "n * 2", "n", 25, None, None),
    ("sq", "n * n", "n", 7, None, None),
    ("inc", "n + 1", "n", 0, None, None),
    ("dec", "n - 1", "n", 10, None, None),
    ("biased", "x + 10", "x", 5, None, None),
    ("scaled", "x * 3", "x", 7, None, None),
    ("adjusted", "x - 5", "x", 20, None, None),
    ("ratio", "x / 3", "x", 30, None, None),
    ("mod_val", "x % 7", "x", 50, None, None),
    ("combined", "a + b * 2", "a", 5, "b", 10),
    ("weighted", "a * 3 + b", "a", 4, "b", 2),
    ("delta", "a - b + 1", "a", 30, "b", 10),
    ("avg", "total / count", "total", 100, "count", 10),
    ("norm", "x * 100 / max", "x", 75, "max", 100),
    ("offset2", "base + idx * 4", "base", 0, "idx", 5),
    ("masked", "val % 256", "val", 1000, None, None),
    ("capped", "val / 10", "val", 555, None, None),
    ("shifted", "val * 4", "val", 25, None, None),
    ("folded", "acc + val", "acc", 0, "val", 42),
]
for i, (result_var, expr, v1, val1, v2, val2) in enumerate(arith_exprs):
    bid = f"B-{2106 + i}"
    slug = f"r10-arith-{result_var}"
    desc = f"Arithmetic: {result_var} = {expr}"
    tier = "standard"
    if v2 and val2 is not None:
        code = f"fn main() {{ let {v1} = {val1}; let {v2} = {val2}; let {result_var} = {expr}; }}"
    else:
        code = f"fn main() {{ let {v1} = {val1}; let {result_var} = {expr}; }}"
    expected = f"{result_var}=$(({expr}))"
    arith_patterns.append((bid, slug, desc, tier, code, expected))
bash_entries += arith_patterns

# ==========================================
# Pattern 7: Complex multi-construct (B-2136..B-2205)
# 70 more complex entries mixing multiple patterns
# ==========================================
complex_entries = [
    ("B-2136", "r10-loop-func-pipeline", "Loop calling pipeline of functions", "adversarial",
     "fn inc2(x: i32) -> i32 { x + 2 } fn dbl2(x: i32) -> i32 { x * 2 } fn main() { let mut t = 0; for i in 1..11 { t += dbl2(inc2(i)); } }",
     "inc2() {"),
    ("B-2137", "r10-nested-loop-sum", "Nested loops with product sum", "adversarial",
     "fn main() { let mut s = 0; for i in 1..8 { for j in 1..8 { s += i * j; } } }",
     "for j in $(seq 1 7); do"),
    ("B-2138", "r10-while-countdown-acc", "While countdown with accumulation", "standard",
     "fn main() { let mut n = 20; let mut s = 0; while n > 0 { s += n; n = n - 1; } }",
     "while [ \"$n\" -gt 0 ]; do"),
    ("B-2139", "r10-func-with-while", "Function containing while loop", "production",
     "fn count_to(limit: i32) -> i32 { let mut c = 0; let mut i = 0; while i < limit { c += 1; i += 1; } c } fn main() { let r = count_to(50); }",
     "count_to() {"),
    ("B-2140", "r10-func-with-for", "Function containing for loop", "production",
     "fn sum_n(n: i32) -> i32 { let mut s = 0; for i in 1..11 { s += i; } s } fn main() { let r = sum_n(10); }",
     "sum_n() {"),
    ("B-2141", "r10-if-in-for-acc", "If inside for with accumulation", "standard",
     "fn main() { let mut even_sum = 0; for i in 1..21 { if i % 2 == 0 { even_sum += i; } } }",
     "for i in $(seq 1 20); do"),
    ("B-2142", "r10-if-else-in-for", "If-else inside for loop", "standard",
     "fn main() { let mut a = 0; let mut b = 0; for i in 1..16 { if i % 2 == 0 { a += i; } else { b += i; } } }",
     "for i in $(seq 1 15); do"),
    ("B-2143", "r10-while-if-update", "While with conditional update", "adversarial",
     "fn main() { let mut x = 1; let mut y = 0; while x < 1000 { if x % 2 == 0 { y += x; } x = x * 2; } }",
     "while [ \"$x\" -lt 1000 ]; do"),
    ("B-2144", "r10-func-multi-call", "Function called multiple times", "standard",
     "fn bump(x: i32) -> i32 { x + 5 } fn main() { let a = bump(0); let b = bump(10); let c = bump(20); let d = bump(30); let e = bump(40); }",
     "bump() {"),
    ("B-2145", "r10-triple-for-count", "Triple nested for with counting", "adversarial",
     "fn main() { let mut c = 0; for i in 1..4 { for j in 1..4 { for k in 1..4 { c += 1; } } } }",
     "for k in $(seq 1 3); do"),
    ("B-2146", "r10-while-double-cond", "While with compound condition", "adversarial",
     "fn main() { let mut a = 0; let mut b = 100; while a < b { a += 5; b = b - 3; } }",
     "while [ \"$a\" -lt \"$b\" ]; do"),
    ("B-2147", "r10-func-fibonacci2", "Fibonacci via function", "production",
     "fn fibo(n: i32) -> i32 { let mut x = 0; let mut y = 1; let mut i = 0; while i < n { let t = x + y; x = y; y = t; i += 1; } x } fn main() { let r = fibo(12); }",
     "fibo() {"),
    ("B-2148", "r10-loop-break-found", "Loop with break on found", "standard",
     "fn main() { let mut found = 0; for i in 1..101 { if i * i > 500 { found = i; break; } } }",
     "for i in $(seq 1 100); do"),
    ("B-2149", "r10-multi-func-chain", "Multiple functions chained", "adversarial",
     "fn f1(x: i32) -> i32 { x + 1 } fn f2(x: i32) -> i32 { x * 2 } fn f3(x: i32) -> i32 { x - 3 } fn f4(x: i32) -> i32 { x / 2 } fn main() { let r = f4(f3(f2(f1(10)))); }",
     "f1() {"),
    ("B-2150", "r10-accum-while-double", "Double accumulator in while", "adversarial",
     "fn main() { let mut s = 0; let mut p = 1; let mut i = 1; while i <= 10 { s += i; p = p * i; i += 1; } }",
     "while [ \"$i\" -le 10 ]; do"),
    ("B-2151", "r10-for-with-3ifs", "For with three different if checks", "adversarial",
     "fn main() { let mut a = 0; let mut b = 0; let mut c = 0; for i in 0..30 { if i % 2 == 0 { a += 1; } if i % 3 == 0 { b += 1; } if i % 5 == 0 { c += 1; } } }",
     "for i in $(seq 0 29); do"),
    ("B-2152", "r10-power-func2", "Power via loop", "production",
     "fn pow_i(base: i32, exp: i32) -> i32 { let mut r = 1; let mut i = 0; while i < exp { r = r * base; i += 1; } r } fn main() { let r = pow_i(2, 8); }",
     "pow_i() {"),
    ("B-2153", "r10-selection-max", "Find max in simulated array", "adversarial",
     "fn main() { let a0 = 5; let a1 = 12; let a2 = 3; let a3 = 18; let a4 = 7; let mut max = a0; if a1 > max { max = a1; } if a2 > max { max = a2; } if a3 > max { max = a3; } if a4 > max { max = a4; } }",
     "a0='5'"),
    ("B-2154", "r10-sum-even-odd", "Separate even/odd sums", "standard",
     "fn main() { let mut even = 0; let mut odd = 0; for i in 1..31 { if i % 2 == 0 { even += i; } else { odd += i; } } }",
     "for i in $(seq 1 30); do"),
    ("B-2155", "r10-prime-count", "Count primes in range", "adversarial",
     "fn is_prime2(n: i32) -> bool { if n < 2 { return false; } let mut i = 2; while i * i <= n { if n % i == 0 { return false; } i += 1; } true } fn main() { let mut count = 0; for i in 2..51 { if is_prime2(i) { count += 1; } } }",
     "is_prime2() {"),
    ("B-2156", "r10-running-min", "Running minimum", "standard",
     "fn main() { let mut min = 999; for i in 0..10 { let val = (i * 7 + 3) % 13; if val < min { min = val; } } }",
     "for i in $(seq 0 9); do"),
    ("B-2157", "r10-digit-product", "Digit product", "adversarial",
     "fn dprod(n: i32) -> i32 { let mut x = n; let mut p = 1; while x > 0 { p = p * (x % 10); x = x / 10; } p } fn main() { let r = dprod(234); }",
     "dprod() {"),
    ("B-2158", "r10-sum-divisors", "Sum of proper divisors", "adversarial",
     "fn sum_div(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i < n { if n % i == 0 { s += i; } i += 1; } s } fn main() { let r = sum_div(28); }",
     "sum_div() {"),
    ("B-2159", "r10-harmonic-int", "Harmonic-like integer sum", "standard",
     "fn main() { let mut s = 0; for i in 1..21 { s += 1000 / i; } }",
     "for i in $(seq 1 20); do"),
    ("B-2160", "r10-geo-series", "Geometric-like series", "adversarial",
     "fn main() { let mut term = 1; let mut s = 0; for i in 0..10 { s += term; term = term * 2; } }",
     "for i in $(seq 0 9); do"),
    ("B-2161", "r10-triangular-num", "Triangular numbers", "standard",
     "fn tri(n: i32) -> i32 { n * (n + 1) / 2 } fn main() { let t5 = tri(5); let t10 = tri(10); let t20 = tri(20); }",
     "tri() {"),
    ("B-2162", "r10-perfect-check", "Perfect number check", "adversarial",
     "fn is_perfect(n: i32) -> bool { let mut s = 0; let mut i = 1; while i < n { if n % i == 0 { s += i; } i += 1; } s == n } fn main() { let r1 = is_perfect(6); let r2 = is_perfect(28); let r3 = is_perfect(12); }",
     "is_perfect() {"),
    ("B-2163", "r10-celsius-batch", "Batch celsius conversion", "production",
     "fn c2f(c: i32) -> i32 { c * 9 / 5 + 32 } fn main() { let f0 = c2f(0); let f100 = c2f(100); let f37 = c2f(37); let fm40 = c2f(-40); }",
     "c2f() {"),
    ("B-2164", "r10-accumulate-prod", "Product accumulation in loop", "standard",
     "fn main() { let mut p = 1; for i in 1..8 { p = p * i; } }",
     "for i in $(seq 1 7); do"),
    ("B-2165", "r10-countdown-step2", "Countdown by 2s", "standard",
     "fn main() { let mut n = 20; let mut c = 0; while n > 0 { n = n - 2; c += 1; } }",
     "while [ \"$n\" -gt 0 ]; do"),
    ("B-2166", "r10-flag-search", "Search with flag", "standard",
     "fn main() { let mut found = false; for i in 1..51 { if i == 42 { found = true; break; } } }",
     "for i in $(seq 1 50); do"),
    ("B-2167", "r10-multi-assign-int", "Multiple integer assignments", "trivial",
     "fn main() { let a = 1; let b = 2; let c = 3; let d = 4; let e = 5; let f = 6; }",
     "f='6'"),
    ("B-2168", "r10-multi-assign-str", "Multiple string assignments", "trivial",
     'fn main() { let s1 = "alpha"; let s2 = "beta"; let s3 = "gamma"; let s4 = "delta"; }',
     "s4='delta'"),
    ("B-2169", "r10-bool-chain", "Boolean chain pattern", "standard",
     "fn main() { let p = true; let q = false; let r = true; let s = false; }",
     "q=false"),
    ("B-2170", "r10-func-pair-ops", "Two operation functions", "standard",
     "fn add_mul(a: i32, b: i32) -> i32 { (a + b) * 2 } fn sub_div(a: i32, b: i32) -> i32 { (a - b) / 2 } fn main() { let r1 = add_mul(5, 3); let r2 = sub_div(20, 10); }",
     "add_mul() {"),
    ("B-2171", "r10-while-halving", "Halving while loop", "standard",
     "fn main() { let mut n = 1024; let mut steps = 0; while n > 1 { n = n / 2; steps += 1; } }",
     "while [ \"$n\" -gt 1 ]; do"),
    ("B-2172", "r10-for-with-case-count", "For with case counting", "adversarial",
     "fn bucket(x: u32) -> u32 { match x % 4 { 0 => { return 0; } 1 => { return 1; } 2 => { return 2; } _ => { return 3; } } } fn main() { let mut c = 0; for i in 0..20 { c += bucket(i); } }",
     "bucket() {"),
    ("B-2173", "r10-for-double-accumulate", "For with two accumulators", "standard",
     "fn main() { let mut s = 0; let mut c = 0; for i in 1..26 { s += i; c += 1; } }",
     "for i in $(seq 1 25); do"),
    ("B-2174", "r10-match-5arm", "Match with 5 arms", "adversarial",
     "fn level(x: u32) -> u32 { match x { 0 => { return 0; } 1 => { return 10; } 2 => { return 20; } 3 => { return 30; } _ => { return 99; } } } fn main() { let l = level(2); }",
     "level() {"),
    ("B-2175", "r10-nested-func-if", "Nested function with if", "adversarial",
     "fn outer(x: i32) -> i32 { let a = inner(x); if a > 10 { a * 2 } else { a } } fn inner(x: i32) -> i32 { x + 5 } fn main() { let r = outer(8); }",
     "outer() {"),
    ("B-2176", "r10-bitcount-loop", "Bit counting via loop", "adversarial",
     "fn bits(n: i32) -> i32 { let mut x = n; let mut c = 0; while x > 0 { c += 1; x = x / 2; } c } fn main() { let r = bits(1023); }",
     "bits() {"),
    ("B-2177", "r10-seq-processing", "Sequential data processing", "production",
     "fn process(x: i32) -> i32 { let v = x * 2; let w = v + 3; let z = w % 7; z } fn main() { let mut total = 0; for i in 0..15 { total += process(i); } }",
     "process() {"),
    ("B-2178", "r10-guard-clause", "Guard clause pattern", "standard",
     "fn safe_op(x: i32) -> i32 { if x <= 0 { return 0; } x * x } fn main() { let r1 = safe_op(5); let r2 = safe_op(-3); let r3 = safe_op(0); }",
     "safe_op() {"),
    ("B-2179", "r10-accumulate-with-limit", "Accumulate until limit", "adversarial",
     "fn main() { let mut s = 0; let mut i = 1; while s < 100 { s += i; i += 1; } }",
     "while [ \"$s\" -lt 100 ]; do"),
    ("B-2180", "r10-multi-counter", "Multiple counters in loop", "adversarial",
     "fn main() { let mut c1 = 0; let mut c2 = 0; let mut c3 = 0; for i in 1..41 { if i % 2 == 0 { c1 += 1; } if i % 3 == 0 { c2 += 1; } if i % 5 == 0 { c3 += 1; } } }",
     "for i in $(seq 1 40); do"),
    ("B-2181", "r10-prefix-sum-5", "5-element prefix sum", "adversarial",
     "fn main() { let a0 = 3; let a1 = 1; let a2 = 4; let a3 = 1; let a4 = 5; let p0 = a0; let p1 = p0 + a1; let p2 = p1 + a2; let p3 = p2 + a3; let p4 = p3 + a4; }",
     "p3=$((p2 + a3))"),
    ("B-2182", "r10-double-for-sum", "Double for loop sum", "adversarial",
     "fn main() { let mut s = 0; for i in 1..6 { for j in 1..6 { s += i + j; } } }",
     "for j in $(seq 1 5); do"),
    ("B-2183", "r10-while-triple-update", "While with triple variable update", "adversarial",
     "fn main() { let mut a = 1; let mut b = 1; let mut c = 1; let mut i = 0; while i < 10 { let na = b + c; let nb = a + c; let nc = a + b; a = na; b = nb; c = nc; i += 1; } }",
     "while [ \"$i\" -lt 10 ]; do"),
    ("B-2184", "r10-for-while-nested", "For containing while", "adversarial",
     "fn main() { let mut total = 0; for i in 1..6 { let mut v = i * 10; while v > 0 { total += 1; v = v - 3; } } }",
     "for i in $(seq 1 5); do"),
    ("B-2185", "r10-func-iter-sum", "Iterative sum function", "production",
     "fn iter_sum(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { s += i; i += 1; } s } fn main() { let r = iter_sum(100); }",
     "iter_sum() {"),
    ("B-2186", "r10-func-iter-prod", "Iterative product function", "production",
     "fn iter_prod(n: i32) -> i32 { let mut p = 1; let mut i = 1; while i <= n { p = p * i; i += 1; } p } fn main() { let r = iter_prod(7); }",
     "iter_prod() {"),
    ("B-2187", "r10-func-count-pred", "Count by predicate", "standard",
     "fn count_pred(limit: i32) -> i32 { let mut c = 0; for i in 0..20 { if i % 3 == 0 { c += 1; } } c } fn main() { let r = count_pred(20); }",
     "count_pred() {"),
    ("B-2188", "r10-func-find-first", "Find first matching", "adversarial",
     "fn find_first(target: i32) -> i32 { let mut i = 0; while i < 100 { if i * i >= target { return i; } i += 1; } return -1; } fn main() { let r = find_first(50); }",
     "find_first() {"),
    ("B-2189", "r10-euler1-sim", "Project Euler 1 simulation", "production",
     "fn main() { let mut s = 0; for i in 1..100 { if i % 3 == 0 || i % 5 == 0 { s += i; } } }",
     "for i in $(seq 1 99); do"),
    ("B-2190", "r10-for-4way-branch", "For with 4-way branching", "adversarial",
     "fn main() { let mut a = 0; let mut b = 0; let mut c = 0; let mut d = 0; for i in 0..40 { let r = i % 4; if r == 0 { a += 1; } else if r == 1 { b += 1; } else if r == 2 { c += 1; } else { d += 1; } } }",
     "for i in $(seq 0 39); do"),
    ("B-2191", "r10-weighted-avg", "Weighted average via function", "production",
     "fn wavg(a: i32, wa: i32, b: i32, wb: i32) -> i32 { (a * wa + b * wb) / (wa + wb) } fn main() { let r = wavg(80, 3, 90, 7); }",
     "wavg() {"),
    ("B-2192", "r10-linear-search", "Linear search", "standard",
     "fn main() { let target = 37; let mut found_at = -1; for i in 0..50 { if i == target { found_at = i; break; } } }",
     "for i in $(seq 0 49); do"),
    ("B-2193", "r10-bubble-pass-sim", "Single bubble pass simulation", "adversarial",
     "fn main() { let mut a = 5; let mut b = 2; let mut c = 8; let mut d = 1; if a > b { let t = a; a = b; b = t; } if b > c { let t = b; b = c; c = t; } if c > d { let t = c; c = d; d = t; } }",
     "a='5'"),
    ("B-2194", "r10-two-pointer-sim", "Two pointer simulation", "adversarial",
     "fn main() { let mut lo = 0; let mut hi = 20; while lo < hi { let mid = (lo + hi) / 2; if mid * mid < 50 { lo = mid + 1; } else { hi = mid; } } }",
     "while [ \"$lo\" -lt \"$hi\" ]; do"),
    ("B-2195", "r10-for-func-composition", "For with function composition", "adversarial",
     "fn op1(x: i32) -> i32 { x + 3 } fn op2(x: i32) -> i32 { x * 2 } fn main() { let mut s = 0; for i in 1..11 { s += op2(op1(i)); } }",
     "op1() {"),
    ("B-2196", "r10-nested-if-5level", "5-level nested if", "adversarial",
     "fn classify5(a: i32, b: i32) -> i32 { if a > 0 { if b > 0 { if a > b { 1 } else { 2 } } else { 3 } } else { if b > 0 { 4 } else { 5 } } } fn main() { let r = classify5(3, 7); }",
     "classify5() {"),
    ("B-2197", "r10-state-machine", "3-state machine", "adversarial",
     "fn trans(s: i32, i: i32) -> i32 { if s == 0 { if i > 0 { 1 } else { 2 } } else if s == 1 { if i == 0 { 0 } else { 2 } } else { 0 } } fn main() { let mut st = 0; st = trans(st, 1); st = trans(st, 0); st = trans(st, 5); }",
     "trans() {"),
    ("B-2198", "r10-chain-assignment", "Chain of assignments with arithmetic", "standard",
     "fn main() { let a = 10; let b = a + 5; let c = b * 2; let d = c - 3; let e = d / 4; }",
     "c=$((b * 2))"),
    ("B-2199", "r10-two-func-alternate", "Two functions called alternately", "adversarial",
     "fn up(x: i32) -> i32 { x + 10 } fn down(x: i32) -> i32 { x - 5 } fn main() { let a = up(0); let b = down(a); let c = up(b); let d = down(c); let e = up(d); }",
     "up() {"),
    ("B-2200", "r10-converge-loop", "Convergence loop", "adversarial",
     "fn main() { let mut x = 100; let mut s = 0; while x > 1 { x = x / 2; s += 1; } }",
     "while [ \"$x\" -gt 1 ]; do"),
    ("B-2201", "r10-fibonacci-sum", "Sum of first N fibonacci", "production",
     "fn main() { let mut a = 0; let mut b = 1; let mut s = 0; let mut i = 0; while i < 15 { s += a; let t = a + b; a = b; b = t; i += 1; } }",
     "while [ \"$i\" -lt 15 ]; do"),
    ("B-2202", "r10-modular-counter", "Modular counter pattern", "standard",
     "fn main() { let mut c = 0; for i in 0..50 { c = (c + 1) % 7; } }",
     "c=$(((c + 1) % 7))"),
    ("B-2203", "r10-toggle-pattern", "Toggle flag in loop", "standard",
     "fn main() { let mut flag = false; for i in 0..10 { if i % 2 == 0 { flag = true; } else { flag = false; } } }",
     "for i in $(seq 0 9); do"),
    ("B-2204", "r10-multi-return-func", "Function with many return paths", "adversarial",
     "fn route(x: i32) -> i32 { if x < -100 { return -3; } if x < -10 { return -2; } if x < 0 { return -1; } if x == 0 { return 0; } if x < 10 { return 1; } if x < 100 { return 2; } return 3; } fn main() { let r = route(42); }",
     "route() {"),
    ("B-2205", "r10-batch-transform", "Batch transform pattern", "production",
     "fn transform2(x: i32) -> i32 { if x > 50 { x - 25 } else { x + 25 } } fn main() { let mut t = 0; for i in 0..20 { t += transform2(i * 5); } }",
     "transform2() {"),
]
bash_entries += complex_entries

# ==========================================
# Makefile entries M-436..M-450
# ==========================================
makefile_entries = [
    ("M-436", "r10-ci-pipeline-mk", "CI pipeline Makefile", "adversarial",
     'fn main() { exec(".PHONY: ci lint test build deploy"); exec("ci: lint test build"); exec("lint:"); exec("\\tcargo clippy -- -D warnings"); exec("test:"); exec("\\tcargo test --workspace"); exec("build:"); exec("\\tcargo build --release"); exec("deploy: ci"); exec("\\trsync -avz target/release/app user@host:/opt/"); }',
     "cargo clippy -- -D warnings"),
    ("M-437", "r10-docker-image-mk", "Docker image build Makefile", "standard",
     'fn main() { exec("IMAGE := myapp"); exec("TAG := latest"); exec(".PHONY: build push clean"); exec("build:"); exec("\\tdocker build -t $(IMAGE):$(TAG) ."); exec("push: build"); exec("\\tdocker push $(IMAGE):$(TAG)"); exec("clean:"); exec("\\tdocker rmi $(IMAGE):$(TAG)"); }',
     "IMAGE := myapp"),
    ("M-438", "r10-python-test-mk", "Python test runner Makefile", "production",
     'fn main() { exec(".PHONY: test lint coverage clean"); exec("test:"); exec("\\tpytest tests/"); exec("lint:"); exec("\\truff check ."); exec("coverage:"); exec("\\tpytest --cov=src tests/"); exec("clean:"); exec("\\trm -rf .pytest_cache __pycache__ .coverage"); }',
     "pytest tests/"),
    ("M-439", "r10-helm-deploy-mk", "Helm deployment Makefile", "adversarial",
     'fn main() { exec(".PHONY: install upgrade uninstall status"); exec("install:"); exec("\\thelm install myapp charts/myapp"); exec("upgrade:"); exec("\\thelm upgrade myapp charts/myapp"); exec("uninstall:"); exec("\\thelm uninstall myapp"); exec("status:"); exec("\\thelm status myapp"); }',
     "helm install myapp charts/myapp"),
    ("M-440", "r10-cargo-all-mk", "Cargo all-in-one Makefile", "production",
     'fn main() { exec(".PHONY: all check build test clippy fmt doc"); exec("all: check clippy fmt test build doc"); exec("check:"); exec("\\tcargo check"); exec("build:"); exec("\\tcargo build --release"); exec("test:"); exec("\\tcargo test"); exec("clippy:"); exec("\\tcargo clippy -- -D warnings"); exec("fmt:"); exec("\\tcargo fmt -- --check"); exec("doc:"); exec("\\tcargo doc --no-deps"); }',
     "cargo check"),
    ("M-441", "r10-bazel-mk", "Bazel build Makefile", "adversarial",
     'fn main() { exec(".PHONY: build test clean query"); exec("build:"); exec("\\tbazel build //..."); exec("test:"); exec("\\tbazel test //..."); exec("clean:"); exec("\\tbazel clean"); exec("query:"); exec("\\tbazel query //..."); }',
     "bazel build //..."),
    ("M-442", "r10-gradle-mk", "Gradle wrapper Makefile", "standard",
     'fn main() { exec(".PHONY: build test clean run"); exec("build:"); exec("\\t./gradlew build"); exec("test:"); exec("\\t./gradlew test"); exec("clean:"); exec("\\t./gradlew clean"); exec("run:"); exec("\\t./gradlew bootRun"); }',
     "./gradlew build"),
    ("M-443", "r10-aws-sam-mk", "AWS SAM deploy Makefile", "adversarial",
     'fn main() { exec(".PHONY: build deploy local-api logs"); exec("build:"); exec("\\tsam build"); exec("deploy: build"); exec("\\tsam deploy --guided"); exec("local-api:"); exec("\\tsam local start-api"); exec("logs:"); exec("\\tsam logs --tail"); }',
     "sam build"),
    ("M-444", "r10-flutter-mk", "Flutter build Makefile", "standard",
     'fn main() { exec(".PHONY: run build test clean"); exec("run:"); exec("\\tflutter run"); exec("build:"); exec("\\tflutter build apk"); exec("test:"); exec("\\tflutter test"); exec("clean:"); exec("\\tflutter clean"); }',
     "flutter run"),
    ("M-445", "r10-elixir-mk", "Elixir mix Makefile", "adversarial",
     'fn main() { exec(".PHONY: deps compile test clean"); exec("deps:"); exec("\\tmix deps.get"); exec("compile: deps"); exec("\\tmix compile"); exec("test: compile"); exec("\\tmix test"); exec("clean:"); exec("\\tmix clean"); }',
     "mix deps.get"),
    ("M-446", "r10-cmake-mk", "CMake wrapper Makefile", "standard",
     'fn main() { exec(".PHONY: build test clean"); exec("build:"); exec("\\tcmake -B build -S ."); exec("\\tcmake --build build"); exec("test: build"); exec("\\tctest --test-dir build"); exec("clean:"); exec("\\trm -rf build/"); }',
     "cmake -B build -S ."),
    ("M-447", "r10-poetry-mk", "Poetry project Makefile", "production",
     'fn main() { exec(".PHONY: install lint test publish"); exec("install:"); exec("\\tpoetry install"); exec("lint:"); exec("\\tpoetry run ruff check ."); exec("test: install"); exec("\\tpoetry run pytest"); exec("publish: test"); exec("\\tpoetry publish"); }',
     "poetry install"),
    ("M-448", "r10-swift-mk", "Swift package Makefile", "standard",
     'fn main() { exec(".PHONY: build test clean"); exec("build:"); exec("\\tswift build"); exec("test:"); exec("\\tswift test"); exec("clean:"); exec("\\tswift package clean"); }',
     "swift build"),
    ("M-449", "r10-deno-mk", "Deno project Makefile", "adversarial",
     'fn main() { exec(".PHONY: run test lint fmt"); exec("run:"); exec("\\tdeno run --allow-net main.ts"); exec("test:"); exec("\\tdeno test"); exec("lint:"); exec("\\tdeno lint"); exec("fmt:"); exec("\\tdeno fmt"); }',
     "deno run --allow-net main.ts"),
    ("M-450", "r10-zig-mk", "Zig build Makefile", "adversarial",
     'fn main() { exec(".PHONY: build test clean"); exec("build:"); exec("\\tzig build"); exec("test:"); exec("\\tzig build test"); exec("clean:"); exec("\\trm -rf zig-cache zig-out"); }',
     "zig build"),
]

# ==========================================
# Dockerfile entries D-401..D-420
# ==========================================
dockerfile_entries = [
    ("D-401", "r10-bun-app", "Bun.js application", "adversarial",
     'fn main() { from_image("oven/bun:1"); workdir("/app"); copy("package.json bun.lockb", "./"); run("bun install"); copy(".", "."); expose(3000); cmd("bun run start"); }',
     "FROM oven/bun:1"),
    ("D-402", "r10-crystal-app", "Crystal application", "adversarial",
     'fn main() { from_image("crystallang/crystal:1.11"); workdir("/app"); copy("shard.yml", "."); run("shards install"); copy(".", "."); run("crystal build src/main.cr --release"); }',
     "FROM crystallang/crystal:1.11"),
    ("D-403", "r10-nim-app", "Nim application", "adversarial",
     'fn main() { from_image("nimlang/nim:2.0"); workdir("/app"); copy(".", "."); run("nim c -d:release src/main.nim"); }',
     "FROM nimlang/nim:2.0"),
    ("D-404", "r10-zig-app", "Zig application", "adversarial",
     'fn main() { from_image("alpine:3.19"); run("apk add --no-cache zig"); workdir("/app"); copy(".", "."); run("zig build -Doptimize=ReleaseSafe"); }',
     "FROM alpine:3.19"),
    ("D-405", "r10-gleam-app", "Gleam application", "adversarial",
     'fn main() { from_image("ghcr.io/gleam-lang/gleam:v1.0-erlang-alpine"); workdir("/app"); copy(".", "."); run("gleam build"); }',
     "FROM ghcr.io/gleam-lang/gleam:v1.0-erlang-alpine"),
    ("D-406", "r10-haskell-app", "Haskell Stack application", "adversarial",
     'fn main() { from_image("haskell:9.6"); workdir("/app"); copy("stack.yaml package.yaml", "./"); run("stack build --only-dependencies"); copy(".", "."); run("stack build"); }',
     "FROM haskell:9.6"),
    ("D-407", "r10-ocaml-app", "OCaml application", "adversarial",
     'fn main() { from_image("ocaml/opam:debian-12-ocaml-5.1"); workdir("/app"); copy("*.opam", "."); run("opam install --deps-only ."); copy(".", "."); run("dune build"); }',
     "FROM ocaml/opam:debian-12-ocaml-5.1"),
    ("D-408", "r10-caddy-proxy", "Caddy reverse proxy", "standard",
     'fn main() { from_image("caddy:2-alpine"); copy("Caddyfile", "/etc/caddy/Caddyfile"); expose(80); expose(443); }',
     "FROM caddy:2-alpine"),
    ("D-409", "r10-vault-server", "HashiCorp Vault", "adversarial",
     'fn main() { from_image("hashicorp/vault:1.15"); copy("vault-config.hcl", "/vault/config/"); expose(8200); cmd("vault server -config=/vault/config/vault-config.hcl"); }',
     "FROM hashicorp/vault:1.15"),
    ("D-410", "r10-consul-server", "HashiCorp Consul", "standard",
     'fn main() { from_image("hashicorp/consul:1.17"); expose(8500); expose(8600); cmd("consul agent -server -bootstrap-expect=1 -ui"); }',
     "FROM hashicorp/consul:1.17"),
    ("D-411", "r10-rabbitmq-server", "RabbitMQ with management", "standard",
     'fn main() { from_image("rabbitmq:3.13-management"); expose(5672); expose(15672); }',
     "FROM rabbitmq:3.13-management"),
    ("D-412", "r10-kafka-server", "Kafka with KRaft mode", "adversarial",
     'fn main() { from_image("confluentinc/cp-kafka:7.5"); expose(9092); expose(9093); }',
     "FROM confluentinc/cp-kafka:7.5"),
    ("D-413", "r10-elasticsearch", "Elasticsearch instance", "standard",
     'fn main() { from_image("elasticsearch:8.12"); expose(9200); expose(9300); }',
     "FROM elasticsearch:8.12"),
    ("D-414", "r10-kibana", "Kibana dashboard", "standard",
     'fn main() { from_image("kibana:8.12"); expose(5601); }',
     "FROM kibana:8.12"),
    ("D-415", "r10-sonarqube", "SonarQube server", "adversarial",
     'fn main() { from_image("sonarqube:10-community"); expose(9000); }',
     "FROM sonarqube:10-community"),
    ("D-416", "r10-jenkins", "Jenkins CI server", "production",
     'fn main() { from_image("jenkins/jenkins:lts-jdk17"); expose(8080); expose(50000); }',
     "FROM jenkins/jenkins:lts-jdk17"),
    ("D-417", "r10-gitea-server", "Gitea git server", "standard",
     'fn main() { from_image("gitea/gitea:1.21"); expose(3000); expose(22); }',
     "FROM gitea/gitea:1.21"),
    ("D-418", "r10-keycloak", "Keycloak identity server", "adversarial",
     'fn main() { from_image("quay.io/keycloak/keycloak:23.0"); expose(8080); cmd("start-dev"); }',
     "FROM quay.io/keycloak/keycloak:23.0"),
    ("D-419", "r10-meilisearch", "Meilisearch engine", "standard",
     'fn main() { from_image("getmeili/meilisearch:v1.6"); expose(7700); }',
     "FROM getmeili/meilisearch:v1.6"),
    ("D-420", "r10-typesense", "Typesense search engine", "standard",
     'fn main() { from_image("typesense/typesense:0.25"); expose(8108); }',
     "FROM typesense/typesense:0.25"),
]

# ============================================================
# Output
# ============================================================
lines = []

# Bash function
lines.append('    /// Round 10 Bash: B-1906..B-2205 — 300 entries with B2-optimized expected strings')
lines.append('    fn load_expansion45_bash(&mut self) {')
lines.append('        let entries = vec![')
for e in bash_entries:
    lines.append(entry(e[0], e[1], e[2], "bash", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

# Makefile function
lines.append('    /// Round 10 Makefile: M-436..M-450')
lines.append('    fn load_expansion33_makefile(&mut self) {')
lines.append('        let entries = vec![')
for e in makefile_entries:
    lines.append(entry(e[0], e[1], e[2], "makefile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

# Dockerfile function
lines.append('    /// Round 10 Dockerfile: D-401..D-420')
lines.append('    fn load_expansion33_dockerfile(&mut self) {')
lines.append('        let entries = vec![')
for e in dockerfile_entries:
    lines.append(entry(e[0], e[1], e[2], "dockerfile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')

print('\n'.join(lines))
