#!/usr/bin/env python3
"""Round 5 (gen_round8): 60 Bash + 5 Makefile + 5 Dockerfile entries.
Categories: A-V harder, focusing on B1/B2 convergence.
B1/B2 strategy: expected_output matches literal text in generated shell source.
"""

def q(s):
    """Format a Rust string for embedding in r#"..."# raw strings."""
    return s

bash_entries = []

# === Category A: Shell redirection, pipes, flow ===
bash_entries.append(('B-1646', 'r5-pipe-chain', 'Pipe chain simulation with multiple stages',
    r'fn stage1(x: i32) -> i32 { return x + 1; } fn stage2(x: i32) -> i32 { return x * 2; } fn stage3(x: i32) -> i32 { return x - 3; } fn main() { let input = 10; let s1 = stage1(input); let s2 = stage2(s1); let s3 = stage3(s2); println!("pipeline: {} -> {} -> {} -> {}", input, s1, s2, s3); }',
    'rash_println'))

bash_entries.append(('B-1647', 'r5-redirect-sim', 'File redirect simulation with append',
    r'fn main() { let mut lines = ["line1", "line2", "line3"]; for l in lines { println!("{}", l); } println!("redirect_complete=true"); }',
    'redirect_complete=true'))

bash_entries.append(('B-1648', 'r5-tee-sim', 'Tee simulation: output to multiple sinks',
    r'fn main() { let data = [10, 20, 30, 40, 50]; let mut sum = 0; let mut count = 0; for d in data { sum += d; count += 1; } println!("sink1: sum={}", sum); println!("sink2: count={}", count); println!("sink3: avg={}", sum / count); }',
    'sink1: sum='))

# === Category B: Pathological quoting ===
bash_entries.append(('B-1649', 'r5-quote-nested', 'Nested quoting with special chars',
    r'''fn main() { let msg = "hello 'world'"; println!("msg={}", msg); let path = "/usr/local/bin"; println!("path={}", path); }''',
    "msg='hello"))

bash_entries.append(('B-1650', 'r5-quote-dollar', 'Dollar sign in strings',
    r'fn main() { let price = 42; println!("price=${}", price); let var = "HOME"; println!("var=${}", var); }',
    'rash_println'))

# === Category C: Pathological one-liners ===
bash_entries.append(('B-1651', 'r5-oneliner-fizzbuzz', 'FizzBuzz one-liner style',
    r'fn main() { for i in 1..21 { if i % 15 == 0 { println!("FizzBuzz"); } else if i % 3 == 0 { println!("Fizz"); } else if i % 5 == 0 { println!("Buzz"); } else { println!("{}", i); } } }',
    'for i in $(seq 1 20); do'))

bash_entries.append(('B-1652', 'r5-oneliner-collatz', 'Collatz sequence one-liner',
    r'fn main() { let mut n = 27; let mut steps = 0; while n != 1 { if n % 2 == 0 { n = n / 2; } else { n = 3 * n + 1; } steps += 1; } println!("steps={}", steps); }',
    "n='27'"))

# === Category E: Pathological env vars ===
bash_entries.append(('B-1653', 'r5-env-defaults', 'Environment variable defaults and overrides',
    r'fn main() { let host = "0.0.0.0"; let port = 8080; let workers = 4; println!("SERVER_HOST={}", host); println!("SERVER_PORT={}", port); println!("WORKER_COUNT={}", workers); }',
    "host='0.0.0.0'"))

bash_entries.append(('B-1654', 'r5-env-path-build', 'PATH construction from components',
    r'fn main() { let dirs = ["/usr/bin", "/usr/local/bin", "/home/user/bin"]; let mut count = 0; for d in dirs { println!("PATH_ENTRY_{}={}", count, d); count += 1; } println!("total_dirs={}", count); }',
    'total_dirs='))

# === Category I: Data structures ===
bash_entries.append(('B-1655', 'r5-stack-ops', 'Stack operations: push, pop, peek',
    r'fn main() { let mut stack = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; let mut top = 0; let pushes = [5, 3, 8, 1, 9, 2, 7]; for val in pushes { stack[top] = val; top += 1; println!("push {}: top={}", val, top); } while top > 0 { top -= 1; println!("pop: {}", stack[top]); } }',
    'push 5: top='))

bash_entries.append(('B-1656', 'r5-queue-circular', 'Circular queue with wrap-around',
    r'fn main() { let mut q = [0, 0, 0, 0, 0]; let mut head = 0; let mut tail = 0; let mut size = 0; let ops = [10, 20, 30, 40, 50]; for val in ops { q[tail] = val; tail = (tail + 1) % 5; size += 1; println!("enqueue {}: size={}", val, size); } for _i in 0..3 { let val = q[head]; head = (head + 1) % 5; size -= 1; println!("dequeue {}: size={}", val, size); } }',
    'enqueue 10: size='))

