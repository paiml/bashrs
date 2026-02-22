#!/usr/bin/env python3
"""Round 13: Expansion 191, IDs B-16315..D-16355 (41 entries)
Focus: Extreme transpiler stress - deep recursion, complex control flow, multi-function interaction
"""

def format_rust_string(s):
    return f'r#"{s}"#'

def entry(id_prefix, num, name, desc, fmt, input_code, expected):
    fmtstr = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}[fmt]
    return f'        self.entries.push(CorpusEntry::new("{id_prefix}-{num}", "{name}", "{desc}",\n            {fmtstr}, CorpusTier::Adversarial,\n            {format_rust_string(input_code)},\n            {format_rust_string(expected)}));'

bash_entries = []
n = 16315

bash_entries.append(entry("B", n, "pipeline-map-filter", "Pipeline: map then filter simulation",
    "bash",
    'fn map_double(x: u32) -> u32 { return x * 2; } fn filter_gt10(x: u32) -> u32 { if x > 10 { return x; } return 0; } fn pipeline(a: u32, b: u32, c: u32, d: u32) -> u32 { let ma: u32 = filter_gt10(map_double(a)); let mb: u32 = filter_gt10(map_double(b)); let mc: u32 = filter_gt10(map_double(c)); let md: u32 = filter_gt10(map_double(d)); return ma + mb + mc + md; } fn main() { println!("{}", pipeline(3, 5, 8, 10)); }',
    "46")); n += 1

bash_entries.append(entry("B", n, "collatz-steps-bounded", "Collatz sequence step counter bounded",
    "bash",
    'fn collatz_steps(n: u32, max_steps: u32) -> u32 { if n <= 1 { return 0; } if max_steps == 0 { return 0; } if n % 2 == 0 { return 1 + collatz_steps(n / 2, max_steps - 1); } return 1 + collatz_steps(3 * n + 1, max_steps - 1); } fn main() { println!("{} {} {} {} {}", collatz_steps(1, 100), collatz_steps(2, 100), collatz_steps(6, 100), collatz_steps(7, 100), collatz_steps(27, 100)); }',
    "0 1 8 16 96")); n += 1

bash_entries.append(entry("B", n, "ackermann-bounded", "Bounded Ackermann function",
    "bash",
    'fn ack(m: u32, n: u32, fuel: u32) -> u32 { if fuel == 0 { return n; } if m == 0 { return n + 1; } if n == 0 { return ack(m - 1, 1, fuel - 1); } return ack(m - 1, ack(m, n - 1, fuel - 1), fuel - 1); } fn main() { println!("{} {} {} {} {}", ack(0, 0, 100), ack(1, 1, 100), ack(2, 2, 100), ack(3, 1, 100), ack(3, 2, 100)); }',
    "1 3 7 13 29")); n += 1

bash_entries.append(entry("B", n, "state-machine-4state", "Four-state state machine",
    "bash",
    'fn transition(state: u32, input: u32) -> u32 { if state == 0 { if input == 0 { return 1; } return 0; } if state == 1 { if input == 0 { return 2; } return 0; } if state == 2 { if input == 0 { return 3; } return 1; } if state == 3 { return 3; } return 0; } fn run_machine(i0: u32, i1: u32, i2: u32, i3: u32) -> u32 { let s1: u32 = transition(0, i0); let s2: u32 = transition(s1, i1); let s3: u32 = transition(s2, i2); let s4: u32 = transition(s3, i3); return s4; } fn main() { println!("{} {} {} {}", run_machine(0, 0, 0, 0), run_machine(1, 0, 0, 0), run_machine(0, 0, 1, 0), run_machine(0, 1, 0, 0)); }',
    "3 2 2 1")); n += 1

bash_entries.append(entry("B", n, "matrix-trace-3x3", "3x3 matrix trace computation",
    "bash",
    'fn trace3(a00: u32, a01: u32, a02: u32, a10: u32, a11: u32, a12: u32, a20: u32, a21: u32, a22: u32) -> u32 { return a00 + a11 + a22; } fn main() { println!("{} {}", trace3(1, 2, 3, 4, 5, 6, 7, 8, 9), trace3(10, 0, 0, 0, 20, 0, 0, 0, 30)); }',
    "15 60")); n += 1

