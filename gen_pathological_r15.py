#!/usr/bin/env python3
"""Round 15: Expansion 193 - extreme stress entries"""

def fmt(s):
    return f'r#"{s}"#'

def e(id_prefix, num, name, desc, ftype, code, expected):
    ft = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}[ftype]
    return f'        self.entries.push(CorpusEntry::new("{id_prefix}-{num}", "{name}", "{desc}",\n            {ft}, CorpusTier::Adversarial,\n            {fmt(code)},\n            {fmt(expected)}));'

b = []
n = 16373

b.append(e("B", n, "pentagonal-numbers", "Pentagonal number sequence", "bash",
    'fn pentagonal(n: u32) -> u32 { return n * (3 * n - 1) / 2; } fn main() { println!("{} {} {} {} {} {}", pentagonal(1), pentagonal(2), pentagonal(3), pentagonal(4), pentagonal(5), pentagonal(10)); }',
    "1 5 12 22 35 145")); n += 1

b.append(e("B", n, "hexagonal-numbers", "Hexagonal number sequence", "bash",
    'fn hexagonal(n: u32) -> u32 { return n * (2 * n - 1); } fn main() { println!("{} {} {} {} {} {}", hexagonal(1), hexagonal(2), hexagonal(3), hexagonal(4), hexagonal(5), hexagonal(10)); }',
    "1 6 15 28 45 190")); n += 1

b.append(e("B", n, "abundant-number-check", "Check if number is abundant", "bash",
    'fn sum_proper_div(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 1; while i < n { if n % i == 0 { s = s + i; } i = i + 1; } return s; } fn is_abundant(n: u32) -> u32 { if sum_proper_div(n) > n { return 1; } return 0; } fn main() { println!("{} {} {} {} {}", is_abundant(12), is_abundant(18), is_abundant(7), is_abundant(28), is_abundant(20)); }',
    "1 1 0 0 1")); n += 1

b.append(e("B", n, "happy-number-check", "Happy number detection", "bash",
    'fn digit_sq_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut v: u32 = n; while v > 0 { let d: u32 = v % 10; s = s + d * d; v = v / 10; } return s; } fn is_happy(n: u32) -> u32 { let mut slow: u32 = n; let mut fast: u32 = digit_sq_sum(n); let mut i: u32 = 0; while i < 100 { if slow == 1 { return 1; } slow = digit_sq_sum(slow); fast = digit_sq_sum(digit_sq_sum(fast)); if slow == fast { if slow == 1 { return 1; } return 0; } i = i + 1; } return 0; } fn main() { println!("{} {} {} {} {}", is_happy(1), is_happy(7), is_happy(10), is_happy(4), is_happy(2)); }',
    "1 1 1 0 0")); n += 1

b.append(e("B", n, "narcissistic-3digit", "Check 3-digit narcissistic numbers", "bash",
    'fn is_narcissistic3(n: u32) -> u32 { let d0: u32 = n % 10; let d1: u32 = (n / 10) % 10; let d2: u32 = n / 100; if d0 * d0 * d0 + d1 * d1 * d1 + d2 * d2 * d2 == n { return 1; } return 0; } fn main() { println!("{} {} {} {} {}", is_narcissistic3(153), is_narcissistic3(370), is_narcissistic3(371), is_narcissistic3(407), is_narcissistic3(100)); }',
    "1 1 1 1 0")); n += 1

b.append(e("B", n, "modular-exponent-chain", "Modular exponentiation chain", "bash",
    'fn powmod(base: u32, exp: u32, m: u32) -> u32 { if exp == 0 { return 1; } let mut result: u32 = 1; let mut b: u32 = base % m; let mut e: u32 = exp; while e > 0 { if e % 2 == 1 { result = (result * b) % m; } b = (b * b) % m; e = e / 2; } return result; } fn main() { println!("{} {} {} {}", powmod(2, 10, 1000), powmod(3, 7, 100), powmod(5, 13, 97), powmod(7, 20, 1000)); }',
    "24 87 70 1")); n += 1

b.append(e("B", n, "count-trailing-zeros-fact", "Count trailing zeros in factorial", "bash",
    'fn trailing_zeros(n: u32) -> u32 { let mut count: u32 = 0; let mut d: u32 = 5; while d <= n { count = count + n / d; d = d * 5; } return count; } fn main() { println!("{} {} {} {} {}", trailing_zeros(5), trailing_zeros(10), trailing_zeros(25), trailing_zeros(100), trailing_zeros(1000)); }',
    "1 2 6 24 249")); n += 1

b.append(e("B", n, "stern-brocot-path", "Stern-Brocot tree path computation", "bash",
    'fn stern(n: u32) -> u32 { if n <= 1 { return n; } if n % 2 == 0 { return stern(n / 2); } return stern(n / 2) + stern(n / 2 + 1); } fn main() { println!("{} {} {} {} {} {} {}", stern(1), stern(2), stern(3), stern(4), stern(5), stern(6), stern(7)); }',
    "1 1 2 1 3 2 3")); n += 1

b.append(e("B", n, "look-and-say-length", "Look-and-say sequence length approximation", "bash",
    'fn next_length(current_len: u32) -> u32 { if current_len <= 1 { return 2; } return current_len + current_len / 3 + 1; } fn las_length(steps: u32) -> u32 { let mut len: u32 = 1; let mut i: u32 = 0; while i < steps { len = next_length(len); i = i + 1; } return len; } fn main() { println!("{} {} {} {}", las_length(0), las_length(1), las_length(2), las_length(3)); }',
    "1 2 3 5")); n += 1