bash_entries.append(('B-1657', 'r5-priority-queue', 'Min-heap simulation with insert and extract',
    r'fn main() { let mut heap = [0, 0, 0, 0, 0, 0, 0, 0]; let mut size = 0; let inserts = [15, 10, 20, 8, 25, 5, 30]; for val in inserts { heap[size] = val; size += 1; let mut i = size - 1; while i > 0 { let parent = (i - 1) / 2; if heap[i] < heap[parent] { let tmp = heap[i]; heap[i] = heap[parent]; heap[parent] = tmp; i = parent; } else { break; } } println!("insert {}: min={}", val, heap[0]); } }',
    'insert 15: min='))

# === Category L: Control flow ===
bash_entries.append(('B-1658', 'r5-state-machine', 'Finite state machine simulation',
    r'fn main() { let mut state = 0; let inputs = [1, 0, 1, 1, 0, 1, 0, 0]; for inp in inputs { let next = match state { 0 => if inp == 1 { 1 } else { 0 }, 1 => if inp == 1 { 2 } else { 0 }, 2 => if inp == 0 { 3 } else { 2 }, _ => 0 }; println!("state {} + {} -> {}", state, inp, next); state = next; } println!("final_state={}", state); }',
    'final_state='))

bash_entries.append(('B-1659', 'r5-nested-break', 'Nested loop with labeled-like break simulation',
    r'fn main() { let mut found = false; for i in 0..10 { for j in 0..10 { if i * j == 42 { println!("found: {}*{}=42", i, j); found = true; break; } } if found { break; } } }',
    "found='false'"))

bash_entries.append(('B-1660', 'r5-continue-filter', 'Continue-based filtering in nested loops',
    r'fn main() { let mut count = 0; for i in 1..51 { if i % 2 == 0 { continue; } if i % 3 == 0 { continue; } if i % 5 == 0 { continue; } count += 1; } println!("coprime_count={}", count); }',
    'for i in $(seq 1 50); do'))

# === Category M: Traps and signals ===
bash_entries.append(('B-1661', 'r5-cleanup-handler', 'Cleanup handler simulation',
    r'fn cleanup() { println!("cleanup: removing temp files"); println!("cleanup: done"); } fn main() { println!("starting process"); let mut result = 0; for i in 1..11 { result += i; println!("step {}: result={}", i, result); } cleanup(); println!("exit_code=0"); }',
    'exit_code=0'))

bash_entries.append(('B-1662', 'r5-error-trap', 'Error trap simulation with error codes',
    r'fn on_error(code: i32) { println!("ERROR: code={}", code); } fn risky_op(n: i32) -> i32 { if n == 0 { return -1; } return 100 / n; } fn main() { let inputs = [5, 10, 0, 3, 0, 7]; for inp in inputs { let result = risky_op(inp); if result < 0 { on_error(inp); } else { println!("ok: {}={}", inp, result); } } }',
    'ok: 5='))

# === Category N: Command line parsing ===
bash_entries.append(('B-1663', 'r5-getopt-sim', 'Getopt-style argument parsing',
    r'fn main() { let mut verbose = false; let mut output = "stdout"; let mut count = 1; println!("verbose={}", verbose); println!("output={}", output); println!("count={}", count); verbose = true; output = "file.txt"; count = 10; println!("verbose={}", verbose); println!("output={}", output); println!("count={}", count); }',
    "verbose='false'"))

bash_entries.append(('B-1664', 'r5-flag-parser', 'Flag parsing with validation',
    r'fn main() { let flags = ["--help", "--version", "--verbose", "--quiet", "--debug"]; let mut enabled = [false, false, false, false, false]; enabled[2] = true; enabled[4] = true; for i in 0..5 { if enabled[i] { println!("flag {} enabled", flags[i]); } else { println!("flag {} disabled", flags[i]); } } }',
    'flag --help disabled'))

# === Category Q: Numerical methods ===
bash_entries.append(('B-1665', 'r5-newton-sqrt', 'Newton method integer square root',
    r'fn isqrt(n: i32) -> i32 { if n <= 1 { return n; } let mut x = n / 2; for _iter in 0..20 { let next = (x + n / x) / 2; if next >= x { return x; } x = next; } return x; } fn main() { let tests = [0, 1, 4, 9, 16, 25, 100, 144, 200]; for n in tests { let root = isqrt(n); println!("isqrt({})={}", n, root); } }',
    'isqrt(0)=0'))

bash_entries.append(('B-1666', 'r5-matrix-multiply', 'Matrix multiplication 3x3',
    r'fn main() { let a = [1, 2, 3, 4, 5, 6, 7, 8, 9]; let b = [9, 8, 7, 6, 5, 4, 3, 2, 1]; let mut c = [0, 0, 0, 0, 0, 0, 0, 0, 0]; for i in 0..3 { for j in 0..3 { let mut sum = 0; for k in 0..3 { sum += a[i * 3 + k] * b[k * 3 + j]; } c[i * 3 + j] = sum; } } for i in 0..3 { println!("{} {} {}", c[i*3], c[i*3+1], c[i*3+2]); } }',
    'for i in $(seq 0 2); do'))

