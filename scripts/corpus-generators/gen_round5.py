#!/usr/bin/env python3
"""Generate Round 5 corpus entries: Harder patterns across categories A-V."""

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

# === Harder numerical methods ===
bash_entries.append(entry("B-1536", "num-modular-exp", "Modular exponentiation",
    "Bash", "Adversarial",
    'fn mod_pow(base: i32, exp: i32, modulus: i32) -> i32 { let mut result = 1; let mut b = base % modulus; let mut e = exp; while e > 0 { if e % 2 == 1 { result = (result * b) % modulus; } e = e / 2; b = (b * b) % modulus; } return result; } fn main() { println!("{}", mod_pow(2, 10, 1000)); println!("{}", mod_pow(3, 13, 1000)); }',
    "24"))

bash_entries.append(entry("B-1537", "num-lcm-via-gcd", "LCM computed via GCD",
    "Bash", "Adversarial",
    'fn gcd(a: i32, b: i32) -> i32 { if b == 0 { return a; } return gcd(b, a % b); } fn lcm(a: i32, b: i32) -> i32 { return (a / gcd(a, b)) * b; } fn main() { println!("{}", lcm(12, 18)); println!("{}", lcm(7, 5)); println!("{}", lcm(100, 75)); }',
    "36"))

bash_entries.append(entry("B-1538", "num-sieve-optimized", "Optimized prime counting",
    "Bash", "Adversarial",
    'fn is_prime(n: i32) -> i32 { if n < 2 { return 0; } if n == 2 { return 1; } if n % 2 == 0 { return 0; } let mut i = 3; while i * i <= n { if n % i == 0 { return 0; } i += 2; } return 1; } fn main() { let mut count = 0; for n in 2..=100 { if is_prime(n) == 1 { count += 1; } } println!("primes_100={}", count); }',
    "primes_100=25"))

bash_entries.append(entry("B-1539", "num-sum-of-squares", "Sum of first N squares",
    "Bash", "Adversarial",
    'fn sum_squares(n: i32) -> i32 { let mut sum = 0; let mut i = 1; while i <= n { sum += i * i; i += 1; } return sum; } fn main() { println!("{}", sum_squares(1)); println!("{}", sum_squares(5)); println!("{}", sum_squares(10)); }',
    "1"))

bash_entries.append(entry("B-1540", "num-harmonic-scaled", "Harmonic sum scaled by 1000",
    "Bash", "Adversarial",
    'fn harmonic(n: i32) -> i32 { let mut sum = 0; let mut k = 1; while k <= n { sum += 1000 / k; k += 1; } return sum; } fn main() { println!("{}", harmonic(1)); println!("{}", harmonic(10)); }',
    "1000"))

# === Harder function patterns ===
bash_entries.append(entry("B-1541", "func-ackermann", "Ackermann function (small values)",
    "Bash", "Adversarial",
    'fn ack(m: i32, n: i32) -> i32 { if m == 0 { return n + 1; } if n == 0 { return ack(m - 1, 1); } return ack(m - 1, ack(m, n - 1)); } fn main() { println!("{}", ack(0, 0)); println!("{}", ack(1, 1)); println!("{}", ack(2, 2)); println!("{}", ack(3, 3)); }',
    "61"))

bash_entries.append(entry("B-1542", "func-fold-reduce", "Manual fold/reduce pattern",
    "Bash", "Adversarial",
    'fn fold_sum(arr: [i32; 5]) -> i32 { let mut acc = 0; for i in 0..5 { acc += arr[i]; } return acc; } fn fold_max(arr: [i32; 5]) -> i32 { let mut acc = arr[0]; for i in 1..5 { if arr[i] > acc { acc = arr[i]; } } return acc; } fn main() { let data = [3, 7, 1, 9, 4]; println!("sum={} max={}", fold_sum(data), fold_max(data)); }',
    "sum=24 max=9"))

bash_entries.append(entry("B-1543", "func-pipeline", "Pipeline: map -> filter -> reduce",
    "Bash", "Adversarial",
    'fn main() { let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; let mut sum = 0; for i in 0..10 { let doubled = data[i] * 2; if doubled > 8 { sum += doubled; } } println!("sum={}", sum); }',
    "sum=90"))

