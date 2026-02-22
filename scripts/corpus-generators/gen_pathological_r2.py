#!/usr/bin/env python3
"""
Round 2: HARDER pathological entries. Pushes transpiler boundaries.
Categories: deeper nesting, larger programs, edge-case arithmetic, complex control flow.
"""

NEXT_ID = 15906
EXPANSION_NUM = 180

def format_rust_string(s):
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

def gen_entries():
    entries = []
    bid = NEXT_ID

    # ============================================================
    # A2: Pipes - Multi-stage with accumulator and error propagation
    # ============================================================
    entries.append((f"B-{bid}", "pipe-error-prop", "Pipeline with error propagation through stages",
        r'''fn stage1(x: i64) -> i64 {
    if x < 0 { return -999; }
    return x * 3;
}
fn stage2(x: i64) -> i64 {
    if x == -999 { return -999; }
    return x + 7;
}
fn stage3(x: i64) -> i64 {
    if x == -999 { return -999; }
    if x % 2 == 0 { return x / 2; }
    return x;
}
fn stage4(x: i64) -> i64 {
    if x == -999 { return -999; }
    return x * x;
}
fn pipeline(input: i64) -> i64 {
    let s1: i64 = stage1(input);
    let s2: i64 = stage2(s1);
    let s3: i64 = stage3(s2);
    let s4: i64 = stage4(s3);
    return s4;
}
fn main() {
    let inputs: [i64; 5] = [5, -1, 0, 10, 3];
    let mut i: i64 = 0;
    while i < 5 {
        let result: i64 = pipeline(inputs[i]);
        println!("in={} out={}", inputs[i], result);
        i = i + 1;
    }
}''', r'''pipeline()'''))
    bid += 1

    # A3: Bidirectional pipe simulation
    entries.append((f"B-{bid}", "pipe-bidirectional", "Bidirectional data flow with producer-consumer-feedback",
        r'''fn produce(step: i64) -> i64 { return step * 10 + 1; }
fn consume(data: i64) -> i64 { return data / 2; }
fn feedback(consumed: i64, step: i64) -> i64 {
    if consumed > 20 { return step + 2; }
    return step + 1;
}
fn main() {
    let mut step: i64 = 1;
    let mut total_consumed: i64 = 0;
    let mut rounds: i64 = 0;
    while step < 10 {
        let data: i64 = produce(step);
        let result: i64 = consume(data);
        total_consumed = total_consumed + result;
        step = feedback(result, step);
        rounds = rounds + 1;
        println!("step={} data={} consumed={} total={}", step, data, result, total_consumed);
    }
    println!("rounds={} total={}", rounds, total_consumed);
}''', r'''produce()'''))
    bid += 1

    # ============================================================
    # B2: Harder quoting with format strings containing braces
    # ============================================================
    entries.append((f"B-{bid}", "quote-format-braces", "Format strings with literal braces and nested interpolation",
        r'''fn format_record(id: i64, name: &str, value: i64) -> i64 {
    println!("[{}] {} = {}", id, name, value);
    return id;
}
fn main() {
    format_record(1, "alpha", 100);
    format_record(2, "beta-gamma", 200);
    format_record(3, "delta_epsilon", 300);
    format_record(4, "zeta.eta", 400);
}''', r'''format_record()'''))
    bid += 1

    # ============================================================
    # C2: Extreme one-liner with 8 chained operations
    # ============================================================
    entries.append((f"B-{bid}", "oneliner-8chain", "Eight chained arithmetic operations in sequence",
        r'''fn f1(x: i64) -> i64 { return x + 1; }
fn f2(x: i64) -> i64 { return x * 2; }
fn f3(x: i64) -> i64 { return x - 3; }
fn f4(x: i64) -> i64 { return x / 2; }
fn f5(x: i64) -> i64 { return x + 10; }
fn f6(x: i64) -> i64 { return x * 3; }
fn f7(x: i64) -> i64 { return x - 7; }
fn f8(x: i64) -> i64 { return x % 100; }
fn main() {
    let r: i64 = f8(f7(f6(f5(f4(f3(f2(f1(5))))))));
    println!("chain={}", r);
}''', r'''f1()'''))
    bid += 1

    # ============================================================
    # D2: Bitmask pattern matching (glob-like)
    # ============================================================
    entries.append((f"B-{bid}", "glob-bitmask-pattern", "Bitmask-based pattern matching like file permissions",
        r'''fn has_read(perms: i64) -> i64 { if perms / 4 % 2 == 1 { return 1; } return 0; }
fn has_write(perms: i64) -> i64 { if perms / 2 % 2 == 1 { return 1; } return 0; }
fn has_exec(perms: i64) -> i64 { if perms % 2 == 1 { return 1; } return 0; }
fn describe_perms(perms: i64) {
    let r: i64 = has_read(perms);
    let w: i64 = has_write(perms);
    let x: i64 = has_exec(perms);
    println!("perms={} r={} w={} x={}", perms, r, w, x);
}
fn main() {
    describe_perms(7);
    describe_perms(6);
    describe_perms(5);
    describe_perms(4);
    describe_perms(0);
}''', r'''has_read()'''))
    bid += 1

    # ============================================================
    # E2: Heredoc with conditional sections
    # ============================================================
    entries.append((f"B-{bid}", "heredoc-conditional-sec", "Heredoc-like output with conditional sections based on flags",
        r'''fn emit_section(section: &str, enabled: i64) {
    if enabled == 1 {
        println!("[{}]", section);
        println!("  enabled = true");
    }
}
fn main() {
    println!("# Configuration");
    emit_section("logging", 1);
    emit_section("metrics", 0);
    emit_section("tracing", 1);
    emit_section("profiling", 0);
    println!("# End");
}''', r'''emit_section()'''))
    bid += 1

    # ============================================================
    # F2: Environment variable cascading with 4 levels
    # ============================================================
    entries.append((f"B-{bid}", "env-cascade-4level", "Four-level cascading environment variable resolution",
        r'''fn resolve4(a: i64, b: i64, c: i64, d: i64) -> i64 {
    if a != 0 { return a; }
    if b != 0 { return b; }
    if c != 0 { return c; }
    return d;
}
fn main() {
    let r1: i64 = resolve4(10, 20, 30, 40);
    let r2: i64 = resolve4(0, 20, 30, 40);
    let r3: i64 = resolve4(0, 0, 30, 40);
    let r4: i64 = resolve4(0, 0, 0, 40);
    let r5: i64 = resolve4(0, 0, 0, 0);
    println!("{} {} {} {} {}", r1, r2, r3, r4, r5);
}''', r'''resolve4()'''))
    bid += 1

    # ============================================================
    # G2: SSH tunneling simulation
    # ============================================================
    entries.append((f"B-{bid}", "ssh-tunnel-sim", "SSH tunnel setup with port forwarding simulation",
        r'''fn tunnel_open(local_port: i64, remote_port: i64, host: i64) -> i64 {
    if host == 0 { return -1; }
    if local_port < 1024 { return -2; }
    if remote_port < 1 { return -3; }
    println!("tunnel: local={} -> remote={} via host={}", local_port, remote_port, host);
    return local_port * 1000 + remote_port;
}
fn tunnel_close(tunnel_id: i64) -> i64 {
    if tunnel_id <= 0 { return -1; }
    println!("closed tunnel {}", tunnel_id);
    return 0;
}
fn main() {
    let t1: i64 = tunnel_open(8080, 80, 1);
    let t2: i64 = tunnel_open(9090, 443, 2);
    let t3: i64 = tunnel_open(80, 80, 1);
    let t4: i64 = tunnel_open(8080, 80, 0);
    println!("tunnels: {} {} {} {}", t1, t2, t3, t4);
    tunnel_close(t1);
    tunnel_close(t2);
    tunnel_close(t3);
}''', r'''tunnel_open()'''))
    bid += 1

    # ============================================================
    # H2: Printf-like format width simulation
    # ============================================================
    entries.append((f"B-{bid}", "print-padded-nums", "Formatted number output with padding simulation",
        r'''fn pad_width(n: i64, width: i64) -> i64 {
    let mut digits: i64 = 0;
    let mut tmp: i64 = n;
    if tmp == 0 { return width - 1; }
    if tmp < 0 { tmp = 0 - tmp; digits = 1; }
    while tmp > 0 {
        digits = digits + 1;
        tmp = tmp / 10;
    }
    if digits >= width { return 0; }
    return width - digits;
}
fn main() {
    let nums: [i64; 5] = [1, 42, 100, 9999, 0];
    let mut i: i64 = 0;
    while i < 5 {
        let padding: i64 = pad_width(nums[i], 6);
        println!("num={} pad={}", nums[i], padding);
        i = i + 1;
    }
}''', r'''pad_width()'''))
    bid += 1

    # ============================================================
    # I2: Awk-like aggregation with multiple columns
    # ============================================================
    entries.append((f"B-{bid}", "awk-multi-col-agg", "Awk-style multi-column aggregation with sum/min/max",
        r'''fn min_val(a: i64, b: i64) -> i64 { if a < b { return a; } return b; }
fn max_val(a: i64, b: i64) -> i64 { if a > b { return a; } return b; }
fn main() {
    let col1: [i64; 5] = [10, 20, 5, 30, 15];
    let col2: [i64; 5] = [100, 50, 200, 75, 150];
    let mut sum1: i64 = 0;
    let mut sum2: i64 = 0;
    let mut min1: i64 = 999999;
    let mut max2: i64 = 0;
    let mut i: i64 = 0;
    while i < 5 {
        sum1 = sum1 + col1[i];
        sum2 = sum2 + col2[i];
        min1 = min_val(min1, col1[i]);
        max2 = max_val(max2, col2[i]);
        i = i + 1;
    }
    println!("sum1={} sum2={} min1={} max2={}", sum1, sum2, min1, max2);
}''', r'''min_val()'''))
    bid += 1

    # ============================================================
    # J2: Binary search tree simulation
    # ============================================================
    entries.append((f"B-{bid}", "ds-bst-search", "Binary search tree search with array-based storage",
        r'''fn bst_left(idx: i64) -> i64 { return 2 * idx + 1; }
fn bst_right(idx: i64) -> i64 { return 2 * idx + 2; }
fn bst_search(tree: [i64; 15], size: i64, target: i64) -> i64 {
    let mut idx: i64 = 0;
    while idx < size {
        if tree[idx] == 0 { return -1; }
        if tree[idx] == target { return idx; }
        if target < tree[idx] {
            idx = bst_left(idx);
        } else {
            idx = bst_right(idx);
        }
    }
    return -1;
}
fn main() {
    let tree: [i64; 15] = [50, 30, 70, 20, 40, 60, 80, 0, 0, 0, 0, 0, 0, 0, 0];
    println!("find 40: idx={}", bst_search(tree, 15, 40));
    println!("find 70: idx={}", bst_search(tree, 15, 70));
    println!("find 99: idx={}", bst_search(tree, 15, 99));
    println!("find 50: idx={}", bst_search(tree, 15, 50));
}''', r'''bst_left()'''))
    bid += 1

    # ============================================================
    # K2: Multi-file sourcing with dependency ordering
    # ============================================================
    entries.append((f"B-{bid}", "source-dep-ordering", "Simulated source file dependency resolution with topological order",
        r'''fn dep_satisfied(loaded: [i64; 6], dep: i64) -> i64 {
    if dep < 0 { return 1; }
    if dep >= 6 { return 0; }
    return loaded[dep];
}
fn can_load(loaded: [i64; 6], deps: [i64; 3]) -> i64 {
    let d0: i64 = dep_satisfied(loaded, deps[0]);
    let d1: i64 = dep_satisfied(loaded, deps[1]);
    let d2: i64 = dep_satisfied(loaded, deps[2]);
    if d0 == 1 {
        if d1 == 1 {
            if d2 == 1 { return 1; }
        }
    }
    return 0;
}
fn main() {
    let mut loaded: [i64; 6] = [0, 0, 0, 0, 0, 0];
    let deps0: [i64; 3] = [-1, -1, -1];
    let deps1: [i64; 3] = [0, -1, -1];
    let deps2: [i64; 3] = [0, 1, -1];
    let deps3: [i64; 3] = [1, 2, -1];
    loaded[0] = can_load(loaded, deps0);
    println!("load 0: {}", loaded[0]);
    loaded[1] = can_load(loaded, deps1);
    println!("load 1: {}", loaded[1]);
    loaded[2] = can_load(loaded, deps2);
    println!("load 2: {}", loaded[2]);
    loaded[3] = can_load(loaded, deps3);
    println!("load 3: {}", loaded[3]);
}''', r'''dep_satisfied()'''))
    bid += 1

    # ============================================================
    # L2: Pathological script - 10 functions with mutual calls
    # ============================================================
    entries.append((f"B-{bid}", "script-10fn-mutual", "10-function program with mutual calling and state threading",
        r'''fn validate_input(x: i64) -> i64 {
    if x < 0 { return 0; }
    if x > 10000 { return 0; }
    return 1;
}
fn normalize(x: i64, scale: i64) -> i64 {
    if scale == 0 { return 0; }
    return x * 100 / scale;
}
fn threshold(x: i64, t: i64) -> i64 {
    if x >= t { return 1; }
    return 0;
}
fn weight(x: i64, w: i64) -> i64 { return x * w / 100; }
fn accumulate(acc: i64, x: i64) -> i64 { return acc + x; }
fn count_pass(acc: i64, passed: i64) -> i64 { return acc + passed; }
fn compute_avg(total: i64, count: i64) -> i64 {
    if count == 0 { return 0; }
    return total / count;
}
fn grade(avg: i64) -> i64 {
    if avg >= 90 { return 4; }
    if avg >= 80 { return 3; }
    if avg >= 70 { return 2; }
    if avg >= 60 { return 1; }
    return 0;
}
fn format_result(g: i64, avg: i64, count: i64) {
    println!("grade={} avg={} count={}", g, avg, count);
}
fn process(vals: [i64; 5], scale: i64, thresh: i64, w: i64) {
    let mut total: i64 = 0;
    let mut passed: i64 = 0;
    let mut i: i64 = 0;
    while i < 5 {
        if validate_input(vals[i]) == 1 {
            let n: i64 = normalize(vals[i], scale);
            let t: i64 = threshold(n, thresh);
            let weighted: i64 = weight(n, w);
            total = accumulate(total, weighted);
            passed = count_pass(passed, t);
        }
        i = i + 1;
    }
    let avg: i64 = compute_avg(total, 5);
    let g: i64 = grade(avg);
    format_result(g, avg, passed);
}
fn main() {
    let vals: [i64; 5] = [85, 92, 78, 65, 95];
    process(vals, 100, 50, 80);
}''', r'''validate_input()'''))
    bid += 1

    # ============================================================
    # M2: 4-level nested for with break propagation
    # ============================================================
    entries.append((f"B-{bid}", "brace-4level-nest-break", "Four-level nested loop with outer break via flag",
        r'''fn main() {
    let mut found: i64 = 0;
    let mut fi: i64 = 0;
    let mut fj: i64 = 0;
    let mut fk: i64 = 0;
    let mut i: i64 = 0;
    while i < 4 {
        if found == 1 { break; }
        let mut j: i64 = 0;
        while j < 4 {
            if found == 1 { break; }
            let mut k: i64 = 0;
            while k < 4 {
                if i + j + k == 7 {
                    if i * j * k == 8 {
                        fi = i;
                        fj = j;
                        fk = k;
                        found = 1;
                        break;
                    }
                }
                k = k + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    println!("found={} at ({},{},{})", found, fi, fj, fk);
}''', r'''found='''))
    bid += 1

    # ============================================================
    # N2: Signal-safe state machine
    # ============================================================
    entries.append((f"B-{bid}", "trap-state-machine", "Signal-handler-safe state machine with cleanup transitions",
        r'''fn transition(state: i64, event: i64) -> i64 {
    if state == 0 {
        if event == 1 { return 1; }
        return 0;
    }
    if state == 1 {
        if event == 2 { return 2; }
        if event == 99 { return 3; }
        return 1;
    }
    if state == 2 {
        if event == 3 { return 0; }
        if event == 99 { return 3; }
        return 2;
    }
    return 3;
}
fn state_name(s: i64) -> i64 { return s; }
fn main() {
    let events: [i64; 8] = [1, 2, 3, 1, 99, 0, 0, 0];
    let mut state: i64 = 0;
    let mut i: i64 = 0;
    while i < 5 {
        let prev: i64 = state;
        state = transition(state, events[i]);
        println!("{} --[{}]--> {}", prev, events[i], state);
        i = i + 1;
    }
}''', r'''transition()'''))
    bid += 1

    # ============================================================
    # O2: Getopts-style argument parser
    # ============================================================
    entries.append((f"B-{bid}", "cli-getopts-parser", "Getopts-style argument parser with short and long options",
        r'''fn is_flag(arg: i64) -> i64 {
    if arg < 0 { return 1; }
    return 0;
}
fn get_flag_id(arg: i64) -> i64 {
    return 0 - arg;
}
fn main() {
    let args: [i64; 8] = [-1, -2, 100, -3, 200, -4, -5, 300];
    let mut flags: [i64; 6] = [0, 0, 0, 0, 0, 0];
    let mut values: [i64; 6] = [0, 0, 0, 0, 0, 0];
    let mut vi: i64 = 0;
    let mut i: i64 = 0;
    while i < 8 {
        if is_flag(args[i]) == 1 {
            let fid: i64 = get_flag_id(args[i]);
            if fid < 6 {
                flags[fid] = 1;
            }
        } else {
            if vi < 6 {
                values[vi] = args[i];
                vi = vi + 1;
            }
        }
        i = i + 1;
    }
    println!("flags: {} {} {} {} {} {}", flags[0], flags[1], flags[2], flags[3], flags[4], flags[5]);
    println!("values: {} {} {}", values[0], values[1], values[2]);
}''', r'''is_flag()'''))
    bid += 1

    # ============================================================
    # Q2: Numerical - Simpson's rule integration
    # ============================================================
    entries.append((f"B-{bid}", "num-simpsons-rule", "Simpson's rule numerical integration (integer approximation)",
        r'''fn f_quadratic(x: i64) -> i64 { return x * x; }
fn simpsons(a: i64, b: i64, n: i64) -> i64 {
    if n < 2 { return 0; }
    let h: i64 = (b - a) / n;
    if h == 0 { return 0; }
    let mut sum: i64 = f_quadratic(a) + f_quadratic(b);
    let mut i: i64 = 1;
    while i < n {
        let x: i64 = a + i * h;
        if i % 2 == 0 {
            sum = sum + 2 * f_quadratic(x);
        } else {
            sum = sum + 4 * f_quadratic(x);
        }
        i = i + 1;
    }
    return sum * h / 3;
}
fn main() {
    let r1: i64 = simpsons(0, 10, 10);
    let r2: i64 = simpsons(0, 100, 100);
    let r3: i64 = simpsons(1, 5, 4);
    println!("int_0_10={} int_0_100={} int_1_5={}", r1, r2, r3);
}''', r'''f_quadratic()'''))
    bid += 1

    # ============================================================
    # R2: Symbolic - Bitwise operations with shift simulation
    # ============================================================
    entries.append((f"B-{bid}", "symbolic-bitwise-ops", "Bitwise-like operations using arithmetic (and, or, xor)",
        r'''fn bit_and(a: i64, b: i64) -> i64 {
    let mut result: i64 = 0;
    let mut bit: i64 = 1;
    let mut i: i64 = 0;
    while i < 16 {
        let a_bit: i64 = (a / bit) % 2;
        let b_bit: i64 = (b / bit) % 2;
        if a_bit == 1 {
            if b_bit == 1 {
                result = result + bit;
            }
        }
        bit = bit * 2;
        i = i + 1;
    }
    return result;
}
fn bit_or(a: i64, b: i64) -> i64 {
    let mut result: i64 = 0;
    let mut bit: i64 = 1;
    let mut i: i64 = 0;
    while i < 16 {
        let a_bit: i64 = (a / bit) % 2;
        let b_bit: i64 = (b / bit) % 2;
        if a_bit == 1 {
            result = result + bit;
        } else if b_bit == 1 {
            result = result + bit;
        }
        bit = bit * 2;
        i = i + 1;
    }
    return result;
}
fn main() {
    println!("and(12,10)={}", bit_and(12, 10));
    println!("and(255,15)={}", bit_and(255, 15));
    println!("or(12,10)={}", bit_or(12, 10));
    println!("or(255,15)={}", bit_or(255, 15));
}''', r'''bit_and()'''))
    bid += 1

    # ============================================================
    # S2: Editor - cursor movement simulation
    # ============================================================
    entries.append((f"B-{bid}", "editor-cursor-move", "Vi-style cursor movement (h/j/k/l) simulation",
        r'''fn move_cursor(row: i64, col: i64, cmd: i64, max_row: i64, max_col: i64) -> i64 {
    if cmd == 104 {
        if col > 0 { return (row * 1000) + col - 1; }
    }
    if cmd == 108 {
        if col < max_col - 1 { return (row * 1000) + col + 1; }
    }
    if cmd == 107 {
        if row > 0 { return ((row - 1) * 1000) + col; }
    }
    if cmd == 106 {
        if row < max_row - 1 { return ((row + 1) * 1000) + col; }
    }
    return row * 1000 + col;
}
fn main() {
    let mut pos: i64 = 5005;
    let cmds: [i64; 6] = [108, 108, 106, 106, 104, 107];
    let mut i: i64 = 0;
    while i < 6 {
        pos = move_cursor(pos / 1000, pos % 1000, cmds[i], 24, 80);
        println!("row={} col={}", pos / 1000, pos % 1000);
        i = i + 1;
    }
}''', r'''move_cursor()'''))
    bid += 1

    # ============================================================
    # T2: Function composition chain
    # ============================================================
    entries.append((f"B-{bid}", "fp-compose-chain", "Function composition with 5-level chain",
        r'''fn add3(x: i64) -> i64 { return x + 3; }
fn mul2(x: i64) -> i64 { return x * 2; }
fn sub1(x: i64) -> i64 { return x - 1; }
fn sqr(x: i64) -> i64 { return x * x; }
fn neg(x: i64) -> i64 { return 0 - x; }
fn compose_5(x: i64) -> i64 {
    return neg(sqr(sub1(mul2(add3(x)))));
}
fn main() {
    let vals: [i64; 4] = [0, 1, 5, 10];
    let mut i: i64 = 0;
    while i < 4 {
        let r: i64 = compose_5(vals[i]);
        println!("f({})={}", vals[i], r);
        i = i + 1;
    }
}''', r'''add3()'''))
    bid += 1

    # ============================================================
    # U2: Invariant-verified bounded recursion
    # ============================================================
    entries.append((f"B-{bid}", "prove-bounded-recursion", "Recursion with decreasing variant proving termination",
        r'''fn ackermann_bounded(m: i64, n: i64, fuel: i64) -> i64 {
    if fuel <= 0 { return -1; }
    if m == 0 { return n + 1; }
    if n == 0 { return ackermann_bounded(m - 1, 1, fuel - 1); }
    let inner: i64 = ackermann_bounded(m, n - 1, fuel - 1);
    if inner == -1 { return -1; }
    return ackermann_bounded(m - 1, inner, fuel - 1);
}
fn main() {
    println!("A(0,0)={}", ackermann_bounded(0, 0, 100));
    println!("A(1,1)={}", ackermann_bounded(1, 1, 100));
    println!("A(2,2)={}", ackermann_bounded(2, 2, 100));
    println!("A(3,1)={}", ackermann_bounded(3, 1, 100));
    println!("A(3,3)={}", ackermann_bounded(3, 3, 500));
}''', r'''ackermann_bounded()'''))
    bid += 1

    # ============================================================
    # V2: Saturating arithmetic (clippy-safe)
    # ============================================================
    entries.append((f"B-{bid}", "clippy-saturating-arith", "Saturating arithmetic operations preventing overflow",
        r'''fn saturating_add(a: i64, b: i64) -> i64 {
    let max: i64 = 999999;
    let min: i64 = -999999;
    if b > 0 {
        if a > max - b { return max; }
    }
    if b < 0 {
        if a < min - b { return min; }
    }
    return a + b;
}
fn saturating_sub(a: i64, b: i64) -> i64 {
    return saturating_add(a, 0 - b);
}
fn main() {
    println!("add: {} {} {} {}", saturating_add(100, 200), saturating_add(999990, 100), saturating_add(-999990, -100), saturating_add(0, 0));
    println!("sub: {} {} {}", saturating_sub(100, 200), saturating_sub(-999990, 100), saturating_sub(500, -500));
}''', r'''saturating_add()'''))
    bid += 1

    # ============================================================
    # W2: C-style struct simulation with array-of-arrays
    # ============================================================
    entries.append((f"B-{bid}", "c-style-struct-sim", "C-style struct using arrays with field offset access",
        r'''fn struct_get(data: [i64; 8], field: i64) -> i64 {
    return data[field];
}
fn struct_set_field(old_val: i64, new_val: i64) -> i64 {
    return new_val;
}
fn point_distance_sq(p1: [i64; 8], p2: [i64; 8]) -> i64 {
    let dx: i64 = struct_get(p1, 0) - struct_get(p2, 0);
    let dy: i64 = struct_get(p1, 1) - struct_get(p2, 1);
    return dx * dx + dy * dy;
}
fn main() {
    let p1: [i64; 8] = [3, 4, 0, 0, 0, 0, 0, 0];
    let p2: [i64; 8] = [6, 8, 0, 0, 0, 0, 0, 0];
    let dist_sq: i64 = point_distance_sq(p1, p2);
    println!("p1=({},{}) p2=({},{}) dist_sq={}", struct_get(p1, 0), struct_get(p1, 1), struct_get(p2, 0), struct_get(p2, 1), dist_sq);
}''', r'''struct_get()'''))
    bid += 1

    return entries, bid