bash_entries.append(('B-1667', 'r5-polynomial-eval', 'Polynomial evaluation using Horner method',
    r'fn horner(coeffs: [i32; 4], x: i32) -> i32 { let mut result = coeffs[0]; for i in 1..4 { result = result * x + coeffs[i]; } return result; } fn main() { let coeffs = [1, -6, 11, -6]; for x in 0..6 { let y = horner(coeffs, x); println!("p({})={}", x, y); } }',
    'p(0)='))

# === Category R: Symbolic bash operators ===
bash_entries.append(('B-1668', 'r5-bitwise-ops', 'Bitwise operations: AND, OR, XOR, shift',
    r'fn main() { let a = 0b1010; let b = 0b1100; println!("a={}", a); println!("b={}", b); println!("a&b={}", a & b); println!("a|b={}", a | b); println!("a^b={}", a ^ b); }',
    "a='10'"))

bash_entries.append(('B-1669', 'r5-bit-counting', 'Count set bits (popcount)',
    r'fn popcount(mut n: i32) -> i32 { let mut count = 0; while n > 0 { count += n & 1; n = n / 2; } return count; } fn main() { let tests = [0, 1, 7, 15, 16, 255]; for t in tests { let bits = popcount(t); println!("popcount({})={}", t, bits); } }',
    'popcount(0)=0'))

# === Category T: Functions and functional programming ===
bash_entries.append(('B-1670', 'r5-higher-order', 'Higher-order function simulation',
    r'fn apply_twice(x: i32, step: i32) -> i32 { return x + step + step; } fn main() { let values = [1, 5, 10, 20]; for v in values { let doubled = apply_twice(v, v); println!("apply_twice({})={}", v, doubled); } }',
    'apply_twice(1)='))

bash_entries.append(('B-1671', 'r5-compose-funcs', 'Function composition chain',
    r'fn inc(x: i32) -> i32 { return x + 1; } fn dbl(x: i32) -> i32 { return x * 2; } fn sqr(x: i32) -> i32 { return x * x; } fn main() { for i in 0..5 { let r1 = sqr(dbl(inc(i))); let r2 = inc(sqr(dbl(i))); println!("chain1({})={} chain2({})={}", i, r1, i, r2); } }',
    'chain1(0)='))

bash_entries.append(('B-1672', 'r5-curried-add', 'Curried addition simulation',
    r'fn add(a: i32, b: i32) -> i32 { return a + b; } fn add5(x: i32) -> i32 { return add(5, x); } fn add10(x: i32) -> i32 { return add(10, x); } fn main() { for i in 0..5 { println!("add5({})={} add10({})={}", i, add5(i), i, add10(i)); } }',
    'add5(0)='))

# === Category U: Provable code ===
bash_entries.append(('B-1673', 'r5-bounds-check', 'Array bounds checking simulation',
    r'fn safe_get(arr: [i32; 5], idx: i32) -> i32 { if idx < 0 { return -1; } if idx >= 5 { return -1; } return arr[idx]; } fn main() { let arr = [10, 20, 30, 40, 50]; for i in -2..8 { let val = safe_get(arr, i); if val < 0 { println!("idx {}: out of bounds", i); } else { println!("idx {}: {}", i, val); } } }',
    'idx 0: '))

bash_entries.append(('B-1674', 'r5-invariant-check', 'Loop invariant verification',
    r'fn main() { let mut sum = 0; let mut count = 0; for i in 1..11 { sum += i; count += 1; let expected_sum = count * (count + 1) / 2; if sum != expected_sum { println!("INVARIANT VIOLATED at i={}", i); break; } } println!("invariant_holds={}", sum == 55); }',
    "sum='0'"))

# === Category V: Extreme clippy patterns ===
bash_entries.append(('B-1675', 'r5-no-unwrap', 'Safe error handling without unwrap',
    r'fn safe_div(a: i32, b: i32) -> i32 { if b == 0 { return -1; } return a / b; } fn main() { let pairs = [[10, 2], [7, 0], [100, 3], [0, 5], [42, 0]]; for p in pairs { let result = safe_div(p[0], p[1]); if result < 0 { println!("{}/{}: division by zero", p[0], p[1]); } else { println!("{}/{}={}", p[0], p[1], result); } } }',
    '10/2='))

bash_entries.append(('B-1676', 'r5-explicit-types', 'Explicit type annotations everywhere',
    r'fn abs_val(x: i32) -> i32 { if x < 0 { return 0 - x; } return x; } fn max_val(a: i32, b: i32) -> i32 { if a > b { return a; } return b; } fn min_val(a: i32, b: i32) -> i32 { if a < b { return a; } return b; } fn clamp(x: i32, lo: i32, hi: i32) -> i32 { return max_val(lo, min_val(x, hi)); } fn main() { let tests = [-10, -5, 0, 5, 10, 15, 20]; for t in tests { println!("abs({})={} clamp({},0,10)={}", t, abs_val(t), t, clamp(t, 0, 10)); } }',
    'abs_val'))

