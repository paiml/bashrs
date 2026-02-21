#!/usr/bin/env python3
"""Round 11: Generate 300 more corpus entries with B2-optimized expected strings.

Focus: More functions, more loop patterns, more string patterns.
All expected strings are exact full trimmed output lines.
"""

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
# More function definitions (B-2206..B-2305, 100 entries)
# Every function name is unique and expected is "name() {"
# ==========================================
funcs = [
    ("B-2206", "r11-dist-sq", "Distance squared", "fn dist_sq(x: i32, y: i32) -> i32 { x * x + y * y }", "dist_sq"),
    ("B-2207", "r11-midpoint", "Midpoint", "fn midpoint(a: i32, b: i32) -> i32 { (a + b) / 2 }", "midpoint"),
    ("B-2208", "r11-ceilDiv", "Ceiling division", "fn ceil_div(a: i32, b: i32) -> i32 { (a + b - 1) / b }", "ceil_div"),
    ("B-2209", "r11-floorDiv", "Floor division", "fn floor_div(a: i32, b: i32) -> i32 { a / b }", "floor_div"),
    ("B-2210", "r11-roundDiv", "Rounded division", "fn round_div(a: i32, b: i32) -> i32 { (a + b / 2) / b }", "round_div"),
    ("B-2211", "r11-saturateAdd", "Saturating add", "fn sat_add(a: i32, b: i32, max: i32) -> i32 { let s = a + b; if s > max { max } else { s } }", "sat_add"),
    ("B-2212", "r11-saturateSub", "Saturating sub", "fn sat_sub(a: i32, b: i32) -> i32 { let d = a - b; if d < 0 { 0 } else { d } }", "sat_sub"),
    ("B-2213", "r11-mapRange", "Map range", "fn map_range(x: i32, in_lo: i32, in_hi: i32, out_lo: i32, out_hi: i32) -> i32 { out_lo + (x - in_lo) * (out_hi - out_lo) / (in_hi - in_lo) }", "map_range"),
    ("B-2214", "r11-isPow2", "Is power of 2", "fn is_pow2(n: i32) -> bool { n > 0 && n % 2 == 0 }", "is_pow2"),
    ("B-2215", "r11-nextMult", "Next multiple", "fn next_mult(x: i32, m: i32) -> i32 { ((x + m - 1) / m) * m }", "next_mult"),
    ("B-2216", "r11-alignUp", "Align up", "fn align_up(x: i32, align: i32) -> i32 { ((x + align - 1) / align) * align }", "align_up"),
    ("B-2217", "r11-alignDown", "Align down", "fn align_down(x: i32, align: i32) -> i32 { (x / align) * align }", "align_down"),
    ("B-2218", "r11-rotLeft", "Rotate left sim", "fn rot_left(x: i32, bits: i32) -> i32 { (x * (1 + bits)) % 256 }", "rot_left"),
    ("B-2219", "r11-scale100", "Scale to percentage", "fn scale100(x: i32, total: i32) -> i32 { x * 100 / total }", "scale100"),
    ("B-2220", "r11-unscale", "Unscale from pct", "fn unscale(pct: i32, total: i32) -> i32 { pct * total / 100 }", "unscale"),
    ("B-2221", "r11-mix2", "Mix two values", "fn mix2(a: i32, b: i32, t: i32) -> i32 { a * (100 - t) / 100 + b * t / 100 }", "mix2"),
    ("B-2222", "r11-smooth", "Smooth step", "fn smooth(x: i32) -> i32 { if x < 0 { 0 } else if x > 100 { 100 } else { x * x / 100 } }", "smooth"),
    ("B-2223", "r11-zigzag", "Zigzag encode", "fn zigzag(x: i32) -> i32 { if x >= 0 { x * 2 } else { (0 - x) * 2 - 1 } }", "zigzag"),
    ("B-2224", "r11-unzigzag", "Zigzag decode", "fn unzigzag(x: i32) -> i32 { if x % 2 == 0 { x / 2 } else { 0 - (x + 1) / 2 } }", "unzigzag"),
    ("B-2225", "r11-hammingW", "Hamming weight sim", "fn hamming_w(n: i32) -> i32 { let mut x = n; let mut c = 0; while x > 0 { c += x % 2; x = x / 2; } c }", "hamming_w"),
    ("B-2226", "r11-hammingD", "Hamming distance sim", "fn hamming_d(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; let mut d = 0; while x > 0 || y > 0 { if x % 2 != y % 2 { d += 1; } x = x / 2; y = y / 2; } d }", "hamming_d"),
    ("B-2227", "r11-logBase2", "Integer log base 2", "fn log2_i(n: i32) -> i32 { let mut x = n; let mut l = 0; while x > 1 { x = x / 2; l += 1; } l }", "log2_i"),
    ("B-2228", "r11-isPerfSq", "Perfect square check", "fn is_perf_sq(n: i32) -> bool { let mut i = 0; while i * i <= n { if i * i == n { return true; } i += 1; } false }", "is_perf_sq"),
    ("B-2229", "r11-sumArith", "Arithmetic sum formula", "fn sum_arith(n: i32) -> i32 { n * (n + 1) / 2 }", "sum_arith"),
    ("B-2230", "r11-sumGeom", "Geometric sum sim", "fn sum_geom(a: i32, r: i32, n: i32) -> i32 { let mut s = 0; let mut t = a; let mut i = 0; while i < n { s += t; t = t * r; i += 1; } s }", "sum_geom"),
]

