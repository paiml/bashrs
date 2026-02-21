#!/usr/bin/env python3
"""
Round 4: EXTREME difficulty. Designed to find and break transpiler boundaries.
Focus: struct usage, complex match, else-if chains, string operations, for-in loops.
"""

NEXT_ID = 15963
EXPANSION_NUM = 182

def format_rust_string(s):
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

def gen_entries():
    entries = []
    bid = NEXT_ID

    # A4: 8-function pipeline with accumulator struct-like pattern
    entries.append((f"B-{bid}", "pipe-8fn-accum", "8-function pipeline accumulating results in array state",
        r'''fn step1(state: [i64; 4]) -> i64 { return state[0] + 1; }
fn step2(state: [i64; 4]) -> i64 { return state[1] * 2; }
fn step3(state: [i64; 4]) -> i64 { return state[2] + state[3]; }
fn step4(a: i64, b: i64, c: i64) -> i64 { return a + b + c; }
fn step5(total: i64) -> i64 { return total % 1000; }
fn step6(x: i64) -> i64 { if x > 500 { return x - 500; } return x; }
fn step7(x: i64, scale: i64) -> i64 { return x * scale / 100; }
fn step8(x: i64) -> i64 { return x * x; }
fn main() {
    let state: [i64; 4] = [10, 20, 30, 40];
    let a: i64 = step1(state);
    let b: i64 = step2(state);
    let c: i64 = step3(state);
    let total: i64 = step4(a, b, c);
    let normed: i64 = step5(total);
    let clamped: i64 = step6(normed);
    let scaled: i64 = step7(clamped, 150);
    let result: i64 = step8(scaled);
    println!("a={} b={} c={} total={} norm={} clamp={} scale={} result={}", a, b, c, total, normed, clamped, scaled, result);
}''', r'''step1()'''))
    bid += 1

    # B4: String with backslash and quotes
    entries.append((f"B-{bid}", "quote-backslash-heavy", "Strings with heavy backslash and double-quote content",
        r'''fn main() {
    let path: &str = "/usr/local/bin";
    let escaped: &str = "line1\\nline2";
    let tabbed: &str = "col1\\tcol2";
    println!("path: {}", path);
    println!("escaped: {}", escaped);
    println!("tabbed: {}", tabbed);
}''', r'''path:'''))
    bid += 1

    # C4: Extreme one-liner - 10 nested function calls
    entries.append((f"B-{bid}", "oneliner-10nested-call", "10-level nested function call chain",
        r'''fn a(x: i64) -> i64 { return x + 1; }
fn b(x: i64) -> i64 { return x * 2; }
fn c(x: i64) -> i64 { return x - 3; }
fn d(x: i64) -> i64 { return x + 7; }
fn e(x: i64) -> i64 { return x * 3; }
fn f(x: i64) -> i64 { return x - 5; }
fn g(x: i64) -> i64 { return x / 2; }
fn h(x: i64) -> i64 { return x + 11; }
fn j(x: i64) -> i64 { return x * x; }
fn k(x: i64) -> i64 { return x % 100; }
fn main() {
    let r: i64 = k(j(h(g(f(e(d(c(b(a(1))))))))));
    println!("10-deep={}", r);
}''', r'''a()'''))
    bid += 1

    # D4: File type detection via magic number matching
    entries.append((f"B-{bid}", "glob-magic-number", "File type detection via magic number pattern matching",
        r'''fn detect_type(magic: [i64; 4]) -> i64 {
    if magic[0] == 127 {
        if magic[1] == 69 {
            if magic[2] == 76 {
                if magic[3] == 70 { return 1; }
            }
        }
    }
    if magic[0] == 80 {
        if magic[1] == 75 { return 2; }
    }
    if magic[0] == 137 {
        if magic[1] == 80 {
            if magic[2] == 78 {
                if magic[3] == 71 { return 3; }
            }
        }
    }
    if magic[0] == 35 {
        if magic[1] == 33 { return 4; }
    }
    return 0;
}
fn main() {
    let elf: [i64; 4] = [127, 69, 76, 70];
    let zip: [i64; 4] = [80, 75, 3, 4];
    let png: [i64; 4] = [137, 80, 78, 71];
    let shebang: [i64; 4] = [35, 33, 47, 98];
    let unknown: [i64; 4] = [0, 0, 0, 0];
    println!("ELF={} ZIP={} PNG={} SHEBANG={} UNK={}", detect_type(elf), detect_type(zip), detect_type(png), detect_type(shebang), detect_type(unknown));
}''', r'''detect_type()'''))
    bid += 1

    # E4: Config generator with sections and subsections
    entries.append((f"B-{bid}", "heredoc-config-nested", "Multi-level config generation with nested sections",
        r'''fn emit_section(name: &str, enabled: i64) {
    println!("[{}]", name);
    println!("enabled = {}", enabled);
}
fn emit_kv(key: &str, val: i64) {
    println!("  {} = {}", key, val);
}
fn emit_subsection(parent: &str, child: &str) {
    println!("  [{}]", child);
}
fn main() {
    emit_section("server", 1);
    emit_kv("port", 8080);
    emit_kv("workers", 4);
    emit_subsection("server", "tls");
    emit_kv("cert_days", 365);
    println!("");
    emit_section("database", 1);
    emit_kv("pool_size", 10);
    emit_kv("timeout", 30);
    println!("");
    emit_section("cache", 0);
}''', r'''emit_section()'''))
    bid += 1

    # F4: Env var validation with type checking
    entries.append((f"B-{bid}", "env-validate-types", "Environment variable validation with range and type checking",
        r'''fn is_port(val: i64) -> i64 {
    if val < 1 { return 0; }
    if val > 65535 { return 0; }
    return 1;
}
fn is_positive(val: i64) -> i64 {
    if val > 0 { return 1; }
    return 0;
}
fn is_percentage(val: i64) -> i64 {
    if val < 0 { return 0; }
    if val > 100 { return 0; }
    return 1;
}
fn validate_config(port: i64, workers: i64, cpu_limit: i64) -> i64 {
    let p: i64 = is_port(port);
    let w: i64 = is_positive(workers);
    let c: i64 = is_percentage(cpu_limit);
    if p == 1 {
        if w == 1 {
            if c == 1 { return 0; }
        }
    }
    return -1;
}
fn main() {
    println!("valid: {}", validate_config(8080, 4, 80));
    println!("bad port: {}", validate_config(0, 4, 80));
    println!("bad workers: {}", validate_config(8080, 0, 80));
    println!("bad cpu: {}", validate_config(8080, 4, 101));
    println!("all bad: {}", validate_config(-1, -1, -1));
}''', r'''is_port()'''))
    bid += 1

    # G4: SSH key exchange simulation
    entries.append((f"B-{bid}", "ssh-kex-sim", "SSH key exchange simulation with Diffie-Hellman-like computation",
        r'''fn mod_pow(base: i64, exp: i64, modulus: i64) -> i64 {
    if modulus == 1 { return 0; }
    let mut result: i64 = 1;
    let mut b: i64 = base % modulus;
    let mut e: i64 = exp;
    while e > 0 {
        if e % 2 == 1 {
            result = (result * b) % modulus;
        }
        e = e / 2;
        b = (b * b) % modulus;
    }
    return result;
}
fn dh_shared_secret(g: i64, p: i64, priv_a: i64, pub_b: i64) -> i64 {
    return mod_pow(pub_b, priv_a, p);
}
fn main() {
    let g: i64 = 5;
    let p: i64 = 23;
    let a_priv: i64 = 6;
    let b_priv: i64 = 15;
    let a_pub: i64 = mod_pow(g, a_priv, p);
    let b_pub: i64 = mod_pow(g, b_priv, p);
    let secret_a: i64 = dh_shared_secret(g, p, a_priv, b_pub);
    let secret_b: i64 = dh_shared_secret(g, p, b_priv, a_pub);
    println!("A_pub={} B_pub={}", a_pub, b_pub);
    println!("secret_A={} secret_B={} match={}", secret_a, secret_b, if secret_a == secret_b { 1 } else { 0 });
}''', r'''mod_pow()'''))
    bid += 1

    # H4: Multi-line table with border drawing
    entries.append((f"B-{bid}", "print-table-bordered", "Table output with border characters and column alignment",
        r'''fn print_border(width: i64) {
    let mut i: i64 = 0;
    while i < width {
        print!("-");
        i = i + 1;
    }
    println!("");
}
fn print_cell(val: i64) {
    print!("| {} ", val);
}
fn main() {
    print_border(25);
    println!("| ID | NAME | SCORE |");
    print_border(25);
    print_cell(1);
    println!("| alpha | 95 |");
    print_cell(2);
    println!("| beta  | 87 |");
    print_cell(3);
    println!("| gamma | 92 |");
    print_border(25);
}''', r'''print_border()'''))
    bid += 1

    # I4: Awk-like conditional aggregation
    entries.append((f"B-{bid}", "awk-conditional-agg", "Awk-like conditional aggregation with group-by simulation",
        r'''fn group_id(record: i64) -> i64 { return record / 100; }
fn record_value(record: i64) -> i64 { return record % 100; }
fn main() {
    let records: [i64; 8] = [150, 120, 230, 245, 110, 340, 355, 160];
    let mut sum_g1: i64 = 0;
    let mut sum_g2: i64 = 0;
    let mut sum_g3: i64 = 0;
    let mut cnt_g1: i64 = 0;
    let mut cnt_g2: i64 = 0;
    let mut cnt_g3: i64 = 0;
    let mut i: i64 = 0;
    while i < 8 {
        let g: i64 = group_id(records[i]);
        let v: i64 = record_value(records[i]);
        if g == 1 {
            sum_g1 = sum_g1 + v;
            cnt_g1 = cnt_g1 + 1;
        } else if g == 2 {
            sum_g2 = sum_g2 + v;
            cnt_g2 = cnt_g2 + 1;
        } else if g == 3 {
            sum_g3 = sum_g3 + v;
            cnt_g3 = cnt_g3 + 1;
        }
        i = i + 1;
    }
    println!("g1: sum={} cnt={}", sum_g1, cnt_g1);
    println!("g2: sum={} cnt={}", sum_g2, cnt_g2);
    println!("g3: sum={} cnt={}", sum_g3, cnt_g3);
}''', r'''group_id()'''))
    bid += 1

    # J4: Doubly-linked list simulation
    entries.append((f"B-{bid}", "ds-doubly-linked", "Doubly-linked list with prev/next pointer arrays",
        r'''fn dll_next(next: [i64; 8], idx: i64) -> i64 {
    if idx < 0 { return -1; }
    return next[idx];
}
fn dll_prev(prev: [i64; 8], idx: i64) -> i64 {
    if idx < 0 { return -1; }
    return prev[idx];
}
fn dll_traverse_fwd(vals: [i64; 8], next: [i64; 8], head: i64) -> i64 {
    let mut sum: i64 = 0;
    let mut cur: i64 = head;
    let mut count: i64 = 0;
    while cur >= 0 {
        sum = sum + vals[cur];
        cur = dll_next(next, cur);
        count = count + 1;
        if count > 8 { break; }
    }
    return sum;
}
fn dll_traverse_rev(vals: [i64; 8], prev: [i64; 8], tail: i64) -> i64 {
    let mut sum: i64 = 0;
    let mut cur: i64 = tail;
    let mut count: i64 = 0;
    while cur >= 0 {
        sum = sum + vals[cur];
        cur = dll_prev(prev, cur);
        count = count + 1;
        if count > 8 { break; }
    }
    return sum;
}
fn main() {
    let vals: [i64; 8] = [10, 20, 30, 40, 0, 0, 0, 0];
    let next: [i64; 8] = [1, 2, 3, -1, -1, -1, -1, -1];
    let prev: [i64; 8] = [-1, 0, 1, 2, -1, -1, -1, -1];
    let fwd_sum: i64 = dll_traverse_fwd(vals, next, 0);
    let rev_sum: i64 = dll_traverse_rev(vals, prev, 3);
    println!("fwd={} rev={} eq={}", fwd_sum, rev_sum, if fwd_sum == rev_sum { 1 } else { 0 });
}''', r'''dll_next()'''))
    bid += 1

    # K4: Dynamic module loader with circular dependency detection
    entries.append((f"B-{bid}", "source-circular-detect", "Module loader with circular dependency detection via visited tracking",
        r'''fn is_loaded(loaded: [i64; 6], mod_id: i64) -> i64 {
    if mod_id < 0 { return 0; }
    if mod_id >= 6 { return 0; }
    return loaded[mod_id];
}
fn is_in_stack(stack: [i64; 6], mod_id: i64, depth: i64) -> i64 {
    let mut i: i64 = 0;
    while i < depth {
        if stack[i] == mod_id { return 1; }
        i = i + 1;
    }
    return 0;
}
fn main() {
    let deps: [i64; 6] = [1, 2, 0, 4, 5, 3];
    let mut loaded: [i64; 6] = [0, 0, 0, 0, 0, 0];
    let mut stack: [i64; 6] = [0, 0, 0, 0, 0, 0];
    let mut i: i64 = 0;
    while i < 6 {
        let dep: i64 = deps[i];
        let circular: i64 = is_in_stack(stack, dep, i);
        if circular == 1 {
            println!("CIRCULAR: mod {} -> mod {}", i, dep);
        } else if is_loaded(loaded, dep) == 0 {
            println!("load mod {} (dep={})", i, dep);
        } else {
            println!("skip mod {} (dep {} already loaded)", i, dep);
        }
        loaded[i] = 1;
        stack[i] = i;
        i = i + 1;
    }
}''', r'''is_loaded()'''))
    bid += 1

    # L4: Interpreter for simple stack-based bytecode
    entries.append((f"B-{bid}", "script-bytecode-vm", "Stack-based bytecode interpreter with 6 opcodes",
        r'''fn vm_push(stack: [i64; 8], sp: i64, val: i64) -> i64 {
    return sp + 1;
}
fn vm_pop(stack: [i64; 8], sp: i64) -> i64 {
    return stack[sp - 1];
}
fn vm_run(prog: [i64; 8], prog_len: i64) -> i64 {
    let mut stack: [i64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut sp: i64 = 0;
    let mut pc: i64 = 0;
    while pc < prog_len {
        let op: i64 = prog[pc];
        if op == 1 {
            pc = pc + 1;
            stack[sp] = prog[pc];
            sp = sp + 1;
        } else if op == 2 {
            let b: i64 = stack[sp - 1];
            let a: i64 = stack[sp - 2];
            sp = sp - 2;
            stack[sp] = a + b;
            sp = sp + 1;
        } else if op == 3 {
            let b: i64 = stack[sp - 1];
            let a: i64 = stack[sp - 2];
            sp = sp - 2;
            stack[sp] = a * b;
            sp = sp + 1;
        }
        pc = pc + 1;
    }
    if sp > 0 { return stack[sp - 1]; }
    return 0;
}
fn main() {
    let prog: [i64; 8] = [1, 3, 1, 4, 2, 1, 2, 3];
    let result: i64 = vm_run(prog, 8);
    println!("vm result: {}", result);
}''', r'''vm_push()'''))
    bid += 1

    # M4: Match with nested if in arms
    entries.append((f"B-{bid}", "brace-match-nested-if", "Match arms containing if-else logic",
        r'''fn process(op: i64, val: i64) -> i64 {
    match op {
        1 => {
            if val > 0 { return val * 2; }
            return 0;
        },
        2 => {
            if val > 100 { return 100; }
            if val < 0 { return 0; }
            return val;
        },
        3 => {
            return val * val;
        },
        _ => {
            return -1;
        },
    }
}
fn main() {
    println!("op1: {} {} {}", process(1, 5), process(1, -3), process(1, 0));
    println!("op2: {} {} {}", process(2, 50), process(2, 200), process(2, -10));
    println!("op3: {}", process(3, 7));
    println!("op9: {}", process(9, 42));
}''', r'''process()'''))
    bid += 1

    # N4: Resource acquisition is initialization (RAII) simulation
    entries.append((f"B-{bid}", "trap-raii-sim", "RAII-like resource management with guaranteed cleanup",
        r'''fn acquire(res_type: i64, res_id: i64) -> i64 {
    println!("acquire type={} id={}", res_type, res_id);
    return res_type * 100 + res_id;
}
fn release(handle: i64) {
    println!("release handle={}", handle);
}
fn use_resource(handle: i64, op: i64) -> i64 {
    if handle <= 0 { return -1; }
    return handle + op;
}
fn main() {
    let h1: i64 = acquire(1, 10);
    let h2: i64 = acquire(2, 20);
    let h3: i64 = acquire(3, 30);
    let r1: i64 = use_resource(h1, 5);
    let r2: i64 = use_resource(h2, 10);
    let r3: i64 = use_resource(h3, 15);
    println!("results: {} {} {}", r1, r2, r3);
    release(h3);
    release(h2);
    release(h1);
}''', r'''acquire()'''))
    bid += 1

    # O4: Full argument parser with help text
    entries.append((f"B-{bid}", "cli-full-parser", "Full CLI parser with flags, values, and help detection",
        r'''fn is_help(arg: i64) -> i64 {
    if arg == -99 { return 1; }
    return 0;
}
fn is_version(arg: i64) -> i64 {
    if arg == -98 { return 1; }
    return 0;
}
fn is_verbose(arg: i64) -> i64 {
    if arg == -97 { return 1; }
    return 0;
}
fn parse_args(args: [i64; 8], len: i64) -> i64 {
    let mut help: i64 = 0;
    let mut version: i64 = 0;
    let mut verbose: i64 = 0;
    let mut positional: i64 = 0;
    let mut i: i64 = 0;
    while i < len {
        if is_help(args[i]) == 1 { help = 1; }
        else if is_version(args[i]) == 1 { version = 1; }
        else if is_verbose(args[i]) == 1 { verbose = 1; }
        else { positional = positional + 1; }
        i = i + 1;
    }
    println!("help={} version={} verbose={} positional={}", help, version, verbose, positional);
    if help == 1 { return 0; }
    if version == 1 { return 1; }
    return 2;
}
fn main() {
    let args1: [i64; 8] = [-99, 0, 0, 0, 0, 0, 0, 0];
    let args2: [i64; 8] = [-97, 42, 43, 0, 0, 0, 0, 0];
    let args3: [i64; 8] = [-98, 0, 0, 0, 0, 0, 0, 0];
    let r1: i64 = parse_args(args1, 1);
    let r2: i64 = parse_args(args2, 3);
    let r3: i64 = parse_args(args3, 1);
    println!("modes: {} {} {}", r1, r2, r3);
}''', r'''is_help()'''))
    bid += 1

    # Q4: Gaussian elimination (partial)
    entries.append((f"B-{bid}", "num-gauss-elim", "Gaussian elimination for 2x2 system",
        r'''fn solve_2x2(a11: i64, a12: i64, b1: i64, a21: i64, a22: i64, b2: i64) -> i64 {
    let det: i64 = a11 * a22 - a12 * a21;
    if det == 0 { return -9999; }
    let x: i64 = (b1 * a22 - b2 * a12) / det;
    return x;
}
fn solve_2x2_y(a11: i64, a12: i64, b1: i64, a21: i64, a22: i64, b2: i64) -> i64 {
    let det: i64 = a11 * a22 - a12 * a21;
    if det == 0 { return -9999; }
    let y: i64 = (a11 * b2 - a21 * b1) / det;
    return y;
}
fn main() {
    let x: i64 = solve_2x2(2, 1, 5, 1, 3, 7);
    let y: i64 = solve_2x2_y(2, 1, 5, 1, 3, 7);
    println!("2x+y=5, x+3y=7: x={} y={}", x, y);
    let x2: i64 = solve_2x2(1, 1, 10, 1, -1, 2);
    let y2: i64 = solve_2x2_y(1, 1, 10, 1, -1, 2);
    println!("x+y=10, x-y=2: x={} y={}", x2, y2);
    let x3: i64 = solve_2x2(1, 2, 0, 2, 4, 0);
    println!("singular: x={}", x3);
}''', r'''solve_2x2()'''))
    bid += 1

    # R4: Symbolic - Expression tree evaluator
    entries.append((f"B-{bid}", "symbolic-expr-tree", "Expression tree evaluator using encoded node array",
        r'''fn node_op(nodes: [i64; 8], idx: i64) -> i64 { return nodes[idx]; }
fn eval_node(nodes: [i64; 8], vals: [i64; 8], idx: i64) -> i64 {
    let op: i64 = nodes[idx];
    if op == 0 { return vals[idx]; }
    let left_idx: i64 = idx * 2 + 1;
    let right_idx: i64 = idx * 2 + 2;
    if left_idx >= 8 { return vals[idx]; }
    if right_idx >= 8 { return vals[idx]; }
    let l: i64 = eval_node(nodes, vals, left_idx);
    let r: i64 = eval_node(nodes, vals, right_idx);
    if op == 1 { return l + r; }
    if op == 2 { return l - r; }
    if op == 3 { return l * r; }
    return l;
}
fn main() {
    let nodes: [i64; 8] = [1, 3, 0, 0, 0, 0, 0, 0];
    let vals: [i64; 8] = [0, 0, 7, 3, 4, 0, 0, 0];
    let result: i64 = eval_node(nodes, vals, 0);
    println!("(3*4)+7={}", result);
}''', r'''node_op()'''))
    bid += 1

    # S4: Editor - line buffer with insert/delete at position
    entries.append((f"B-{bid}", "editor-line-buffer", "Line buffer editor with insert and delete at arbitrary position",
        r'''fn shift_right(buf: [i64; 8], pos: i64, len: i64) -> i64 {
    return len + 1;
}
fn shift_left(buf: [i64; 8], pos: i64, len: i64) -> i64 {
    if len == 0 { return 0; }
    return len - 1;
}
fn main() {
    let mut buf: [i64; 8] = [65, 66, 67, 68, 0, 0, 0, 0];
    let mut len: i64 = 4;
    println!("initial: len={}", len);
    len = shift_right(buf, 2, len);
    buf[2] = 88;
    println!("insert X at 2: len={} buf[2]={}", len, buf[2]);
    len = shift_left(buf, 1, len);
    println!("delete at 1: len={}", len);
    len = shift_right(buf, 0, len);
    buf[0] = 90;
    println!("insert Z at 0: len={} buf[0]={}", len, buf[0]);
}''', r'''shift_right()'''))
    bid += 1

    # T4: Church numerals simulation
    entries.append((f"B-{bid}", "fp-church-numerals", "Church numeral encoding: zero, succ, add, mul",
        r'''fn church_zero() -> i64 { return 0; }
fn church_succ(n: i64) -> i64 { return n + 1; }
fn church_add(a: i64, b: i64) -> i64 { return a + b; }
fn church_mul(a: i64, b: i64) -> i64 { return a * b; }
fn church_pred(n: i64) -> i64 { if n <= 0 { return 0; } return n - 1; }
fn church_is_zero(n: i64) -> i64 { if n == 0 { return 1; } return 0; }
fn church_sub(a: i64, b: i64) -> i64 {
    let mut result: i64 = a;
    let mut i: i64 = 0;
    while i < b {
        result = church_pred(result);
        i = i + 1;
    }
    return result;
}
fn main() {
    let zero: i64 = church_zero();
    let one: i64 = church_succ(zero);
    let two: i64 = church_succ(one);
    let three: i64 = church_add(one, two);
    let six: i64 = church_mul(two, three);
    let five: i64 = church_sub(six, one);
    println!("0={} 1={} 2={} 3={} 6={} 5={}", zero, one, two, three, six, five);
    println!("is_zero(0)={} is_zero(3)={}", church_is_zero(zero), church_is_zero(three));
}''', r'''church_zero()'''))
    bid += 1

    # U4: Division and modular arithmetic with proof assertions
    entries.append((f"B-{bid}", "prove-div-mod-inv", "Division-modulus invariant: a == (a/b)*b + (a%b) for all valid b",
        r'''fn verify_div_mod(a: i64, b: i64) -> i64 {
    if b == 0 { return 1; }
    let q: i64 = a / b;
    let r: i64 = a % b;
    let reconstructed: i64 = q * b + r;
    if reconstructed == a { return 1; }
    return 0;
}
fn main() {
    let test_vals: [i64; 8] = [0, 1, 7, 10, 100, -7, -10, 42];
    let divisors: [i64; 4] = [1, 3, 7, 10];
    let mut i: i64 = 0;
    let mut all_pass: i64 = 1;
    while i < 8 {
        let mut j: i64 = 0;
        while j < 4 {
            let ok: i64 = verify_div_mod(test_vals[i], divisors[j]);
            if ok == 0 {
                println!("FAIL: a={} b={}", test_vals[i], divisors[j]);
                all_pass = 0;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    println!("all_pass={}", all_pass);
}''', r'''verify_div_mod()'''))
    bid += 1

    # V4: Defensive coding patterns
    entries.append((f"B-{bid}", "clippy-defensive-code", "Defensive coding: null checks, bounds, and error propagation",
        r'''fn safe_index(arr: [i64; 8], idx: i64, len: i64) -> i64 {
    if idx < 0 { return -1; }
    if idx >= len { return -1; }
    return arr[idx];
}
fn safe_divide(a: i64, b: i64) -> i64 {
    if b == 0 { return 0; }
    return a / b;
}
fn safe_sqrt_int(n: i64) -> i64 {
    if n < 0 { return -1; }
    let mut x: i64 = n;
    let mut y: i64 = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + safe_divide(n, x)) / 2;
    }
    return x;
}
fn main() {
    let arr: [i64; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    println!("idx: {} {} {} {}", safe_index(arr, 0, 8), safe_index(arr, 7, 8), safe_index(arr, 8, 8), safe_index(arr, -1, 8));
    println!("div: {} {} {}", safe_divide(100, 3), safe_divide(100, 0), safe_divide(0, 5));
    println!("sqrt: {} {} {} {}", safe_sqrt_int(100), safe_sqrt_int(2), safe_sqrt_int(0), safe_sqrt_int(-4));
}''', r'''safe_index()'''))
    bid += 1

    # W4: C-style bitfield operations
    entries.append((f"B-{bid}", "c-style-bitfield", "C-style bitfield packing and unpacking operations",
        r'''fn pack_rgb(r: i64, g: i64, b: i64) -> i64 {
    return r * 65536 + g * 256 + b;
}
fn unpack_r(packed: i64) -> i64 { return packed / 65536; }
fn unpack_g(packed: i64) -> i64 { return (packed / 256) % 256; }
fn unpack_b(packed: i64) -> i64 { return packed % 256; }
fn blend(c1: i64, c2: i64, alpha: i64) -> i64 {
    let r1: i64 = unpack_r(c1);
    let g1: i64 = unpack_g(c1);
    let b1: i64 = unpack_b(c1);
    let r2: i64 = unpack_r(c2);
    let g2: i64 = unpack_g(c2);
    let b2: i64 = unpack_b(c2);
    let r: i64 = (r1 * alpha + r2 * (100 - alpha)) / 100;
    let g: i64 = (g1 * alpha + g2 * (100 - alpha)) / 100;
    let b: i64 = (b1 * alpha + b2 * (100 - alpha)) / 100;
    return pack_rgb(r, g, b);
}
fn main() {
    let red: i64 = pack_rgb(255, 0, 0);
    let blue: i64 = pack_rgb(0, 0, 255);
    let blended: i64 = blend(red, blue, 50);
    println!("red={} r={} g={} b={}", red, unpack_r(red), unpack_g(red), unpack_b(red));
    println!("blue={} r={} g={} b={}", blue, unpack_r(blue), unpack_g(blue), unpack_b(blue));
    println!("blend={} r={} g={} b={}", blended, unpack_r(blended), unpack_g(blended), unpack_b(blended));
}''', r'''pack_rgb()'''))
    bid += 1

    return entries, bid