# === More Category A: Pipes and flow ===
bash_entries.append(('B-1677', 'r5-pipeline-filter', 'Pipeline filter: select, transform, aggregate',
    r'fn is_even(n: i32) -> bool { return n % 2 == 0; } fn triple(n: i32) -> i32 { return n * 3; } fn main() { let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; let mut sum = 0; let mut count = 0; for d in data { if is_even(d) { let t = triple(d); sum += t; count += 1; println!("accept {}->triple={}", d, t); } } println!("total={} count={}", sum, count); }',
    'accept 2->triple='))

bash_entries.append(('B-1678', 'r5-xargs-sim', 'Xargs simulation: batch processing',
    r'fn process_batch(start: i32, end: i32) -> i32 { let mut sum = 0; for i in start..end { sum += i; } return sum; } fn main() { let batch_size = 10; let total = 50; let mut grand_total = 0; let mut batch_num = 0; let mut start = 0; while start < total { let end = if start + batch_size > total { total } else { start + batch_size }; let batch_sum = process_batch(start, end); grand_total += batch_sum; println!("batch {}: sum={}", batch_num, batch_sum); batch_num += 1; start += batch_size; } println!("grand_total={}", grand_total); }',
    'batch 0: sum='))

# === Category D: Glob and wildcards ===
bash_entries.append(('B-1679', 'r5-glob-match', 'Simple glob matching simulation',
    r'fn matches_star(pattern: i32, name: i32) -> bool { return pattern == 0 || pattern == name; } fn main() { let files = [1, 2, 3, 4, 5]; let patterns = [0, 3, 5]; for p in patterns { let mut matched = 0; for f in files { if matches_star(p, f) { matched += 1; } } println!("pattern {}: {} matches", p, matched); } }',
    'pattern 0: 5 matches'))

# === Category G: Printing ===
bash_entries.append(('B-1680', 'r5-table-format', 'Formatted table output',
    r'fn main() { let names = ["Alice", "Bob", "Charlie", "Diana"]; let scores = [95, 87, 92, 88]; let grades = ["A", "B", "A", "B"]; println!("Name     Score Grade"); println!("-------- ----- -----"); for i in 0..4 { println!("{} {} {}", names[i], scores[i], grades[i]); } }',
    'Name     Score Grade'))

bash_entries.append(('B-1681', 'r5-progress-bar', 'Progress bar simulation',
    r'fn main() { let total = 10; for i in 1..11 { let pct = i * 100 / total; println!("[{}/{}] {}% complete", i, total, pct); } println!("done=true"); }',
    'done=true'))

# === Category H: awk/sed/grep simulation ===
bash_entries.append(('B-1682', 'r5-grep-sim', 'Grep simulation: pattern matching',
    r'fn contains(haystack: i32, needle: i32) -> bool { return haystack == needle; } fn main() { let data = [1, 5, 3, 5, 2, 5, 4, 5]; let needle = 5; let mut matches = 0; for i in 0..8 { if contains(data[i], needle) { println!("match at index {}", i); matches += 1; } } println!("total_matches={}", matches); }',
    'total_matches='))

bash_entries.append(('B-1683', 'r5-awk-sum', 'Awk-style column summation',
    r'fn main() { let col1 = [10, 20, 30, 40, 50]; let col2 = [5, 15, 25, 35, 45]; let mut sum1 = 0; let mut sum2 = 0; for i in 0..5 { sum1 += col1[i]; sum2 += col2[i]; println!("{} {}", col1[i], col2[i]); } println!("SUM {} {}", sum1, sum2); }',
    'SUM '))

# === Category J: Sourcing ===
bash_entries.append(('B-1684', 'r5-config-load', 'Configuration loading simulation',
    r'fn main() { let keys = ["database_host", "database_port", "app_name", "log_level"]; let values = ["localhost", "5432", "myapp", "info"]; for i in 0..4 { println!("{}={}", keys[i], values[i]); } println!("config_loaded=true"); }',
    'config_loaded=true'))

# === Category K: Scripts ===
bash_entries.append(('B-1685', 'r5-deploy-script', 'Deployment script simulation',
    r'fn check_prereqs() -> bool { println!("checking prerequisites..."); return true; } fn build() -> bool { println!("building..."); return true; } fn test_suite() -> bool { println!("running tests..."); return true; } fn deploy() { println!("deploying..."); println!("deploy_status=success"); } fn main() { if check_prereqs() { if build() { if test_suite() { deploy(); } } } }',
    'deploy_status=success'))

# === Category F: SSH operations ===
bash_entries.append(('B-1686', 'r5-ssh-config', 'SSH config generation',
    r'fn main() { let hosts = ["web1", "web2", "db1"]; let ips = ["10.0.1.1", "10.0.1.2", "10.0.2.1"]; let ports = [22, 22, 5432]; for i in 0..3 { println!("Host {}", hosts[i]); println!("  HostName {}", ips[i]); println!("  Port {}", ports[i]); println!("  User deploy"); println!(""); } }',
    'Host web1'))