bash_entries.append(entry("B", n, "hash-combine-4way", "Four-way hash combination",
    "bash",
    'fn hash1(x: u32) -> u32 { return (x * 31 + 7) % 1000; } fn hash2(x: u32) -> u32 { return (x * 37 + 13) % 1000; } fn hash3(x: u32) -> u32 { return (x * 41 + 19) % 1000; } fn hash4(x: u32) -> u32 { return (x * 43 + 23) % 1000; } fn combine(x: u32) -> u32 { return (hash1(x) + hash2(x) + hash3(x) + hash4(x)) % 10000; } fn main() { println!("{} {} {}", combine(0), combine(42), combine(255)); }',
    "62 3514 6830")); n += 1

bash_entries.append(entry("B", n, "mutual-recursion-even-odd", "Mutual recursion: is_even/is_odd",
    "bash",
    'fn is_even(n: u32) -> u32 { if n == 0 { return 1; } return is_odd(n - 1); } fn is_odd(n: u32) -> u32 { if n == 0 { return 0; } return is_even(n - 1); } fn main() { println!("{} {} {} {} {} {}", is_even(0), is_even(1), is_even(4), is_odd(0), is_odd(3), is_odd(6)); }',
    "1 0 1 0 1 0")); n += 1

bash_entries.append(entry("B", n, "triangle-numbers-sum", "Triangle number sums",
    "bash",
    'fn tri(n: u32) -> u32 { return n * (n + 1) / 2; } fn tri_sum(n: u32) -> u32 { let mut s: u32 = 0; let mut i: u32 = 1; while i <= n { s = s + tri(i); i = i + 1; } return s; } fn main() { println!("{} {} {} {} {}", tri_sum(1), tri_sum(2), tri_sum(3), tri_sum(5), tri_sum(10)); }',
    "1 4 10 35 220")); n += 1

bash_entries.append(entry("B", n, "caesar-cipher-bounded", "Caesar cipher with bounded shift",
    "bash",
    'fn shift_char(c: u32, shift: u32) -> u32 { return (c + shift) % 26; } fn encode3(a: u32, b: u32, c: u32, shift: u32) -> u32 { return shift_char(a, shift) * 10000 + shift_char(b, shift) * 100 + shift_char(c, shift); } fn main() { println!("{} {} {}", encode3(0, 1, 2, 3), encode3(25, 0, 1, 1), encode3(10, 20, 5, 13)); }',
    "30405 0102 231808")); n += 1

bash_entries.append(entry("B", n, "stack-push-pop-sim", "Stack push/pop simulation with accumulator",
    "bash",
    'fn push_val(stack: u32, val: u32) -> u32 { return stack * 100 + val; } fn pop_val(stack: u32) -> u32 { return stack / 100; } fn top_val(stack: u32) -> u32 { return stack % 100; } fn main() { let s0: u32 = 0; let s1: u32 = push_val(s0, 10); let s2: u32 = push_val(s1, 20); let s3: u32 = push_val(s2, 30); println!("{} {} {} {}", top_val(s3), top_val(pop_val(s3)), top_val(pop_val(pop_val(s3))), s3); }',
    "30 20 10 102030")); n += 1

bash_entries.append(entry("B", n, "compose-3fn-pipeline", "Three function composition pipeline",
    "bash",
    'fn add10(x: u32) -> u32 { return x + 10; } fn mul5(x: u32) -> u32 { return x * 5; } fn sub3(x: u32) -> u32 { if x >= 3 { return x - 3; } return 0; } fn compose_amc(x: u32) -> u32 { return sub3(mul5(add10(x))); } fn compose_cma(x: u32) -> u32 { return add10(mul5(sub3(x))); } fn main() { println!("{} {} {} {}", compose_amc(0), compose_amc(5), compose_cma(5), compose_cma(10)); }',
    "47 72 20 45")); n += 1

bash_entries.append(entry("B", n, "nested-if-8level", "Eight-level nested if chains",
    "bash",
    'fn classify8(x: u32) -> u32 { if x < 10 { return 1; } if x < 20 { return 2; } if x < 30 { return 3; } if x < 40 { return 4; } if x < 50 { return 5; } if x < 60 { return 6; } if x < 70 { return 7; } return 8; } fn main() { println!("{} {} {} {} {} {} {} {}", classify8(5), classify8(15), classify8(25), classify8(35), classify8(45), classify8(55), classify8(65), classify8(99)); }',
    "1 2 3 4 5 6 7 8")); n += 1

