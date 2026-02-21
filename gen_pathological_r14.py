#!/usr/bin/env python3
"""Round 14: Expansion 192, harder entries
Focus: Complex recursion, multi-function interaction, edge cases
"""

def format_rust_string(s):
    return f'r#"{s}"#'

def entry(id_prefix, num, name, desc, fmt, input_code, expected):
    fmtstr = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}[fmt]
    return f'        self.entries.push(CorpusEntry::new("{id_prefix}-{num}", "{name}", "{desc}",\n            {fmtstr}, CorpusTier::Adversarial,\n            {format_rust_string(input_code)},\n            {format_rust_string(expected)}));'

bash_entries = []
n = 16349  # continues from round 13

# Complex multi-function programs
bash_entries.append(entry("B", n, "mergesort-count-inversions", "Count inversions via merge sort logic",
    "bash",
    'fn count_inv(a: u32, b: u32, c: u32, d: u32) -> u32 { let mut inv: u32 = 0; if a > b { inv = inv + 1; } if a > c { inv = inv + 1; } if a > d { inv = inv + 1; } if b > c { inv = inv + 1; } if b > d { inv = inv + 1; } if c > d { inv = inv + 1; } return inv; } fn main() { println!("{} {} {} {}", count_inv(1, 2, 3, 4), count_inv(4, 3, 2, 1), count_inv(2, 1, 4, 3), count_inv(1, 3, 2, 4)); }',
    "0 6 2 1")); n += 1

bash_entries.append(entry("B", n, "fibonacci-matrix-power", "Fibonacci via matrix exponentiation trace",
    "bash",
    'fn fib_fast(n: u32) -> u32 { if n <= 1 { return n; } let mut a: u32 = 0; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let t: u32 = a + b; a = b; b = t; i = i + 1; } return b; } fn fib_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 0; while i <= n { s = s + fib_fast(i); i = i + 1; } return s; } fn main() { println!("{} {} {} {} {}", fib_sum(0), fib_sum(1), fib_sum(5), fib_sum(10), fib_fast(15)); }',
    "0 1 12 143 610")); n += 1

bash_entries.append(entry("B", n, "tower-of-hanoi-moves", "Tower of Hanoi move counter",
    "bash",
    'fn hanoi_moves(n: u32) -> u32 { if n == 0 { return 0; } return 2 * hanoi_moves(n - 1) + 1; } fn main() { println!("{} {} {} {} {} {}", hanoi_moves(0), hanoi_moves(1), hanoi_moves(2), hanoi_moves(3), hanoi_moves(5), hanoi_moves(10)); }',
    "0 1 3 7 31 1023")); n += 1

bash_entries.append(entry("B", n, "josephus-problem", "Josephus problem solver",
    "bash",
    'fn josephus(n: u32, k: u32) -> u32 { if n == 1 { return 0; } return (josephus(n - 1, k) + k) % n; } fn main() { println!("{} {} {} {} {}", josephus(1, 2), josephus(5, 2), josephus(7, 3), josephus(10, 2), josephus(10, 3)); }',
    "0 2 3 4 3")); n += 1

bash_entries.append(entry("B", n, "lucas-numbers-bounded", "Lucas numbers (cousin of Fibonacci)",
    "bash",
    'fn lucas(n: u32) -> u32 { if n == 0 { return 2; } if n == 1 { return 1; } let mut a: u32 = 2; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let t: u32 = a + b; a = b; b = t; i = i + 1; } return b; } fn main() { println!("{} {} {} {} {} {}", lucas(0), lucas(1), lucas(2), lucas(3), lucas(5), lucas(10)); }',
    "2 1 3 4 11 123")); n += 1

bash_entries.append(entry("B", n, "euler-totient-naive", "Euler totient function (naive)",
    "bash",
    'fn gcd(a: u32, b: u32) -> u32 { if b == 0 { return a; } return gcd(b, a % b); } fn totient(n: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 1; while i <= n { if gcd(i, n) == 1 { count = count + 1; } i = i + 1; } return count; } fn main() { println!("{} {} {} {} {}", totient(1), totient(6), totient(10), totient(12), totient(15)); }',
    "1 2 4 4 8")); n += 1

bash_entries.append(entry("B", n, "digital-root-loop", "Digital root via iterative digit sum",
    "bash",
    'fn digit_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut v: u32 = n; while v > 0 { s = s + v % 10; v = v / 10; } return s; } fn digital_root(n: u32) -> u32 { let mut v: u32 = n; while v >= 10 { v = digit_sum(v); } return v; } fn main() { println!("{} {} {} {} {}", digital_root(0), digital_root(9), digital_root(38), digital_root(12345), digital_root(99999)); }',
    "0 9 2 6 9")); n += 1