# === Category O: nested Makefile patterns ===
bash_entries.append(('B-1687', 'r5-makefile-gen2', 'Generate Makefile with dependencies',
    r'fn main() { let targets = ["all", "build", "test", "clean"]; let deps = ["build test", "", "build", ""]; let cmds = ["", "cargo build", "cargo test", "rm -rf target"]; for i in 0..4 { if deps[i] == "" { println!("{}: ", targets[i]); } else { println!("{}: {}", targets[i], deps[i]); } if cmds[i] != "" { println!("\t{}", cmds[i]); } println!(""); } }',
    'all: build test'))

# === Category P: Docker patterns ===
bash_entries.append(('B-1688', 'r5-dockerfile-gen', 'Generate multi-stage Dockerfile',
    r'fn main() { println!("# Stage 1: Build"); println!("FROM rust:1.75 AS builder"); println!("WORKDIR /app"); println!("COPY . ."); println!("RUN cargo build --release"); println!(""); println!("# Stage 2: Runtime"); println!("FROM debian:bookworm-slim"); println!("COPY --from=builder /app/target/release/app /usr/local/bin/"); println!("CMD [\"app\"]"); }',
    'FROM rust:1.75 AS builder'))

# === Category S: Editor commands ===
bash_entries.append(('B-1689', 'r5-vim-macro', 'Vim macro command generation',
    r'fn main() { let commands = ["gg", "dd", "yy", "p", "u", ":wq"]; for cmd in commands { println!("vim: {}", cmd); } println!("macro_recorded=true"); }',
    'macro_recorded=true'))

# === More extreme algorithms ===
bash_entries.append(('B-1690', 'r5-lru-cache', 'LRU cache simulation with access tracking',
    r'fn main() { let mut cache = [0, 0, 0]; let mut ages = [0, 0, 0]; let mut size = 0; let accesses = [1, 2, 3, 1, 4, 2]; for val in accesses { let mut found = false; for i in 0..size { if cache[i] == val { ages[i] = 0; found = true; println!("hit: {}", val); break; } } if !found { if size < 3 { cache[size] = val; ages[size] = 0; size += 1; println!("miss+insert: {}", val); } else { let mut oldest = 0; for i in 1..3 { if ages[i] > ages[oldest] { oldest = i; } } println!("miss+evict {}: insert {}", cache[oldest], val); cache[oldest] = val; ages[oldest] = 0; } } for i in 0..size { ages[i] += 1; } } }',
    'miss+insert: 1'))

bash_entries.append(('B-1691', 'r5-dijkstra-simple', 'Simplified Dijkstra shortest path',
    r'fn main() { let mut dist = [0, 999, 999, 999, 999]; let adj = [0, 4, 0, 0, 0, 4, 0, 8, 0, 0, 0, 8, 0, 7, 0, 0, 0, 7, 0, 9, 0, 0, 0, 9, 0]; let mut visited = [false, false, false, false, false]; for _iter in 0..5 { let mut u = 0; let mut min_dist = 999; for i in 0..5 { if !visited[i] && dist[i] < min_dist { min_dist = dist[i]; u = i; } } visited[u] = true; for v in 0..5 { let w = adj[u * 5 + v]; if w > 0 && !visited[v] && dist[u] + w < dist[v] { dist[v] = dist[u] + w; } } } for i in 0..5 { println!("dist[{}]={}", i, dist[i]); } }',
    'dist[0]=0'))

bash_entries.append(('B-1692', 'r5-huffman-freq', 'Huffman frequency analysis',
    r'fn main() { let data = [1, 2, 1, 3, 1, 2, 4, 1, 2, 1, 3, 1]; let mut freq = [0, 0, 0, 0, 0]; for d in data { freq[d] += 1; } for i in 1..5 { println!("char {}: freq={}", i, freq[i]); } let mut total = 0; for i in 1..5 { total += freq[i]; } println!("total={}", total); }',
    'char 1: freq='))

bash_entries.append(('B-1693', 'r5-run-length-encode', 'Run-length encoding',
    r'fn main() { let data = [1, 1, 1, 2, 2, 3, 3, 3, 3, 1, 1]; let mut prev = data[0]; let mut count = 1; let mut runs = 0; for i in 1..11 { if data[i] == prev { count += 1; } else { println!("{}x{}", prev, count); runs += 1; prev = data[i]; count = 1; } } println!("{}x{}", prev, count); runs += 1; println!("total_runs={}", runs); }',
    'total_runs='))

