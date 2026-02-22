#!/usr/bin/env python3
"""Generate Round 6 corpus entries: Categories A-V, harder patterns."""

def format_rust_string(s):
    if '"' in s and '\\' in s:
        return 'r##"' + s + '"##'
    elif '"' in s:
        return 'r#"' + s + '"#'
    else:
        return 'r#"' + s + '"#'

def entry(id, name, desc, fmt, tier, input_code, expected):
    inp = format_rust_string(input_code)
    return f'            CorpusEntry::new("{id}", "{name}", "{desc}",\n                CorpusFormat::{fmt}, CorpusTier::{tier},\n                {inp},\n                "{expected}"),'

bash_entries = []

# === A. Shell redirection/pipes/flow (B-1556..B-1560) ===
bash_entries.append(entry("B-1556", "flow-cascade-if-return", "Cascading if with early returns",
    "Bash", "Adversarial",
    'fn classify_temp(t: i32) -> &str { if t > 100 { return "boiling"; } if t > 50 { return "hot"; } if t > 20 { return "warm"; } if t > 0 { return "cool"; } return "freezing"; } fn main() { println!("{}", classify_temp(150)); println!("{}", classify_temp(75)); println!("{}", classify_temp(30)); println!("{}", classify_temp(10)); println!("{}", classify_temp(-5)); }',
    "boiling"))

bash_entries.append(entry("B-1557", "flow-while-countdown", "While countdown with step",
    "Bash", "Adversarial",
    'fn main() { let mut n = 20; while n > 0 { if n % 5 == 0 { println!("tick={}", n); } n -= 3; } println!("done"); }',
    "tick=20"))

bash_entries.append(entry("B-1558", "flow-nested-match", "Match inside if inside loop",
    "Bash", "Adversarial",
    'fn main() { for i in 1..=6 { if i % 2 == 0 { match i { 2 => println!("two"), 4 => println!("four"), 6 => println!("six"), _ => println!("even"), } } } }',
    "two"))

# === B. Quoting patterns (B-1559..B-1561) ===
bash_entries.append(entry("B-1559", "quote-mixed-escapes", "String with mixed special chars",
    "Bash", "Adversarial",
    r'fn main() { let msg = "hello world"; println!("msg={}", msg); let n = 42; println!("n={}", n); }',
    "msg=hello world"))

bash_entries.append(entry("B-1560", "quote-empty-string", "Empty string handling",
    "Bash", "Adversarial",
    r'fn main() { let s = ""; let t = "nonempty"; if s == "" { println!("empty"); } if t == "" { println!("also_empty"); } else { println!("has_content"); } }',
    "empty"))

# === C. One-liners (B-1561..B-1563) ===
bash_entries.append(entry("B-1561", "oneliner-sum-range", "Sum of range in single expression",
    "Bash", "Adversarial",
    'fn main() { let mut s = 0; for i in 1..=100 { s += i; } println!("{}", s); }',
    "5050"))

bash_entries.append(entry("B-1562", "oneliner-product", "Product of small range",
    "Bash", "Adversarial",
    'fn main() { let mut p = 1; for i in 1..=6 { p = p * i; } println!("{}", p); }',
    "720"))

bash_entries.append(entry("B-1563", "oneliner-count-div3", "Count numbers divisible by 3",
    "Bash", "Adversarial",
    'fn main() { let mut c = 0; for i in 1..=30 { if i % 3 == 0 { c += 1; } } println!("{}", c); }',
    "10"))

# === I. Data structures (B-1564..B-1567) ===
bash_entries.append(entry("B-1564", "ds-queue-ring", "Ring buffer queue simulation",
    "Bash", "Adversarial",
    'fn main() { let mut buf = [0, 0, 0, 0]; let mut head = 0; let mut tail = 0; for v in [10, 20, 30] { buf[tail] = v; tail = (tail + 1) % 4; } while head != tail { println!("{}", buf[head]); head = (head + 1) % 4; } }',
    "10"))

bash_entries.append(entry("B-1565", "ds-histogram", "Frequency histogram via array",
    "Bash", "Adversarial",
    'fn main() { let data = [1, 2, 3, 2, 1, 3, 3, 2, 1, 1]; let mut freq = [0, 0, 0, 0]; for i in 0..10 { freq[data[i]] += 1; } println!("1:{} 2:{} 3:{}", freq[1], freq[2], freq[3]); }',
    "1:4 2:3 3:3"))