def gen_makefile_r4(start_bid):
    entries = []
    bid = start_bid

    entries.append((f"M-{bid}", "make-cross-compile", "Cross-compilation Makefile with multiple target architectures",
        "Makefile",
        '''TARGETS := linux-amd64 linux-arm64 darwin-amd64 darwin-arm64
BINARY := myapp
VERSION := $(shell git describe --tags 2>/dev/null || echo dev)

.PHONY: all clean $(TARGETS)

all: $(TARGETS)

define build_target
$(1):
\t@echo "Building $(1)..."
\tGOOS=$$(echo $(1) | cut -d- -f1) GOARCH=$$(echo $(1) | cut -d- -f2) go build -o dist/$(BINARY)-$(1) .
endef

$(foreach target,$(TARGETS),$(eval $(call build_target,$(target))))

clean:
\trm -rf dist''',
        '''TARGETS :='''))
    bid += 1

    entries.append((f"M-{bid}", "make-docker-compose", "Makefile orchestrating Docker Compose with environment switching",
        "Makefile",
        '''ENV ?= dev
COMPOSE := docker compose -f docker-compose.yml -f docker-compose.$(ENV).yml
PROJECT := myapp-$(ENV)

.PHONY: up down build logs ps test deploy

up:
\t$(COMPOSE) -p $(PROJECT) up -d

down:
\t$(COMPOSE) -p $(PROJECT) down

build:
\t$(COMPOSE) -p $(PROJECT) build --no-cache

logs:
\t$(COMPOSE) -p $(PROJECT) logs -f

ps:
\t$(COMPOSE) -p $(PROJECT) ps

test: up
\t$(COMPOSE) -p $(PROJECT) exec app pytest
\t$(MAKE) down

deploy: build
\t@echo "Deploying $(ENV)..."
\t$(COMPOSE) -p $(PROJECT) up -d --remove-orphans''',
        '''ENV ?='''))
    bid += 1

    entries.append((f"M-{bid}", "make-c-library", "C library Makefile with shared and static targets",
        "Makefile",
        '''LIB := libmath
VERSION := 1.0.0
CC := gcc
CFLAGS := -Wall -fPIC -O2
SRCS := $(wildcard src/*.c)
OBJS := $(SRCS:src/%.c=obj/%.o)

.PHONY: all static shared clean install

all: static shared

static: lib/$(LIB).a

shared: lib/$(LIB).so.$(VERSION)

lib/$(LIB).a: $(OBJS)
\t@mkdir -p lib
\tar rcs $@ $^

lib/$(LIB).so.$(VERSION): $(OBJS)
\t@mkdir -p lib
\t$(CC) -shared -Wl,-soname,$(LIB).so.1 -o $@ $^
\tcd lib && ln -sf $(LIB).so.$(VERSION) $(LIB).so.1
\tcd lib && ln -sf $(LIB).so.1 $(LIB).so

obj/%.o: src/%.c
\t@mkdir -p obj
\t$(CC) $(CFLAGS) -c $< -o $@

clean:
\trm -rf obj lib''',
        '''LIB :='''))
    bid += 1

    return entries, bid