bash_entries.append(entry("B", n, "retry-with-backoff", "Retry with exponential backoff counter",
    "bash",
    'fn backoff_delay(attempt: u32) -> u32 { let mut delay: u32 = 1; let mut i: u32 = 0; while i < attempt { delay = delay * 2; i = i + 1; } return delay; } fn total_delay(max_attempts: u32) -> u32 { let mut total: u32 = 0; let mut i: u32 = 0; while i < max_attempts { total = total + backoff_delay(i); i = i + 1; } return total; } fn main() { println!("{} {} {} {} {}", total_delay(1), total_delay(2), total_delay(3), total_delay(5), total_delay(8)); }',
    "1 3 7 31 255")); n += 1

bash_entries.append(entry("B", n, "arg-parse-bitmap", "Argument parsing bitmap simulation",
    "bash",
    'fn set_flag(bitmap: u32, bit: u32) -> u32 { let mut mask: u32 = 1; let mut i: u32 = 0; while i < bit { mask = mask * 2; i = i + 1; } return bitmap + mask; } fn has_flag(bitmap: u32, bit: u32) -> u32 { let mut mask: u32 = 1; let mut i: u32 = 0; while i < bit { mask = mask * 2; i = i + 1; } if (bitmap / mask) % 2 == 1 { return 1; } return 0; } fn main() { let b: u32 = set_flag(set_flag(set_flag(0, 0), 2), 4); println!("{} {} {} {} {}", b, has_flag(b, 0), has_flag(b, 1), has_flag(b, 2), has_flag(b, 4)); }',
    "21 1 0 1 1")); n += 1

bash_entries.append(entry("B", n, "newton-isqrt-convergence", "Newton method integer sqrt convergence",
    "bash",
    'fn isqrt(n: u32) -> u32 { if n == 0 { return 0; } let mut x: u32 = n; let mut i: u32 = 0; while i < 20 { let nx: u32 = (x + n / x) / 2; if nx >= x { return x; } x = nx; i = i + 1; } return x; } fn main() { println!("{} {} {} {} {} {}", isqrt(0), isqrt(1), isqrt(4), isqrt(10), isqrt(100), isqrt(10000)); }',
    "0 1 2 3 10 100")); n += 1

bash_entries.append(entry("B", n, "polynomial-eval-horner", "Horner method polynomial evaluation",
    "bash",
    'fn horner4(x: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 { return ((a3 * x + a2) * x + a1) * x + a0; } fn main() { println!("{} {} {} {}", horner4(0, 1, 2, 3, 4), horner4(1, 1, 2, 3, 4), horner4(2, 1, 2, 3, 4), horner4(10, 1, 2, 3, 4)); }',
    "1 10 49 4321")); n += 1

bash_entries.append(entry("B", n, "fold-left-sum-product", "Fold-left sum and product simulation",
    "bash",
    'fn fold_sum(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 { return a + b + c + d + e; } fn fold_prod(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 { return a * b * c * d * e; } fn fold_max(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 { let mut m: u32 = a; if b > m { m = b; } if c > m { m = c; } if d > m { m = d; } if e > m { m = e; } return m; } fn main() { println!("{} {} {}", fold_sum(1, 2, 3, 4, 5), fold_prod(1, 2, 3, 4, 5), fold_max(3, 1, 4, 1, 5)); }',
    "15 120 5")); n += 1

bash_entries.append(entry("B", n, "binary-search-iterative", "Iterative binary search",
    "bash",
    'fn bsearch(target: u32, size: u32) -> u32 { let mut lo: u32 = 0; let mut hi: u32 = size; while lo < hi { let mid: u32 = (lo + hi) / 2; if mid == target { return 1; } if mid < target { lo = mid + 1; } else { hi = mid; } } return 0; } fn main() { println!("{} {} {} {} {}", bsearch(5, 10), bsearch(0, 10), bsearch(9, 10), bsearch(10, 10), bsearch(3, 5)); }',
    "1 1 1 0 1")); n += 1

bash_entries.append(entry("B", n, "checked-arith-chain", "Checked arithmetic chain with saturation",
    "bash",
    'fn safe_add(a: u32, b: u32, max_val: u32) -> u32 { if a > max_val - b { return max_val; } return a + b; } fn safe_mul(a: u32, b: u32, max_val: u32) -> u32 { if b > 0 { if a > max_val / b { return max_val; } } return a * b; } fn chain(x: u32) -> u32 { let a: u32 = safe_mul(x, x, 10000); let b: u32 = safe_add(a, 100, 10000); return b; } fn main() { println!("{} {} {} {}", chain(5), chain(50), chain(100), chain(200)); }',
    "125 2600 10000 10000")); n += 1