bash_entries.append(entry("B-1566", "ds-prefix-sum", "Prefix sum array",
    "Bash", "Adversarial",
    'fn main() { let arr = [3, 1, 4, 1, 5]; let mut pre = [0, 0, 0, 0, 0]; pre[0] = arr[0]; for i in 1..5 { pre[i] = pre[i - 1] + arr[i]; } for i in 0..5 { println!("{}", pre[i]); } }',
    "3"))

bash_entries.append(entry("B-1567", "ds-dot-product", "Dot product of two vectors",
    "Bash", "Adversarial",
    'fn main() { let a = [1, 2, 3, 4, 5]; let b = [5, 4, 3, 2, 1]; let mut dot = 0; for i in 0..5 { dot += a[i] * b[i]; } println!("dot={}", dot); }',
    "dot=35"))

# === L. Control flow braces/semicolons (B-1568..B-1570) ===
bash_entries.append(entry("B-1568", "ctrl-deeply-nested-while", "Triple nested while loops",
    "Bash", "Adversarial",
    'fn main() { let mut total = 0; let mut i = 1; while i <= 3 { let mut j = 1; while j <= 3 { let mut k = 1; while k <= 3 { total += 1; k += 1; } j += 1; } i += 1; } println!("total={}", total); }',
    "total=27"))

bash_entries.append(entry("B-1569", "ctrl-for-continue-break", "For with continue and break mixed",
    "Bash", "Adversarial",
    'fn main() { let mut s = 0; for i in 1..=20 { if i % 2 == 0 { continue; } if i > 15 { break; } s += i; } println!("{}", s); }',
    "64"))

bash_entries.append(entry("B-1570", "ctrl-match-range-sim", "Match simulating range checks",
    "Bash", "Adversarial",
    'fn bucket(x: i32) -> i32 { match x { 0 => 0, 1 => 1, 2 => 1, 3 => 2, 4 => 2, 5 => 2, _ => 3 } } fn main() { for i in 0..=7 { println!("{}->{}", i, bucket(i)); } }',
    "0->0"))

# === Q. Numerical methods (B-1571..B-1575) ===
bash_entries.append(entry("B-1571", "num-newton-cbrt", "Cube root via Newton method (integer)",
    "Bash", "Adversarial",
    'fn icbrt(n: i32) -> i32 { if n <= 0 { return 0; } let mut x = n; while x * x * x > n { x = (2 * x + n / (x * x)) / 3; } return x; } fn main() { println!("{}", icbrt(27)); println!("{}", icbrt(64)); println!("{}", icbrt(125)); println!("{}", icbrt(8)); }',
    "3"))

bash_entries.append(entry("B-1572", "num-matrix-trace", "Trace of a 3x3 matrix (diagonal sum)",
    "Bash", "Adversarial",
    'fn main() { let m = [1, 0, 0, 0, 5, 0, 0, 0, 9]; let trace = m[0] + m[4] + m[8]; println!("trace={}", trace); }',
    "trace=15"))

bash_entries.append(entry("B-1573", "num-triangular", "Triangular number computation",
    "Bash", "Adversarial",
    'fn triangular(n: i32) -> i32 { return n * (n + 1) / 2; } fn main() { for i in [1, 5, 10, 20, 100] { println!("T({})={}", i, triangular(i)); } }',
    "T(1)=1"))

bash_entries.append(entry("B-1574", "num-perfect-number", "Check if number is perfect",
    "Bash", "Adversarial",
    'fn is_perfect(n: i32) -> i32 { let mut sum = 0; let mut d = 1; while d < n { if n % d == 0 { sum += d; } d += 1; } if sum == n { return 1; } return 0; } fn main() { for n in [6, 28, 12, 496] { println!("{}:{}", n, is_perfect(n)); } }',
    "6:1"))

bash_entries.append(entry("B-1575", "num-manhattan-dist", "Manhattan distance 2D",
    "Bash", "Adversarial",
    'fn abs_val(x: i32) -> i32 { if x < 0 { return 0 - x; } return x; } fn manhattan(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 { return abs_val(x1 - x2) + abs_val(y1 - y2); } fn main() { println!("{}", manhattan(0, 0, 3, 4)); println!("{}", manhattan(1, 1, 4, 5)); println!("{}", manhattan(0, 0, 0, 0)); }',
    "7"))

