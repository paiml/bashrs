#!/usr/bin/env python3
"""Round 35: 25 Bash + 3 Makefile + 3 Dockerfile = 31 entries
B-13715..B-13739, M-745..M-747, D-656..D-658
Method names: load_expansion69_bash, load_expansion57_makefile, load_expansion57_dockerfile

Categories:
1. Ackermann-class recursion (deeply recursive, mutual recursive)
2. Complex match expressions (match with guards, nested match)
3. Array + for-in combinations
4. Multi-function programs (5+ functions cooperating)
5. Numerical edge cases (modular arithmetic, prime testing, perfect numbers, digit sums)
6. Accumulator patterns (histogram counting, frequency)
7. String arrays iterated
8. Boolean chain conditions (complex if with && and ||)
9. Nested control flow (while inside match arm, if inside for-in)
10. Early return patterns (multiple return points)
"""

def fmt(s):
    """Format a Rust string for corpus entry."""
    if '"' in s or '\\' in s:
        if '"#' in s:
            return f'r##"{s}"##'
        return f'r#"{s}"#'
    return f'r#"{s}"#'


def main():
    lines = []

    # =========================================================================
    # BASH entries B-13715..B-13739
    # =========================================================================
    lines.append("    pub fn load_expansion69_bash(&mut self) {")

    bid = 13715

    # --- 1. Ackermann-class recursion ---
    # B-13715: Ackermann function (bounded)
    code = r'''fn ack(m: i32, n: i32) -> i32 {
    if m == 0 {
        return n + 1;
    }
    if n == 0 {
        return ack(m - 1, 1);
    }
    ack(m - 1, ack(m, n - 1))
}

fn main() {
    let mut m = 0;
    while m <= 3 {
        let mut n = 0;
        while n <= 3 {
            println!("{}", ack(m, n));
            n = n + 1;
        }
        m = m + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "ackermann_bounded", "Ackermann function for m<=3 n<=3",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "ack()"));')
    bid += 1

    # B-13716: Mutual recursion — ping/pong countdown
    code = r'''fn ping(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    pong(n - 1) + 1
}

fn pong(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    ping(n - 2) + 2
}

fn main() {
    let mut i = 0;
    while i <= 12 {
        println!("{}: {}", i, ping(i));
        i = i + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "mutual_ping_pong", "Mutual recursion ping/pong countdown",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "ping()"));')
    bid += 1

    # B-13717: Deep recursion — recursive sum of digits of factorial-like product
    code = r'''fn digit_sum(n: i32) -> i32 {
    if n < 10 {
        return n;
    }
    n % 10 + digit_sum(n / 10)
}

fn recursive_product(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    let p = n * recursive_product(n - 1);
    digit_sum(p)
}

fn main() {
    let mut i = 1;
    while i <= 8 {
        println!("{}: {}", i, recursive_product(i));
        i = i + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "deep_recursive_digit_product", "Recursive product with digit sum reduction",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "digit_sum()"));')
    bid += 1

    # --- 2. Complex match expressions ---
    # B-13718: Match with multiple return values via dispatch
    code = r'''fn op_dispatch(code: i32, a: i32, b: i32) -> i32 {
    match code {
        1 => a + b,
        2 => a - b,
        3 => a * b,
        4 => a / b,
        5 => a % b,
        _ => 0,
    }
}

fn main() {
    let ops = [1, 2, 3, 4, 5, 6];
    for op in ops {
        println!("{}: {}", op, op_dispatch(op, 20, 4));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "match_op_dispatch", "Match-based arithmetic operator dispatch",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "op_dispatch()"));')
    bid += 1

    # B-13719: Nested match — day category from day number
    code = r'''fn day_type(day: i32) -> i32 {
    match day {
        1 => match 1 { 1 => 10, _ => 0, },
        2 => match 2 { 2 => 20, _ => 0, },
        3 => 30,
        4 => 40,
        5 => 50,
        6 => match 6 { 6 => 60, _ => 0, },
        7 => match 7 { 7 => 70, _ => 0, },
        _ => 0,
    }
}

fn main() {
    let mut d = 1;
    while d <= 8 {
        println!("{}: {}", d, day_type(d));
        d = d + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "match_nested_day", "Nested match for day classification",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "day_type()"));')
    bid += 1

    # B-13720: Match as expression in let binding
    code = r'''fn classify(score: i32) -> i32 {
    let grade = match score / 10 {
        10 => 5,
        9 => 5,
        8 => 4,
        7 => 3,
        6 => 2,
        _ => 1,
    };
    grade * 20
}