def gen_dockerfile_r4(start_bid):
    entries = []
    bid = start_bid

    entries.append((f"D-{bid}", "docker-monorepo-build", "Monorepo Dockerfile building multiple services from shared base",
        "Dockerfile",
        '''FROM node:20-alpine AS base
WORKDIR /app
COPY package.json package-lock.json ./
COPY packages/shared/package.json packages/shared/
COPY packages/api/package.json packages/api/
COPY packages/web/package.json packages/web/
RUN npm ci

FROM base AS shared
COPY packages/shared/ packages/shared/
RUN npm run build -w packages/shared

FROM shared AS api
COPY packages/api/ packages/api/
RUN npm run build -w packages/api

FROM shared AS web
COPY packages/web/ packages/web/
RUN npm run build -w packages/web

FROM node:20-alpine AS api-runtime
WORKDIR /app
COPY --from=api /app/packages/api/dist ./dist
COPY --from=api /app/node_modules ./node_modules
EXPOSE 3000
CMD ["node", "dist/index.js"]''',
        '''FROM node:'''))
    bid += 1

    entries.append((f"D-{bid}", "docker-distroless", "Distroless container with minimal attack surface",
        "Dockerfile",
        '''FROM golang:1.22 AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath -ldflags="-s -w" -o /server ./cmd/server

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /server /server
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/server"]''',
        '''FROM golang:'''))
    bid += 1

    entries.append((f"D-{bid}", "docker-init-system", "Dockerfile with tini init system and signal handling",
        "Dockerfile",
        '''FROM python:3.12-slim
RUN apt-get update && apt-get install -y --no-install-recommends tini && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
ENTRYPOINT ["tini", "--"]
CMD ["python", "app.py"]
HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8000/health')"
EXPOSE 8000
USER 1000:1000''',
        '''FROM python:'''))
    bid += 1

    return entries, bid


def emit_rust_code(bash_entries, makefile_entries, dockerfile_entries, expansion_num):
    lines = []
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
    makefile_entries, next_bid = gen_makefile_r4(next_bid)
    dockerfile_entries, next_bid = gen_dockerfile_r4(next_bid)
    total = len(bash_entries) + len(makefile_entries) + len(dockerfile_entries)
    print(f"// Round 4: {len(bash_entries)} bash + {len(makefile_entries)} makefile + {len(dockerfile_entries)} dockerfile = {total} entries")
    print(f"// B-IDs: B-{NEXT_ID}..{next_bid - 1}")
    print(f"// Expansion function: {EXPANSION_NUM}")
    print()
    print(emit_rust_code(bash_entries, makefile_entries, dockerfile_entries, EXPANSION_NUM))
