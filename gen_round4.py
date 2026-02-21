#!/usr/bin/env python3
"""Generate Round 4 corpus entries: Categories A-V with pathological edge cases."""

def format_rust_string(s):
    """Format a Rust string literal, using raw strings if needed."""
    if '"' in s and '\\' in s:
        return 'r##"' + s + '"##'
    elif '"' in s:
        return 'r#"' + s + '"#'
    else:
        return 'r#"' + s + '"#'

def entry(id, name, desc, fmt, tier, input_code, expected):
    """Generate a CorpusEntry::new() call."""
    inp = format_rust_string(input_code)
    return f'            CorpusEntry::new("{id}", "{name}", "{desc}",\n                CorpusFormat::{fmt}, CorpusTier::{tier},\n                {inp},\n                "{expected}"),'

# ============================================================
# BASH entries: B-1501 onwards
# ============================================================
bash_entries = []

# === Category L: Control flow, braces, semicolons ===
bash_entries.append(entry("B-1501", "ctrl-nested-if-elif-chain", "Deep if/elif/else chain with 5 branches",
    "Bash", "Adversarial",
    'fn classify(n: i32) -> i32 { if n > 100 { return 5; } if n > 50 { return 4; } if n > 25 { return 3; } if n > 10 { return 2; } if n > 0 { return 1; } return 0; } fn main() { println!("{}", classify(75)); println!("{}", classify(5)); println!("{}", classify(0)); }',
    "4"))

bash_entries.append(entry("B-1502", "ctrl-while-accumulate", "While loop with running sum and early break",
    "Bash", "Adversarial",
    'fn main() { let mut sum = 0; let mut i = 1; while i <= 100 { sum += i; if sum > 50 { break; } i += 1; } println!("sum={} i={}", sum, i); }',
    "sum=55 i=10"))

bash_entries.append(entry("B-1503", "ctrl-for-nested-break", "Nested for loops with break in inner",
    "Bash", "Adversarial",
    'fn main() { let mut count = 0; for i in 0..5 { for j in 0..5 { if j >= i { break; } count += 1; } } println!("count={}", count); }',
    "count=10"))

bash_entries.append(entry("B-1504", "ctrl-match-fallthrough", "Match with many literal arms",
    "Bash", "Adversarial",
    'fn grade(score: i32) -> &str { match score { 10 => "A+", 9 => "A", 8 => "B", 7 => "C", 6 => "D", _ => "F" } } fn main() { for i in [10, 9, 8, 7, 6, 5, 3] { println!("{} -> {}", i, grade(i)); } }',
    "10 -> A+"))

bash_entries.append(entry("B-1505", "ctrl-continue-skip-evens", "Continue to skip even numbers",
    "Bash", "Adversarial",
    'fn main() { let mut sum = 0; for i in 1..=20 { if i % 2 == 0 { continue; } sum += i; } println!("odd_sum={}", sum); }',
    "odd_sum=100"))

# === Category Q: Numerical methods ===
bash_entries.append(entry("B-1506", "num-integer-sqrt", "Integer square root via Newton method",
    "Bash", "Adversarial",
    'fn isqrt(n: i32) -> i32 { if n <= 1 { return n; } let mut x = n; let mut y = (x + 1) / 2; while y < x { x = y; y = (x + n / x) / 2; } return x; } fn main() { println!("{}", isqrt(0)); println!("{}", isqrt(1)); println!("{}", isqrt(4)); println!("{}", isqrt(9)); println!("{}", isqrt(16)); println!("{}", isqrt(100)); }',
    "10"))

bash_entries.append(entry("B-1507", "num-gcd-euclidean", "GCD via Euclidean algorithm",
    "Bash", "Adversarial",
    'fn gcd(a: i32, b: i32) -> i32 { if b == 0 { return a; } return gcd(b, a % b); } fn main() { println!("{}", gcd(48, 18)); println!("{}", gcd(100, 75)); println!("{}", gcd(17, 13)); }',
    "6"))