fn main() {
    let scores = [100, 95, 82, 74, 65, 51];
    for s in scores {
        println!("{}: {}", s, classify(s));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "match_let_grade", "Match as let-binding expression for grading",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "classify()"));')
    bid += 1

    # --- 3. Array + for-in combinations ---
    # B-13721: For-in with function call on each element
    code = r'''fn cube(x: i32) -> i32 {
    x * x * x
}

fn main() {
    let nums = [2, 3, 5, 7, 11];
    let mut total = 0;
    for n in nums {
        total = total + cube(n);
    }
    println!("{}", total);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "forin_fn_cube_sum", "For-in with cube function accumulation",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "cube()"));')
    bid += 1

    # B-13722: For-in inside a function
    code = r'''fn sum_above(threshold: i32) -> i32 {
    let data = [15, 3, 27, 8, 42, 1, 36, 11];
    let mut total = 0;
    for d in data {
        if d > threshold {
            total = total + d;
        }
    }
    total
}

fn main() {
    println!("{}", sum_above(10));
    println!("{}", sum_above(20));
    println!("{}", sum_above(30));
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "forin_in_fn_threshold", "For-in inside function with threshold filter",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "sum_above()"));')
    bid += 1

    # B-13723: String array iteration
    code = r'''fn main() {
    let fruits = ["apple", "banana", "cherry", "date", "elderberry"];
    for f in fruits {
        println!("fruit: {}", f);
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "string_array_forin", "Iterate string array with for-in",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "for f in"));')
    bid += 1

    # --- 4. Multi-function programs (5+ functions) ---
    # B-13724: Pipeline of 5 transforms
    code = r'''fn add3(x: i32) -> i32 { x + 3 }
fn mul2(x: i32) -> i32 { x * 2 }
fn sub1(x: i32) -> i32 { x - 1 }
fn sqr(x: i32) -> i32 { x * x }
fn halve(x: i32) -> i32 { x / 2 }

fn pipeline(x: i32) -> i32 {
    halve(sqr(sub1(mul2(add3(x)))))
}

fn main() {
    let mut i = 0;
    while i < 6 {
        println!("{}: {}", i, pipeline(i));
        i = i + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "multi_fn_pipeline5", "Pipeline of 5 transform functions",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "pipeline()"));')
    bid += 1

    # B-13725: 6 cooperating functions — stats calculator
    code = r'''fn arr_sum(n: i32) -> i32 {
    let data = [10, 20, 30, 40, 50, 60];
    let mut s = 0;
    let mut i = 0;
    while i < n {
        s = s + data[i];
        i = i + 1;
    }
    s
}

fn arr_min(n: i32) -> i32 {
    let data = [10, 20, 30, 40, 50, 60];
    let mut m = 9999;
    let mut i = 0;
    while i < n {
        if data[i] < m { m = data[i]; }
        i = i + 1;
    }
    m
}

fn arr_max(n: i32) -> i32 {
    let data = [10, 20, 30, 40, 50, 60];
    let mut m = 0;
    let mut i = 0;
    while i < n {
        if data[i] > m { m = data[i]; }
        i = i + 1;
    }
    m
}

fn arr_mean(n: i32) -> i32 {
    arr_sum(n) / n
}

fn arr_range(n: i32) -> i32 {
    arr_max(n) - arr_min(n)
}

fn report(n: i32) -> i32 {
    let s = arr_sum(n);
    let mn = arr_min(n);
    let mx = arr_max(n);
    println!("sum={} min={} max={} mean={} range={}", s, mn, mx, arr_mean(n), arr_range(n));
    s
}

fn main() {
    report(6);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "multi_fn_stats6", "6 cooperating stats functions",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "report()"));')
    bid += 1

    # --- 5. Numerical edge cases ---
    # B-13726: Is prime checker
    code = r'''fn is_prime(n: i32) -> i32 {
    if n < 2 {
        return 0;
    }
    let mut d = 2;
    while d * d <= n {
        if n % d == 0 {
            return 0;
        }
        d = d + 1;
    }
    1
}

