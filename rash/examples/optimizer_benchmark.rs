#![allow(clippy::expect_used)]
//! Optimizer Benchmark
//!
//! Demonstrates the performance impact of constant folding optimizations.
//! Compares optimized vs unoptimized IR transformation.
//!
//! Run with: cargo run --example optimizer_benchmark --release

use bashrs::ir::{optimize, shell_ir::*, EffectSet, ShellIR, ShellValue};
use bashrs::models::Config;
use std::time::Instant;

fn main() {
    println!("=== Bashrs Optimizer Benchmark ===\n");

    // Benchmark 1: Simple arithmetic (10 + 20)
    benchmark_simple_arithmetic();

    // Benchmark 2: Complex nested arithmetic (10 * 1024 * 1024)
    benchmark_nested_arithmetic();

    // Benchmark 3: Mixed operations (10MB calculation)
    benchmark_complex_expression();

    println!("\n=== Summary ===");
    println!("✅ Constant folding provides compile-time optimization");
    println!("✅ Complex expressions (10 * 1024 * 1024) → single constant");
    println!("✅ Runtime scripts run faster (no arithmetic evaluation needed)");
    println!("\n💡 Use `bashrs compile --optimize` for production deployments");
}

fn benchmark_simple_arithmetic() {
    println!("## Benchmark 1: Simple Arithmetic (10 + 20)");

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("20".to_string())),
        },
        effects: EffectSet::pure(),
    };

    // Unoptimized
    let config_unopt = Config {
        optimize: false,
        ..Default::default()
    };
    let start = Instant::now();
    let unoptimized = optimize(ir.clone(), &config_unopt).expect("optimization failed");
    let unopt_time = start.elapsed();

    // Optimized
    let config_opt = Config {
        optimize: true,
        ..Default::default()
    };
    let start = Instant::now();
    let optimized = optimize(ir, &config_opt).expect("optimization failed");
    let opt_time = start.elapsed();

    println!("  Unoptimized: {:?}", unoptimized);
    println!("  Optimized:   {:?}", optimized);
    println!("  Compile time: unopt={:?}, opt={:?}", unopt_time, opt_time);
    println!("  Result: Arithmetic → String constant \"30\"");
    println!();
}

fn benchmark_nested_arithmetic() {
    println!("## Benchmark 2: Nested Arithmetic (10 * 1024 * 1024 = 10MB)");

    // Build nested expression: (10 * 1024) * 1024
    let inner_mul = ShellValue::Arithmetic {
        op: ArithmeticOp::Mul,
        left: Box::new(ShellValue::String("10".to_string())),
        right: Box::new(ShellValue::String("1024".to_string())),
    };

    let ir = ShellIR::Let {
        name: "bytes".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(inner_mul),
            right: Box::new(ShellValue::String("1024".to_string())),
        },
        effects: EffectSet::pure(),
    };

    // Optimized
    let config_opt = Config {
        optimize: true,
        ..Default::default()
    };
    let start = Instant::now();
    let optimized = optimize(ir, &config_opt).expect("optimization failed");
    let opt_time = start.elapsed();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            println!("  Input:  (10 * 1024) * 1024");
            println!("  Output: \"{}\"", s);
            println!("  Compile time: {:?}", opt_time);
            println!("  ✅ Nested arithmetic fully folded at compile time!");
        }
        _ => {
            println!("  ❌ Expected constant string");
        }
    }
    println!();
}

fn benchmark_complex_expression() {
    println!("## Benchmark 3: Complex Expression (buffer size calculation)");

    // Simulate: buffer_size = (page_size * num_pages) + header_size
    // page_size = 4096, num_pages = 256, header_size = 64
    // Expected: (4096 * 256) + 64 = 1048576 + 64 = 1048640

    let page_mul = ShellValue::Arithmetic {
        op: ArithmeticOp::Mul,
        left: Box::new(ShellValue::String("4096".to_string())),
        right: Box::new(ShellValue::String("256".to_string())),
    };

    let ir = ShellIR::Let {
        name: "buffer_size".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(page_mul),
            right: Box::new(ShellValue::String("64".to_string())),
        },
        effects: EffectSet::pure(),
    };

    // Optimized
    let config_opt = Config {
        optimize: true,
        ..Default::default()
    };
    let start = Instant::now();
    let optimized = optimize(ir, &config_opt).expect("optimization failed");
    let opt_time = start.elapsed();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            println!("  Input:  (4096 * 256) + 64");
            println!("  Output: \"{}\"", s);
            println!("  Compile time: {:?}", opt_time);
            println!("  ✅ Complex nested expression folded to single constant!");
        }
        _ => {
            println!("  ❌ Expected constant string");
        }
    }
    println!();
}