# Generate remaining 75 entries systematically (B-2231..B-2305)
ops = [
    ("add3", "a + b + c", "(a: i32, b: i32, c: i32) -> i32"),
    ("mul3", "a * b * c", "(a: i32, b: i32, c: i32) -> i32"),
    ("avg3", "(a + b + c) / 3", "(a: i32, b: i32, c: i32) -> i32"),
    ("range3", "c - a", "(a: i32, b: i32, c: i32) -> i32"),
    ("dot2", "a * c + b * d", "(a: i32, b: i32, c: i32, d: i32) -> i32"),
]

for i, (name, expr, sig) in enumerate(ops):
    bid = f"B-{2231 + i}"
    funcs.append((bid, f"r11-{name}", f"{name} computation",
                   f"fn {name}{sig} {{ {expr} }} fn main() {{ let r = {name}(1, 2, 3); }}",
                   name))

# More single-arg functions
single_funcs = [
    ("dbl_inc", "x * 2 + 1"), ("half_dec", "x / 2 - 1"), ("sq_mod", "x * x % 100"),
    ("cube_mod", "x * x * x % 1000"), ("inv100", "100 - x"), ("neg_plus1", "0 - x + 1"),
    ("x3p5", "x * 3 + 5"), ("x5m2", "x * 5 - 2"), ("x7m3", "x * 7 % 3"),
    ("div3p1", "x / 3 + 1"), ("mod11", "x % 11"), ("mod13", "x % 13"),
    ("mod17", "x % 17"), ("mod19", "x % 19"), ("mod23", "x % 23"),
    ("x2p3", "x * 2 + 3"), ("x4m1", "x * 4 - 1"), ("x6p7", "x * 6 + 7"),
    ("x8m5", "x * 8 - 5"), ("x9p2", "x * 9 + 2"),
]

for i, (name, expr) in enumerate(single_funcs):
    bid = f"B-{2236 + i}"
    funcs.append((bid, f"r11-{name}", f"{name} function",
                   f"fn {name}(x: i32) -> i32 {{ {expr} }} fn main() {{ let r = {name}(10); }}",
                   name))

# Two-arg predicate functions
predicates = [
    ("gt", "a > b"), ("lt", "a < b"), ("ge", "a >= b"), ("le", "a <= b"),
    ("eq", "a == b"), ("ne", "a != b"),
    ("both_pos", "a > 0 && b > 0"), ("any_neg", "a < 0 || b < 0"),
    ("both_even", "a % 2 == 0 && b % 2 == 0"), ("any_odd", "a % 2 != 0 || b % 2 != 0"),
    ("sum_pos", "a + b > 0"), ("diff_pos", "a - b > 0"),
    ("prod_pos", "a * b > 0"), ("a_divb", "a % b == 0"),
    ("in_range1", "a >= 0 && a < b"),
]