fn main() {
    let mut count = 0;
    let mut i = 2;
    while i <= 100 {
        if is_prime(i) == 1 {
            count = count + 1;
            println!("{}", i);
        }
        i = i + 1;
    }
    println!("total: {}", count);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "num_primes_to_100", "Prime number sieve up to 100",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "is_prime()"));')
    bid += 1

    # B-13727: Perfect number checker
    code = r'''fn is_perfect(n: i32) -> i32 {
    let mut sum = 0;
    let mut d = 1;
    while d < n {
        if n % d == 0 {
            sum = sum + d;
        }
        d = d + 1;
    }
    if sum == n { 1 } else { 0 }
}

fn main() {
    let mut i = 2;
    while i <= 500 {
        if is_perfect(i) == 1 {
            println!("{}", i);
        }
        i = i + 1;
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "num_perfect_numbers", "Perfect number detection up to 500",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "is_perfect()"));')
    bid += 1

    # B-13728: Digit sum with modular arithmetic
    code = r'''fn digit_sum(n: i32) -> i32 {
    let mut x = n;
    let mut s = 0;
    while x > 0 {
        s = s + x % 10;
        x = x / 10;
    }
    s
}

fn digital_root(n: i32) -> i32 {
    let mut x = n;
    while x >= 10 {
        x = digit_sum(x);
    }
    x
}

fn main() {
    let tests = [0, 1, 9, 10, 99, 123, 9999, 12345];
    for t in tests {
        println!("{}: {}", t, digital_root(t));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "num_digital_root", "Digital root via iterated digit sum",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "digital_root()"));')
    bid += 1

    # B-13729: Modular exponentiation
    code = r'''fn mod_pow(base: i32, exp: i32, modulus: i32) -> i32 {
    let mut result = 1;
    let mut b = base % modulus;
    let mut e = exp;
    while e > 0 {
        if e % 2 == 1 {
            result = (result * b) % modulus;
        }
        e = e / 2;
        b = (b * b) % modulus;
    }
    result
}

fn main() {
    println!("{}", mod_pow(2, 10, 1000));
    println!("{}", mod_pow(3, 7, 100));
    println!("{}", mod_pow(5, 13, 97));
    println!("{}", mod_pow(7, 20, 1000));
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "num_mod_pow", "Modular exponentiation via binary method",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "mod_pow()"));')
    bid += 1

    # --- 6. Accumulator patterns ---
    # B-13730: Histogram counting via match-based buckets
    code = r'''fn bucket(val: i32) -> i32 {
    match val / 10 {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        _ => 5,
    }
}

fn main() {
    let data = [5, 12, 27, 33, 45, 8, 19, 31, 42, 3, 50, 25, 38, 11, 47];
    let mut b0 = 0;
    let mut b1 = 0;
    let mut b2 = 0;
    let mut b3 = 0;
    let mut b4 = 0;
    let mut b5 = 0;
    for d in data {
        let b = bucket(d);
        if b == 0 { b0 = b0 + 1; }
        if b == 1 { b1 = b1 + 1; }
        if b == 2 { b2 = b2 + 1; }
        if b == 3 { b3 = b3 + 1; }
        if b == 4 { b4 = b4 + 1; }
        if b == 5 { b5 = b5 + 1; }
    }
    println!("{} {} {} {} {} {}", b0, b1, b2, b3, b4, b5);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "acc_histogram_bucket", "Histogram bucketing via match and accumulators",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "bucket()"));')
    bid += 1

    # B-13731: Frequency counting — count even vs odd
    code = r'''fn main() {
    let data = [3, 8, 15, 22, 7, 30, 11, 44, 9, 16, 25, 38];
    let mut even_count = 0;
    let mut odd_count = 0;
    let mut even_sum = 0;
    let mut odd_sum = 0;
    for d in data {
        if d % 2 == 0 {
            even_count = even_count + 1;
            even_sum = even_sum + d;
        } else {
            odd_count = odd_count + 1;
            odd_sum = odd_sum + d;
        }
    }
    println!("even: {} sum={}", even_count, even_sum);
    println!("odd: {} sum={}", odd_count, odd_sum);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "acc_even_odd_freq", "Even/odd frequency and sum accumulation",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "for d in"));')
    bid += 1

    # --- 7. String arrays ---
    # B-13732: String array with counter
    code = r'''fn main() {
    let colors = ["red", "green", "blue", "yellow", "cyan", "magenta"];
    let mut count = 0;
    for c in colors {
        println!("color {}: {}", count, c);
        count = count + 1;
    }
    println!("total: {}", count);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "string_array_colors", "String array color iteration with counter",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "for c in"));')
    bid += 1

    # B-13733: String array of commands
    code = r'''fn main() {
    let commands = ["init", "build", "test", "lint", "deploy"];
    let mut step = 1;
    for cmd in commands {
        println!("step {}: {}", step, cmd);
        step = step + 1;
    }
    println!("done: {} steps", step - 1);
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "string_array_commands", "String array of pipeline commands",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "for cmd in"));')
    bid += 1

    # --- 8. Boolean chain conditions ---
    # B-13734: Complex boolean condition with && and ||
    code = r'''fn classify_age(age: i32) -> i32 {
    if age >= 0 && age < 13 {
        return 1;
    }
    if age >= 13 && age < 18 {
        return 2;
    }
    if age >= 18 && age < 65 {
        return 3;
    }
    if age >= 65 && age <= 120 {
        return 4;
    }
    0
}