bash_entries.append(entry("B", n, "bitfield-pack-unpack", "Bitfield pack and unpack via decimal",
    "bash",
    'fn pack(high: u32, mid: u32, low: u32) -> u32 { return high * 10000 + mid * 100 + low; } fn unpack_high(packed: u32) -> u32 { return packed / 10000; } fn unpack_mid(packed: u32) -> u32 { return (packed / 100) % 100; } fn unpack_low(packed: u32) -> u32 { return packed % 100; } fn main() { let p: u32 = pack(12, 34, 56); println!("{} {} {} {}", p, unpack_high(p), unpack_mid(p), unpack_low(p)); }',
    "123456 12 34 56")); n += 1

bash_entries.append(entry("B", n, "vm-3register", "Three-register virtual machine",
    "bash",
    'fn vm_step(r0: u32, r1: u32, r2: u32, op: u32) -> u32 { if op == 0 { return r0 + r1; } if op == 1 { return r0 * r1; } if op == 2 { return r1 + r2; } if op == 3 { return r0 + r2; } return 0; } fn run_program(init: u32) -> u32 { let r0: u32 = init; let r1: u32 = init + 1; let r2: u32 = init + 2; let t1: u32 = vm_step(r0, r1, r2, 0); let t2: u32 = vm_step(t1, r2, r0, 1); let t3: u32 = vm_step(t2, t1, r1, 2); return t3; } fn main() { println!("{} {} {}", run_program(1), run_program(5), run_program(10)); }',
    "12 105 462")); n += 1

bash_entries.append(entry("B", n, "catalan-number-recursive", "Catalan number via recursion",
    "bash",
    'fn catalan(n: u32) -> u32 { if n <= 1 { return 1; } let mut result: u32 = 0; let mut i: u32 = 0; while i < n { result = result + catalan(i) * catalan(n - 1 - i); i = i + 1; } return result; } fn main() { println!("{} {} {} {} {}", catalan(0), catalan(1), catalan(2), catalan(3), catalan(4)); }',
    "1 1 2 5 14")); n += 1

bash_entries.append(entry("B", n, "matrix-multiply-2x2", "2x2 matrix multiply",
    "bash",
    'fn mm00(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 { return a * e + b * g; } fn mm01(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 { return a * f + b * h; } fn mm10(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 { return c * e + d * g; } fn mm11(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 { return c * f + d * h; } fn main() { println!("{} {} {} {}", mm00(1, 2, 3, 4, 5, 6, 7, 8), mm01(1, 2, 3, 4, 5, 6, 7, 8), mm10(1, 2, 3, 4, 5, 6, 7, 8), mm11(1, 2, 3, 4, 5, 6, 7, 8)); }',
    "19 22 43 50")); n += 1

bash_entries.append(entry("B", n, "extended-gcd-lcm", "GCD and LCM computation",
    "bash",
    'fn gcd(a: u32, b: u32) -> u32 { if b == 0 { return a; } return gcd(b, a % b); } fn lcm(a: u32, b: u32) -> u32 { return a / gcd(a, b) * b; } fn main() { println!("{} {} {} {} {} {}", gcd(12, 8), gcd(100, 75), gcd(17, 13), lcm(4, 6), lcm(12, 8), lcm(7, 5)); }',
    "4 25 1 12 24 35")); n += 1

bash_entries.append(entry("B", n, "partition-count-simple", "Simple partition counting",
    "bash",
    'fn partitions(n: u32, max_part: u32) -> u32 { if n == 0 { return 1; } if max_part == 0 { return 0; } if max_part > n { return partitions(n, n); } return partitions(n - max_part, max_part) + partitions(n, max_part - 1); } fn main() { println!("{} {} {} {} {}", partitions(0, 0), partitions(1, 1), partitions(3, 3), partitions(5, 5), partitions(7, 7)); }',
    "1 1 3 7 15")); n += 1

bash_entries.append(entry("B", n, "stirling-second-kind", "Stirling numbers of the second kind",
    "bash",
    'fn stirling2(n: u32, k: u32) -> u32 { if n == 0 { if k == 0 { return 1; } return 0; } if k == 0 { return 0; } if k > n { return 0; } return k * stirling2(n - 1, k) + stirling2(n - 1, k - 1); } fn main() { println!("{} {} {} {} {}", stirling2(0, 0), stirling2(3, 2), stirling2(4, 2), stirling2(4, 3), stirling2(5, 3)); }',
    "1 3 7 6 25")); n += 1

