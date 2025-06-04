fn main() {
    use rash::{transpile, Config};
    
    let deep_nesting = "fn main() { let x = ".to_string() + &"(".repeat(50) + "42" + &")".repeat(50) + "; }";
    let result = transpile(&deep_nesting, Config::default());
    println!("Deep nesting (50 levels): {:?}", result.is_ok());
    
    let incomplete = "fn main() { let x = ((((((((((((((((((((((((((((((";
    let result2 = transpile(incomplete, Config::default());
    println!("Incomplete parens: {:?}", result2.is_ok());
}