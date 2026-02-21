#!/usr/bin/env python3
"""
Generate pathological corpus entries across 23 categories (A-W).
Round 1: Moderate difficulty. Each successive round gets harder.
"""

import sys

NEXT_ID = 15850  # Starting B-ID
EXPANSION_NUM = 179  # Next expansion function number

def format_rust_string(s):
    """Format a string for inclusion in Rust r#\"...\"# raw strings."""
    # Raw strings can contain anything except the closing sequence "#
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

def gen_entries():
    entries = []
    bid = NEXT_ID

    # ============================================================
    # Category A: Shell Redirection, Pipes, and Flow
    # ============================================================

    # A-1: Function with pipe-like output chaining
    entries.append((f"B-{bid}", "pipe-chain-output", "Function chaining simulating pipe data flow",
        r'''fn transform_a(x: i64) -> i64 { return x * 2; }
fn transform_b(x: i64) -> i64 { return x + 10; }
fn transform_c(x: i64) -> i64 { return x / 3; }
fn main() {
    let input: i64 = 15;
    let stage1: i64 = transform_a(input);
    let stage2: i64 = transform_b(stage1);
    let stage3: i64 = transform_c(stage2);
    println!("pipe: {} -> {} -> {} -> {}", input, stage1, stage2, stage3);
}''', r'''transform_a()'''))
    bid += 1

    # A-2: Multi-stage filter pipeline
    entries.append((f"B-{bid}", "filter-pipeline", "Multi-stage filter with conditional pass-through",
        r'''fn is_even(n: i64) -> i64 {
    if n % 2 == 0 { return 1; }
    return 0;
}
fn is_positive(n: i64) -> i64 {
    if n > 0 { return 1; }
    return 0;
}
fn main() {
    let vals: [i64; 8] = [3, -2, 4, -7, 8, 1, -6, 10];
    let mut count: i64 = 0;
    let mut i: i64 = 0;
    while i < 8 {
        if is_even(vals[i]) == 1 {
            if is_positive(vals[i]) == 1 {
                println!("pass: {}", vals[i]);
                count = count + 1;
            }
        }
        i = i + 1;
    }
    println!("total_pass={}", count);
}''', r'''is_even()'''))
    bid += 1

    # A-3: Redirect simulation - output to multiple channels
    entries.append((f"B-{bid}", "multi-channel-output", "Simulate stdout/stderr redirection with tagged output",
        r'''fn log_info(msg: &str, val: i64) {
    println!("[INFO] {} {}", msg, val);
}
fn log_error(msg: &str, code: i64) {
    println!("[ERROR] {} code={}", msg, code);
}
fn process(x: i64) -> i64 {
    if x < 0 {
        log_error("negative input", x);
        return -1;
    }
    log_info("processed", x * 2);
    return x * 2;
}
fn main() {
    let r1: i64 = process(5);
    let r2: i64 = process(-3);
    let r3: i64 = process(10);
    println!("results: {} {} {}", r1, r2, r3);
}''', r'''log_info()'''))
    bid += 1

    # A-4: Tee-like operation - process and accumulate
    entries.append((f"B-{bid}", "tee-accumulate", "Tee-like pattern: process value and accumulate running total",
        r'''fn main() {
    let data: [i64; 6] = [10, 20, 30, 40, 50, 60];
    let mut running_sum: i64 = 0;
    let mut max_val: i64 = 0;
    let mut i: i64 = 0;
    while i < 6 {
        running_sum = running_sum + data[i];
        if data[i] > max_val {
            max_val = data[i];
        }
        println!("item={} sum={} max={}", data[i], running_sum, max_val);
        i = i + 1;
    }
    println!("final sum={} max={}", running_sum, max_val);
}''', r'''final sum='''))
    bid += 1

    # A-5: Flow control with early exit simulation
    entries.append((f"B-{bid}", "early-exit-flow", "Pipeline with early termination on error condition",
        r'''fn validate(x: i64) -> i64 {
    if x < 0 { return -1; }
    if x > 1000 { return -2; }
    return 0;
}
fn transform(x: i64) -> i64 { return x * x; }
fn main() {
    let inputs: [i64; 5] = [5, -1, 100, 2000, 42];
    let mut i: i64 = 0;
    while i < 5 {
        let err: i64 = validate(inputs[i]);
        if err != 0 {
            println!("skip {} err={}", inputs[i], err);
        } else {
            let result: i64 = transform(inputs[i]);
            println!("ok {} -> {}", inputs[i], result);
        }
        i = i + 1;
    }
}''', r'''validate()'''))
    bid += 1

    # ============================================================
    # Category B: Pathological Quoting (Shell + Makefile contexts)
    # ============================================================

    # B-1: Strings with special chars
    entries.append((f"B-{bid}", "quote-special-chars", "Strings containing shell metacharacters",
        r'''fn main() {
    let msg1: &str = "hello world";
    let msg2: &str = "price is $5";
    let msg3: &str = "yes & no";
    let msg4: &str = "pipe | char";
    println!("{}", msg1);
    println!("{}", msg2);
    println!("{}", msg3);
    println!("{}", msg4);
}''', r'''hello world'''))
    bid += 1

    # B-2: Nested quoting in function args
    entries.append((f"B-{bid}", "quote-nested-args", "Function arguments with quoted special strings",
        r'''fn describe(name: &str, count: i64) {
    println!("item: {} x{}", name, count);
}
fn main() {
    describe("widget-a", 3);
    describe("bolt #5", 10);
    describe("size=large", 1);
    describe("type:hex", 7);
}''', r'''describe()'''))
    bid += 1

    # B-3: String comparison edge cases
    entries.append((f"B-{bid}", "quote-string-compare", "String equality with special characters",
        r'''fn check(s: &str, expected: &str) -> i64 {
    if s == expected { return 1; }
    return 0;
}
fn main() {
    let r1: i64 = check("abc", "abc");
    let r2: i64 = check("a b", "a b");
    let r3: i64 = check("", "");
    let r4: i64 = check("x", "y");
    println!("eq={} sp={} empty={} neq={}", r1, r2, r3, r4);
}''', r'''check()'''))
    bid += 1

    # ============================================================
    # Category C: Pathological One-Liners
    # ============================================================

    # C-1: Dense arithmetic chain
    entries.append((f"B-{bid}", "oneliner-arith-chain", "Dense arithmetic expression chain in single expression",
        r'''fn main() {
    let x: i64 = 2 + 3 * 4 - 1;
    let y: i64 = x * x + x - x / 2;
    let z: i64 = y % 7 + y / 3 - 1;
    println!("x={} y={} z={}", x, y, z);
}''', r'''x='''))
    bid += 1

    # C-2: Deeply nested ternary-like
    entries.append((f"B-{bid}", "oneliner-nested-cond", "Deeply nested conditional one-liner simulation",
        r'''fn classify(x: i64) -> i64 {
    if x > 100 { return 4; }
    if x > 50 { return 3; }
    if x > 10 { return 2; }
    if x > 0 { return 1; }
    return 0;
}
fn main() {
    println!("{} {} {} {} {}", classify(200), classify(75), classify(25), classify(5), classify(-1));
}''', r'''classify()'''))
    bid += 1

    # C-3: Single-line multi-operation
    entries.append((f"B-{bid}", "oneliner-multi-op", "Multiple operations compressed into minimal code",
        r'''fn f(a: i64, b: i64, c: i64) -> i64 { return a * b + c; }
fn g(x: i64) -> i64 { return x * x - 1; }
fn main() {
    let r: i64 = f(g(2), g(3), g(4));
    println!("result={}", r);
}''', r'''result='''))
    bid += 1

    # ============================================================
    # Category D: Pathological Glob and Wildcards
    # ============================================================

    # D-1: Pattern matching simulation
    entries.append((f"B-{bid}", "glob-pattern-match", "Pattern matching with wildcard-like character checks",
        r'''fn starts_with_a(c: i64) -> i64 {
    if c == 97 { return 1; }
    return 0;
}
fn is_digit(c: i64) -> i64 {
    if c >= 48 {
        if c <= 57 { return 1; }
    }
    return 0;
}
fn main() {
    let chars: [i64; 6] = [97, 98, 48, 57, 65, 49];
    let mut i: i64 = 0;
    while i < 6 {
        let sa: i64 = starts_with_a(chars[i]);
        let dig: i64 = is_digit(chars[i]);
        println!("char={} is_a={} is_digit={}", chars[i], sa, dig);
        i = i + 1;
    }
}''', r'''starts_with_a()'''))
    bid += 1

    # D-2: Path component matching
    entries.append((f"B-{bid}", "glob-path-components", "Path component separator counting and extraction",
        r'''fn count_separators(path: [i64; 10], len: i64, sep: i64) -> i64 {
    let mut count: i64 = 0;
    let mut i: i64 = 0;
    while i < len {
        if path[i] == sep {
            count = count + 1;
        }
        i = i + 1;
    }
    return count;
}
fn main() {
    let path: [i64; 10] = [47, 104, 47, 115, 47, 102, 0, 0, 0, 0];
    let depth: i64 = count_separators(path, 6, 47);
    println!("depth={}", depth);
}''', r'''count_separators()'''))
    bid += 1

    # ============================================================
    # Category E: Pathological Heredoc simulation
    # ============================================================

    # E-1: Multi-line string builder
    entries.append((f"B-{bid}", "heredoc-multiline-build", "Multi-line string construction simulating heredoc",
        r'''fn main() {
    println!("line 1: header");
    println!("line 2: body starts");
    println!("line 3: data=42");
    println!("line 4: body ends");
    println!("line 5: footer");
}''', r'''line 1: header'''))
    bid += 1

    # E-2: Template with variable interpolation
    entries.append((f"B-{bid}", "heredoc-template-vars", "Template-style output with variable substitution",
        r'''fn emit_config(host: &str, port: i64, workers: i64) {
    println!("server:");
    println!("  host: {}", host);
    println!("  port: {}", port);
    println!("  workers: {}", workers);
    println!("  timeout: 30");
}
fn main() {
    emit_config("localhost", 8080, 4);
}''', r'''emit_config()'''))
    bid += 1

    # ============================================================
    # Category F: Pathological Environment Variables
    # ============================================================

    # F-1: Environment defaults simulation
    entries.append((f"B-{bid}", "env-defaults-sim", "Simulated environment variable with default fallback",
        r'''fn get_or_default(val: i64, default_val: i64) -> i64 {
    if val == 0 { return default_val; }
    return val;
}
fn main() {
    let home: i64 = 0;
    let path: i64 = 100;
    let shell: i64 = 0;
    let h: i64 = get_or_default(home, 42);
    let p: i64 = get_or_default(path, 42);
    let s: i64 = get_or_default(shell, 99);
    println!("HOME={} PATH={} SHELL={}", h, p, s);
}''', r'''get_or_default()'''))
    bid += 1

    # F-2: Environment variable chaining
    entries.append((f"B-{bid}", "env-chain-expand", "Chained variable expansion with cascading defaults",
        r'''fn resolve(primary: i64, fallback1: i64, fallback2: i64) -> i64 {
    if primary != 0 { return primary; }
    if fallback1 != 0 { return fallback1; }
    return fallback2;
}
fn main() {
    let r1: i64 = resolve(0, 0, 99);
    let r2: i64 = resolve(0, 50, 99);
    let r3: i64 = resolve(10, 50, 99);
    println!("r1={} r2={} r3={}", r1, r2, r3);
}''', r'''resolve()'''))
    bid += 1

    # ============================================================
    # Category G: SSH Operations simulation
    # ============================================================

    # G-1: SSH-like command execution
    entries.append((f"B-{bid}", "ssh-remote-exec-sim", "Simulate remote command execution with error codes",
        r'''fn ssh_exec(host: i64, cmd: i64) -> i64 {
    if host == 0 { return -1; }
    if cmd == 0 { return -2; }
    return host + cmd;
}
fn main() {
    let r1: i64 = ssh_exec(1, 10);
    let r2: i64 = ssh_exec(0, 10);
    let r3: i64 = ssh_exec(2, 0);
    let r4: i64 = ssh_exec(3, 20);
    println!("exec: {} {} {} {}", r1, r2, r3, r4);
}''', r'''ssh_exec()'''))
    bid += 1

    # G-2: SSH key fingerprint computation
    entries.append((f"B-{bid}", "ssh-fingerprint-sim", "SSH key fingerprint hash simulation",
        r'''fn hash_step(h: i64, byte: i64) -> i64 {
    return ((h * 31) + byte) % 65536;
}
fn fingerprint(key: [i64; 8], len: i64) -> i64 {
    let mut h: i64 = 0;
    let mut i: i64 = 0;
    while i < len {
        h = hash_step(h, key[i]);
        i = i + 1;
    }
    return h;
}
fn main() {
    let key1: [i64; 8] = [65, 66, 67, 68, 69, 70, 71, 72];
    let key2: [i64; 8] = [72, 71, 70, 69, 68, 67, 66, 65];
    let fp1: i64 = fingerprint(key1, 8);
    let fp2: i64 = fingerprint(key2, 8);
    println!("fp1={} fp2={} same={}", fp1, fp2, if fp1 == fp2 { 1 } else { 0 });
}''', r'''hash_step()'''))
    bid += 1

    # ============================================================
    # Category H: Pathological Printing
    # ============================================================

    # H-1: Complex format string patterns
    entries.append((f"B-{bid}", "print-complex-format", "Multiple format specifiers and mixed types",
        r'''fn main() {
    let a: i64 = 42;
    let b: i64 = -7;
    let s: &str = "hello";
    println!("a={} b={} s={} sum={}", a, b, s, a + b);
    println!("[{} | {} | {}]", a, b, s);
    println!("({},{},{})", a, b, a * b);
}''', r'''a=42'''))
    bid += 1

    # H-2: Aligned table output
    entries.append((f"B-{bid}", "print-table-aligned", "Table-style aligned output with headers and rows",
        r'''fn print_row(name: &str, qty: i64, price: i64) {
    println!("{} {} {}", name, qty, price);
}
fn main() {
    println!("ITEM QTY PRICE");
    println!("---- --- -----");
    print_row("bolt", 100, 5);
    print_row("nut", 200, 3);
    print_row("screw", 50, 8);
    let total: i64 = 100 * 5 + 200 * 3 + 50 * 8;
    println!("TOTAL: {}", total);
}''', r'''print_row()'''))
    bid += 1

    # ============================================================
    # Category I: Pathological Awk/Sed/Grep/Tr simulation
    # ============================================================

    # I-1: Field extraction (awk-like)
    entries.append((f"B-{bid}", "awk-field-extract", "Awk-like field extraction from structured data",
        r'''fn extract_field(record: [i64; 5], field: i64) -> i64 {
    return record[field];
}
fn main() {
    let r1: [i64; 5] = [10, 20, 30, 40, 50];
    let r2: [i64; 5] = [11, 22, 33, 44, 55];
    let r3: [i64; 5] = [99, 88, 77, 66, 55];
    println!("$1: {} {} {}", extract_field(r1, 0), extract_field(r2, 0), extract_field(r3, 0));
    println!("$3: {} {} {}", extract_field(r1, 2), extract_field(r2, 2), extract_field(r3, 2));
    println!("$5: {} {} {}", extract_field(r1, 4), extract_field(r2, 4), extract_field(r3, 4));
}''', r'''extract_field()'''))
    bid += 1

    # I-2: Character substitution (tr-like)
    entries.append((f"B-{bid}", "tr-char-substitute", "Character-by-character substitution like tr",
        r'''fn tr_char(c: i64, from: i64, to: i64) -> i64 {
    if c == from { return to; }
    return c;
}
fn main() {
    let input: [i64; 5] = [97, 98, 97, 99, 97];
    let mut i: i64 = 0;
    while i < 5 {
        let out: i64 = tr_char(input[i], 97, 120);
        println!("tr: {} -> {}", input[i], out);
        i = i + 1;
    }
}''', r'''tr_char()'''))
    bid += 1

    # I-3: Line grep simulation
    entries.append((f"B-{bid}", "grep-line-filter", "Grep-like line filtering by pattern match",
        r'''fn matches(line_tag: i64, pattern: i64) -> i64 {
    if line_tag == pattern { return 1; }
    return 0;
}
fn main() {
    let tags: [i64; 6] = [1, 2, 1, 3, 1, 2];
    let pattern: i64 = 1;
    let mut matched: i64 = 0;
    let mut i: i64 = 0;
    while i < 6 {
        if matches(tags[i], pattern) == 1 {
            println!("match at line {}", i);
            matched = matched + 1;
        }
        i = i + 1;
    }
    println!("total matches: {}", matched);
}''', r'''matches()'''))
    bid += 1

    # ============================================================
    # Category J: Pathological Data Structures
    # ============================================================

    # J-1: Stack with array
    entries.append((f"B-{bid}", "ds-stack-array", "Stack implementation using fixed array",
        r'''fn push(stack: [i64; 8], top: i64, val: i64) -> i64 {
    return top + 1;
}
fn peek(stack: [i64; 8], top: i64) -> i64 {
    return stack[top];
}
fn main() {
    let mut stack: [i64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut top: i64 = -1;
    stack[0] = 10;
    top = 0;
    stack[1] = 20;
    top = 1;
    stack[2] = 30;
    top = 2;
    println!("top={} peek={}", top, peek(stack, top));
    top = top - 1;
    println!("top={} peek={}", top, peek(stack, top));
}''', r'''push()'''))
    bid += 1

    # J-2: Queue circular buffer
    entries.append((f"B-{bid}", "ds-queue-circular", "Circular buffer queue simulation",
        r'''fn enqueue_idx(rear: i64, cap: i64) -> i64 {
    return (rear + 1) % cap;
}
fn dequeue_idx(front: i64, cap: i64) -> i64 {
    return (front + 1) % cap;
}
fn main() {
    let cap: i64 = 4;
    let mut front: i64 = 0;
    let mut rear: i64 = 0;
    let mut buf: [i64; 4] = [0, 0, 0, 0];
    buf[rear] = 10;
    rear = enqueue_idx(rear, cap);
    buf[rear] = 20;
    rear = enqueue_idx(rear, cap);
    buf[rear] = 30;
    rear = enqueue_idx(rear, cap);
    let v1: i64 = buf[front];
    front = dequeue_idx(front, cap);
    let v2: i64 = buf[front];
    front = dequeue_idx(front, cap);
    println!("deq1={} deq2={} rear={} front={}", v1, v2, rear, front);
}''', r'''enqueue_idx()'''))
    bid += 1

    # J-3: Hash table (open addressing)
    entries.append((f"B-{bid}", "ds-hash-open-addr", "Hash table with open addressing probe",
        r'''fn hash_key(key: i64, capacity: i64) -> i64 {
    let h: i64 = key % capacity;
    if h < 0 { return h + capacity; }
    return h;
}
fn probe(start: i64, attempt: i64, capacity: i64) -> i64 {
    return (start + attempt) % capacity;
}
fn main() {
    let cap: i64 = 8;
    let keys: [i64; 5] = [10, 18, 26, 3, 11];
    let mut i: i64 = 0;
    while i < 5 {
        let h: i64 = hash_key(keys[i], cap);
        let p1: i64 = probe(h, 1, cap);
        let p2: i64 = probe(h, 2, cap);
        println!("key={} hash={} p1={} p2={}", keys[i], h, p1, p2);
        i = i + 1;
    }
}''', r'''hash_key()'''))
    bid += 1

    # ============================================================
    # Category K: Pathological Sourcing simulation
    # ============================================================

    # K-1: Module-like function imports
    entries.append((f"B-{bid}", "source-module-funcs", "Multi-module function call pattern simulating sourcing",
        r'''fn math_add(a: i64, b: i64) -> i64 { return a + b; }
fn math_mul(a: i64, b: i64) -> i64 { return a * b; }
fn str_len(s: &str) -> i64 { return 5; }
fn util_max(a: i64, b: i64) -> i64 {
    if a > b { return a; }
    return b;
}
fn main() {
    let sum: i64 = math_add(10, 20);
    let prod: i64 = math_mul(3, 7);
    let mx: i64 = util_max(sum, prod);
    println!("sum={} prod={} max={}", sum, prod, mx);
}''', r'''math_add()'''))
    bid += 1

    # K-2: Config file simulation
    entries.append((f"B-{bid}", "source-config-vars", "Simulated config file sourcing with overridable variables",
        r'''fn config_default_port() -> i64 { return 8080; }
fn config_default_workers() -> i64 { return 4; }
fn config_default_timeout() -> i64 { return 30; }
fn override_if_set(current: i64, override_val: i64) -> i64 {
    if override_val != 0 { return override_val; }
    return current;
}
fn main() {
    let port: i64 = override_if_set(config_default_port(), 9090);
    let workers: i64 = override_if_set(config_default_workers(), 0);
    let timeout: i64 = override_if_set(config_default_timeout(), 60);
    println!("port={} workers={} timeout={}", port, workers, timeout);
}''', r'''config_default_port()'''))
    bid += 1

    # ============================================================
    # Category L: Pathological Scripts (complex multi-function)
    # ============================================================

    # L-1: Installation script simulation
    entries.append((f"B-{bid}", "script-install-sim", "Installation script with dependency checks and rollback",
        r'''fn check_dep(dep_id: i64) -> i64 {
    if dep_id == 1 { return 1; }
    if dep_id == 2 { return 1; }
    if dep_id == 3 { return 0; }
    return 0;
}
fn install_pkg(pkg_id: i64) -> i64 {
    if check_dep(pkg_id) == 0 {
        println!("FAIL: dep {} missing", pkg_id);
        return -1;
    }
    println!("OK: installed {}", pkg_id);
    return 0;
}
fn main() {
    let r1: i64 = install_pkg(1);
    let r2: i64 = install_pkg(2);
    let r3: i64 = install_pkg(3);
    let success: i64 = if r1 == 0 { if r2 == 0 { if r3 == 0 { 1 } else { 0 } } else { 0 } } else { 0 };
    println!("all_ok={}", success);
}''', r'''check_dep()'''))
    bid += 1

    # ============================================================
    # Category M: Pathological Braces, Semicolons, Control Flow
    # ============================================================

    # M-1: Deeply nested if-else
    entries.append((f"B-{bid}", "brace-deep-if-else", "Deeply nested if-else with multiple brace levels",
        r'''fn classify_deep(x: i64, y: i64, z: i64) -> i64 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                return 1;
            } else {
                return 2;
            }
        } else {
            if z > 0 {
                return 3;
            } else {
                return 4;
            }
        }
    } else {
        if y > 0 {
            if z > 0 {
                return 5;
            } else {
                return 6;
            }
        } else {
            return 7;
        }
    }
}
fn main() {
    println!("{} {} {} {} {} {} {}", classify_deep(1, 1, 1), classify_deep(1, 1, -1), classify_deep(1, -1, 1), classify_deep(1, -1, -1), classify_deep(-1, 1, 1), classify_deep(-1, 1, -1), classify_deep(-1, -1, -1));
}''', r'''classify_deep()'''))
    bid += 1

    # M-2: Loop with multiple break/continue conditions
    entries.append((f"B-{bid}", "brace-break-continue", "Complex loop with conditional break and continue",
        r'''fn main() {
    let mut total: i64 = 0;
    let mut skipped: i64 = 0;
    let mut i: i64 = 0;
    while i < 20 {
        if i % 3 == 0 {
            skipped = skipped + 1;
            i = i + 1;
            continue;
        }
        if i > 15 {
            break;
        }
        total = total + i;
        i = i + 1;
    }
    println!("total={} skipped={} i={}", total, skipped, i);
}''', r'''total='''))
    bid += 1

    # M-3: Nested loops with labeled-like control
    entries.append((f"B-{bid}", "brace-nested-loop-ctrl", "Nested loops with outer loop control variable",
        r'''fn main() {
    let mut found_i: i64 = -1;
    let mut found_j: i64 = -1;
    let mut done: i64 = 0;
    let mut i: i64 = 0;
    while i < 5 {
        if done == 1 { break; }
        let mut j: i64 = 0;
        while j < 5 {
            if i * 5 + j == 13 {
                found_i = i;
                found_j = j;
                done = 1;
                break;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    println!("found at ({}, {})", found_i, found_j);
}''', r'''found at'''))
    bid += 1

    # ============================================================
    # Category N: Pathological Traps simulation
    # ============================================================

    # N-1: Cleanup handler pattern
    entries.append((f"B-{bid}", "trap-cleanup-sim", "Trap-like cleanup handler simulation",
        r'''fn create_temp() -> i64 {
    println!("created temp file");
    return 1;
}
fn cleanup(temp_id: i64) {
    println!("cleanup: removing temp {}", temp_id);
}
fn do_work(temp_id: i64) -> i64 {
    if temp_id == 0 { return -1; }
    println!("working with temp {}", temp_id);
    return 0;
}
fn main() {
    let tmp: i64 = create_temp();
    let result: i64 = do_work(tmp);
    cleanup(tmp);
    println!("exit code: {}", result);
}''', r'''create_temp()'''))
    bid += 1

    # N-2: Signal handler simulation
    entries.append((f"B-{bid}", "trap-signal-handler", "Signal handler dispatch table simulation",
        r'''fn handle_signal(sig: i64) -> i64 {
    if sig == 1 { println!("HUP: reload"); return 0; }
    if sig == 2 { println!("INT: interrupt"); return 1; }
    if sig == 15 { println!("TERM: terminate"); return 1; }
    if sig == 9 { println!("KILL: force"); return 2; }
    println!("SIG{}: unknown", sig);
    return -1;
}
fn main() {
    let r1: i64 = handle_signal(1);
    let r2: i64 = handle_signal(2);
    let r3: i64 = handle_signal(15);
    let r4: i64 = handle_signal(99);
    println!("results: {} {} {} {}", r1, r2, r3, r4);
}''', r'''handle_signal()'''))
    bid += 1

    # ============================================================
    # Category O: Pathological Command Line Parsing
    # ============================================================

    # O-1: Option parser
    entries.append((f"B-{bid}", "cli-option-parser", "Command-line option parsing with flags and values",
        r'''fn parse_flag(flag: i64) -> i64 {
    if flag == 1 { return 1; }
    return 0;
}
fn parse_value(val: i64, default: i64) -> i64 {
    if val != 0 { return val; }
    return default;
}
fn main() {
    let verbose: i64 = parse_flag(1);
    let debug: i64 = parse_flag(0);
    let port: i64 = parse_value(8080, 3000);
    let workers: i64 = parse_value(0, 4);
    println!("verbose={} debug={} port={} workers={}", verbose, debug, port, workers);
}''', r'''parse_flag()'''))
    bid += 1

    # O-2: Subcommand dispatch
    entries.append((f"B-{bid}", "cli-subcommand-dispatch", "CLI subcommand dispatch with argument validation",
        r'''fn cmd_build(opt: i64) -> i64 {
    println!("build: opt={}", opt);
    return 0;
}
fn cmd_test(opt: i64) -> i64 {
    println!("test: opt={}", opt);
    return 0;
}
fn cmd_deploy(opt: i64) -> i64 {
    println!("deploy: opt={}", opt);
    return 0;
}
fn dispatch(cmd: i64, opt: i64) -> i64 {
    if cmd == 1 { return cmd_build(opt); }
    if cmd == 2 { return cmd_test(opt); }
    if cmd == 3 { return cmd_deploy(opt); }
    println!("unknown command: {}", cmd);
    return -1;
}
fn main() {
    let r1: i64 = dispatch(1, 42);
    let r2: i64 = dispatch(2, 0);
    let r3: i64 = dispatch(99, 0);
    println!("codes: {} {} {}", r1, r2, r3);
}''', r'''cmd_build()'''))
    bid += 1

    # ============================================================
    # Category P: Pathological Makefiles
    # ============================================================
    # (These go in as Makefile format entries)

    # ============================================================
    # Category Q: Numerical Methods with Edge Cases
    # ============================================================

    # Q-1: Newton's method with edge cases
    entries.append((f"B-{bid}", "num-newton-edge", "Newton's method with zero derivative protection",
        r'''fn newton_step(x: i64, fx: i64, dfx: i64) -> i64 {
    if dfx == 0 { return x; }
    return x - fx / dfx;
}
fn f_val(x: i64) -> i64 { return x * x - 4; }
fn df_val(x: i64) -> i64 { return 2 * x; }
fn main() {
    let mut x: i64 = 10;
    let mut i: i64 = 0;
    while i < 5 {
        let fx: i64 = f_val(x);
        let dfx: i64 = df_val(x);
        x = newton_step(x, fx, dfx);
        println!("iter={} x={} f(x)={}", i, x, f_val(x));
        i = i + 1;
    }
}''', r'''newton_step()'''))
    bid += 1

    # Q-2: Fixed-point arithmetic
    entries.append((f"B-{bid}", "num-fixed-point", "Fixed-point arithmetic with scaling",
        r'''fn fp_mul(a: i64, b: i64, scale: i64) -> i64 {
    return (a * b) / scale;
}
fn fp_div(a: i64, b: i64, scale: i64) -> i64 {
    if b == 0 { return 0; }
    return (a * scale) / b;
}
fn fp_add(a: i64, b: i64) -> i64 { return a + b; }
fn main() {
    let scale: i64 = 1000;
    let a: i64 = 1500;
    let b: i64 = 2500;
    let product: i64 = fp_mul(a, b, scale);
    let quotient: i64 = fp_div(a, b, scale);
    let sum: i64 = fp_add(a, b);
    println!("a={} b={} mul={} div={} sum={}", a, b, product, quotient, sum);
}''', r'''fp_mul()'''))
    bid += 1

    # ============================================================
    # Category R: Pathological Symbolic Bash
    # ============================================================

    # R-1: Boolean operator chains
    entries.append((f"B-{bid}", "symbolic-bool-chain", "Boolean AND/OR chain evaluation",
        r'''fn and_op(a: i64, b: i64) -> i64 {
    if a != 0 {
        if b != 0 { return 1; }
    }
    return 0;
}
fn or_op(a: i64, b: i64) -> i64 {
    if a != 0 { return 1; }
    if b != 0 { return 1; }
    return 0;
}
fn main() {
    let t: i64 = 1;
    let f: i64 = 0;
    println!("T&&T={} T&&F={} F&&T={} F&&F={}", and_op(t, t), and_op(t, f), and_op(f, t), and_op(f, f));
    println!("T||T={} T||F={} F||T={} F||F={}", or_op(t, t), or_op(t, f), or_op(f, t), or_op(f, f));
}''', r'''and_op()'''))
    bid += 1

    # R-2: Dollar-sign expansion simulation
    entries.append((f"B-{bid}", "symbolic-dollar-expand", "Variable expansion with dollar-like indirect access",
        r'''fn indirect(vars: [i64; 5], idx: i64) -> i64 {
    if idx < 0 { return -1; }
    if idx >= 5 { return -1; }
    return vars[idx];
}
fn main() {
    let vars: [i64; 5] = [100, 200, 300, 400, 500];
    let mut i: i64 = 0;
    while i < 5 {
        let val: i64 = indirect(vars, i);
        println!("${} = {}", i, val);
        i = i + 1;
    }
    let bad: i64 = indirect(vars, 99);
    println!("$99 = {}", bad);
}''', r'''indirect()'''))
    bid += 1

    # ============================================================
    # Category S: Editor Commands simulation
    # ============================================================

    # S-1: Buffer operations (vi-like)
    entries.append((f"B-{bid}", "editor-buffer-ops", "Text buffer insert/delete operations (vi-like)",
        r'''fn buf_insert(buf: [i64; 8], pos: i64, val: i64, len: i64) -> i64 {
    return len + 1;
}
fn buf_delete(len: i64, pos: i64) -> i64 {
    if pos >= len { return len; }
    return len - 1;
}
fn main() {
    let mut len: i64 = 0;
    let buf: [i64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    len = buf_insert(buf, 0, 65, len);
    len = buf_insert(buf, 1, 66, len);
    len = buf_insert(buf, 2, 67, len);
    println!("after insert: len={}", len);
    len = buf_delete(len, 1);
    println!("after delete: len={}", len);
    len = buf_delete(len, 99);
    println!("after bad delete: len={}", len);
}''', r'''buf_insert()'''))
    bid += 1

    # ============================================================
    # Category T: Functions, Closures, Functional Programming
    # ============================================================

    # T-1: Higher-order function simulation
    entries.append((f"B-{bid}", "fp-higher-order", "Higher-order function pattern with function pointers",
        r'''fn apply_twice(x: i64, offset: i64) -> i64 {
    let first: i64 = x + offset;
    let second: i64 = first + offset;
    return second;
}
fn square_then_add(x: i64, addend: i64) -> i64 {
    return x * x + addend;
}
fn main() {
    let r1: i64 = apply_twice(10, 5);
    let r2: i64 = apply_twice(0, 100);
    let r3: i64 = square_then_add(5, 10);
    let r4: i64 = square_then_add(3, square_then_add(2, 1));
    println!("twice: {} {} sq: {} nested: {}", r1, r2, r3, r4);
}''', r'''apply_twice()'''))
    bid += 1

    # T-2: Map/filter/reduce simulation
    entries.append((f"B-{bid}", "fp-map-filter-reduce", "Map-filter-reduce pipeline over arrays",
        r'''fn map_double(x: i64) -> i64 { return x * 2; }
fn filter_gt5(x: i64) -> i64 {
    if x > 5 { return 1; }
    return 0;
}
fn main() {
    let data: [i64; 6] = [1, 3, 5, 7, 9, 2];
    let mut sum: i64 = 0;
    let mut count: i64 = 0;
    let mut i: i64 = 0;
    while i < 6 {
        let mapped: i64 = map_double(data[i]);
        if filter_gt5(mapped) == 1 {
            sum = sum + mapped;
            count = count + 1;
        }
        i = i + 1;
    }
    println!("sum={} count={}", sum, count);
}''', r'''map_double()'''))
    bid += 1

    # T-3: Fold/accumulate
    entries.append((f"B-{bid}", "fp-fold-accumulate", "Fold/accumulate pattern with custom combiner",
        r'''fn combine(acc: i64, x: i64) -> i64 {
    return acc * 10 + x;
}
fn main() {
    let digits: [i64; 5] = [1, 2, 3, 4, 5];
    let mut acc: i64 = 0;
    let mut i: i64 = 0;
    while i < 5 {
        acc = combine(acc, digits[i]);
        println!("step {}: acc={}", i, acc);
        i = i + 1;
    }
    println!("final={}", acc);
}''', r'''combine()'''))
    bid += 1

    # ============================================================
    # Category U: Provable Code (Miri-safe patterns)
    # ============================================================

    # U-1: Bounded loop provability
    entries.append((f"B-{bid}", "prove-bounded-loop", "Provably terminating bounded loop with invariant",
        r'''fn sum_range(start: i64, end: i64) -> i64 {
    let mut total: i64 = 0;
    let mut i: i64 = start;
    while i < end {
        total = total + i;
        i = i + 1;
    }
    return total;
}
fn main() {
    let s1: i64 = sum_range(1, 11);
    let s2: i64 = sum_range(0, 0);
    let s3: i64 = sum_range(5, 8);
    println!("sum(1..11)={} sum(0..0)={} sum(5..8)={}", s1, s2, s3);
}''', r'''sum_range()'''))
    bid += 1

    # U-2: Overflow-safe arithmetic
    entries.append((f"B-{bid}", "prove-overflow-safe", "Arithmetic with explicit overflow checks",
        r'''fn safe_add(a: i64, b: i64, max_val: i64) -> i64 {
    if a > max_val - b { return max_val; }
    return a + b;
}
fn safe_mul(a: i64, b: i64, max_val: i64) -> i64 {
    if b == 0 { return 0; }
    if a > max_val / b { return max_val; }
    return a * b;
}
fn main() {
    let max_val: i64 = 1000;
    println!("add: {} {} {}", safe_add(100, 200, max_val), safe_add(900, 200, max_val), safe_add(500, 500, max_val));
    println!("mul: {} {} {}", safe_mul(10, 20, max_val), safe_mul(100, 100, max_val), safe_mul(0, 999, max_val));
}''', r'''safe_add()'''))
    bid += 1

    # ============================================================
    # Category V: Clippy-Pedantic Patterns
    # ============================================================

    # V-1: Explicit type conversions
    entries.append((f"B-{bid}", "clippy-explicit-conv", "Explicit type handling avoiding implicit conversions",
        r'''fn i64_to_bool(x: i64) -> i64 {
    if x != 0 { return 1; }
    return 0;
}
fn bool_to_i64(b: i64) -> i64 { return b; }
fn clamp(x: i64, low: i64, high: i64) -> i64 {
    if x < low { return low; }
    if x > high { return high; }
    return x;
}
fn main() {
    println!("bool: {} {} {}", i64_to_bool(0), i64_to_bool(1), i64_to_bool(-5));
    println!("clamp: {} {} {}", clamp(5, 0, 10), clamp(-5, 0, 10), clamp(15, 0, 10));
}''', r'''i64_to_bool()'''))
    bid += 1

    # ============================================================
    # Category W: C Mixed with Bash patterns
    # ============================================================

    # W-1: C-like memory operations
    entries.append((f"B-{bid}", "c-style-memops", "C-style memory operations: alloc, copy, compare",
        r'''fn memset(buf: [i64; 8], val: i64, n: i64) -> i64 {
    return val;
}
fn memcmp(a: [i64; 8], b: [i64; 8], n: i64) -> i64 {
    let mut i: i64 = 0;
    while i < n {
        if a[i] != b[i] { return a[i] - b[i]; }
        i = i + 1;
    }
    return 0;
}
fn main() {
    let a: [i64; 8] = [1, 2, 3, 4, 0, 0, 0, 0];
    let b: [i64; 8] = [1, 2, 3, 4, 0, 0, 0, 0];
    let c: [i64; 8] = [1, 2, 3, 5, 0, 0, 0, 0];
    let eq_ab: i64 = memcmp(a, b, 4);
    let eq_ac: i64 = memcmp(a, c, 4);
    println!("a==b: {} a==c: {}", eq_ab, eq_ac);
}''', r'''memcmp()'''))
    bid += 1

    # W-2: C-style string operations (null-terminated arrays)
    entries.append((f"B-{bid}", "c-style-strlen", "C-style strlen on null-terminated integer array",
        r'''fn strlen(s: [i64; 8], max: i64) -> i64 {
    let mut len: i64 = 0;
    while len < max {
        if s[len] == 0 { return len; }
        len = len + 1;
    }
    return len;
}
fn main() {
    let s1: [i64; 8] = [72, 101, 108, 108, 111, 0, 0, 0];
    let s2: [i64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let s3: [i64; 8] = [65, 66, 67, 68, 69, 70, 71, 72];
    println!("len1={} len2={} len3={}", strlen(s1, 8), strlen(s2, 8), strlen(s3, 8));
}''', r'''strlen()'''))
    bid += 1

    return entries, bid