for i, (name, expr) in enumerate(predicates):
    bid = f"B-{2256 + i}"
    fn_name = f"pred_{name}"
    funcs.append((bid, f"r11-pred-{name}", f"Predicate {name}",
                   f"fn {fn_name}(a: i32, b: i32) -> bool {{ {expr} }} fn main() {{ let r = {fn_name}(5, 3); }}",
                   fn_name))

# More complex functions with while loops
while_funcs = [
    ("B-2271", "r11-pow3", "Power of 3", "fn pow3(e: i32) -> i32 { let mut r = 1; let mut i = 0; while i < e { r = r * 3; i += 1; } r }", "pow3"),
    ("B-2272", "r11-pow5", "Power of 5", "fn pow5(e: i32) -> i32 { let mut r = 1; let mut i = 0; while i < e { r = r * 5; i += 1; } r }", "pow5"),
    ("B-2273", "r11-sumSq", "Sum of squares", "fn sum_sq(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { s += i * i; i += 1; } s }", "sum_sq"),
    ("B-2274", "r11-sumCube", "Sum of cubes", "fn sum_cube(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { s += i * i * i; i += 1; } s }", "sum_cube"),
    ("B-2275", "r11-prodRange", "Product of range", "fn prod_range(a: i32, b: i32) -> i32 { let mut p = 1; let mut i = a; while i <= b { p = p * i; i += 1; } p }", "prod_range"),
    ("B-2276", "r11-countWhile", "Count while condition", "fn count_while(limit: i32) -> i32 { let mut n = 1; let mut c = 0; while n < limit { n = n * 2; c += 1; } c }", "count_while"),
    ("B-2277", "r11-sumOdds", "Sum of odd numbers", "fn sum_odds(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { if i % 2 != 0 { s += i; } i += 1; } s }", "sum_odds"),
    ("B-2278", "r11-sumEvens", "Sum of even numbers", "fn sum_evens(n: i32) -> i32 { let mut s = 0; let mut i = 1; while i <= n { if i % 2 == 0 { s += i; } i += 1; } s }", "sum_evens"),
    ("B-2279", "r11-countPrimes", "Count primes to N", "fn cnt_primes(n: i32) -> i32 { let mut c = 0; let mut i = 2; while i <= n { let mut j = 2; let mut ok = true; while j * j <= i { if i % j == 0 { ok = false; break; } j += 1; } if ok { c += 1; } i += 1; } c }", "cnt_primes"),
    ("B-2280", "r11-nthPrime", "Nth prime sim", "fn nth_prime(n: i32) -> i32 { let mut c = 0; let mut x = 2; while c < n { let mut j = 2; let mut ok = true; while j * j <= x { if x % j == 0 { ok = false; break; } j += 1; } if ok { c += 1; } if c < n { x += 1; } } x }", "nth_prime"),
    ("B-2281", "r11-gcdExtend", "Extended GCD", "fn gcd_ext(a: i32, b: i32) -> i32 { let mut x = a; let mut y = b; while y != 0 { let t = y; y = x % y; x = t; } x }", "gcd_ext"),
    ("B-2282", "r11-tribonacci", "Tribonacci", "fn tribonacci(n: i32) -> i32 { let mut a = 0; let mut b = 0; let mut c = 1; let mut i = 0; while i < n { let t = a + b + c; a = b; b = c; c = t; i += 1; } a }", "tribonacci"),
    ("B-2283", "r11-lucas", "Lucas numbers", "fn lucas(n: i32) -> i32 { let mut a = 2; let mut b = 1; let mut i = 0; while i < n { let t = a + b; a = b; b = t; i += 1; } a }", "lucas"),
    ("B-2284", "r11-pell", "Pell numbers", "fn pell(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let t = 2 * b + a; a = b; b = t; i += 1; } a }", "pell"),
    ("B-2285", "r11-jacobsthal", "Jacobsthal numbers", "fn jacobsthal(n: i32) -> i32 { let mut a = 0; let mut b = 1; let mut i = 0; while i < n { let t = b + 2 * a; a = b; b = t; i += 1; } a }", "jacobsthal"),
]

for e in while_funcs:
    funcs.append(e)