bash_entries.append(entry("B-1508", "num-fibonacci-iterative", "Fibonacci via iterative loop",
    "Bash", "Adversarial",
    'fn fib(n: i32) -> i32 { if n <= 1 { return n; } let mut a = 0; let mut b = 1; let mut i = 2; while i <= n { let c = a + b; a = b; b = c; i += 1; } return b; } fn main() { for i in 0..=10 { println!("fib({})={}", i, fib(i)); } }',
    "fib(10)=55"))

bash_entries.append(entry("B-1509", "num-prime-sieve-simple", "Simple prime check function",
    "Bash", "Adversarial",
    'fn is_prime(n: i32) -> i32 { if n < 2 { return 0; } let mut i = 2; while i * i <= n { if n % i == 0 { return 0; } i += 1; } return 1; } fn main() { let mut count = 0; for n in 2..=50 { if is_prime(n) == 1 { count += 1; } } println!("primes_to_50={}", count); }',
    "primes_to_50=15"))

bash_entries.append(entry("B-1510", "num-factorial-iterative", "Factorial via iterative multiplication",
    "Bash", "Adversarial",
    'fn factorial(n: i32) -> i32 { let mut result = 1; let mut i = 2; while i <= n { result = result * i; i += 1; } return result; } fn main() { println!("{}", factorial(1)); println!("{}", factorial(5)); println!("{}", factorial(8)); }',
    "120"))

# === Category T: Functions, closures, functional programming ===
bash_entries.append(entry("B-1511", "func-higher-order-apply", "Apply function via dispatch",
    "Bash", "Adversarial",
    'fn double(x: i32) -> i32 { return x * 2; } fn triple(x: i32) -> i32 { return x * 3; } fn apply(op: i32, val: i32) -> i32 { if op == 2 { return double(val); } return triple(val); } fn main() { println!("{}", apply(2, 5)); println!("{}", apply(3, 5)); }',
    "10"))

bash_entries.append(entry("B-1512", "func-recursive-tower", "Tower of recursion 3 levels deep",
    "Bash", "Adversarial",
    'fn a(n: i32) -> i32 { if n <= 0 { return 1; } return b(n - 1) + 1; } fn b(n: i32) -> i32 { if n <= 0 { return 2; } return c(n - 1) + 2; } fn c(n: i32) -> i32 { if n <= 0 { return 3; } return a(n - 1) + 3; } fn main() { println!("{}", a(6)); }',
    "12"))

bash_entries.append(entry("B-1513", "func-composition-chain", "Chained function composition f(g(h(x)))",
    "Bash", "Adversarial",
    'fn add5(x: i32) -> i32 { return x + 5; } fn mul3(x: i32) -> i32 { return x * 3; } fn sub2(x: i32) -> i32 { return x - 2; } fn main() { let x = 4; let result = add5(mul3(sub2(x))); println!("{}", result); }',
    "11"))

bash_entries.append(entry("B-1514", "func-mutual-recursion-even-odd", "Mutual recursion for even/odd check",
    "Bash", "Adversarial",
    'fn is_even(n: i32) -> i32 { if n == 0 { return 1; } return is_odd(n - 1); } fn is_odd(n: i32) -> i32 { if n == 0 { return 0; } return is_even(n - 1); } fn main() { for i in 0..=6 { println!("{}: even={} odd={}", i, is_even(i), is_odd(i)); } }',
    "6: even=1 odd=0"))

# === Category I: Data structures ===
bash_entries.append(entry("B-1515", "ds-array-reverse", "Reverse array in-place",
    "Bash", "Adversarial",
    'fn main() { let arr = [10, 20, 30, 40, 50]; for i in [4, 3, 2, 1, 0] { println!("{}", arr[i]); } }',
    "50"))