def gen_makefile_entries(start_bid):
    """Generate Makefile corpus entries for pathological quoting and nested includes."""
    entries = []
    bid = start_bid

    # P-O1: Pathological Makefile quoting
    entries.append((f"M-{bid}", "make-pathological-quote", "Makefile with complex variable quoting and shell escapes",
        "Makefile",
        '''CC := gcc
CFLAGS := -Wall -Wextra -O2
SOURCES := main.c utils.c
OBJECTS := $(SOURCES:.c=.o)

.PHONY: all clean

all: $(OBJECTS)
\t@echo "Building with: $(CC) $(CFLAGS)"
\t$(CC) $(CFLAGS) -o app $(OBJECTS)

%.o: %.c
\t$(CC) $(CFLAGS) -c $< -o $@

clean:
\t@echo "Cleaning build artifacts"
\trm -f $(OBJECTS) app''',
        '''CC :='''))
    bid += 1

    # P-O2: Nested variable expansion
    entries.append((f"M-{bid}", "make-nested-var-expand", "Makefile with nested variable references and recursive expansion",
        "Makefile",
        '''PREFIX := /usr/local
BINDIR := $(PREFIX)/bin
LIBDIR := $(PREFIX)/lib
MANDIR := $(PREFIX)/share/man

INSTALL_DIRS := $(BINDIR) $(LIBDIR) $(MANDIR)

.PHONY: install dirs

dirs:
\t@for d in $(INSTALL_DIRS); do mkdir -p "$$d"; done

install: dirs
\t@echo "Installing to $(PREFIX)"
\tcp -f app $(BINDIR)/app
\t@echo "Done"''',
        '''PREFIX :='''))
    bid += 1

    # P-O3: Conditional Makefile
    entries.append((f"M-{bid}", "make-conditional-complex", "Makefile with ifdef/ifndef/ifeq conditionals",
        "Makefile",
        '''DEBUG ?= 0
PLATFORM ?= linux

ifeq ($(DEBUG),1)
CFLAGS := -g -O0 -DDEBUG
else
CFLAGS := -O2 -DNDEBUG
endif

ifeq ($(PLATFORM),linux)
LDFLAGS := -lpthread
else ifeq ($(PLATFORM),macos)
LDFLAGS := -framework Foundation
else
LDFLAGS :=
endif

.PHONY: build
build:
\t@echo "Platform: $(PLATFORM) Debug: $(DEBUG)"
\t@echo "CFLAGS: $(CFLAGS)"
\t@echo "LDFLAGS: $(LDFLAGS)"''',
        '''DEBUG ?='''))
    bid += 1

    return entries, bid