# Remaining functions to reach 100 entries (B-2286..B-2305)
more_funcs = [
    ("B-2286", "r11-catalan", "Catalan number sim", "fn catalan(n: i32) -> i32 { let mut c = 1; let mut i = 0; while i < n { c = c * 2 * (2 * i + 1) / (i + 2); i += 1; } c }", "catalan"),
    ("B-2287", "r11-stirling", "Stirling approx sim", "fn stirling(n: i32) -> i32 { let mut r = 1; let mut i = 1; while i <= n { r = r * i; i += 1; } r }", "stirling"),
    ("B-2288", "r11-ackermann-small", "Ackermann-like bounded", "fn ack_sm(m: i32, n: i32) -> i32 { if m == 0 { n + 1 } else if n == 0 { ack_sm(m - 1, 1) } else { ack_sm(m - 1, ack_sm(m, n - 1)) } }", "ack_sm"),
    ("B-2289", "r11-eulerTot", "Euler totient sim", "fn euler_tot(n: i32) -> i32 { let mut c = 0; let mut i = 1; while i < n { let mut a = i; let mut b = n; while b != 0 { let t = b; b = a % b; a = t; } if a == 1 { c += 1; } i += 1; } c }", "euler_tot"),
    ("B-2290", "r11-tetrahedral", "Tetrahedral number", "fn tetra(n: i32) -> i32 { n * (n + 1) * (n + 2) / 6 }", "tetra"),
    ("B-2291", "r11-pentagonal", "Pentagonal number", "fn penta(n: i32) -> i32 { n * (3 * n - 1) / 2 }", "penta"),
    ("B-2292", "r11-hexagonal", "Hexagonal number", "fn hexa(n: i32) -> i32 { n * (2 * n - 1) }", "hexa"),
    ("B-2293", "r11-isHappy", "Happy number check", "fn is_happy(n: i32) -> bool { let mut x = n; let mut i = 0; while i < 20 { let mut s = 0; while x > 0 { let d = x % 10; s += d * d; x = x / 10; } x = s; if x == 1 { return true; } i += 1; } false }", "is_happy"),
    ("B-2294", "r11-digitalRoot", "Digital root", "fn droot(n: i32) -> i32 { let mut x = n; while x >= 10 { let mut s = 0; while x > 0 { s += x % 10; x = x / 10; } x = s; } x }", "droot"),
    ("B-2295", "r11-repunit", "Repunit number", "fn repunit(n: i32) -> i32 { let mut r = 0; let mut i = 0; while i < n { r = r * 10 + 1; i += 1; } r }", "repunit"),
    ("B-2296", "r11-kaprekar", "Kaprekar routine step", "fn kap_step(n: i32) -> i32 { let d0 = n % 10; let d1 = n / 10 % 10; let d2 = n / 100 % 10; let d3 = n / 1000; let big = d3 * 1000 + d2 * 100 + d1 * 10 + d0; let small = d0 * 1000 + d1 * 100 + d2 * 10 + d3; if big > small { big - small } else { small - big } }", "kap_step"),
    ("B-2297", "r11-narcissistic", "Narcissistic check sim", "fn is_narc(n: i32) -> bool { let d0 = n % 10; let d1 = n / 10 % 10; let d2 = n / 100; d0 * d0 * d0 + d1 * d1 * d1 + d2 * d2 * d2 == n }", "is_narc"),
    ("B-2298", "r11-isAbundant", "Abundant number check", "fn is_abundant(n: i32) -> bool { let mut s = 0; let mut i = 1; while i < n { if n % i == 0 { s += i; } i += 1; } s > n }", "is_abundant"),
    ("B-2299", "r11-isDeficient", "Deficient number check", "fn is_deficient(n: i32) -> bool { let mut s = 0; let mut i = 1; while i < n { if n % i == 0 { s += i; } i += 1; } s < n }", "is_deficient"),
    ("B-2300", "r11-abundant-count", "Count abundant numbers", "fn cnt_abun(limit: i32) -> i32 { let mut c = 0; let mut n = 1; while n <= limit { let mut s = 0; let mut i = 1; while i < n { if n % i == 0 { s += i; } i += 1; } if s > n { c += 1; } n += 1; } c }", "cnt_abun"),
    ("B-2301", "r11-totientSum", "Totient sum", "fn tot_sum(n: i32) -> i32 { let mut s = 0; let mut k = 1; while k <= n { let mut c = 0; let mut i = 1; while i <= k { if k % i == 0 && i % 2 != 0 { c += 1; } i += 1; } s += c; k += 1; } s }", "tot_sum"),
    ("B-2302", "r11-baseConvert", "Base conversion sim", "fn to_base(n: i32, b: i32) -> i32 { let mut x = n; let mut r = 0; let mut p = 1; while x > 0 { r += (x % b) * p; x = x / b; p = p * 10; } r }", "to_base"),
    ("B-2303", "r11-fromBase", "From base to decimal", "fn from_base(n: i32, b: i32) -> i32 { let mut x = n; let mut r = 0; let mut p = 1; while x > 0 { r += (x % 10) * p; x = x / 10; p = p * b; } r }", "from_base"),
    ("B-2304", "r11-fastPow", "Fast power sim", "fn fast_pow(b: i32, e: i32) -> i32 { let mut result = 1; let mut base = b; let mut exp = e; while exp > 0 { if exp % 2 == 1 { result = result * base; } base = base * base; exp = exp / 2; } result }", "fast_pow"),
    ("B-2305", "r11-modPow", "Modular fast power", "fn mod_pow(b: i32, e: i32, m: i32) -> i32 { let mut result = 1; let mut base = b % m; let mut exp = e; while exp > 0 { if exp % 2 == 1 { result = result * base % m; } base = base * base % m; exp = exp / 2; } result }", "mod_pow"),
]
funcs += more_funcs