bash_entries.append(entry("B-1516", "ds-array-sum-product", "Compute sum and product of array",
    "Bash", "Adversarial",
    'fn main() { let arr = [1, 2, 3, 4, 5]; let mut sum = 0; let mut prod = 1; for i in 0..5 { sum += arr[i]; prod = prod * arr[i]; } println!("sum={} prod={}", sum, prod); }',
    "sum=15 prod=120"))

bash_entries.append(entry("B-1517", "ds-array-find-max-min", "Find max and min in array",
    "Bash", "Adversarial",
    'fn main() { let arr = [42, 17, 93, 8, 56]; let mut max = arr[0]; let mut min = arr[0]; for i in 1..5 { if arr[i] > max { max = arr[i]; } if arr[i] < min { min = arr[i]; } } println!("max={} min={}", max, min); }',
    "max=93 min=8"))

# === Category A: Control flow edge cases ===
bash_entries.append(entry("B-1518", "flow-deeply-nested-conditions", "4-level deep nested if",
    "Bash", "Adversarial",
    'fn main() { let x = 15; if x > 0 { if x > 5 { if x > 10 { if x > 20 { println!("huge"); } else { println!("large"); } } else { println!("medium"); } } else { println!("small"); } } else { println!("zero_or_neg"); } }',
    "large"))

bash_entries.append(entry("B-1519", "flow-for-with-accumulator-pattern", "For loop accumulator pattern",
    "Bash", "Adversarial",
    'fn main() { let mut max = 0; for i in 1..=10 { let sq = i * i; if sq > max { max = sq; } } println!("max_sq={}", max); }',
    "max_sq=100"))

# === Category E: Environment variables ===
bash_entries.append(entry("B-1520", "env-path-with-default", "Env var with default value",
    "Bash", "Adversarial",
    r'fn main() { let home = std::env::var("HOME").unwrap_or("/tmp".to_string()); println!("home={}", home); }',
    'home='))

bash_entries.append(entry("B-1521", "env-multiple-vars", "Read multiple env vars",
    "Bash", "Adversarial",
    r'fn main() { let user = std::env::var("USER").unwrap_or("nobody".to_string()); let shell = std::env::var("SHELL").unwrap_or("/bin/sh".to_string()); println!("user={} shell={}", user, shell); }',
    'user='))

# === Category N: Command line parsing ===
bash_entries.append(entry("B-1522", "cli-positional-args", "Positional argument access",
    "Bash", "Adversarial",
    'fn main() { let args = std::env::args(); let first = args.get(1).unwrap_or("default"); println!("arg1={}", first); }',
    'arg1='))

bash_entries.append(entry("B-1523", "cli-arg-count", "Count command line arguments",
    "Bash", "Adversarial",
    'fn main() { let count = std::env::args().len(); println!("argc={}", count); }',
    'argc='))

# === Category V: Clippy-pedantic patterns ===
bash_entries.append(entry("B-1524", "clippy-no-unwrap", "Error handling without unwrap via if/else",
    "Bash", "Adversarial",
    'fn safe_div(a: i32, b: i32) -> i32 { if b == 0 { return -1; } return a / b; } fn main() { println!("{}", safe_div(10, 3)); println!("{}", safe_div(10, 0)); println!("{}", safe_div(100, 7)); }',
    "3"))

bash_entries.append(entry("B-1525", "clippy-bounds-check", "Array bounds checking pattern",
    "Bash", "Adversarial",
    'fn safe_get(arr: [i32; 5], idx: i32) -> i32 { if idx < 0 { return -1; } if idx >= 5 { return -1; } return arr[idx]; } fn main() { let data = [10, 20, 30, 40, 50]; println!("{}", safe_get(data, 2)); println!("{}", safe_get(data, 5)); println!("{}", safe_get(data, -1)); }',
    "30"))