# === T. Functions/closures/functional programming (B-1576..B-1580) ===
bash_entries.append(entry("B-1576", "func-map-array", "Map operation over array",
    "Bash", "Adversarial",
    'fn square(x: i32) -> i32 { return x * x; } fn main() { let arr = [1, 2, 3, 4, 5]; for i in 0..5 { println!("{}", square(arr[i])); } }',
    "1"))

bash_entries.append(entry("B-1577", "func-filter-count", "Filter and count pattern",
    "Bash", "Adversarial",
    'fn main() { let arr = [12, 5, 8, 3, 17, 9, 22, 1]; let mut count = 0; for i in 0..8 { if arr[i] >= 10 { count += 1; } } println!("count={}", count); }',
    "count=3"))

bash_entries.append(entry("B-1578", "func-compose-3", "Three-function composition",
    "Bash", "Adversarial",
    'fn inc(x: i32) -> i32 { return x + 1; } fn dbl(x: i32) -> i32 { return x * 2; } fn sqr(x: i32) -> i32 { return x * x; } fn main() { let x = 3; println!("{}", sqr(dbl(inc(x)))); println!("{}", inc(dbl(sqr(x)))); }',
    "64"))

bash_entries.append(entry("B-1579", "func-recursive-sum", "Recursive sum of 1..n",
    "Bash", "Adversarial",
    'fn rsum(n: i32) -> i32 { if n <= 0 { return 0; } return n + rsum(n - 1); } fn main() { println!("{}", rsum(10)); println!("{}", rsum(100)); }',
    "55"))

bash_entries.append(entry("B-1580", "func-recursive-power", "Recursive exponentiation",
    "Bash", "Adversarial",
    'fn rpow(base: i32, exp: i32) -> i32 { if exp == 0 { return 1; } return base * rpow(base, exp - 1); } fn main() { println!("{}", rpow(2, 10)); println!("{}", rpow(3, 5)); println!("{}", rpow(5, 3)); }',
    "1024"))

# === U. Provable code patterns (B-1581..B-1583) ===
bash_entries.append(entry("B-1581", "prove-bounded-loop", "Provably terminating bounded loop",
    "Bash", "Adversarial",
    'fn main() { let n = 10; let mut sum = 0; let mut i = 0; while i < n { sum += i; i += 1; } println!("sum={}", sum); }',
    "sum=45"))

bash_entries.append(entry("B-1582", "prove-total-function", "Total function (all inputs produce output)",
    "Bash", "Adversarial",
    'fn safe_mod(a: i32, b: i32) -> i32 { if b == 0 { return 0; } return a % b; } fn main() { println!("{}", safe_mod(10, 3)); println!("{}", safe_mod(10, 0)); println!("{}", safe_mod(7, 7)); println!("{}", safe_mod(0, 5)); }',
    "1"))

bash_entries.append(entry("B-1583", "prove-invariant", "Loop invariant: sum = i*(i-1)/2",
    "Bash", "Adversarial",
    'fn main() { let mut sum = 0; for i in 0..10 { sum += i; let expected = (i + 1) * i / 2; if sum != expected { println!("INVARIANT_BROKEN"); return; } } println!("invariant_holds sum={}", sum); }',
    "invariant_holds sum=45"))

# === V. Extreme clippy patterns (B-1584..B-1586) ===
bash_entries.append(entry("B-1584", "clippy-no-else", "Avoid needless else after return",
    "Bash", "Adversarial",
    'fn validate(x: i32) -> i32 { if x < 0 { return -1; } if x > 100 { return -2; } return x; } fn main() { println!("{}", validate(-5)); println!("{}", validate(50)); println!("{}", validate(200)); }',
    "-1"))

bash_entries.append(entry("B-1585", "clippy-manual-swap", "Manual swap vs std::mem::swap",
    "Bash", "Adversarial",
    'fn main() { let mut a = 10; let mut b = 20; let temp = a; a = b; b = temp; println!("a={} b={}", a, b); }',
    "a=20 b=10"))

bash_entries.append(entry("B-1586", "clippy-if-same-then-else", "Avoid identical if/else arms",
    "Bash", "Adversarial",
    'fn min_val(a: i32, b: i32) -> i32 { if a <= b { return a; } return b; } fn main() { println!("{}", min_val(3, 7)); println!("{}", min_val(9, 2)); println!("{}", min_val(5, 5)); }',
    "3"))

