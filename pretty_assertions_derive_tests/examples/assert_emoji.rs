#![allow(clippy::eq_op)]

use pretty_assertions_derive_tests::{assert_eq_emoji, assert_ne_emoji};

fn main() {
    println!("Deliberate `assert_eq` panic:");
    println!("---");
    let result = std::panic::catch_unwind(|| assert_eq_emoji!(3, 2));
    assert!(result.is_err(), "example did not panic");
    println!();

    println!("Deliberate `assert_ne` panic:");
    println!("---");
    let result = std::panic::catch_unwind(|| assert_ne_emoji!(3, 3, "additional {}", "details"));
    assert!(result.is_err(), "example did not panic");
    println!();
}