# Build bash entries from funcs
for bid, slug, desc, code, fname in funcs:
    tier = "standard"
    if "while" in code.lower() and code.count("while") > 1:
        tier = "adversarial"
    elif "while" in code.lower():
        tier = "production"
    bash_entries.append((bid, slug, desc, tier, code + f' fn main() {{ let r = {fname}(5); }}' if 'fn main' not in code else code, f"{fname}() {{"))

# ==========================================
# Pattern: More for loops (B-2306..B-2355, 50 entries)
# ==========================================
for i, (start, end, var) in enumerate([
    (1,10,"a"),(1,20,"b"),(1,30,"c"),(1,40,"d"),(1,50,"e"),
    (0,9,"f"),(0,19,"g"),(0,29,"h"),(0,39,"j"),(0,49,"k"),
    (1,12,"m"),(1,15,"n"),(1,18,"p"),(1,22,"q"),(1,25,"r"),
    (2,11,"s"),(2,21,"t"),(3,13,"u"),(3,23,"v"),(4,14,"w"),
    (1,33,"aa"),(1,44,"ab"),(1,55,"ac"),(1,66,"ad"),(1,77,"ae"),
    (0,5,"af"),(0,6,"ag"),(0,7,"ah"),(0,8,"ai"),(0,10,"aj"),
    (1,14,"ak"),(1,16,"al"),(1,17,"am"),(1,19,"an"),(1,23,"ao"),
    (5,15,"ap"),(5,25,"aq"),(5,35,"ar"),(10,20,"at"),(10,30,"au"),
    (1,4,"av"),(1,3,"aw"),(1,2,"ax"),(0,3,"ay"),(0,2,"az"),
    (0,1,"ba"),(1,99,"bb"),(1,88,"bc"),(1,77,"bd"),(1,66,"be"),
]):
    bid = f"B-{2306 + i}"
    slug = f"r11-for-{var}-{start}-{end}"
    desc = f"For loop {var} from {start} to {end}"
    tier = "standard" if end <= 20 else "adversarial"
    code = f"fn main() {{ let mut s = 0; for {var} in {start}..{end+1} {{ s += {var}; }} }}"
    expected = f"for {var} in $(seq {start} {end}); do"
    bash_entries.append((bid, slug, desc, tier, code, expected))