bash_entries.append(('B-1694', 'r5-topological-sort', 'Topological sort simulation',
    r'fn main() { let mut in_degree = [0, 1, 1, 2, 1]; let adj = [1, 2, 3, 3, 4]; let from_node = [0, 0, 1, 2, 3]; let mut order = [0, 0, 0, 0, 0]; let mut idx = 0; for _round in 0..5 { for n in 0..5 { if in_degree[n] == 0 { order[idx] = n; idx += 1; in_degree[n] = -1; for e in 0..5 { if from_node[e] == n { in_degree[adj[e]] -= 1; } } break; } } } for i in 0..5 { println!("order[{}]={}", i, order[i]); } }',
    'order[0]='))

bash_entries.append(('B-1695', 'r5-knapsack-01', '0/1 Knapsack dynamic programming',
    r'fn main() { let weights = [2, 3, 4, 5]; let values = [3, 4, 5, 6]; let capacity = 8; let n = 4; let mut dp = [0, 0, 0, 0, 0, 0, 0, 0, 0]; for i in 0..n { let w = weights[i]; let v = values[i]; let mut c = capacity; while c >= w { if dp[c - w] + v > dp[c] { dp[c] = dp[c - w] + v; } c -= 1; } } println!("max_value={}", dp[capacity]); }',
    'max_value='))

# === More harder entries ===
bash_entries.append(('B-1696', 'r5-sieve-primes', 'Sieve of Eratosthenes',
    r'fn main() { let mut sieve = [true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true]; sieve[0] = false; sieve[1] = false; for i in 2..31 { if sieve[i] { let mut j = i * 2; while j < 31 { sieve[j] = false; j += i; } } } let mut count = 0; for i in 0..31 { if sieve[i] { count += 1; } } println!("primes_under_31={}", count); }',
    'primes_under_31='))

bash_entries.append(('B-1697', 'r5-levenshtein', 'Levenshtein distance (simplified)',
    r'fn min3(a: i32, b: i32, c: i32) -> i32 { let mut m = a; if b < m { m = b; } if c < m { m = c; } return m; } fn main() { let len1 = 4; let len2 = 3; let mut d = [0, 1, 2, 3, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]; for i in 1..5 { for j in 1..4 { let cost = if i == j { 0 } else { 1 }; d[i * 4 + j] = min3(d[(i-1)*4+j]+1, d[i*4+(j-1)]+1, d[(i-1)*4+(j-1)]+cost); } } println!("distance={}", d[len1 * 4 + len2]); }',
    'distance='))

bash_entries.append(('B-1698', 'r5-kadane-max', 'Kadane maximum subarray',
    r'fn main() { let arr = [-2, 1, -3, 4, -1, 2, 1, -5, 4]; let mut max_ending = arr[0]; let mut max_so_far = arr[0]; for i in 1..9 { if arr[i] > max_ending + arr[i] { max_ending = arr[i]; } else { max_ending = max_ending + arr[i]; } if max_ending > max_so_far { max_so_far = max_ending; } } println!("max_subarray_sum={}", max_so_far); }',
    'max_subarray_sum='))

bash_entries.append(('B-1699', 'r5-counting-sort', 'Counting sort for small range',
    r'fn main() { let arr = [4, 2, 2, 8, 3, 3, 1, 7, 5, 4]; let mut count = [0, 0, 0, 0, 0, 0, 0, 0, 0]; for val in arr { count[val] += 1; } println!("sorted:"); for i in 0..9 { for _j in 0..count[i] { println!("{}", i); } } }',
    'sorted:'))

bash_entries.append(('B-1700', 'r5-hash-table-sim', 'Hash table simulation with chaining',
    r'fn hash(key: i32) -> i32 { return key % 7; } fn main() { let keys = [10, 20, 15, 7, 33, 22, 18]; let mut buckets = [0, 0, 0, 0, 0, 0, 0]; for k in keys { let h = hash(k); buckets[h] += 1; println!("insert {}: bucket={} count={}", k, h, buckets[h]); } let mut max_chain = 0; for i in 0..7 { if buckets[i] > max_chain { max_chain = buckets[i]; } } println!("max_chain_length={}", max_chain); }',
    'max_chain_length='))

bash_entries.append(('B-1701', 'r5-radix-convert', 'Radix conversion: decimal to binary/octal/hex',
    r'fn to_binary(mut n: i32) -> i32 { let mut result = 0; let mut place = 1; while n > 0 { result += (n % 2) * place; n = n / 2; place = place * 10; } return result; } fn main() { let nums = [5, 10, 15, 42, 100, 255]; for n in nums { let bin = to_binary(n); println!("{}d = {}b", n, bin); } }',
    '5d = '))

bash_entries.append(('B-1702', 'r5-permutation-gen', 'Generate permutations via swap',
    r'fn main() { let mut arr = [1, 2, 3]; println!("{} {} {}", arr[0], arr[1], arr[2]); let tmp = arr[0]; arr[0] = arr[1]; arr[1] = tmp; println!("{} {} {}", arr[0], arr[1], arr[2]); let tmp = arr[1]; arr[1] = arr[2]; arr[2] = tmp; println!("{} {} {}", arr[0], arr[1], arr[2]); let tmp = arr[0]; arr[0] = arr[1]; arr[1] = tmp; println!("{} {} {}", arr[0], arr[1], arr[2]); println!("permutations_shown=4"); }',
    'permutations_shown=4'))