def gen_dockerfile_entries(start_bid):
    """Generate Dockerfile entries for multi-stage and complex layering."""
    entries = []
    bid = start_bid

    # P-D1: Multi-stage Docker build
    entries.append((f"D-{bid}", "docker-multistage-build", "Multi-stage Dockerfile with builder and runtime stages",
        "Dockerfile",
        '''FROM rust:1.75 AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/app /usr/local/bin/app
EXPOSE 8080
ENTRYPOINT ["app"]''',
        '''FROM rust:'''))
    bid += 1

    # P-D2: Complex layering with ARG and ENV
    entries.append((f"D-{bid}", "docker-arg-env-layers", "Dockerfile with ARG/ENV layering and conditional RUN",
        "Dockerfile",
        '''ARG BASE_IMAGE=ubuntu:22.04
FROM ${BASE_IMAGE}

ARG APP_VERSION=1.0.0
ARG BUILD_DATE
ENV APP_VERSION=${APP_VERSION}
ENV APP_HOME=/opt/app

RUN groupadd -r appuser && useradd -r -g appuser appuser
WORKDIR ${APP_HOME}
COPY --chown=appuser:appuser . .
RUN chmod +x entrypoint.sh

USER appuser
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s CMD curl -f http://localhost:3000/health || exit 1
CMD ["./entrypoint.sh"]''',
        '''ARG BASE_IMAGE'''))
    bid += 1

    # P-D3: Multi-stage with test stage
    entries.append((f"D-{bid}", "docker-test-stage", "Dockerfile with dedicated test stage between build and runtime",
        "Dockerfile",
        '''FROM node:20-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --production=false

FROM deps AS test
COPY . .
RUN npm run lint && npm test

FROM deps AS build
COPY . .
RUN npm run build && npm prune --production

FROM node:20-alpine AS runtime
WORKDIR /app
COPY --from=build /app/dist ./dist
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/package.json ./
USER node
EXPOSE 8080
CMD ["node", "dist/index.js"]''',
        '''FROM node:'''))
    bid += 1

    return entries, bid