# === More numerical entries ===
bash_entries.append(entry("B-1526", "num-collatz-steps", "Collatz conjecture step counter",
    "Bash", "Adversarial",
    'fn collatz_steps(n: i32) -> i32 { let mut x = n; let mut steps = 0; while x != 1 { if x % 2 == 0 { x = x / 2; } else { x = 3 * x + 1; } steps += 1; } return steps; } fn main() { println!("{}", collatz_steps(1)); println!("{}", collatz_steps(6)); println!("{}", collatz_steps(27)); }',
    "0"))

bash_entries.append(entry("B-1527", "num-power-iterative", "Iterative integer exponentiation",
    "Bash", "Adversarial",
    'fn power(base: i32, exp: i32) -> i32 { let mut result = 1; let mut i = 0; while i < exp { result = result * base; i += 1; } return result; } fn main() { println!("2^10={}", power(2, 10)); println!("3^5={}", power(3, 5)); println!("5^0={}", power(5, 0)); }',
    "2^10=1024"))

bash_entries.append(entry("B-1528", "num-abs-diff", "Absolute difference computation",
    "Bash", "Adversarial",
    'fn abs_diff(a: i32, b: i32) -> i32 { if a > b { return a - b; } return b - a; } fn main() { println!("{}", abs_diff(10, 3)); println!("{}", abs_diff(3, 10)); println!("{}", abs_diff(5, 5)); }',
    "7"))

bash_entries.append(entry("B-1529", "num-digit-sum", "Sum digits of a number",
    "Bash", "Adversarial",
    'fn digit_sum(n: i32) -> i32 { let mut x = n; let mut sum = 0; while x > 0 { sum += x % 10; x = x / 10; } return sum; } fn main() { println!("{}", digit_sum(123)); println!("{}", digit_sum(9999)); println!("{}", digit_sum(0)); }',
    "6"))

bash_entries.append(entry("B-1530", "num-count-digits", "Count number of digits",
    "Bash", "Adversarial",
    'fn count_digits(n: i32) -> i32 { if n == 0 { return 1; } let mut x = n; let mut count = 0; while x > 0 { count += 1; x = x / 10; } return count; } fn main() { println!("{}", count_digits(0)); println!("{}", count_digits(9)); println!("{}", count_digits(100)); println!("{}", count_digits(99999)); }',
    "1"))

# === More function patterns ===
bash_entries.append(entry("B-1531", "func-ternary-chain", "Chained ternary-like if expressions",
    "Bash", "Adversarial",
    'fn sign(x: i32) -> i32 { if x > 0 { return 1; } if x < 0 { return -1; } return 0; } fn main() { println!("{} {} {}", sign(42), sign(-7), sign(0)); }',
    "1 -1 0"))

bash_entries.append(entry("B-1532", "func-max3", "Maximum of three values",
    "Bash", "Adversarial",
    'fn max3(a: i32, b: i32, c: i32) -> i32 { let mut m = a; if b > m { m = b; } if c > m { m = c; } return m; } fn main() { println!("{}", max3(3, 7, 5)); println!("{}", max3(10, 2, 8)); println!("{}", max3(1, 1, 1)); }',
    "7"))

bash_entries.append(entry("B-1533", "func-clamp-with-return", "Clamp using early returns",
    "Bash", "Adversarial",
    'fn clamp(x: i32, lo: i32, hi: i32) -> i32 { if x < lo { return lo; } if x > hi { return hi; } return x; } fn main() { println!("{}", clamp(-5, 0, 100)); println!("{}", clamp(50, 0, 100)); println!("{}", clamp(200, 0, 100)); }',
    "0"))

bash_entries.append(entry("B-1534", "func-lerp-integer", "Integer linear interpolation",
    "Bash", "Adversarial",
    'fn lerp(a: i32, b: i32, t_pct: i32) -> i32 { return a + (b - a) * t_pct / 100; } fn main() { println!("{}", lerp(0, 100, 0)); println!("{}", lerp(0, 100, 50)); println!("{}", lerp(0, 100, 100)); println!("{}", lerp(10, 20, 75)); }',
    "0"))