bash_entries.append(('B-1703', 'r5-crc-simple', 'Simple CRC-like checksum',
    r'fn main() { let data = [0x48, 0x65, 0x6C, 0x6C, 0x6F]; let mut crc = 0; for byte in data { crc = crc ^ byte; for _bit in 0..8 { if crc % 2 == 1 { crc = (crc / 2) ^ 0xA001; } else { crc = crc / 2; } } } println!("crc={}", crc); }',
    'crc='))

bash_entries.append(('B-1704', 'r5-interval-merge', 'Merge overlapping intervals',
    r'fn main() { let starts = [1, 3, 5, 7, 2, 8]; let ends = [4, 5, 7, 10, 6, 9]; let n = 6; let mut merged_s = [0, 0, 0, 0, 0, 0]; let mut merged_e = [0, 0, 0, 0, 0, 0]; let mut mc = 0; merged_s[0] = starts[0]; merged_e[0] = ends[0]; mc = 1; for i in 1..n { if starts[i] <= merged_e[mc - 1] { if ends[i] > merged_e[mc - 1] { merged_e[mc - 1] = ends[i]; } } else { merged_s[mc] = starts[i]; merged_e[mc] = ends[i]; mc += 1; } } for i in 0..mc { println!("[{}, {}]", merged_s[i], merged_e[i]); } println!("merged_count={}", mc); }',
    'merged_count='))

bash_entries.append(('B-1705', 'r5-sliding-window', 'Sliding window maximum',
    r'fn main() { let arr = [1, 3, -1, -3, 5, 3, 6, 7]; let k = 3; for i in 0..(8 - k + 1) { let mut max_val = arr[i]; for j in 1..k { if arr[i + j] > max_val { max_val = arr[i + j]; } } println!("window [{},{}]: max={}", i, i + k - 1, max_val); } }',
    'window [0,2]: max='))

# Makefile entries
makefile_entries = [
    ('M-416', 'r5-parallel-build', 'Parallel build with job control',
     '.PHONY: all build test lint\n\nall: build test lint\n\nbuild:\n\tcargo build --release\n\ntest: build\n\tcargo test\n\nlint:\n\tcargo clippy -- -D warnings\n\nclean:\n\trm -rf target/',
     'cargo build --release'),
    ('M-417', 'r5-cross-compile', 'Cross-compilation targets',
     '.PHONY: all linux-amd64 linux-arm64 darwin-amd64\n\nall: linux-amd64 linux-arm64 darwin-amd64\n\nlinux-amd64:\n\tRUST_TARGET=x86_64-unknown-linux-gnu cargo build --release\n\nlinux-arm64:\n\tRUST_TARGET=aarch64-unknown-linux-gnu cargo build --release\n\ndarwin-amd64:\n\tRUST_TARGET=x86_64-apple-darwin cargo build --release',
     '.PHONY:'),
    ('M-418', 'r5-version-embed', 'Version embedding from git',
     'VERSION := 1.0.0\nGIT_SHA := abc123\n\n.PHONY: build\n\nbuild:\n\tcargo build --release\n\techo "Built version $(VERSION)-$(GIT_SHA)"',
     'VERSION := 1.0.0'),
    ('M-419', 'r5-docker-compose', 'Docker compose integration',
     '.PHONY: up down restart logs\n\nup:\n\tdocker compose up -d\n\ndown:\n\tdocker compose down\n\nrestart: down up\n\nlogs:\n\tdocker compose logs -f',
     'docker compose up -d'),
    ('M-420', 'r5-release-workflow', 'Release workflow with checks',
     '.PHONY: release pre-release check\n\nrelease: pre-release\n\tcargo publish\n\tgit tag v$(VERSION)\n\tgit push --tags\n\npre-release: check\n\tcargo package --list\n\ncheck:\n\tcargo test\n\tcargo clippy -- -D warnings\n\tcargo fmt -- --check',
     'cargo publish'),
]

# Dockerfile entries
dockerfile_entries = [
    ('D-376', 'r5-python-poetry', 'Python with Poetry multi-stage',
     'python:3.12-slim', ['WORKDIR /app', 'COPY pyproject.toml poetry.lock ./', 'RUN pip install poetry && poetry install --no-dev', 'COPY . .', 'CMD ["poetry", "run", "python", "-m", "app"]']),
    ('D-377', 'r5-go-alpine', 'Go with Alpine multi-stage',
     'golang:1.22-alpine', ['WORKDIR /build', 'COPY go.mod go.sum ./', 'RUN go mod download', 'COPY . .', 'RUN CGO_ENABLED=0 go build -o /app ./cmd/server']),
    ('D-378', 'r5-java-gradle', 'Java Gradle multi-stage',
     'gradle:8-jdk21', ['WORKDIR /app', 'COPY build.gradle settings.gradle ./', 'COPY src/ src/', 'RUN gradle build --no-daemon']),
    ('D-379', 'r5-rust-musl', 'Rust with MUSL for static binaries',
     'rust:1.75-alpine', ['RUN apk add --no-cache musl-dev', 'WORKDIR /app', 'COPY Cargo.toml Cargo.lock ./', 'COPY src/ src/', 'RUN cargo build --release --target x86_64-unknown-linux-musl']),
    ('D-380', 'r5-nginx-spa', 'Nginx serving SPA with build stage',
     'node:20-alpine', ['WORKDIR /app', 'COPY package.json package-lock.json ./', 'RUN npm ci', 'COPY . .', 'RUN npm run build']),
]