def gen_makefile_r2(start_bid):
    entries = []
    bid = start_bid

    # Makefile with shell function and quoting
    entries.append((f"M-{bid}", "make-shell-fn-quote", "Makefile using shell functions with complex quoting",
        "Makefile",
        '''SRCS := $(wildcard src/*.c)
OBJS := $(patsubst src/%.c,build/%.o,$(SRCS))

define compile_template
build/$(1).o: src/$(1).c
\t@mkdir -p build
\t$$(CC) $$(CFLAGS) -c $$< -o $$@
endef

$(foreach src,$(basename $(notdir $(SRCS))),$(eval $(call compile_template,$(src))))

.PHONY: all clean
all: $(OBJS)
\t@echo "Built $(words $(OBJS)) objects"

clean:
\trm -rf build''',
        '''SRCS :='''))
    bid += 1

    # Recursive Makefile with submakes
    entries.append((f"M-{bid}", "make-recursive-submake", "Recursive Makefile with sub-directory builds",
        "Makefile",
        '''SUBDIRS := lib app test

.PHONY: all clean $(SUBDIRS)

all: $(SUBDIRS)

$(SUBDIRS):
\t$(MAKE) -C $@ all

clean:
\t@for dir in $(SUBDIRS); do $(MAKE) -C $$dir clean; done

app: lib
test: lib app''',
        '''SUBDIRS :='''))
    bid += 1

    # Makefile with auto-dependency generation
    entries.append((f"M-{bid}", "make-auto-deps", "Makefile with automatic header dependency generation",
        "Makefile",
        '''CC := gcc
CFLAGS := -Wall -MMD -MP
SRCS := main.c parser.c lexer.c
OBJS := $(SRCS:.c=.o)
DEPS := $(SRCS:.c=.d)

.PHONY: all clean

all: compiler

compiler: $(OBJS)
\t$(CC) -o $@ $^

%.o: %.c
\t$(CC) $(CFLAGS) -c $< -o $@

-include $(DEPS)

clean:
\trm -f $(OBJS) $(DEPS) compiler''',
        '''CC :='''))
    bid += 1

    return entries, bid