bash_entries.append(entry("B", n, "derangement-count", "Count derangements (subfactorials)",
    "bash",
    'fn derange(n: u32) -> u32 { if n == 0 { return 1; } if n == 1 { return 0; } return (n - 1) * (derange(n - 1) + derange(n - 2)); } fn main() { println!("{} {} {} {} {} {}", derange(0), derange(1), derange(2), derange(3), derange(4), derange(5)); }',
    "1 0 1 2 9 44")); n += 1

bash_entries.append(entry("B", n, "ring-buffer-position", "Ring buffer position tracking",
    "bash",
    'fn ring_pos(writes: u32, size: u32) -> u32 { return writes % size; } fn ring_full(writes: u32, size: u32) -> u32 { if writes >= size { return 1; } return 0; } fn main() { println!("{} {} {} {} {} {}", ring_pos(0, 4), ring_pos(3, 4), ring_pos(7, 4), ring_full(2, 4), ring_full(4, 4), ring_full(10, 4)); }',
    "0 3 3 0 1 1")); n += 1

# Makefile entries
make_entries = []
mn = n

make_entries.append(entry("M", mn, "make-dep-graph-depth", "Makefile: dependency graph depth calculation",
    "makefile",
    'fn dep_depth(targets: u32, deps_per: u32) -> u32 { if targets <= 1 { return 0; } return 1 + dep_depth(targets / deps_per, deps_per); } fn main() { println!("{} {} {}", dep_depth(1, 2), dep_depth(8, 2), dep_depth(27, 3)); }',
    "dep_depth() {")); mn += 1

make_entries.append(entry("M", mn, "make-phony-counter", "Makefile: PHONY target counter",
    "makefile",
    'fn count_phony(total: u32, phony: u32) -> u32 { return phony * 100 / total; } fn main() { println!("{} {}", count_phony(10, 3), count_phony(20, 15)); }',
    "count_phony() {")); mn += 1

make_entries.append(entry("M", mn, "make-recursive-depth", "Makefile: recursive make depth tracker",
    "makefile",
    'fn make_depth(level: u32, max_level: u32) -> u32 { if level >= max_level { return level; } return make_depth(level + 1, max_level); } fn main() { println!("{} {}", make_depth(0, 5), make_depth(2, 8)); }',
    "make_depth() {")); mn += 1

# Dockerfile entries
docker_entries = []
dn = mn

docker_entries.append(entry("D", dn, "docker-layer-cache-sim", "Dockerfile: layer cache simulation",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn cache_hit(layer_hash: u32, cache_hash: u32) -> u32 { if layer_hash == cache_hash { return 1; } return 0; } fn total_hits(h1: u32, h2: u32, h3: u32) -> u32 { return h1 + h2 + h3; } fn main() { from_image("node", "20-slim"); println!("{} {}", cache_hit(100, 100), total_hits(1, 0, 1)); }',
    "FROM node:20-slim")); dn += 1

docker_entries.append(entry("D", dn, "docker-healthcheck-interval", "Dockerfile: healthcheck interval calc",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn health_checks_per_hour(interval_sec: u32) -> u32 { return 3600 / interval_sec; } fn main() { from_image("redis", "7-alpine"); println!("{} {}", health_checks_per_hour(30), health_checks_per_hour(60)); }',
    "FROM redis:7-alpine")); dn += 1

docker_entries.append(entry("D", dn, "docker-multi-arch-count", "Dockerfile: multi-arch platform count",
    "dockerfile",
    'fn from_image(i: &str, t: &str) {} fn platforms(arches: u32) -> u32 { return arches; } fn build_time(arches: u32, base_time: u32) -> u32 { return arches * base_time; } fn main() { from_image("golang", "1.22"); println!("{} {}", platforms(3), build_time(3, 120)); }',
    "FROM golang:1.22")); dn += 1

# Print the expansion function
print(f"    fn load_expansion191_bash(&mut self) {{")
for e in bash_entries:
    print(e)
print(f"    }}")
print()
print(f"    fn load_expansion191_makefile(&mut self) {{")
for e in make_entries:
    print(e)
print(f"    }}")
print()
print(f"    fn load_expansion191_dockerfile(&mut self) {{")
for e in docker_entries:
    print(e)
print(f"    }}")

print(f"\n// Total entries: {len(bash_entries)} bash + {len(make_entries)} makefile + {len(docker_entries)} dockerfile = {len(bash_entries) + len(make_entries) + len(docker_entries)}")