bash_entries.append(entry("B-1535", "func-binary-search", "Binary search in sorted array",
    "Bash", "Adversarial",
    'fn bsearch(arr: [i32; 8], target: i32) -> i32 { let mut lo = 0; let mut hi = 7; while lo <= hi { let mid = (lo + hi) / 2; if arr[mid] == target { return mid; } if arr[mid] < target { lo = mid + 1; } else { hi = mid - 1; } } return -1; } fn main() { let arr = [2, 5, 8, 12, 16, 23, 38, 56]; println!("{}", bsearch(arr, 23)); println!("{}", bsearch(arr, 99)); }',
    "5"))

# ============================================================
# MAKEFILE entries: M-391 onwards
# ============================================================
makefile_entries = []

# === Category O: Nested Makefiles, includes, complex patterns ===
makefile_entries.append(entry("M-391", "make-multi-target-deps", "Multiple targets with shared deps",
    "Makefile", "Adversarial",
    r'fn main() { let cc = "gcc"; let cflags = "-Wall -O2"; exec("all: build test"); exec("build: main.o utils.o"); println!("\t$(CC) $(CFLAGS) -o app main.o utils.o"); exec("test: test.o utils.o"); println!("\t$(CC) $(CFLAGS) -o test_app test.o utils.o"); }',
    "CC := gcc"))

makefile_entries.append(entry("M-392", "make-pattern-rule", "Pattern rule for .c to .o",
    "Makefile", "Adversarial",
    r'fn main() { let cc = "gcc"; let cflags = "-Wall -Wextra -std=c11"; exec("%.o: %.c"); println!("\t$(CC) $(CFLAGS) -c $< -o $@"); }',
    "CC := gcc"))

makefile_entries.append(entry("M-393", "make-conditional-platform", "Platform conditional compilation",
    "Makefile", "Adversarial",
    r'fn main() { let os = "linux"; let cc = "gcc"; exec("all: build"); println!("ifeq ($(OS),linux)"); println!("\tCC := gcc"); println!("else"); println!("\tCC := clang"); println!("endif"); }',
    "CC := gcc"))

makefile_entries.append(entry("M-394", "make-phony-targets", ".PHONY declaration with multiple targets",
    "Makefile", "Adversarial",
    r'fn main() { exec(".PHONY: all clean test lint"); exec("all: build"); exec("clean:"); println!("\trm -rf build/"); exec("test:"); println!("\tcargo test"); exec("lint:"); println!("\tcargo clippy"); }',
    ".PHONY: all clean test lint"))

makefile_entries.append(entry("M-395", "make-recursive-make", "Recursive make for subdirectories",
    "Makefile", "Adversarial",
    r'fn main() { let subdirs = "lib src tests"; exec("all:"); println!("\tfor dir in $(SUBDIRS); do $(MAKE) -C $$dir; done"); exec("clean:"); println!("\tfor dir in $(SUBDIRS); do $(MAKE) -C $$dir clean; done"); }',
    "SUBDIRS := lib src tests"))

makefile_entries.append(entry("M-396", "make-auto-deps", "Automatic dependency generation",
    "Makefile", "Adversarial",
    r'fn main() { let cc = "gcc"; let srcs = "main.c utils.c"; exec("-include $(SRCS:.c=.d)"); exec("%.o: %.c"); println!("\t$(CC) -MMD -MP -c $< -o $@"); }',
    "CC := gcc"))

makefile_entries.append(entry("M-397", "make-define-block", "Multi-line define block",
    "Makefile", "Adversarial",
    r'fn main() { exec("define HELP_TEXT"); println!("Usage: make [target]"); println!("Targets:"); println!("  build  - Build the project"); println!("  test   - Run tests"); println!("  clean  - Clean build artifacts"); exec("endef"); exec("help:"); println!("\t@echo \"$(HELP_TEXT)\""); }',
    "define HELP_TEXT"))