def gen_dockerfile_r2(start_bid):
    entries = []
    bid = start_bid

    # Dockerfile with complex COPY and chained RUN
    entries.append((f"D-{bid}", "docker-chained-run", "Dockerfile with chained RUN commands and layer optimization",
        "Dockerfile",
        '''FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \\
    apt-get install -y --no-install-recommends \\
        build-essential \\
        curl \\
        git \\
        python3 \\
        python3-pip && \\
    rm -rf /var/lib/apt/lists/* && \\
    apt-get clean

WORKDIR /app
COPY requirements.txt .
RUN pip3 install --no-cache-dir -r requirements.txt

COPY . .
RUN python3 -m pytest tests/ && \\
    python3 setup.py install

EXPOSE 5000
CMD ["python3", "-m", "flask", "run", "--host=0.0.0.0"]''',
        '''FROM ubuntu:'''))
    bid += 1

    # Multi-stage with build cache optimization
    entries.append((f"D-{bid}", "docker-cache-optimize", "Multi-stage Dockerfile with build cache optimization layers",
        "Dockerfile",
        '''FROM golang:1.22-alpine AS builder
RUN apk add --no-cache git make

WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download

COPY . .
ARG VERSION=dev
RUN CGO_ENABLED=0 go build -ldflags="-s -w -X main.version=${VERSION}" -o /app ./cmd/server

FROM alpine:3.19 AS certs
RUN apk add --no-cache ca-certificates

FROM scratch
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app /app
USER 65534:65534
ENTRYPOINT ["/app"]''',
        '''FROM golang:'''))
    bid += 1

    # Dockerfile with ONBUILD and complex ENV
    entries.append((f"D-{bid}", "docker-onbuild-env", "Dockerfile with ONBUILD triggers and complex ENV layering",
        "Dockerfile",
        '''FROM python:3.12-slim AS base
ENV PYTHONDONTWRITEBYTECODE=1 \\
    PYTHONUNBUFFERED=1 \\
    PIP_NO_CACHE_DIR=1 \\
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /app

FROM base AS deps
COPY pyproject.toml poetry.lock ./
RUN pip install poetry && \\
    poetry config virtualenvs.create false && \\
    poetry install --no-dev --no-interaction

FROM base AS dev
COPY pyproject.toml poetry.lock ./
RUN pip install poetry && \\
    poetry config virtualenvs.create false && \\
    poetry install --no-interaction
COPY . .
CMD ["pytest", "--cov=app"]

FROM deps AS production
COPY app/ app/
HEALTHCHECK --interval=10s CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8000/health')"
CMD ["gunicorn", "app.main:app", "-w", "4", "-b", "0.0.0.0:8000"]''',
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
    makefile_entries, next_bid = gen_makefile_r2(next_bid)
    dockerfile_entries, next_bid = gen_dockerfile_r2(next_bid)

    print(f"// Round 2: {len(bash_entries)} bash + {len(makefile_entries)} makefile + {len(dockerfile_entries)} dockerfile = {len(bash_entries) + len(makefile_entries) + len(dockerfile_entries)} entries")
    print(f"// B-IDs: B-{NEXT_ID}..B/M/D-{next_bid - 1}")
    print(f"// Expansion function: {EXPANSION_NUM}")
    print()
    print(emit_rust_code(bash_entries, makefile_entries, dockerfile_entries, EXPANSION_NUM))
    print()
    print(f"// Call in load_full():")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_bash();")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_makefile();")
    print(f"//   registry.load_expansion{EXPANSION_NUM}_dockerfile();")