# === E. Environment variables (B-1587..B-1588) ===
bash_entries.append(entry("B-1587", "env-flag-toggle", "Boolean flag from env",
    "Bash", "Adversarial",
    'fn main() { let verbose = 0; if verbose == 1 { println!("debug: verbose mode on"); } println!("running"); }',
    "running"))

bash_entries.append(entry("B-1588", "env-config-defaults", "Configuration with defaults",
    "Bash", "Adversarial",
    'fn main() { let port = 8080; let workers = 4; let debug = 0; println!("port={} workers={} debug={}", port, workers, debug); }',
    "port=8080 workers=4 debug=0"))

# === R. Symbolic bash patterns (B-1589..B-1591) ===
bash_entries.append(entry("B-1589", "sym-negation-double", "Double negation patterns",
    "Bash", "Adversarial",
    'fn main() { let x = 5; let y = 0; if x != 0 { println!("nonzero"); } if y == 0 { println!("zero"); } }',
    "nonzero"))

bash_entries.append(entry("B-1590", "sym-bitwise-and", "Bitwise AND for power-of-two check",
    "Bash", "Adversarial",
    'fn is_pow2(n: i32) -> i32 { if n <= 0 { return 0; } if n & (n - 1) == 0 { return 1; } return 0; } fn main() { for n in [1, 2, 3, 4, 7, 8, 16, 15] { println!("{}:{}", n, is_pow2(n)); } }',
    "1:1"))

bash_entries.append(entry("B-1591", "sym-compound-assign", "Compound assignment operators",
    "Bash", "Adversarial",
    'fn main() { let mut x = 10; x += 5; println!("{}", x); x -= 3; println!("{}", x); x *= 2; println!("{}", x); x /= 4; println!("{}", x); x %= 3; println!("{}", x); }',
    "15"))

# === M. Traps (B-1592..B-1593) ===
bash_entries.append(entry("B-1592", "trap-cleanup-pattern", "Cleanup function pattern",
    "Bash", "Adversarial",
    'fn cleanup() { println!("cleanup_done"); } fn main() { println!("start"); println!("work"); cleanup(); }',
    "start"))

bash_entries.append(entry("B-1593", "trap-guard-pattern", "Guard clause pattern",
    "Bash", "Adversarial",
    'fn process(x: i32) -> i32 { if x <= 0 { return -1; } if x > 1000 { return -1; } return x * 2; } fn main() { println!("{}", process(0)); println!("{}", process(50)); println!("{}", process(9999)); }',
    "-1"))

# === N. Command line parsing (B-1594..B-1595) ===
bash_entries.append(entry("B-1594", "cli-flag-parser", "Simple flag parser simulation",
    "Bash", "Adversarial",
    'fn main() { let verbose = 0; let quiet = 1; if verbose == 1 { println!("verbose mode"); } if quiet == 1 { println!("quiet mode"); } println!("done"); }',
    "quiet mode"))

bash_entries.append(entry("B-1595", "cli-subcommand-dispatch", "Subcommand dispatch pattern",
    "Bash", "Adversarial",
    'fn cmd_build() { println!("building"); } fn cmd_test() { println!("testing"); } fn cmd_run() { println!("running"); } fn main() { let cmd = 2; if cmd == 1 { cmd_build(); } if cmd == 2 { cmd_test(); } if cmd == 3 { cmd_run(); } }',
    "testing"))

# Makefile entries M-406..M-410
makefile_entries = []

makefile_entries.append(entry("M-406", "make-install-uninstall", "Install/uninstall targets",
    "Makefile", "Adversarial",
    r'fn main() { exec("PREFIX := /usr/local"); exec("install:"); println!("\tinstall -m 755 target/release/app $(PREFIX)/bin/"); exec("uninstall:"); println!("\trm -f $(PREFIX)/bin/app"); }',
    "PREFIX := /usr/local"))

makefile_entries.append(entry("M-407", "make-phony-all", "Phony targets with .PHONY",
    "Makefile", "Adversarial",
    r'fn main() { exec(".PHONY: all clean test lint"); exec("all: lint test build"); exec("build:"); println!("\tcargo build --release"); exec("test:"); println!("\tcargo test"); exec("lint:"); println!("\tcargo clippy -- -D warnings"); exec("clean:"); println!("\tcargo clean"); }',
    ".PHONY: all clean test lint"))