fn main() {
    let ages = [0, 5, 13, 17, 18, 30, 65, 100, 121];
    for a in ages {
        println!("{}: {}", a, classify_age(a));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "bool_chain_age_classify", "Age classification with && range conditions",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "classify_age()"));')
    bid += 1

    # B-13735: Multi-condition validation with || chains
    code = r'''fn is_valid(x: i32, y: i32) -> i32 {
    if (x > 0 && y > 0) || (x < 0 && y < 0) {
        return 1;
    }
    if x == 0 || y == 0 {
        return 2;
    }
    0
}

fn main() {
    println!("{}", is_valid(3, 5));
    println!("{}", is_valid(-2, -7));
    println!("{}", is_valid(0, 5));
    println!("{}", is_valid(3, -1));
    println!("{}", is_valid(0, 0));
    println!("{}", is_valid(-4, 8));
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "bool_chain_or_validation", "Multi-condition validation with || chains",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "is_valid()"));')
    bid += 1

    # --- 9. Nested control flow ---
    # B-13736: If inside for-in inside function
    code = r'''fn count_in_range(lo: i32, hi: i32) -> i32 {
    let values = [4, 17, 23, 8, 55, 12, 31, 7, 42, 19];
    let mut count = 0;
    for v in values {
        if v >= lo && v <= hi {
            count = count + 1;
        }
    }
    count
}

fn main() {
    println!("0-10: {}", count_in_range(0, 10));
    println!("10-25: {}", count_in_range(10, 25));
    println!("20-50: {}", count_in_range(20, 50));
    println!("0-100: {}", count_in_range(0, 100));
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "nested_if_in_forin", "If conditions inside for-in loop in function",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "count_in_range()"));')
    bid += 1

    # B-13737: While inside match (dispatch loop iterations)
    code = r'''fn run_task(task: i32) -> i32 {
    let iters = match task {
        1 => 5,
        2 => 10,
        3 => 15,
        _ => 3,
    };
    let mut sum = 0;
    let mut i = 0;
    while i < iters {
        sum = sum + i;
        i = i + 1;
    }
    sum
}

fn main() {
    let tasks = [1, 2, 3, 4];
    for t in tasks {
        println!("task {}: {}", t, run_task(t));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "nested_while_in_match", "While loop controlled by match dispatch",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "run_task()"));')
    bid += 1

    # --- 10. Early return patterns ---
    # B-13738: Multiple return points — input validator
    code = r'''fn validate(code: i32, value: i32) -> i32 {
    if code < 0 {
        return -1;
    }
    if code == 0 {
        return 0;
    }
    if value < 0 {
        return -2;
    }
    if value > 1000 {
        return -3;
    }
    if code > 100 {
        return -4;
    }
    code * value
}