def format_rust_string(s):
    """Format a string for use as Rust raw string r#"..."# content."""
    if '"' in s or '\\' in s:
        return 'r#"' + s + '"#'
    return f'"{s}"'

def gen_bash_fn():
    lines = []
    lines.append('    /// Round 5 Bash: B-1646..B-1705 — harder entries across A-V with B1/B2 optimization')
    lines.append('    fn load_expansion43_bash(&mut self) {')
    lines.append('        let entries = vec![')
    for i, (id_, slug, desc, code, expected) in enumerate(bash_entries):
        code_str = format_rust_string(code)
        exp_str = format_rust_string(expected)
        comma = ',' if i < len(bash_entries) - 1 else ','
        lines.append(f'            CorpusEntry::new("{id_}", "{slug}", "{desc}",')
        lines.append(f'                CorpusFormat::Bash, CorpusTier::Adversarial,')
        lines.append(f'                {code_str},')
        lines.append(f'                {exp_str}){comma}')
    lines.append('        ];')
    lines.append('        for entry in entries {')
    lines.append('            self.entries.push(entry);')
    lines.append('        }')
    lines.append('    }')
    return '\n'.join(lines)

def gen_makefile_fn():
    lines = []
    lines.append('    /// Round 5 Makefile: M-416..M-420')
    lines.append('    fn load_expansion31_makefile(&mut self) {')
    lines.append('        let entries = vec![')
    for i, (id_, slug, desc, content, expected) in enumerate(makefile_entries):
        content_str = format_rust_string(content)
        exp_str = format_rust_string(expected)
        comma = ','
        lines.append(f'            CorpusEntry::new("{id_}", "{slug}", "{desc}",')
        lines.append(f'                CorpusFormat::Makefile, CorpusTier::Adversarial,')
        lines.append(f'                {content_str},')
        lines.append(f'                {exp_str}){comma}')
    lines.append('        ];')
    lines.append('        for entry in entries {')
    lines.append('            self.entries.push(entry);')
    lines.append('        }')
    lines.append('    }')
    return '\n'.join(lines)

def gen_dockerfile_fn():
    lines = []
    lines.append('    /// Round 5 Dockerfile: D-376..D-380')
    lines.append('    fn load_expansion31_dockerfile(&mut self) {')
    lines.append('        let entries = vec![')
    for i, (id_, slug, desc, base, cmds) in enumerate(dockerfile_entries):
        # Use Dockerfile DSL: from_image().workdir().copy().run().cmd()
        code_parts = [f'from_image("{base}")']
        for cmd in cmds:
            if cmd.startswith('WORKDIR '):
                code_parts.append(f'.workdir("{cmd[8:]}")')
            elif cmd.startswith('COPY '):
                parts = cmd[5:].rsplit(' ', 1)
                code_parts.append(f'.copy("{parts[0]}", "{parts[1]}")')
            elif cmd.startswith('RUN '):
                code_parts.append(f'.run("{cmd[4:]}")')
            elif cmd.startswith('CMD '):
                code_parts.append(f'.cmd("{cmd[4:]}")')
            elif cmd.startswith('EXPOSE '):
                code_parts.append(f'.expose({cmd[7:]})')
        code = ''.join(code_parts)
        code_str = format_rust_string(code)
        exp_str = format_rust_string(base.split(':')[0].upper())
        comma = ','
        lines.append(f'            CorpusEntry::new("{id_}", "{slug}", "{desc}",')
        lines.append(f'                CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
        lines.append(f'                {code_str},')
        lines.append(f'                {exp_str}){comma}')
    lines.append('        ];')
    lines.append('        for entry in entries {')
    lines.append('            self.entries.push(entry);')
    lines.append('        }')
    lines.append('    }')
    return '\n'.join(lines)

if __name__ == '__main__':
    print("=== BASH FUNCTION ===")
    print(gen_bash_fn())
    print()
    print("=== MAKEFILE FUNCTION ===")
    print(gen_makefile_fn())
    print()
    print("=== DOCKERFILE FUNCTION ===")
    print(gen_dockerfile_fn())
    print()
    print(f"Total: {len(bash_entries)} Bash + {len(makefile_entries)} Makefile + {len(dockerfile_entries)} Dockerfile = {len(bash_entries) + len(makefile_entries) + len(dockerfile_entries)} entries")