bash_entries.append(entry("B", n, "perfect-number-check", "Perfect number checker",
    "bash",
    'fn sum_divisors(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 1; while i < n { if n % i == 0 { s = s + i; } i = i + 1; } return s; } fn is_perfect(n: u32) -> u32 { if sum_divisors(n) == n { return 1; } return 0; } fn main() { println!("{} {} {} {} {}", is_perfect(6), is_perfect(28), is_perfect(12), is_perfect(496), is_perfect(100)); }',
    "1 1 0 1 0")); n += 1

bash_entries.append(entry("B", n, "collatz-max-sequence", "Collatz sequence maximum value",
    "bash",
    'fn collatz_max(n: u32, fuel: u32) -> u32 { if n <= 1 { return n; } if fuel == 0 { return n; } let mut max_val: u32 = n; let next: u32 = if n % 2 == 0 { n / 2 } else { 3 * n + 1 }; let sub_max: u32 = collatz_max(next, fuel - 1); if sub_max > max_val { max_val = sub_max; } return max_val; }  fn main() { println!("{} {} {} {}", collatz_max(1, 100), collatz_max(3, 100), collatz_max(7, 100), collatz_max(27, 200)); }',
    "1 16 52 9232")); n += 1

bash_entries.append(entry("B", n, "prime-counting-naive", "Prime counting function (naive)",
    "bash",
    'fn is_prime(n: u32) -> u32 { if n < 2 { return 0; } let mut i: u32 = 2; while i * i <= n { if n % i == 0 { return 0; } i = i + 1; } return 1; } fn count_primes(up_to: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 2; while i <= up_to { count = count + is_prime(i); i = i + 1; } return count; } fn main() { println!("{} {} {} {} {}", count_primes(2), count_primes(10), count_primes(20), count_primes(50), count_primes(100)); }',
    "1 4 8 15 25")); n += 1

bash_entries.append(entry("B", n, "sieve-count-composites", "Count composites up to N",
    "bash",
    'fn is_composite(n: u32) -> u32 { if n < 2 { return 0; } let mut i: u32 = 2; while i * i <= n { if n % i == 0 { return 1; } i = i + 1; } return 0; } fn count_composites(up_to: u32) -> u32 { let mut count: u32 = 0; let mut i: u32 = 2; while i <= up_to { count = count + is_composite(i); i = i + 1; } return count; } fn main() { println!("{} {} {} {}", count_composites(10), count_composites(20), count_composites(50), count_composites(100)); }',
    "5 11 34 74")); n += 1

bash_entries.append(entry("B", n, "motzkin-numbers", "Motzkin numbers recursive",
    "bash",
    'fn motzkin(n: u32) -> u32 { if n <= 1 { return 1; } let mut result: u32 = motzkin(n - 1); let mut k: u32 = 0; while k <= n - 2 { result = result + motzkin(k) * motzkin(n - 2 - k); k = k + 1; } return result; } fn main() { println!("{} {} {} {} {} {}", motzkin(0), motzkin(1), motzkin(2), motzkin(3), motzkin(4), motzkin(5)); }',
    "1 1 2 4 9 21")); n += 1

bash_entries.append(entry("B", n, "permutation-parity", "Permutation parity (even/odd) checker",
    "bash",
    'fn count_inv4(a: u32, b: u32, c: u32, d: u32) -> u32 { let mut inv: u32 = 0; if a > b { inv = inv + 1; } if a > c { inv = inv + 1; } if a > d { inv = inv + 1; } if b > c { inv = inv + 1; } if b > d { inv = inv + 1; } if c > d { inv = inv + 1; } return inv % 2; } fn main() { println!("{} {} {} {} {}", count_inv4(1, 2, 3, 4), count_inv4(2, 1, 3, 4), count_inv4(4, 3, 2, 1), count_inv4(3, 1, 4, 2), count_inv4(1, 3, 2, 4)); }',
    "0 1 0 0 1")); n += 1

bash_entries.append(entry("B", n, "tribonacci-sequence", "Tribonacci number sequence",
    "bash",
    'fn trib(n: u32) -> u32 { if n == 0 { return 0; } if n == 1 { return 0; } if n == 2 { return 1; } let mut a: u32 = 0; let mut b: u32 = 0; let mut c: u32 = 1; let mut i: u32 = 3; while i <= n { let t: u32 = a + b + c; a = b; b = c; c = t; i = i + 1; } return c; } fn main() { println!("{} {} {} {} {} {} {}", trib(0), trib(1), trib(2), trib(3), trib(4), trib(5), trib(10)); }',
    "0 0 1 1 2 4 149")); n += 1

bash_entries.append(entry("B", n, "jacobsthal-numbers", "Jacobsthal number sequence",
    "bash",
    'fn jacobsthal(n: u32) -> u32 { if n == 0 { return 0; } if n == 1 { return 1; } let mut a: u32 = 0; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let t: u32 = b + 2 * a; a = b; b = t; i = i + 1; } return b; } fn main() { println!("{} {} {} {} {} {} {}", jacobsthal(0), jacobsthal(1), jacobsthal(2), jacobsthal(3), jacobsthal(4), jacobsthal(5), jacobsthal(8)); }',
    "0 1 1 3 5 11 85")); n += 1