# ==========================================
# Pattern: More while loops (B-2356..B-2405, 50 entries)
# ==========================================
while_configs = [
    ("aa", "lt", 5), ("ab", "lt", 10), ("ac", "lt", 15), ("ad", "lt", 25),
    ("ae", "lt", 35), ("af", "lt", 45), ("ag", "lt", 55), ("ah", "lt", 65),
    ("ai", "lt", 75), ("aj", "lt", 85), ("ak", "lt", 95), ("al", "gt", 0),
    ("am", "gt", 5), ("an", "gt", 10), ("ao", "gt", 15), ("ap", "gt", 25),
    ("aq", "gt", 35), ("ar", "gt", 45), ("at", "gt", 50), ("au", "gt", 75),
    ("av", "lt", 3), ("aw", "lt", 7), ("ax", "lt", 12), ("ay", "lt", 17),
    ("az", "lt", 22), ("ba", "lt", 27), ("bb", "lt", 32), ("bc", "lt", 37),
    ("bd", "lt", 42), ("be", "lt", 47), ("bf", "lt", 52), ("bg", "lt", 57),
    ("bh", "lt", 62), ("bi", "lt", 67), ("bj", "lt", 72), ("bk", "lt", 77),
    ("bl", "lt", 82), ("bm", "lt", 87), ("bn", "lt", 92), ("bo", "lt", 97),
    ("bp", "gt", 2), ("bq", "gt", 7), ("br", "gt", 12), ("bs", "gt", 17),
    ("bt", "gt", 22), ("bu", "gt", 27), ("bv", "gt", 32), ("bw", "gt", 37),
    ("bx", "gt", 42), ("by", "gt", 47),
]
for i, (var, op, limit) in enumerate(while_configs):
    bid = f"B-{2356 + i}"
    slug = f"r11-while-{var}-{op}-{limit}"
    desc = f"While {var} {op} {limit}"
    tier = "standard" if limit <= 20 else "adversarial"
    if op == "lt":
        code = f'fn main() {{ let mut {var} = 0; while {var} < {limit} {{ {var} += 1; }} }}'
        expected = f'while [ "${var}" -lt {limit} ]; do'
    else:
        code = f'fn main() {{ let mut {var} = {limit + 10}; while {var} > {limit} {{ {var} = {var} - 1; }} }}'
        expected = f'while [ "${var}" -gt {limit} ]; do'
    bash_entries.append((bid, slug, desc, tier, code, expected))