fn main() {
    println!("{}", validate(-1, 50));
    println!("{}", validate(0, 50));
    println!("{}", validate(5, -10));
    println!("{}", validate(5, 2000));
    println!("{}", validate(200, 50));
    println!("{}", validate(5, 50));
    println!("{}", validate(10, 100));
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "early_return_validator", "Multiple early returns for input validation",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "validate()"));')
    bid += 1

    # B-13739: Early return — binary search with guard
    code = r'''fn find_index(target: i32) -> i32 {
    if target < 0 {
        return -1;
    }
    let arr = [2, 5, 8, 12, 16, 23, 38, 56, 72, 91];
    let mut lo = 0;
    let mut hi = 9;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        if arr[mid] == target {
            return mid;
        }
        if arr[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    -1
}

fn main() {
    let tests = [2, 23, 91, 50, 0, 8];
    for t in tests {
        println!("{}: idx={}", t, find_index(t));
    }
}'''
    lines.append(f'        self.entries.push(CorpusEntry::new("B-{bid}", "early_return_bsearch", "Binary search with early return on guard and match",')
    lines.append(f'            CorpusFormat::Bash, CorpusTier::Adversarial,')
    lines.append(f'            {fmt(code)},')
    lines.append(f'            "find_index()"));')
    bid += 1

    lines.append("    }")
    lines.append("")

    # =========================================================================
    # MAKEFILE entries M-745..M-747
    # =========================================================================
    lines.append("    pub fn load_expansion57_makefile(&mut self) {")

    # M-745: Autoconf-like pattern detection
    lines.append('        self.entries.push(CorpusEntry::new("M-745", "mk_autoconf_detect", "Autoconf-like feature detection Makefile",')
    lines.append('            CorpusFormat::Makefile, CorpusTier::Adversarial,')
    lines.append(r'''            r#"HAS_OPENSSL := $(shell pkg-config --exists openssl 2>/dev/null && echo yes || echo no)\nHAS_ZLIB := $(shell pkg-config --exists zlib 2>/dev/null && echo yes || echo no)\nHAS_CURL := $(shell command -v curl >/dev/null 2>&1 && echo yes || echo no)\n\nCFLAGS = -Wall -Wextra -O2\nLDFLAGS =\n\nifeq ($(HAS_OPENSSL),yes)\n  CFLAGS += -DHAVE_OPENSSL $(shell pkg-config --cflags openssl)\n  LDFLAGS += $(shell pkg-config --libs openssl)\nendif\nifeq ($(HAS_ZLIB),yes)\n  CFLAGS += -DHAVE_ZLIB\n  LDFLAGS += -lz\nendif\n\n.PHONY: all config clean\n\nconfig:\n\t@echo "OpenSSL: $(HAS_OPENSSL)"\n\t@echo "Zlib: $(HAS_ZLIB)"\n\t@echo "Curl: $(HAS_CURL)"\n\nall: config myapp\n\nmyapp: main.c\n\t$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)\n\nclean:\n\trm -f myapp"#,''')
    lines.append('            "HAS_OPENSSL"));')

    # M-746: Recursive make with subdirectories
    lines.append('        self.entries.push(CorpusEntry::new("M-746", "mk_recursive_subdirs", "Recursive make across subdirectories",')
    lines.append('            CorpusFormat::Makefile, CorpusTier::Adversarial,')
    lines.append(r'''            r#"SUBDIRS = lib/core lib/io lib/net src\n\n.PHONY: all clean test $(SUBDIRS)\n\nall: $(SUBDIRS)\n\n$(SUBDIRS):\n\t$(MAKE) -C $@ all\n\nsrc: lib/core lib/io lib/net\n\ntest:\n\t@for dir in $(SUBDIRS); do \\\n\t\t$(MAKE) -C $$dir test || exit 1; \\\n\tdone\n\nclean:\n\t@for dir in $(SUBDIRS); do \\\n\t\t$(MAKE) -C $$dir clean; \\\n\tdone\n\ninstall: all\n\tinstall -d $(DESTDIR)/usr/local/bin\n\tinstall -m 755 src/myapp $(DESTDIR)/usr/local/bin/"#,''')
    lines.append('            "SUBDIRS"));')

    # M-747: Parallel jobs with dependency tracking
    lines.append('        self.entries.push(CorpusEntry::new("M-747", "mk_parallel_deps", "Parallel build with fine-grained dependencies",')
    lines.append('            CorpusFormat::Makefile, CorpusTier::Adversarial,')
    lines.append(r'''            r#"SRCS := $(wildcard src/*.c)\nOBJS := $(SRCS:src/%.c=build/%.o)\nDEPS := $(OBJS:.o=.d)\nTARGET = build/app\n\nCFLAGS = -Wall -O2 -MMD -MP\nLDFLAGS = -pthread\n\n.PHONY: all clean\n\nall: $(TARGET)\n\n$(TARGET): $(OBJS) | build\n\t$(CC) $(LDFLAGS) -o $@ $^\n\nbuild/%.o: src/%.c | build\n\t$(CC) $(CFLAGS) -c -o $@ $<\n\nbuild:\n\tmkdir -p build\n\n-include $(DEPS)\n\nclean:\n\trm -rf build\n\n.SUFFIXES:"#,''')
    lines.append('            "DEPS"));')

    lines.append("    }")
    lines.append("")

    # =========================================================================
    # DOCKERFILE entries D-656..D-658
    # =========================================================================
    lines.append("    pub fn load_expansion57_dockerfile(&mut self) {")

    # D-656: Python with wheels
    lines.append('        self.entries.push(CorpusEntry::new("D-656", "dk_python_wheels", "Python app with wheel caching",')
    lines.append('            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
    lines.append(r'            "FROM python:3.12-slim AS builder\nWORKDIR /app\nRUN pip install --no-cache-dir pip-tools\nCOPY requirements.in .\nRUN pip-compile requirements.in -o requirements.txt\nRUN pip wheel --wheel-dir=/wheels -r requirements.txt\n\nFROM python:3.12-slim\nWORKDIR /app\nCOPY --from=builder /wheels /wheels\nCOPY --from=builder /app/requirements.txt .\nRUN pip install --no-cache-dir --no-index --find-links=/wheels -r requirements.txt && rm -rf /wheels\nCOPY src/ src/\nCOPY main.py .\nEXPOSE 8000\nCMD [\"python\", \"main.py\"]",')
    lines.append('            "wheels"));')

    # D-657: Node.js with yarn
    lines.append('        self.entries.push(CorpusEntry::new("D-657", "dk_node_yarn", "Node.js app with yarn berry and PnP",')
    lines.append('            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
    lines.append(r'            "FROM node:22-alpine AS builder\nWORKDIR /app\nRUN corepack enable\nCOPY package.json yarn.lock .yarnrc.yml ./\nCOPY .yarn/ .yarn/\nRUN yarn install --immutable\nCOPY tsconfig.json ./\nCOPY src/ src/\nRUN yarn build\n\nFROM node:22-alpine\nWORKDIR /app\nRUN corepack enable\nCOPY --from=builder /app/package.json /app/yarn.lock /app/.yarnrc.yml ./\nCOPY --from=builder /app/.yarn/ .yarn/\nCOPY --from=builder /app/.pnp.cjs /app/.pnp.loader.mjs ./\nCOPY --from=builder /app/dist/ dist/\nEXPOSE 3000\nCMD [\"yarn\", \"node\", \"dist/index.js\"]",')
    lines.append('            "corepack"));')

    # D-658: Multi-arch build
    lines.append('        self.entries.push(CorpusEntry::new("D-658", "dk_multi_arch", "Multi-architecture build with buildx",')
    lines.append('            CorpusFormat::Dockerfile, CorpusTier::Adversarial,')
    lines.append(r'            "FROM --platform=$BUILDPLATFORM golang:1.22 AS builder\nARG TARGETPLATFORM\nARG BUILDPLATFORM\nARG TARGETOS\nARG TARGETARCH\nWORKDIR /app\nCOPY go.mod go.sum ./\nRUN go mod download\nCOPY . .\nRUN CGO_ENABLED=0 GOOS=${TARGETOS} GOARCH=${TARGETARCH} go build -ldflags=\"-s -w\" -o /app/server ./cmd/server\n\nFROM gcr.io/distroless/static-debian12\nCOPY --from=builder /app/server /server\nEXPOSE 8080\nENTRYPOINT [\"/server\"]",')
    lines.append('            "TARGETARCH"));')

    lines.append("    }")

    print("\n".join(lines))


if __name__ == "__main__":
    main()