bash_entries.append(entry("B", n, "pell-numbers", "Pell number sequence",
    "bash",
    'fn pell(n: u32) -> u32 { if n == 0 { return 0; } if n == 1 { return 1; } let mut a: u32 = 0; let mut b: u32 = 1; let mut i: u32 = 2; while i <= n { let t: u32 = 2 * b + a; a = b; b = t; i = i + 1; } return b; } fn main() { println!("{} {} {} {} {} {}", pell(0), pell(1), pell(2), pell(3), pell(4), pell(5)); }',
    "0 1 2 5 12 29")); n += 1

bash_entries.append(entry("B", n, "padovan-sequence", "Padovan number sequence",
    "bash",
    'fn padovan(n: u32) -> u32 { if n <= 2 { return 1; } let mut a: u32 = 1; let mut b: u32 = 1; let mut c: u32 = 1; let mut i: u32 = 3; while i <= n { let t: u32 = a + b; a = b; b = c; c = t; i = i + 1; } return c; } fn main() { println!("{} {} {} {} {} {} {}", padovan(0), padovan(1), padovan(2), padovan(5), padovan(8), padovan(10), padovan(12)); }',
    "1 1 1 2 4 7 12")); n += 1

bash_entries.append(entry("B", n, "hofstadter-q-bounded", "Hofstadter Q-sequence (bounded)",
    "bash",
    'fn q_seq(n: u32) -> u32 { if n <= 2 { return 1; } let mut arr_1: u32 = 1; let mut arr_2: u32 = 1; let mut result: u32 = 1; let mut i: u32 = 3; while i <= n { let prev: u32 = result; let pprev: u32 = arr_2; result = arr_1; arr_1 = pprev; arr_2 = prev; i = i + 1; } return result; } fn main() { println!("{} {} {} {}", q_seq(1), q_seq(2), q_seq(5), q_seq(10)); }',
    "1 1 1 1")); n += 1

# Makefile entries
make_entries = []
mn = n

make_entries.append(entry("M", mn, "make-variable-expand", "Makefile: variable expansion depth",
    "makefile",
    'fn expand_depth(refs: u32, max_depth: u32) -> u32 { if refs <= 1 { return 1; } if max_depth == 0 { return refs; } return 1 + expand_depth(refs - 1, max_depth - 1); } fn main() { println!("{} {}", expand_depth(5, 10), expand_depth(3, 2)); }',
    "expand_depth() {")); mn += 1

make_entries.append(entry("M", mn, "make-pattern-rule-match", "Makefile: pattern rule matching",
    "makefile",
    'fn pattern_matches(targets: u32, patterns: u32) -> u32 { if patterns == 0 { return 0; } return targets / patterns; } fn main() { println!("{} {}", pattern_matches(20, 4), pattern_matches(100, 10)); }',
    "pattern_matches() {")); mn += 1

make_entries.append(entry("M", mn, "make-auto-dep-gen", "Makefile: auto dependency generation count",
    "makefile",
    'fn auto_deps(sources: u32, headers_per: u32) -> u32 { return sources * (1 + headers_per); } fn main() { println!("{} {}", auto_deps(10, 3), auto_deps(50, 5)); }',
    "auto_deps() {")); mn += 1

# Dockerfile entries
docker_entries = []
dn = mn

docker_entries.append(entry("D", dn, "docker-build-cache-layers", "Dockerfile: build cache layer optimization",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn cache_savings(total_layers: u32, cached: u32, avg_time: u32) -> u32 { return cached * avg_time; } fn main() { from_image("python", "3.12-slim"); println!("{} {}", cache_savings(10, 7, 30), cache_savings(5, 3, 60)); }',
    "FROM python:3.12-slim")); dn += 1

docker_entries.append(entry("D", dn, "docker-registry-tags", "Dockerfile: registry tag management",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn tag_count(versions: u32, variants: u32) -> u32 { return versions * variants; } fn main() { from_image("alpine", "3.19"); println!("{} {}", tag_count(5, 3), tag_count(10, 4)); }',
    "FROM alpine:3.19")); dn += 1

docker_entries.append(entry("D", dn, "docker-compose-services", "Dockerfile: compose service count",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn total_containers(services: u32, replicas: u32) -> u32 { return services * replicas; } fn main() { from_image("traefik", "v3.0"); println!("{} {}", total_containers(5, 3), total_containers(8, 2)); }',
    "FROM traefik:v3.0")); dn += 1

# Print the expansion function
print(f"    fn load_expansion192_bash(&mut self) {{")
for e in bash_entries:
    print(e)
print(f"    }}")
print()
print(f"    fn load_expansion192_makefile(&mut self) {{")
for e in make_entries:
    print(e)
print(f"    }}")
print()
print(f"    fn load_expansion192_dockerfile(&mut self) {{")
for e in docker_entries:
    print(e)
print(f"    }}")
