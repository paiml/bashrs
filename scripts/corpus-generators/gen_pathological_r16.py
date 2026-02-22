#!/usr/bin/env python3
"""Round 16: Expansion 194 - pushing transpiler limits"""

def fmt(s):
    return f'r#"{s}"#'

def e(id_prefix, num, name, desc, ftype, code, expected):
    ft = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}[ftype]
    return f'        self.entries.push(CorpusEntry::new("{id_prefix}-{num}", "{name}", "{desc}",\n            {ft}, CorpusTier::Adversarial,\n            {fmt(code)},\n            {fmt(expected)}));'

b = []
n = 16394

b.append(e("B", n, "karatsuba-multiply", "Karatsuba multiplication simulation", "bash",
    'fn karat(x: u32, y: u32) -> u32 { if x < 10 { return x * y; } if y < 10 { return x * y; } let half: u32 = 2; let xh: u32 = x / 100; let xl: u32 = x % 100; let yh: u32 = y / 100; let yl: u32 = y % 100; let z0: u32 = xl * yl; let z2: u32 = xh * yh; let z1: u32 = (xh + xl) * (yh + yl) - z2 - z0; return z2 * 10000 + z1 * 100 + z0; } fn main() { println!("{} {} {}", karat(12, 34), karat(56, 78), karat(99, 99)); }',
    "408 4368 9801")); n += 1

b.append(e("B", n, "run-length-count", "Run-length encoding counter", "bash",
    'fn count_runs(a: u32, b: u32, c: u32, d: u32, e2: u32) -> u32 { let mut runs: u32 = 1; if b != a { runs = runs + 1; } if c != b { runs = runs + 1; } if d != c { runs = runs + 1; } if e2 != d { runs = runs + 1; } return runs; } fn main() { println!("{} {} {} {}", count_runs(1, 1, 1, 1, 1), count_runs(1, 2, 3, 4, 5), count_runs(1, 1, 2, 2, 3), count_runs(5, 5, 5, 3, 3)); }',
    "1 5 3 2")); n += 1

b.append(e("B", n, "longest-increasing-sub", "Longest increasing subsequence length for 5 elements", "bash",
    'fn lis5(a: u32, b: u32, c: u32, d: u32, e2: u32) -> u32 { let mut best: u32 = 1; let mut lb: u32 = 1; let mut lc: u32 = 1; let mut ld: u32 = 1; let mut le: u32 = 1; if b > a { lb = 2; } if c > a { lc = 2; } if c > b { if lb + 1 > lc { lc = lb + 1; } } if d > a { ld = 2; } if d > b { if lb + 1 > ld { ld = lb + 1; } } if d > c { if lc + 1 > ld { ld = lc + 1; } } if e2 > a { le = 2; } if e2 > b { if lb + 1 > le { le = lb + 1; } } if e2 > c { if lc + 1 > le { le = lc + 1; } } if e2 > d { if ld + 1 > le { le = ld + 1; } } best = lb; if lc > best { best = lc; } if ld > best { best = ld; } if le > best { best = le; } return best; } fn main() { println!("{} {} {} {}", lis5(1, 2, 3, 4, 5), lis5(5, 4, 3, 2, 1), lis5(3, 1, 4, 1, 5), lis5(2, 1, 3, 2, 4)); }',
    "5 1 3 3")); n += 1

b.append(e("B", n, "edit-distance-bounded", "Edit distance bounded approximation", "bash",
    'fn min2(a: u32, b: u32) -> u32 { if a < b { return a; } return b; } fn min3(a: u32, b: u32, c: u32) -> u32 { return min2(min2(a, b), c); } fn edit_approx(len1: u32, len2: u32, common: u32) -> u32 { let max_len: u32 = if len1 > len2 { len1 } else { len2 }; return max_len - common; } fn main() { println!("{} {} {} {}", edit_approx(5, 5, 5), edit_approx(5, 3, 2), edit_approx(10, 8, 6), edit_approx(7, 7, 0)); }',
    "0 3 4 7")); n += 1