bash_entries.append(entry("B-1544", "func-dispatch-match", "Function dispatch via match",
    "Bash", "Adversarial",
    'fn add(a: i32, b: i32) -> i32 { return a + b; } fn sub(a: i32, b: i32) -> i32 { return a - b; } fn mul(a: i32, b: i32) -> i32 { return a * b; } fn calc(op: i32, a: i32, b: i32) -> i32 { match op { 1 => add(a, b), 2 => sub(a, b), 3 => mul(a, b), _ => 0 } } fn main() { println!("{}", calc(1, 10, 5)); println!("{}", calc(2, 10, 5)); println!("{}", calc(3, 10, 5)); println!("{}", calc(4, 10, 5)); }',
    "15"))

# === Harder control flow ===
bash_entries.append(entry("B-1545", "ctrl-linear-search", "Linear search in array",
    "Bash", "Adversarial",
    'fn main() { let data = [3, 7, 2, 9, 1, 5, 8, 4, 6, 10]; let mut found = -1; let mut i = 0; while i < 10 { if data[i] == 5 { found = i; break; } i += 1; } println!("found_at={}", found); }',
    "found_at=5"))

bash_entries.append(entry("B-1546", "ctrl-fizzbuzz", "Classic FizzBuzz via conditionals",
    "Bash", "Adversarial",
    'fn fizzbuzz(n: i32) { let mut i = 1; while i <= n { if i % 15 == 0 { println!("FizzBuzz"); } else if i % 3 == 0 { println!("Fizz"); } else if i % 5 == 0 { println!("Buzz"); } else { println!("{}", i); } i += 1; } } fn main() { fizzbuzz(15); }',
    "FizzBuzz"))

bash_entries.append(entry("B-1547", "ctrl-bubble-sort", "Bubble sort implementation",
    "Bash", "Adversarial",
    'fn main() { let mut arr = [64, 34, 25, 12, 22, 11, 90]; let n = 7; let mut i = 0; while i < n - 1 { let mut j = 0; while j < n - i - 1 { if arr[j] > arr[j + 1] { let temp = arr[j]; arr[j] = arr[j + 1]; arr[j + 1] = temp; } j += 1; } i += 1; } for k in 0..7 { println!("{}", arr[k]); } }',
    "11"))

bash_entries.append(entry("B-1548", "ctrl-selection-sort", "Selection sort implementation",
    "Bash", "Adversarial",
    'fn main() { let mut arr = [29, 10, 14, 37, 13]; let n = 5; let mut i = 0; while i < n - 1 { let mut min_idx = i; let mut j = i + 1; while j < n { if arr[j] < arr[min_idx] { min_idx = j; } j += 1; } if min_idx != i { let temp = arr[i]; arr[i] = arr[min_idx]; arr[min_idx] = temp; } i += 1; } for k in 0..5 { println!("{}", arr[k]); } }',
    "10"))

# === Data structures ===
bash_entries.append(entry("B-1549", "ds-matrix-2x2", "2x2 matrix multiplication",
    "Bash", "Adversarial",
    'fn main() { let a = [1, 2, 3, 4]; let b = [5, 6, 7, 8]; let c0 = a[0] * b[0] + a[1] * b[2]; let c1 = a[0] * b[1] + a[1] * b[3]; let c2 = a[2] * b[0] + a[3] * b[2]; let c3 = a[2] * b[1] + a[3] * b[3]; println!("{} {}", c0, c1); println!("{} {}", c2, c3); }',
    "19 22"))

bash_entries.append(entry("B-1550", "ds-stack-sim", "Stack simulation via array",
    "Bash", "Adversarial",
    'fn main() { let mut stack = [0, 0, 0, 0, 0]; let mut top = 0; for v in [10, 20, 30, 40, 50] { stack[top] = v; top += 1; } while top > 0 { top -= 1; println!("{}", stack[top]); } }',
    "50"))

