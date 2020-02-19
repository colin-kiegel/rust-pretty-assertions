//! # Pretty Assertions
//!
//! When writing tests in Rust, you'll probably use `assert_eq!(a, b)` _a lot_.
//!
//! If such a test fails, it will present all the details of `a` and `b`.
//! But you have to spot the differences yourself, which is not always straightforward,
//! like here:
//!
//! ![standard assertion](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions/v0.6.1/examples/standard_assertion.png)
//!
//! Wouldn't that task be _much_ easier with a colorful diff?
//!
//! ![pretty assertion](https://raw.githubusercontent.com/colin-kiegel/rust-pretty-assertions/v0.6.1/examples/pretty_assertion.png)
//!
//! Yep — and you only need **one line of code** to make it happen:
//!
//! ```rust,ignore
//! use pretty_assertions::{assert_eq, assert_ne};
//! ```
//!
//! <details>
//! <summary>Show the example behind the screenshots above.</summary>
//!
//! ```rust,ignore
//! // 1. add the `pretty_assertions` dependency to `Cargo.toml`.
//! // 2. insert this line at the top of each module, as needed
//! use pretty_assertions::{assert_eq, assert_ne};
//!
//! fn main() {
//!     #[derive(Debug, PartialEq)]
//!     struct Foo {
//!         lorem: &'static str,
//!         ipsum: u32,
//!         dolor: Result<String, String>,
//!     }
//!
//!     let x = Some(Foo { lorem: "Hello World!", ipsum: 42, dolor: Ok("hey".to_string())});
//!     let y = Some(Foo { lorem: "Hello Wrold!", ipsum: 42, dolor: Ok("hey ho!".to_string())});
//!
//!     assert_eq!(x, y);
//! }
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
//! ```rust,ignore
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

extern crate ansi_term;
extern crate difference;

#[cfg(windows)]
extern crate ctor;
#[cfg(windows)]
extern crate output_vt100;

mod format_changeset;

pub use ansi_term::Style;

pub use crate::format_changeset::Comparison; // private use; but required to be public for use in exported macros
pub use crate::format_changeset::Config; // private use; but required to be public for use in exported macros

#[cfg(windows)]
use ctor::*;
#[cfg(windows)]
#[ctor]
fn init() {
    output_vt100::try_init().ok(); // Do not panic on fail
}

#[cfg(not(feature = "labels"))]
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_labels_assert_eq!(left: $left, right: $right, $($arg)*)
    });
    ($left:expr, $right:expr) => ({
        $crate::with_labels_assert_eq!(left: $left, right: $right)
    });
}

#[cfg(feature = "labels")]
#[macro_export]
macro_rules! assert_eq {
    ($($arg:tt)+) => ({
        $crate::with_labels_assert_eq!($($arg)+)
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! with_labels_assert_eq_impl_ {
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr, $separator:expr, $($arg:tt)+) => ({
        let mut config = $crate::Config::new();
        config.maybe_left_label = Some(stringify!($left_label));
        config.maybe_right_label = Some(stringify!($right_label));
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!("assertion failed: `({} == {})`{}{}\
                        \n\
                        \n{}\
                        \n",
                        config.maybe_left_label.unwrap_or(config.default_left_label),
                        config.maybe_right_label.unwrap_or(config.default_right_label),
                        $separator,
                        format_args!($($arg)+),
                        $crate::Comparison::new(config, left_val, right_val))
                }
            }
        }
    });
}

#[macro_export]
macro_rules! with_labels_assert_eq {
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr, $($arg:tt)+) => ({
        $crate::with_labels_assert_eq_impl_!($left_label: $left, $right_label: $right, ": ", $($arg)+)
    });
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr,) => ({
        $crate::with_labels_assert_eq_impl_!($left_label: $left, $right_label: $right, "", "")
    });
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr) => ({
        $crate::with_labels_assert_eq_impl_!($left_label: $left, $right_label: $right, "", "")
    });
    // ($left:ident, $right_label:ident : $right:expr, $($arg:tt)*) => ({
    //     $crate::with_labels_assert_eq!($left: $left, $right_label: $right, $($arg)*)
    // });
    // ($left:ident, $right_label:ident : $right:expr) => ({
    //     $crate::with_labels_assert_eq!($left: $left, $right_label: $right)
    // });
    // ($left_label:ident : $left:expr, $right:ident, $($arg:tt)*) => ({
    //     $crate::with_labels_assert_eq!($left_label: $left, $right: $right, $($arg)*)
    // });
    // ($left_label:ident : $left:expr, $right:ident) => ({
    //     $crate::with_labels_assert_eq!($left_label: $left, $right: $right)
    // });
    ($left:ident, $right:ident, $($arg:tt)*) => ({
        $crate::with_labels_assert_eq!($left: $left, $right: $right, $($arg)*)
    });
    ($left:ident, $right:ident) => ({
        $crate::with_labels_assert_eq!($left: $left, $right: $right)
    });
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_labels_assert_eq!(left: $left, right: $right, $($arg)*)
    });
    ($left:expr, $right:expr) => ({
        $crate::with_labels_assert_eq!(left: $left, right: $right)
    });
}