b.append(e("B", n, "sum-of-cubes-range", "Sum of cubes in a range", "bash",
    'fn cube_sum(lo: u32, hi: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = lo; while i <= hi { s = s + i * i * i; i = i + 1; } return s; } fn main() { println!("{} {} {} {}", cube_sum(1, 1), cube_sum(1, 3), cube_sum(1, 5), cube_sum(5, 10)); }',
    "1 36 225 2850")); n += 1

b.append(e("B", n, "cantor-pairing", "Cantor pairing function", "bash",
    'fn cantor_pair(x: u32, y: u32) -> u32 { return (x + y) * (x + y + 1) / 2 + y; } fn main() { println!("{} {} {} {} {}", cantor_pair(0, 0), cantor_pair(1, 0), cantor_pair(0, 1), cantor_pair(2, 3), cantor_pair(5, 5)); }',
    "0 1 2 18 60")); n += 1

b.append(e("B", n, "tetrahedral-numbers", "Tetrahedral number computation", "bash",
    'fn tetrahedral(n: u32) -> u32 { return n * (n + 1) * (n + 2) / 6; } fn main() { println!("{} {} {} {} {} {}", tetrahedral(1), tetrahedral(2), tetrahedral(3), tetrahedral(4), tetrahedral(5), tetrahedral(10)); }',
    "1 4 10 20 35 220")); n += 1

b.append(e("B", n, "sum-of-primes-below", "Sum of primes below N", "bash",
    'fn is_prime(n: u32) -> u32 { if n < 2 { return 0; } let mut i: u32 = 2; while i * i <= n { if n % i == 0 { return 0; } i = i + 1; } return 1; } fn sum_primes(up_to: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 2; while i <= up_to { if is_prime(i) == 1 { s = s + i; } i = i + 1; } return s; } fn main() { println!("{} {} {} {}", sum_primes(10), sum_primes(20), sum_primes(50), sum_primes(100)); }',
    "17 77 328 1060")); n += 1

b.append(e("B", n, "digit-factorial-sum", "Sum of factorials of digits", "bash",
    'fn fact(n: u32) -> u32 { if n <= 1 { return 1; } return n * fact(n - 1); } fn digit_fact_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut v: u32 = n; while v > 0 { s = s + fact(v % 10); v = v / 10; } return s; } fn main() { println!("{} {} {} {}", digit_fact_sum(145), digit_fact_sum(40585), digit_fact_sum(123), digit_fact_sum(999)); }',
    "145 40585 9 1088640")); n += 1

b.append(e("B", n, "champernowne-digit", "Champernowne digit at position", "bash",
    'fn digits_in(n: u32) -> u32 { if n < 10 { return 1; } if n < 100 { return 2; } if n < 1000 { return 3; } return 4; } fn total_digits(n: u32) -> u32 { let mut total: u32 = 0; let mut i: u32 = 1; while i <= n { total = total + digits_in(i); i = i + 1; } return total; } fn main() { println!("{} {} {} {}", total_digits(9), total_digits(99), total_digits(999), total_digits(10)); }',
    "9 189 2889 11")); n += 1

# Makefile
m = []
mn = n

m.append(e("M", mn, "make-topological-sort-depth", "Makefile: topological sort depth estimate", "makefile",
    'fn topo_depth(nodes: u32, edges: u32) -> u32 { if edges == 0 { return 1; } return 1 + edges / nodes; } fn main() { println!("{} {}", topo_depth(10, 15), topo_depth(5, 0)); }',
    "topo_depth() {")); mn += 1

m.append(e("M", mn, "make-obj-count", "Makefile: object file count from sources", "makefile",
    'fn obj_count(c_files: u32, cpp_files: u32) -> u32 { return c_files + cpp_files; } fn main() { println!("{} {}", obj_count(10, 5), obj_count(100, 50)); }',
    "obj_count() {")); mn += 1

m.append(e("M", mn, "make-rebuild-fraction", "Makefile: rebuild fraction estimate", "makefile",
    'fn rebuild_pct(changed: u32, total: u32) -> u32 { return changed * 100 / total; } fn main() { println!("{} {}", rebuild_pct(3, 10), rebuild_pct(50, 200)); }',
    "rebuild_pct() {")); mn += 1

# Docker
d = []
dn = mn

d.append(e("D", dn, "docker-image-size-sum", "Dockerfile: image size accumulation", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn image_size(base: u32, layers: u32, avg_layer: u32) -> u32 { return base + layers * avg_layer; } fn main() { from_image("ubuntu", "22.04"); println!("{} {}", image_size(77, 5, 10), image_size(5, 10, 20)); }',
    "FROM ubuntu:22.04")); dn += 1

d.append(e("D", dn, "docker-env-var-count", "Dockerfile: environment variable counting", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn total_env(base_env: u32, added: u32) -> u32 { return base_env + added; } fn main() { from_image("node", "22-bookworm"); println!("{} {}", total_env(5, 3), total_env(10, 7)); }',
    "FROM node:22-bookworm")); dn += 1

d.append(e("D", dn, "docker-copy-layers", "Dockerfile: COPY instruction layer count", "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn copy_layers(files: u32, dirs: u32) -> u32 { return files + dirs; } fn main() { from_image("rust", "1.78-slim"); println!("{} {}", copy_layers(5, 2), copy_layers(20, 8)); }',
    "FROM rust:1.78-slim")); dn += 1

print(f"    fn load_expansion193_bash(&mut self) {{")
for x in b: print(x)
print(f"    }}\n")
print(f"    fn load_expansion193_makefile(&mut self) {{")
for x in m: print(x)
print(f"    }}\n")
print(f"    fn load_expansion193_dockerfile(&mut self) {{")
for x in d: print(x)
print(f"    }}")