makefile_entries.append(entry("M-398", "make-vpath-search", "VPATH for source directory search",
    "Makefile", "Adversarial",
    r'fn main() { exec("VPATH = src:include:lib"); let cc = "gcc"; exec("app: main.o module.o"); println!("\t$(CC) -o $@ $^"); exec("%.o: %.c"); println!("\t$(CC) -I include -c $< -o $@"); }',
    "VPATH = src:include:lib"))

makefile_entries.append(entry("M-399", "make-install-target", "Install target with PREFIX",
    "Makefile", "Adversarial",
    r'fn main() { exec("PREFIX ?= /usr/local"); exec("install: build"); println!("\tinstall -d $(DESTDIR)$(PREFIX)/bin"); println!("\tinstall -m 755 app $(DESTDIR)$(PREFIX)/bin/app"); exec("uninstall:"); println!("\trm -f $(DESTDIR)$(PREFIX)/bin/app"); }',
    "PREFIX ?= /usr/local"))

makefile_entries.append(entry("M-400", "make-release-debug", "Debug vs release build modes",
    "Makefile", "Adversarial",
    r'fn main() { let cc = "gcc"; exec("ifdef DEBUG"); println!("CFLAGS := -g -O0 -DDEBUG"); exec("else"); println!("CFLAGS := -O2 -DNDEBUG"); exec("endif"); exec("build:"); println!("\t$(CC) $(CFLAGS) -o app main.c"); }',
    "CC := gcc"))

# ============================================================
# DOCKERFILE entries: D-351 onwards
# ============================================================
dockerfile_entries = []

# === Category P: Multi-stage, complex layering, nested logic ===
dockerfile_entries.append(entry("D-351", "docker-multi-stage-rust", "Multi-stage Rust build with slim runtime",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM rust:1.75 AS builder"); exec("WORKDIR /app"); exec("COPY Cargo.toml Cargo.lock ./"); exec("RUN mkdir src && echo \"fn main() {}\" > src/main.rs && cargo build --release"); exec("COPY src/ src/"); exec("RUN cargo build --release"); exec("FROM debian:bookworm-slim"); exec("COPY --from=builder /app/target/release/app /usr/local/bin/"); exec("CMD [\"app\"]"); }',
    "FROM rust:1.75 AS builder"))

dockerfile_entries.append(entry("D-352", "docker-multi-stage-node", "Multi-stage Node.js with pruned deps",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM node:20-alpine AS deps"); exec("WORKDIR /app"); exec("COPY package*.json ./"); exec("RUN npm ci --only=production"); exec("FROM node:20-alpine AS build"); exec("WORKDIR /app"); exec("COPY . ."); exec("RUN npm ci && npm run build"); exec("FROM node:20-alpine"); exec("COPY --from=deps /app/node_modules ./node_modules"); exec("COPY --from=build /app/dist ./dist"); exec("CMD [\"node\", \"dist/index.js\"]"); }',
    "FROM node:20-alpine AS deps"))

dockerfile_entries.append(entry("D-353", "docker-multi-stage-go", "Multi-stage Go with scratch runtime",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM golang:1.22-alpine AS builder"); exec("WORKDIR /app"); exec("COPY go.* ./"); exec("RUN go mod download"); exec("COPY . ."); exec("RUN CGO_ENABLED=0 go build -ldflags=\"-s -w\" -o /app/server"); exec("FROM scratch"); exec("COPY --from=builder /app/server /server"); exec("ENTRYPOINT [\"/server\"]"); }',
    "FROM golang:1.22-alpine AS builder"))

dockerfile_entries.append(entry("D-354", "docker-arg-env-matrix", "ARG and ENV interaction pattern",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("ARG BASE_IMAGE=python:3.12-slim"); exec("ARG APP_VERSION=1.0.0"); exec("FROM ${BASE_IMAGE}"); let app_version = "1.0.0"; let port = "8080"; exec("WORKDIR /app"); exec("COPY requirements.txt ."); exec("RUN pip install --no-cache-dir -r requirements.txt"); exec("COPY . ."); exec("EXPOSE ${PORT}"); exec("CMD [\"python\", \"app.py\"]"); }',
    "ARG BASE_IMAGE=python:3.12-slim"))