# ==========================================
# Makefile entries M-451..M-465
# ==========================================
makefile_entries = [
    ("M-451", "r11-pnpm-mk", "pnpm project Makefile", "standard",
     'fn main() { exec(".PHONY: install dev build test"); exec("install:"); exec("\\tpnpm install"); exec("dev:"); exec("\\tpnpm dev"); exec("build: install"); exec("\\tpnpm build"); exec("test:"); exec("\\tpnpm test"); }',
     "pnpm install"),
    ("M-452", "r11-yarn-mk", "Yarn project Makefile", "standard",
     'fn main() { exec(".PHONY: install build test lint"); exec("install:"); exec("\\tyarn install"); exec("build:"); exec("\\tyarn build"); exec("test:"); exec("\\tyarn test"); exec("lint:"); exec("\\tyarn lint"); }',
     "yarn install"),
    ("M-453", "r11-ansible-mk", "Ansible playbook Makefile", "adversarial",
     'fn main() { exec(".PHONY: deploy check lint"); exec("deploy:"); exec("\\tansible-playbook -i inventory site.yml"); exec("check:"); exec("\\tansible-playbook -i inventory site.yml --check"); exec("lint:"); exec("\\tansible-lint site.yml"); }',
     "ansible-playbook -i inventory site.yml"),
    ("M-454", "r11-pulumi-mk", "Pulumi IaC Makefile", "adversarial",
     'fn main() { exec(".PHONY: preview up destroy stack"); exec("preview:"); exec("\\tpulumi preview"); exec("up:"); exec("\\tpulumi up -y"); exec("destroy:"); exec("\\tpulumi destroy -y"); exec("stack:"); exec("\\tpulumi stack"); }',
     "pulumi preview"),
    ("M-455", "r11-nix-mk", "Nix build Makefile", "adversarial",
     'fn main() { exec(".PHONY: build shell check update"); exec("build:"); exec("\\tnix build"); exec("shell:"); exec("\\tnix develop"); exec("check:"); exec("\\tnix flake check"); exec("update:"); exec("\\tnix flake update"); }',
     "nix build"),
    ("M-456", "r11-uv-mk", "Python uv Makefile", "production",
     'fn main() { exec(".PHONY: install sync test lint"); exec("install:"); exec("\\tuv pip install -r requirements.txt"); exec("sync:"); exec("\\tuv pip sync requirements.txt"); exec("test:"); exec("\\tuv run pytest"); exec("lint:"); exec("\\tuv run ruff check ."); }',
     "uv pip install -r requirements.txt"),
    ("M-457", "r11-pixi-mk", "Pixi conda Makefile", "adversarial",
     'fn main() { exec(".PHONY: install run test shell"); exec("install:"); exec("\\tpixi install"); exec("run:"); exec("\\tpixi run start"); exec("test:"); exec("\\tpixi run test"); exec("shell:"); exec("\\tpixi shell"); }',
     "pixi install"),
    ("M-458", "r11-just-mk", "Justfile wrapper Makefile", "standard",
     'fn main() { exec(".PHONY: build test lint clean"); exec("build:"); exec("\\tjust build"); exec("test:"); exec("\\tjust test"); exec("lint:"); exec("\\tjust lint"); exec("clean:"); exec("\\tjust clean"); }',
     "just build"),
    ("M-459", "r11-earthly-mk", "Earthly CI Makefile", "adversarial",
     'fn main() { exec(".PHONY: build test lint docker"); exec("build:"); exec("\\tearthly +build"); exec("test:"); exec("\\tearthly +test"); exec("lint:"); exec("\\tearthly +lint"); exec("docker:"); exec("\\tearthly +docker"); }',
     "earthly +build"),
    ("M-460", "r11-mage-mk", "Mage Go Makefile", "adversarial",
     'fn main() { exec(".PHONY: build test lint clean"); exec("build:"); exec("\\tmage build"); exec("test:"); exec("\\tmage test"); exec("lint:"); exec("\\tmage lint"); exec("clean:"); exec("\\tmage clean"); }',
     "mage build"),
    ("M-461", "r11-tilt-mk", "Tilt dev Makefile", "standard",
     'fn main() { exec(".PHONY: up down ci"); exec("up:"); exec("\\ttilt up"); exec("down:"); exec("\\ttilt down"); exec("ci:"); exec("\\ttilt ci"); }',
     "tilt up"),
    ("M-462", "r11-skaffold-mk", "Skaffold Makefile", "adversarial",
     'fn main() { exec(".PHONY: dev run deploy delete"); exec("dev:"); exec("\\tskaffold dev"); exec("run:"); exec("\\tskaffold run"); exec("deploy:"); exec("\\tskaffold deploy"); exec("delete:"); exec("\\tskaffold delete"); }',
     "skaffold dev"),
    ("M-463", "r11-buf-mk", "Buf protobuf Makefile", "adversarial",
     'fn main() { exec(".PHONY: generate lint format breaking"); exec("generate:"); exec("\\tbuf generate"); exec("lint:"); exec("\\tbuf lint"); exec("format:"); exec("\\tbuf format -w"); exec("breaking:"); exec("\\tbuf breaking --against .git"); }',
     "buf generate"),
    ("M-464", "r11-wasm-pack-mk", "wasm-pack Makefile", "production",
     'fn main() { exec(".PHONY: build test pack publish"); exec("build:"); exec("\\twasm-pack build --target web"); exec("test:"); exec("\\twasm-pack test --headless --chrome"); exec("pack:"); exec("\\twasm-pack pack"); exec("publish:"); exec("\\twasm-pack publish"); }',
     "wasm-pack build --target web"),
    ("M-465", "r11-trunk-mk", "Trunk WASM Makefile", "standard",
     'fn main() { exec(".PHONY: serve build clean"); exec("serve:"); exec("\\ttrunk serve"); exec("build:"); exec("\\ttrunk build --release"); exec("clean:"); exec("\\ttrunk clean"); }',
     "trunk serve"),
]