#[cfg(not(feature = "labels"))]
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_labels_assert_ne!(left: $left, right: $right, $($arg)*)
    });
    ($left:expr, $right:expr) => ({
        $crate::with_labels_assert_ne!(left: $left, right: $right)
    });
}

#[cfg(feature = "labels")]
#[macro_export]
macro_rules! assert_ne {
    ($($arg:tt)+) => ({
        $crate::with_labels_assert_ne!($($arg)+)
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! with_labels_assert_ne_impl_ {
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr, $separator:expr, $($arg:tt)+) => ({
        let mut config = $crate::Config::new();
        config.maybe_left_label = Some(stringify!($left_label));
        config.maybe_right_label = Some(stringify!($right_label));
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if *left_val == *right_val {
                    let left_dbg = format!("{:?}", *left_val);
                    let right_dbg = format!("{:?}", *right_val);
                    if left_dbg != right_dbg {
                        panic!("assertion failed: `({} != {})`{}{}\
                            \n\
                            \n{}\
                            \n{}: According to the `PartialEq` implementation, both of the values \
                            are partially equivalent, even if the `Debug` outputs differ.\
                            \n\
                            \n",
                            config.maybe_left_label.unwrap_or(config.default_left_label),
                            config.maybe_right_label.unwrap_or(config.default_right_label),
                            $separator,
                            format_args!($($arg)+),
                            $crate::Comparison::new(config, left_val, right_val),
                            $crate::Style::new()
                                .bold()
                                .underline()
                                .paint("Note"))
                    }
                    panic!("assertion failed: `({} != {})`{}{}\
                        \n\
                        \n{}:\
                        \n{:#?}\
                        \n\
                        \n",
                        config.maybe_left_label.unwrap_or(config.default_left_label),
                        config.maybe_right_label.unwrap_or(config.default_right_label),
                        $separator,
                        format_args!($($arg)+),
                        $crate::Style::new().bold().paint("Both sides"),
                        left_val)
                }
            }
        }
    });
}

#[macro_export]
macro_rules! with_labels_assert_ne {
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr, $($arg:tt)+) => ({
        $crate::with_labels_assert_ne_impl_!($left_label: $left, $right_label: $right, ": ", $($arg)+)
    });
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr,) => ({
        $crate::with_labels_assert_ne_impl_!($left_label: $left, $right_label: $right, "", "")
    });
    ($left_label:ident : $left:expr, $right_label:ident : $right:expr) => ({
        $crate::with_labels_assert_ne_impl_!($left_label: $left, $right_label: $right, "", "")
    });
    // ($left:ident, $right_label:ident : $right:expr, $($arg:tt)*) => ({
    //     $crate::with_labels_assert_ne!($left: $left, $right_label: $right, $($arg:tt)*)
    // });
    // ($left:ident, $right_label:ident : $right:expr) => ({
    //     $crate::with_labels_assert_ne!($left: $left, $right_label: $right)
    // });
    // ($left_label:ident : $left:expr, $right:ident, $($arg:tt)*) => ({
    //     $crate::with_labels_assert_ne!($left_label: $left, $right: $right, $($arg:tt)*)
    // });
    // ($left_label:ident : $left:expr, $right:ident) => ({
    //     $crate::with_labels_assert_ne!($left_label: $left, $right: $right)
    // });
    ($left:ident, $right:ident, $($arg:tt)*) => ({
        $crate::with_labels_assert_ne!($left: $left, $right: $right, $($arg)*)
    });
    ($left:ident, $right:ident) => ({
        $crate::with_labels_assert_ne!($left: $left, $right: $right)
    });
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_labels_assert_ne!(left: $left, right: $right, $($arg)*)
    });
    ($left:expr, $right:expr) => ({
        $crate::with_labels_assert_ne!(left: $left, right: $right)
    });
}
