// Fibonacci implementations in Rust

/// Recursive Fibonacci implementation
/// This is not efficient for large numbers due to repeated calculations
pub fn fibonacci_recursive(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2),
    }
}

/// Iterative Fibonacci implementation
/// More efficient than recursive version
pub fn fibonacci_iterative(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    
    let mut a = 0;
    let mut b = 1;
    
    for _ in 1..n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

/// Using the closed-form formula (Binet's formula)
/// This provides O(1) time complexity but is subject to floating-point precision issues
pub fn fibonacci_formula(n: u32) -> u64 {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    ((phi.powi(n as i32) - (1.0 - phi).powi(n as i32)) / 5.0_f64.sqrt()).round() as u64
}

// Example usage
fn main() {
    println!("First 10 Fibonacci numbers:");
    for i in 0..10 {
        println!("fibonacci({}) = {}", i, fibonacci_iterative(i));
    }
} 