makefile_entries.append(entry("M-408", "make-conditional-os", "Conditional OS detection",
    "Makefile", "Adversarial",
    r'fn main() { exec("UNAME := $(shell uname -s)"); exec("build:"); println!("\t@echo Building for $(UNAME)"); println!("\tcargo build --release"); }',
    "UNAME := $(shell uname"))

makefile_entries.append(entry("M-409", "make-generate-docs", "Documentation generation targets",
    "Makefile", "Adversarial",
    r'fn main() { exec("docs:"); println!("\tcargo doc --no-deps"); println!("\tmdbook build book/"); exec("docs-serve:"); println!("\tmdbook serve book/ --port 3000"); }',
    "cargo doc --no-deps"))

makefile_entries.append(entry("M-410", "make-release-tag", "Release with version tag",
    "Makefile", "Adversarial",
    'fn main() { exec("VERSION := 1.0.0"); exec("release:"); println!("\\tcargo build --release"); println!("\\tgit tag -a v$(VERSION) -m Release"); println!("\\tgit push --tags"); }',
    "VERSION := 1.0.0"))

# Dockerfile entries D-366..D-370
dockerfile_entries = []

dockerfile_entries.append(entry("D-366", "docker-python-uvicorn", "Python FastAPI with uvicorn",
    "Dockerfile", "Adversarial",
    r'fn main() { from_image("python:3.12-slim"); workdir("/app"); copy("requirements.txt", "."); run("pip install --no-cache-dir -r requirements.txt"); copy(".", "."); expose(8000); cmd("uvicorn main:app --host 0.0.0.0 --port 8000"); }',
    "FROM python:3.12-slim"))

dockerfile_entries.append(entry("D-367", "docker-bun-runtime", "Bun JavaScript runtime",
    "Dockerfile", "Adversarial",
    r'fn main() { from_image("oven/bun:1"); workdir("/app"); copy("package.json bun.lockb", "."); run("bun install --frozen-lockfile"); copy(".", "."); expose(3000); cmd("bun run start"); }',
    "FROM oven/bun:1"))

dockerfile_entries.append(entry("D-368", "docker-gradle-spring", "Spring Boot with Gradle",
    "Dockerfile", "Adversarial",
    r'fn builder() { from_image("gradle:8-jdk21"); workdir("/app"); copy(".", "."); run("gradle bootJar --no-daemon"); } fn main() { from_image("eclipse-temurin:21-jre"); copy("--from=builder /app/build/libs/app.jar", "/app/app.jar"); expose(8080); entrypoint("java -jar /app/app.jar"); }',
    "FROM gradle:8-jdk21"))

dockerfile_entries.append(entry("D-369", "docker-rust-chef", "Rust with cargo-chef caching",
    "Dockerfile", "Adversarial",
    r'fn planner() { from_image("rust:1.75"); run("cargo install cargo-chef"); workdir("/app"); copy(".", "."); run("cargo chef prepare --recipe-path recipe.json"); } fn builder() { from_image("rust:1.75"); run("cargo install cargo-chef"); workdir("/app"); copy("--from=planner /app/recipe.json", "recipe.json"); run("cargo chef cook --release --recipe-path recipe.json"); copy(".", "."); run("cargo build --release"); } fn main() { from_image("debian:bookworm-slim"); copy("--from=builder /app/target/release/app", "/usr/local/bin/"); cmd("app"); }',
    "FROM rust:1.75"))

dockerfile_entries.append(entry("D-370", "docker-terraform-apply", "Terraform infrastructure Dockerfile",
    "Dockerfile", "Adversarial",
    r'fn main() { from_image("hashicorp/terraform:1.7"); workdir("/infra"); copy(".", "."); run("terraform init"); cmd("terraform plan"); }',
    "FROM hashicorp/terraform:1.7"))


def generate_load_function(name, entries_str):
    return f"""    fn {name}(&mut self) {{
        let entries = vec![
{entries_str}
        ];
        for e in entries {{ self.entries.push(e); }}
    }}
"""

bash_str = "\n".join(bash_entries)
make_str = "\n".join(makefile_entries)
docker_str = "\n".join(dockerfile_entries)

print(generate_load_function("load_expansion41_bash", bash_str))
print(generate_load_function("load_expansion29_makefile", make_str))
print(generate_load_function("load_expansion29_dockerfile", docker_str))
