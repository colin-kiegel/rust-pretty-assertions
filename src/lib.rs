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

mod comparison;
mod config;

pub use ansi_term::{Color, Colour, Style}; // re-exported for easier/direct usage when customizing Config

pub use crate::comparison::Comparison; // private use; but required to be public for use in exported macros
pub use crate::config::Config; // private use; but required to be public for use in exported macros

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
    ($($arg:tt)+) => ({
        $crate::with_config_assert_eq!($crate::Config { auto_label: false, ..Default::default() }, $($arg)+)
    });
}

#[cfg(feature = "labels")]
#[macro_export]
macro_rules! assert_eq {
    ($($arg:tt)+) => ({
        $crate::with_config_assert_eq!($crate::Config { auto_label: true, ..Default::default() }, $($arg)+)
    });
}

#[macro_export]
macro_rules! with_config_assert_eq {
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr, $($arg:tt)+) => ({
        $crate::_with_config_assert_eq_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, ": ", $($arg)+)
    });
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr,) => ({
        $crate::_with_config_assert_eq_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, "", "")
    });
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr) => ({
        $crate::_with_config_assert_eq_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, "", "")
    });
    // ($config: expr, $left:ident, $label_right:ident : $right:expr, $($arg:tt)*) => ({
    //     $crate::with_config_assert_eq!($config, $left: $left, $label_right: $right, $($arg)*)
    // });
    // ($config: expr, $left:ident, $label_right:ident : $right:expr) => ({
    //     $crate::with_config_assert_eq!($config, $left: $left, $label_right: $right)
    // });
    // ($config: expr, $label_left:ident : $left:expr, $right:ident, $($arg:tt)*) => ({
    //     $crate::with_config_assert_eq!($config, $label_left: $left, $right: $right, $($arg)*)
    // });
    // ($config: expr, $label_left:ident : $left:expr, $right:ident) => ({
    //     $crate::with_config_assert_eq!($config, $label_left: $left, $right: $right)
    // });
    ($config: expr, $left:ident, $right:ident, $($arg:tt)+) => ({
        if $config.auto_label {
            $crate::_with_config_assert_eq_impl_!($config, stringify!($left), $left, stringify!($right), $right, ": ", $($arg)+);
        } else {
            $crate::_with_config_assert_eq_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, ": ", $($arg)+);
        }
    });
    ($config: expr, $left:ident, $right:ident,) => ({
        if $config.auto_label {
            $crate::_with_config_assert_eq_impl_!($config, stringify!($left), $left, stringify!($right), $right, "", "");
        } else {
            $crate::_with_config_assert_eq_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, "", "");
        }
    });
    ($config: expr, $left:ident, $right:ident) => ({
        if $config.auto_label {
            $crate::_with_config_assert_eq_impl_!($config, stringify!($left), $left, stringify!($right), $right, "", "");
        } else {
            $crate::_with_config_assert_eq_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, "", "");
        }
    });
    ($config: expr, $left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_config_assert_eq!($config, left: $left, right: $right, $($arg)*)
    });
    ($config: expr, $left:expr, $right:expr) => ({
        $crate::with_config_assert_eq!($config, left: $left, right: $right)
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! _with_config_assert_eq_impl_ {
    ($config: expr, $label_left:expr, $left:expr, $label_right:expr, $right:expr, $separator:expr, $($arg:tt)+) => ({
        let mut config = $config;
        config._maybe_label_left = Some($label_left);
        config._maybe_label_right = Some($label_right);
        match (&($left), &($right)) {
            (val_left, val_right) => {
                if !(*val_left == *val_right) {
                    panic!("assertion failed: `({} == {})`{}{}\
                        \n\
                        \n{}\
                        \n",
                        $label_left,
                        $label_right,
                        $separator,
                        format_args!($($arg)+),
                        $crate::Comparison::new(config, val_left, val_right)
                    )
                }
            }
        }
    });
}

#[cfg(not(feature = "labels"))]
#[macro_export]
macro_rules! assert_ne {
    ($($arg:tt)+) => ({
        $crate::with_config_assert_ne!($crate::Config { auto_label: false, ..Default::default() }, $($arg)+)
    });
}

#[cfg(feature = "labels")]
#[macro_export]
macro_rules! assert_ne {
    ($($arg:tt)+) => ({
        $crate::with_config_assert_ne!($crate::Config { auto_label: true, ..Default::default() }, $($arg)+)
    });
}

#[macro_export]
macro_rules! with_config_assert_ne {
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr, $($arg:tt)+) => ({
        $crate::_with_config_assert_ne_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, ": ", $($arg)+)
    });
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr,) => ({
        $crate::_with_config_assert_ne_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, "", "")
    });
    ($config: expr, $label_left:ident : $left:expr, $label_right:ident : $right:expr) => ({
        $crate::_with_config_assert_ne_impl_!($config, stringify!($label_left), $left, stringify!($label_right), $right, "", "")
    });
    // ($config: expr, $left:ident, $label_right:ident : $right:expr, $($arg:tt)*) => ({
    //     $crate::with_config_assert_ne!($config, $left: $left, $label_right: $right, $($arg)*)
    // });
    // ($config: expr, $left:ident, $label_right:ident : $right:expr) => ({
    //     $crate::with_config_assert_ne!($config, $left: $left, $label_right: $right)
    // });
    // ($config: expr, $label_left:ident : $left:expr, $right:ident, $($arg:tt)*) => ({
    //     $crate::with_config_assert_ne!($config, $label_left: $left, $right: $right, $($arg)*)
    // });
    // ($config: expr, $label_left:ident : $left:expr, $right:ident) => ({
    //     $crate::with_config_assert_ne!($config, $label_left: $left, $right: $right)
    // });
    ($config: expr, $left:ident, $right:ident, $($arg:tt)+) => ({
        if $config.auto_label {
            $crate::_with_config_assert_ne_impl_!($config, stringify!($left), $left, stringify!($right), $right, ": ", $($arg)+);
        } else {
            $crate::_with_config_assert_ne_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, ": ", $($arg)+);
        }
    });
    ($config: expr, $left:ident, $right:ident,) => ({
        if $config.auto_label {
            $crate::_with_config_assert_ne_impl_!($config, stringify!($left), $left, stringify!($right), $right, "", "");
        } else {
            $crate::_with_config_assert_ne_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, "", "");
        }
    });
    ($config: expr, $left:ident, $right:ident) => ({
        if $config.auto_label {
            $crate::_with_config_assert_ne_impl_!($config, stringify!($left), $left, stringify!($right), $right, "", "");
        } else {
            $crate::_with_config_assert_ne_impl_!($config, $config.default_label_left, $left, $config.default_label_right, $right, "", "");
        }
    });
    ($config: expr, $left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::with_config_assert_ne!($config, left: $left, right: $right, $($arg)*)
    });
    ($config: expr, $left:expr, $right:expr) => ({
        $crate::with_config_assert_ne!($config, left: $left, right: $right)
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! _with_config_assert_ne_impl_ {
    ($config: expr, $label_left:expr, $left:expr, $label_right:expr, $right:expr, $separator:expr, $($arg:tt)+) => ({
        let mut config = $config;
        config._maybe_label_left = Some($label_left);
        config._maybe_label_right = Some($label_right);
        match (&($left), &($right)) {
            (val_left, val_right) => {
                if *val_left == *val_right {
                    let left_dbg = format!("{:?}", *val_left);
                    let right_dbg = format!("{:?}", *val_right);
                    if left_dbg != right_dbg {
                        panic!("assertion failed: `({} != {})`{}{}\
                            \n\
                            \n{}\
                            \n{}: According to the `PartialEq` implementation, both of the values \
                            are partially equivalent, even if the `Debug` outputs differ.\
                            \n\
                            \n",
                            $label_left,
                            $label_right,
                            $separator,
                            format_args!($($arg)+),
                            $crate::Comparison::new(config, val_left, val_right),
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
                        $label_left,
                        $label_right,
                        $separator,
                        format_args!($($arg)+),
                        $crate::Style::new().bold().paint("Both sides"),
                        val_left)
                }
            }
        }
    });
}