# === Combo algorithms ===
bash_entries.append(entry("B-1551", "combo-kadane", "Kadane max subarray sum",
    "Bash", "Adversarial",
    'fn main() { let arr = [2, -3, 4, -1, -2, 1, 5, -3]; let mut max_sum = arr[0]; let mut cur_sum = arr[0]; let mut i = 1; while i < 8 { if cur_sum + arr[i] > arr[i] { cur_sum = cur_sum + arr[i]; } else { cur_sum = arr[i]; } if cur_sum > max_sum { max_sum = cur_sum; } i += 1; } println!("max_subarray={}", max_sum); }',
    "max_subarray=7"))

bash_entries.append(entry("B-1552", "combo-two-sum", "Two-sum with nested loops",
    "Bash", "Adversarial",
    'fn main() { let arr = [2, 7, 11, 15]; let target = 9; let n = 4; let mut i = 0; while i < n { let mut j = i + 1; while j < n { if arr[i] + arr[j] == target { println!("{} {}", i, j); } j += 1; } i += 1; } }',
    "0 1"))

bash_entries.append(entry("B-1553", "combo-insertion-sort", "Insertion sort",
    "Bash", "Adversarial",
    'fn main() { let mut arr = [5, 3, 8, 1, 9, 2, 7, 4, 6, 10]; let n = 10; let mut i = 1; while i < n { let key = arr[i]; let mut j = i - 1; while j >= 0 { if arr[j] > key { arr[j + 1] = arr[j]; j -= 1; } else { break; } } arr[j + 1] = key; i += 1; } for k in 0..10 { println!("{}", arr[k]); } }',
    "1"))

bash_entries.append(entry("B-1554", "combo-binary-search-generic", "Binary search proven correct",
    "Bash", "Adversarial",
    'fn bsearch(arr: [i32; 10], target: i32, n: i32) -> i32 { let mut lo = 0; let mut hi = n - 1; while lo <= hi { let mid = lo + (hi - lo) / 2; if arr[mid] == target { return mid; } if arr[mid] < target { lo = mid + 1; } else { hi = mid - 1; } } return -1; } fn main() { let arr = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19]; println!("{}", bsearch(arr, 7, 10)); println!("{}", bsearch(arr, 12, 10)); println!("{}", bsearch(arr, 1, 10)); println!("{}", bsearch(arr, 19, 10)); }',
    "3"))

bash_entries.append(entry("B-1555", "combo-dutch-flag", "Dutch national flag partition",
    "Bash", "Adversarial",
    'fn main() { let mut arr = [2, 0, 1, 2, 1, 0, 0, 2, 1, 0]; let mut lo = 0; let mut mid = 0; let mut hi = 9; while mid <= hi { if arr[mid] == 0 { let temp = arr[lo]; arr[lo] = arr[mid]; arr[mid] = temp; lo += 1; mid += 1; } else if arr[mid] == 1 { mid += 1; } else { let temp = arr[hi]; arr[hi] = arr[mid]; arr[mid] = temp; hi -= 1; } } for k in 0..10 { println!("{}", arr[k]); } }',
    "0"))

# === Makefile entries: M-401..M-405 ===
makefile_entries = []

makefile_entries.append(entry("M-401", "make-docker-compose", "Docker compose build targets",
    "Makefile", "Adversarial",
    r'fn main() { exec("up:"); println!("\tdocker compose up -d"); exec("down:"); println!("\tdocker compose down"); exec("logs:"); println!("\tdocker compose logs -f"); exec("rebuild:"); println!("\tdocker compose build --no-cache"); println!("\tdocker compose up -d"); }',
    "docker compose up -d"))

makefile_entries.append(entry("M-402", "make-coverage-target", "Test coverage with threshold",
    "Makefile", "Adversarial",
    r'fn main() { let min_coverage = "85"; exec("coverage:"); println!("\tcargo llvm-cov --lib --fail-under-lines $(MIN_COVERAGE)"); exec("coverage-report:"); println!("\tcargo llvm-cov --lib --html"); }',
    "MIN_COVERAGE := 85"))

makefile_entries.append(entry("M-403", "make-version-git", "Version from git tag",
    "Makefile", "Adversarial",
    r'fn main() { exec("VERSION := $(shell git describe --tags --always)"); exec("build:"); println!("\tcargo build --release"); println!("\t@echo Built version $(VERSION)"); }',
    "VERSION := $(shell git describe"))