# ==========================================
# Dockerfile entries D-421..D-435
# ==========================================
dockerfile_entries = [
    ("D-421", "r11-valkey", "Valkey cache", "standard",
     'fn main() { from_image("valkey/valkey:7.2"); expose(6379); }',
     "FROM valkey/valkey:7.2"),
    ("D-422", "r11-dragonfly", "Dragonfly cache", "adversarial",
     'fn main() { from_image("docker.dragonflydb.io/dragonflydb/dragonfly"); expose(6379); }',
     "FROM docker.dragonflydb.io/dragonflydb/dragonfly"),
    ("D-423", "r11-clickhouse", "ClickHouse database", "standard",
     'fn main() { from_image("clickhouse/clickhouse-server:24.1"); expose(8123); expose(9000); }',
     "FROM clickhouse/clickhouse-server:24.1"),
    ("D-424", "r11-cockroachdb", "CockroachDB", "adversarial",
     'fn main() { from_image("cockroachdb/cockroach:v23.2"); expose(26257); expose(8080); cmd("start-single-node --insecure"); }',
     "FROM cockroachdb/cockroach:v23.2"),
    ("D-425", "r11-timescaledb", "TimescaleDB", "adversarial",
     'fn main() { from_image("timescale/timescaledb:latest-pg16"); expose(5432); }',
     "FROM timescale/timescaledb:latest-pg16"),
    ("D-426", "r11-nats-server", "NATS messaging", "standard",
     'fn main() { from_image("nats:2.10-alpine"); expose(4222); expose(8222); }',
     "FROM nats:2.10-alpine"),
    ("D-427", "r11-envoy-proxy", "Envoy proxy", "adversarial",
     'fn main() { from_image("envoyproxy/envoy:v1.29"); copy("envoy.yaml", "/etc/envoy/envoy.yaml"); expose(10000); expose(9901); }',
     "FROM envoyproxy/envoy:v1.29"),
    ("D-428", "r11-vector-sink", "Vector log sink", "adversarial",
     'fn main() { from_image("timberio/vector:0.35-alpine"); copy("vector.toml", "/etc/vector/vector.toml"); expose(8686); }',
     "FROM timberio/vector:0.35-alpine"),
    ("D-429", "r11-loki-server", "Grafana Loki", "standard",
     'fn main() { from_image("grafana/loki:2.9"); expose(3100); cmd("-config.file=/etc/loki/local-config.yaml"); }',
     "FROM grafana/loki:2.9"),
    ("D-430", "r11-tempo-server", "Grafana Tempo", "adversarial",
     'fn main() { from_image("grafana/tempo:2.3"); expose(3200); expose(4317); }',
     "FROM grafana/tempo:2.3"),
    ("D-431", "r11-mimir-server", "Grafana Mimir", "adversarial",
     'fn main() { from_image("grafana/mimir:2.11"); expose(8080); expose(9009); }',
     "FROM grafana/mimir:2.11"),
    ("D-432", "r11-qdrant-db", "Qdrant vector DB", "standard",
     'fn main() { from_image("qdrant/qdrant:v1.7"); expose(6333); expose(6334); }',
     "FROM qdrant/qdrant:v1.7"),
    ("D-433", "r11-weaviate-db", "Weaviate vector DB", "adversarial",
     'fn main() { from_image("semitechnologies/weaviate:1.23"); expose(8080); expose(50051); }',
     "FROM semitechnologies/weaviate:1.23"),
    ("D-434", "r11-chromadb", "ChromaDB vector DB", "standard",
     'fn main() { from_image("chromadb/chroma:0.4"); expose(8000); }',
     "FROM chromadb/chroma:0.4"),
    ("D-435", "r11-ollama", "Ollama LLM server", "standard",
     'fn main() { from_image("ollama/ollama:latest"); expose(11434); }',
     "FROM ollama/ollama:latest"),
]

# ============================================================
# Output
# ============================================================
lines = []

lines.append('    /// Round 11 Bash: B-2206..B-2405 — 200 entries')
lines.append('    fn load_expansion46_bash(&mut self) {')
lines.append('        let entries = vec![')
for e in bash_entries:
    lines.append(entry(e[0], e[1], e[2], "bash", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

lines.append('    /// Round 11 Makefile: M-451..M-465')
lines.append('    fn load_expansion34_makefile(&mut self) {')
lines.append('        let entries = vec![')
for e in makefile_entries:
    lines.append(entry(e[0], e[1], e[2], "makefile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

lines.append('    /// Round 11 Dockerfile: D-421..D-435')
lines.append('    fn load_expansion34_dockerfile(&mut self) {')
lines.append('        let entries = vec![')
for e in dockerfile_entries:
    lines.append(entry(e[0], e[1], e[2], "dockerfile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')

print('\n'.join(lines))