def emit_rust_code(bash_entries, makefile_entries, dockerfile_entries, expansion_num):
    """Emit Rust code for inserting into registry.rs"""
    lines = []

    # Bash entries
    lines.append(f"    fn load_expansion{expansion_num}_bash(&mut self) {{")
    for bid, name, desc, code, expected in bash_entries:
        rust_code = format_rust_string(code)
        rust_expected = format_rust_string(expected)
        lines.append(f'        self.entries.push(CorpusEntry::new("{bid}", "{name}", "{desc}",')
        lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
        lines.append(f'            {rust_code},')
        lines.append(f'            {rust_expected}));')
    lines.append("    }")
    lines.append("")

    # Makefile entries
    lines.append(f"    pub fn load_expansion{expansion_num}_makefile(&mut self) {{")
    for bid, name, desc, fmt, code, expected in makefile_entries:
        rust_code = format_rust_string(code)
        rust_expected = format_rust_string(expected)
        lines.append(f'        self.entries.push(CorpusEntry::new("{bid}", "{name}", "{desc}",')
        lines.append(f'            CorpusFormat::Makefile, CorpusTier::Adversarial,')
        lines.append(f'            {rust_code},')
        lines.append(f'            {rust_expected}));')
    lines.append("    }")
    lines.append("")

    # Dockerfile entries
    lines.append(f"    pub fn load_expansion{expansion_num}_dockerfile(&mut self) {{")
    for bid, name, desc, fmt, code, expected in dockerfile_entries:
        rust_code = format_rust_string(code)
        rust_expected = format_rust_string(expected)
        lines.append(f'        self.entries.push(CorpusEntry::new("{bid}", "{name}", "{desc}",')
        lines.append(f'            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
        lines.append(f'            {rust_code},')
        lines.append(f'            {rust_expected}));')
    lines.append("    }")

    return "\n".join(lines)


if __name__ == "__main__":
    bash_entries, next_bid = gen_entries()
    makefile_entries, next_bid = gen_makefile_entries(next_bid)
    dockerfile_entries, next_bid = gen_dockerfile_entries(next_bid)

    print(f"// Round 1: {len(bash_entries)} bash + {len(makefile_entries)} makefile + {len(dockerfile_entries)} dockerfile = {len(bash_entries) + len(makefile_entries) + len(dockerfile_entries)} entries")
    print(f"// B-IDs: B-{NEXT_ID}..B-{next_bid - 1}")
    print(f"// Expansion function: {EXPANSION_NUM}")
    print()
    print(emit_rust_code(bash_entries, makefile_entries, dockerfile_entries, EXPANSION_NUM))
    print()
    print(f"// Call in load_full():")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_bash();")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_makefile();")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_dockerfile();")