b.append(e("B", n, "knapsack-01-small", "0/1 knapsack for 4 items", "bash",
    'fn max2(a: u32, b: u32) -> u32 { if a > b { return a; } return b; } fn knap4(cap: u32, w1: u32, v1: u32, w2: u32, v2: u32, w3: u32, v3: u32, w4: u32, v4: u32) -> u32 { let mut best: u32 = 0; let mut i: u32 = 0; while i < 16 { let mut weight: u32 = 0; let mut value: u32 = 0; if i % 2 == 1 { weight = weight + w1; value = value + v1; } if (i / 2) % 2 == 1 { weight = weight + w2; value = value + v2; } if (i / 4) % 2 == 1 { weight = weight + w3; value = value + v3; } if (i / 8) % 2 == 1 { weight = weight + w4; value = value + v4; } if weight <= cap { if value > best { best = value; } } i = i + 1; } return best; } fn main() { println!("{} {} {}", knap4(10, 5, 10, 4, 40, 6, 30, 3, 50), knap4(7, 3, 30, 4, 50, 5, 60, 2, 20), knap4(15, 5, 10, 5, 10, 5, 10, 5, 10)); }',
    "90 80 40")); n += 1

b.append(e("B", n, "coin-change-count", "Coin change counting (1,5,10 cents)", "bash",
    'fn coin_count(amount: u32) -> u32 { let tens: u32 = amount / 10; let rem_after_tens: u32 = amount % 10; let fives: u32 = rem_after_tens / 5; let ones: u32 = rem_after_tens % 5; return tens + fives + ones; } fn main() { println!("{} {} {} {} {}", coin_count(1), coin_count(5), coin_count(10), coin_count(17), coin_count(99)); }',
    "1 1 1 3 13")); n += 1

b.append(e("B", n, "gray-code-convert", "Binary to Gray code conversion", "bash",
    'fn to_gray(n: u32) -> u32 { return n / 2; } fn from_gray(g: u32) -> u32 { let mut n: u32 = g; let mut mask: u32 = g / 2; while mask > 0 { n = n + mask; mask = mask / 2; } return n; } fn main() { println!("{} {} {} {} {} {}", to_gray(0), to_gray(1), to_gray(5), to_gray(15), from_gray(0), from_gray(5)); }',
    "0 0 2 7 0 6")); n += 1

b.append(e("B", n, "hamming-distance", "Hamming distance between two numbers", "bash",
    'fn hamming(a: u32, b: u32) -> u32 { let mut diff: u32 = 0; let mut x: u32 = a; let mut y: u32 = b; while x > 0 { if x % 2 != y % 2 { diff = diff + 1; } x = x / 2; y = y / 2; } while y > 0 { if y % 2 == 1 { diff = diff + 1; } y = y / 2; } return diff; } fn main() { println!("{} {} {} {}", hamming(0, 0), hamming(1, 4), hamming(7, 0), hamming(15, 15)); }',
    "0 2 3 0")); n += 1

b.append(e("B", n, "zigzag-sequence", "Zigzag sequence generation", "bash",
    'fn zigzag(n: u32) -> u32 { if n % 2 == 0 { return n / 2 + 1; } return 0 - (n / 2 + 1); } fn zigzag_sum(count: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 0; while i < count { if i % 2 == 0 { s = s + (i / 2 + 1); } else { if s >= (i / 2 + 1) { s = s - (i / 2 + 1); } } i = i + 1; } return s; } fn main() { println!("{} {} {} {}", zigzag_sum(1), zigzag_sum(2), zigzag_sum(4), zigzag_sum(6)); }',
    "1 0 1 0")); n += 1

b.append(e("B", n, "luhn-checksum", "Luhn algorithm checksum", "bash",
    'fn luhn_digit(d: u32, double_it: u32) -> u32 { if double_it == 0 { return d; } let dd: u32 = d * 2; if dd > 9 { return dd - 9; } return dd; } fn luhn4(d1: u32, d2: u32, d3: u32, d4: u32) -> u32 { let s: u32 = luhn_digit(d1, 1) + luhn_digit(d2, 0) + luhn_digit(d3, 1) + luhn_digit(d4, 0); return s % 10; } fn main() { println!("{} {} {}", luhn4(7, 9, 9, 2), luhn4(1, 2, 3, 4), luhn4(0, 0, 0, 0)); }',
    "4 0 0")); n += 1

b.append(e("B", n, "bit-reverse-8bit", "Reverse bits of 8-bit number", "bash",
    'fn reverse_bits8(n: u32) -> u32 { let mut result: u32 = 0; let mut val: u32 = n; let mut i: u32 = 0; while i < 8 { result = result * 2 + val % 2; val = val / 2; i = i + 1; } return result; } fn main() { println!("{} {} {} {} {}", reverse_bits8(0), reverse_bits8(1), reverse_bits8(128), reverse_bits8(170), reverse_bits8(255)); }',
    "0 128 1 85 255")); n += 1