dockerfile_entries.append(entry("D-355", "docker-healthcheck-pattern", "Healthcheck with retries",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM python:3.12-slim"); exec("WORKDIR /app"); exec("COPY . ."); exec("RUN pip install --no-cache-dir -r requirements.txt"); exec("HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://localhost:8080/health || exit 1"); exec("EXPOSE 8080"); exec("CMD [\"python\", \"app.py\"]"); }',
    "HEALTHCHECK --interval=30s"))

dockerfile_entries.append(entry("D-356", "docker-cache-mount", "BuildKit cache mount for pip",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM python:3.12"); exec("WORKDIR /app"); exec("COPY requirements.txt ."); exec("RUN --mount=type=cache,target=/root/.cache/pip pip install -r requirements.txt"); exec("COPY . ."); exec("CMD [\"python\", \"main.py\"]"); }',
    "FROM python:3.12"))

dockerfile_entries.append(entry("D-357", "docker-multi-stage-test", "Test stage in multi-stage build",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM rust:1.75 AS builder"); exec("WORKDIR /app"); exec("COPY . ."); exec("RUN cargo build --release"); exec("FROM builder AS tester"); exec("RUN cargo test"); exec("FROM debian:bookworm-slim AS runtime"); exec("COPY --from=builder /app/target/release/app /usr/local/bin/"); exec("CMD [\"app\"]"); }',
    "FROM builder AS tester"))

dockerfile_entries.append(entry("D-358", "docker-user-security", "Non-root user security pattern",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM node:20-alpine"); exec("RUN addgroup -g 1001 -S appgroup && adduser -u 1001 -S appuser -G appgroup"); exec("WORKDIR /app"); exec("COPY --chown=appuser:appgroup . ."); exec("RUN npm ci --only=production"); exec("USER appuser"); exec("CMD [\"node\", \"index.js\"]"); }',
    "USER appuser"))

dockerfile_entries.append(entry("D-359", "docker-multi-platform", "Multi-platform build args",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM --platform=$BUILDPLATFORM golang:1.22 AS builder"); exec("ARG TARGETPLATFORM"); exec("ARG BUILDPLATFORM"); exec("WORKDIR /app"); exec("COPY . ."); exec("RUN GOOS=$(echo $TARGETPLATFORM | cut -d/ -f1) GOARCH=$(echo $TARGETPLATFORM | cut -d/ -f2) go build -o /app/server"); exec("FROM gcr.io/distroless/static"); exec("COPY --from=builder /app/server /server"); exec("CMD [\"/server\"]"); }',
    "FROM --platform=$BUILDPLATFORM golang:1.22 AS builder"))

dockerfile_entries.append(entry("D-360", "docker-layer-optimization", "Layer optimization with combined RUN",
    "Dockerfile", "Adversarial",
    r'fn main() { exec("FROM ubuntu:22.04"); exec("RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*"); exec("WORKDIR /app"); exec("COPY . ."); exec("CMD [\"/app/entrypoint.sh\"]"); }',
    "FROM ubuntu:22.04"))


# ============================================================
# Generate the Rust source code
# ============================================================

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

print("// === PASTE INTO registry.rs (before the closing } of impl CorpusRegistry) ===")
print()
print(generate_load_function("load_expansion39_bash", bash_str))
print(generate_load_function("load_expansion27_makefile", make_str))
print(generate_load_function("load_expansion27_dockerfile", docker_str))

print("// === PASTE INTO load_full() ===")
print("//        registry.load_expansion39_bash();")
print("//        registry.load_expansion27_makefile();")
print("//        registry.load_expansion27_dockerfile();")
