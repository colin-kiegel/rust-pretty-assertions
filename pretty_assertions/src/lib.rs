//! # Pretty Assertions
//!
//! When writing tests in Rust, you'll probably use `assert_eq!(a, b)` _a lot_.
//!
//! If such a test fails, it will present all the details of `a` and `b`.
//! But you have to spot the differences yourself, which is not always straightforward,
//! like here:
//!
//! ![standard assertion](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions/2d2357ff56d22c51a86b2f1cfe6efcee9f5a8081/examples/standard_assertion.png)
//!
//! Wouldn't that task be _much_ easier with a colorful diff?
//!
//! ![pretty assertion](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions/2d2357ff56d22c51a86b2f1cfe6efcee9f5a8081/examples/pretty_assertion.png)
//!
//! Yep — and you only need **one line of code** to make it happen:
//!
//! ```rust
//! use pretty_assertions::{assert_eq, assert_ne};
//! ```
//!
//! <details>
//! <summary>Show the example behind the screenshots above.</summary>
//!
//! ```rust,should_panic
//! // 1. add the `pretty_assertions` dependency to `Cargo.toml`.
//! // 2. insert this line at the top of each module, as needed
//! use pretty_assertions::{assert_eq, assert_ne};
//!
//! #[derive(Debug, PartialEq)]
//! struct Foo {
//!     lorem: &'static str,
//!     ipsum: u32,
//!     dolor: Result<String, String>,
//! }
//!
//! let x = Some(Foo { lorem: "Hello World!", ipsum: 42, dolor: Ok("hey".to_string())});
//! let y = Some(Foo { lorem: "Hello Wrold!", ipsum: 42, dolor: Ok("hey ho!".to_string())});
//!
//! assert_eq!(x, y);
//! ```
//! </details>
//!
//! ## Tip
//!
//! Specify it as [`[dev-dependencies]`](http://doc.crates.io/specifying-dependencies.html#development-dependencies)
//! and it will only be used for compiling tests, examples, and benchmarks.
//! This way the compile time of `cargo build` won't be affected!
//!
//! Also add `#[cfg(test)]` to your `use` statements, like this:
//!
//! ```rust
//! #[cfg(test)]
//! use pretty_assertions::{assert_eq, assert_ne};
//! ```
//!
//! ## Note
//!
//! * Since `Rust 2018` edition, you need to declare
//!   `use pretty_assertions::{assert_eq, assert_ne};` per module.
//!   Before you would write `#[macro_use] extern crate pretty_assertions;`.
//! * The replacement is only effective in your own crate, not in other libraries
//!   you include.
//! * `assert_ne` is also switched to multi-line presentation, but does _not_ show
//!   a diff.

#![deny(clippy::all, missing_docs, unsafe_code)]

pub use ansi_term::Style;
use std::fmt::{self, Debug, Display};

mod printer;

#[cfg(windows)]
use ctor::*;
#[cfg(windows)]
#[ctor]
fn init() {
    output_vt100::try_init().ok(); // Do not panic on fail
}

/// A comparison of two values.
///
/// Where both values implement `Debug`, the comparison can be displayed as a pretty diff.
///
/// ```
/// use pretty_assertions::Comparison;
///
/// print!("{}", Comparison::new(&123, &134));
/// ```
///
/// The values may have different types, although in practice they are usually the same.
pub struct Comparison<'a, TLeft, TRight>
where
    TLeft: ?Sized,
    TRight: ?Sized,
{
    left: &'a TLeft,
    right: &'a TRight,
}

impl<'a, TLeft, TRight> Comparison<'a, TLeft, TRight>
where
    TLeft: ?Sized,
    TRight: ?Sized,
{
    /// Store two values to be compared in future.
    ///
    /// Expensive diffing is deferred until calling `Debug::fmt`.
    pub fn new(left: &'a TLeft, right: &'a TRight) -> Comparison<'a, TLeft, TRight> {
        Comparison { left, right }
    }
}

impl<'a, TLeft, TRight> Display for Comparison<'a, TLeft, TRight>
where
    TLeft: Debug + ?Sized,
    TRight: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // To diff arbitary types, render them as debug strings
        let left_debug = format!("{:#?}", self.left);
        let right_debug = format!("{:#?}", self.right);
        // And then diff the debug output
        printer::write_header(f)?;
        printer::write_lines(f, &left_debug, &right_debug)
    }
}

pretty_assertions_derive::derive_assert_eq! {
    /// Asserts that two expressions are equal to each other (using [`PartialEq`]).
    ///
    /// On panic, this macro will print a diff derived from [`Debug`] representation of
    /// each value.
    ///
    /// This is a drop in replacement for [`std::assert_eq!`].
    /// You can provide a custom panic message if desired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pretty_assertions::assert_eq;
    ///
    /// let a = 3;
    /// let b = 1 + 2;
    /// assert_eq!(a, b);
    ///
    /// assert_eq!(a, b, "we are testing addition with {} and {}", a, b);
    /// ```
    (assert_eq, |left_val, right_val, has_message, message| {
        ::std::panic!("assertion failed: `(left == right)`{}{}\
           \n\
           \n{}\
           \n",
           if has_message { ": " } else { "" },
           message,
           $crate::Comparison::new(left_val, right_val)
        )
    })
}

pretty_assertions_derive::derive_assert_ne! {
    /// Asserts that two expressions are not equal to each other (using [`PartialEq`]).
    ///
    /// On panic, this macro will print the values of the expressions with their
    /// [`Debug`] representations.
    ///
    /// This is a drop in replacement for [`std::assert_ne!`].
    /// You can provide a custom panic message if desired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pretty_assertions::assert_ne;
    ///
    /// let a = 3;
    /// let b = 2;
    /// assert_ne!(a, b);
    ///
    /// assert_ne!(a, b, "we are testing that the values are not equal");
    /// ```
    (assert_ne, |left_val, right_val, has_message, message| {
        let left_dbg = ::std::format!("{:?}", &*left_val);
        let right_dbg = ::std::format!("{:?}", &*right_val);
        if left_dbg != right_dbg {
            ::std::panic!("assertion failed: `(left != right)`{}{}\
                \n\
                \n{}\
                \n{}: According to the `PartialEq` implementation, both of the values \
                  are partially equivalent, even if the `Debug` outputs differ.\
                \n\
                \n",
                if has_message { ": " } else { "" },
                message,
                $crate::Comparison::new(left_val, right_val),
                $crate::Style::new()
                    .bold()
                    .underline()
                    .paint("Note")
            )
        }

        ::std::panic!("assertion failed: `(left != right)`{}{}\
            \n\
            \n{}:\
            \n{:#?}\
            \n\
            \n",
            if has_message { ": " } else { "" },
            message,
            $crate::Style::new().bold().paint("Both sides"),
            left_val
        )
    })
}