makefile_entries.append(entry("M-404", "make-cross-compile", "Cross-compilation targets",
    "Makefile", "Adversarial",
    r'fn main() { let cc = "gcc"; exec("build-linux:"); println!("\tCGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o bin/app-linux"); exec("build-darwin:"); println!("\tCGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -o bin/app-darwin"); exec("build-all: build-linux build-darwin"); }',
    "CC := gcc"))

makefile_entries.append(entry("M-405", "make-protobuf", "Protobuf code generation",
    "Makefile", "Adversarial",
    r'fn main() { exec("PROTO_DIR := proto"); exec("OUT_DIR := gen"); exec("proto:"); println!("\tprotoc --go_out=$(OUT_DIR) $(PROTO_DIR)/*.proto"); exec("proto-clean:"); println!("\trm -rf $(OUT_DIR)/*.go"); }',
    "PROTO_DIR := proto"))

# === Dockerfile entries: D-361..D-365 ===
dockerfile_entries = []

dockerfile_entries.append(entry("D-361", "docker-zig-build", "Zig cross-compilation Dockerfile",
    "Dockerfile", "Adversarial",
    r'fn builder() { from_image("ghcr.io/ziglang/zig:0.12.0"); workdir("/app"); copy(".", "."); run("zig build -Drelease-safe"); } fn main() { from_image("debian:bookworm-slim"); copy("--from=builder /app/zig-out/bin/app", "/usr/local/bin/"); cmd("app"); }',
    "FROM ghcr.io/ziglang/zig:0.12.0"))

dockerfile_entries.append(entry("D-362", "docker-elixir-release", "Elixir mix release",
    "Dockerfile", "Adversarial",
    r'fn builder() { from_image("elixir:1.16-alpine"); workdir("/app"); run("mix local.hex --force && mix local.rebar --force"); copy("mix.exs mix.lock", "."); run("mix deps.get --only prod && mix deps.compile"); copy(".", "."); run("MIX_ENV=prod mix release"); } fn main() { from_image("alpine:3.19"); run("apk add --no-cache libstdc++ openssl ncurses-libs"); copy("--from=builder /app/_build/prod/rel/myapp", "/app"); cmd("/app/bin/myapp start"); }',
    "FROM elixir:1.16-alpine"))

dockerfile_entries.append(entry("D-363", "docker-ruby-rails", "Ruby on Rails Dockerfile",
    "Dockerfile", "Adversarial",
    r'fn main() { from_image("ruby:3.3-slim"); run("apt-get update && apt-get install -y build-essential libpq-dev && rm -rf /var/lib/apt/lists/*"); workdir("/app"); copy("Gemfile Gemfile.lock", "."); run("bundle install"); copy(".", "."); expose(3000); cmd("bundle exec rails server -b 0.0.0.0"); }',
    "FROM ruby:3.3-slim"))

dockerfile_entries.append(entry("D-364", "docker-php-composer", "PHP with Composer",
    "Dockerfile", "Adversarial",
    r'fn builder() { from_image("composer:2"); workdir("/app"); copy("composer.json composer.lock", "."); run("composer install --no-dev --optimize-autoloader"); } fn main() { from_image("php:8.3-fpm-alpine"); copy("--from=builder /app/vendor", "/var/www/html/vendor"); copy(".", "/var/www/html"); expose(9000); cmd("php-fpm"); }',
    "FROM composer:2"))

dockerfile_entries.append(entry("D-365", "docker-deno-compile", "Deno compile to single binary",
    "Dockerfile", "Adversarial",
    r'fn builder() { from_image("denoland/deno:1.40"); workdir("/app"); copy(".", "."); run("deno compile --allow-net --allow-read --output server main.ts"); } fn main() { from_image("debian:bookworm-slim"); copy("--from=builder /app/server", "/usr/local/bin/"); expose(8000); cmd("server"); }',
    "FROM denoland/deno:1.40"))


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

print(generate_load_function("load_expansion40_bash", bash_str))
print(generate_load_function("load_expansion28_makefile", make_str))
print(generate_load_function("load_expansion28_dockerfile", docker_str))