b.append(e("B", n, "count-set-bits", "Count set bits (popcount)", "bash",
    'fn popcount(n: u32) -> u32 { let mut count: u32 = 0; let mut v: u32 = n; while v > 0 { count = count + v % 2; v = v / 2; } return count; } fn main() { println!("{} {} {} {} {}", popcount(0), popcount(1), popcount(7), popcount(255), popcount(1023)); }',
    "0 1 3 8 10")); n += 1

b.append(e("B", n, "next-power-of-two", "Find next power of two", "bash",
    'fn next_pow2(n: u32) -> u32 { if n == 0 { return 1; } let mut p: u32 = 1; while p < n { p = p * 2; } return p; } fn main() { println!("{} {} {} {} {} {}", next_pow2(0), next_pow2(1), next_pow2(3), next_pow2(5), next_pow2(16), next_pow2(17)); }',
    "1 1 4 8 16 32")); n += 1

b.append(e("B", n, "integer-log2", "Integer log base 2", "bash",
    'fn ilog2(n: u32) -> u32 { if n <= 1 { return 0; } let mut log_val: u32 = 0; let mut v: u32 = n; while v > 1 { v = v / 2; log_val = log_val + 1; } return log_val; } fn main() { println!("{} {} {} {} {} {}", ilog2(1), ilog2(2), ilog2(4), ilog2(7), ilog2(16), ilog2(1000)); }',
    "0 1 2 2 4 9")); n += 1

b.append(e("B", n, "interleave-bits", "Interleave bits of two numbers", "bash",
    'fn interleave(x: u32, y: u32) -> u32 { let mut result: u32 = 0; let mut bit: u32 = 1; let mut xv: u32 = x; let mut yv: u32 = y; let mut pos: u32 = 1; let mut i: u32 = 0; while i < 8 { result = result + (xv % 2) * pos; pos = pos * 2; result = result + (yv % 2) * pos; pos = pos * 2; xv = xv / 2; yv = yv / 2; i = i + 1; } return result; } fn main() { println!("{} {} {} {}", interleave(0, 0), interleave(1, 0), interleave(0, 1), interleave(3, 3)); }',
    "0 1 2 15")); n += 1

# Makefile
m = []
mn = n

m.append(e("M", mn, "make-incremental-build-est", "Makefile: incremental build time estimate", "makefile",
    'fn incremental_time(changed: u32, total: u32, full_time: u32) -> u32 { return (changed * full_time + total - 1) / total; } fn main() { println!("{} {}", incremental_time(2, 10, 100), incremental_time(5, 20, 60)); }',
    "incremental_time() {")); mn += 1

m.append(e("M", mn, "make-suffix-rule-count", "Makefile: suffix rule counting", "makefile",
    'fn suffix_rules(extensions: u32) -> u32 { return extensions * (extensions - 1); } fn main() { println!("{} {}", suffix_rules(3), suffix_rules(5)); }',
    "suffix_rules() {")); mn += 1

m.append(e("M", mn, "make-job-slots", "Makefile: parallel job slot calculation", "makefile",
    'fn job_slots(cores: u32, load_factor: u32) -> u32 { return cores * load_factor / 100; } fn main() { println!("{} {}", job_slots(8, 150), job_slots(16, 200)); }',
    "job_slots() {")); mn += 1

# Docker
d = []
dn = mn

d.append(e("D", dn, "docker-stage-count", "Dockerfile: multi-stage build count", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn stage_count(builds: u32, tests: u32, deploy: u32) -> u32 { return builds + tests + deploy; } fn main() { from_image("gcc", "13"); println!("{} {}", stage_count(2, 1, 1), stage_count(3, 2, 1)); }',
    "FROM gcc:13")); dn += 1

d.append(e("D", dn, "docker-network-ports", "Dockerfile: network port mapping", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn port_map(internal: u32, offset: u32) -> u32 { return internal + offset; } fn main() { from_image("httpd", "2.4"); println!("{} {}", port_map(80, 8000), port_map(443, 8000)); }',
    "FROM httpd:2.4")); dn += 1

d.append(e("D", dn, "docker-arg-defaults", "Dockerfile: ARG default value handling", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn arg_default(provided: u32, default_val: u32) -> u32 { if provided > 0 { return provided; } return default_val; } fn main() { from_image("maven", "3.9"); println!("{} {}", arg_default(0, 42), arg_default(10, 42)); }',
    "FROM maven:3.9")); dn += 1

print(f"    fn load_expansion194_bash(&mut self) {{")
for x in b: print(x)
print(f"    }}\n")
print(f"    fn load_expansion194_makefile(&mut self) {{")
for x in m: print(x)
print(f"    }}\n")
print(f"    fn load_expansion194_dockerfile(&mut self) {{")
for x in d: print(x)
print(f"    }}")
