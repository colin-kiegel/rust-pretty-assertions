//! # pretty_assertions_derive
//!
//! Used to produce custom `assert_eq` and `assert_ne` drop-in implementations, without
//! the boilerplate.
//!
//! For further documentation, see [`derive_assert_eq!`] and [`derive_assert_ne!`].

#![deny(clippy::all, unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

struct ParsedItem {
    attrs: Vec<syn::Attribute>,
    body: Box<syn::Expr>,
    name: syn::PathSegment,
    ident_left: syn::Ident,
    ident_right: syn::Ident,
    ident_has_message: syn::Ident,
    ident_message: syn::Ident,
}

impl From<syn::ExprTuple> for ParsedItem {
    fn from(item: syn::ExprTuple) -> Self {
        let syn::ExprTuple { attrs, elems, .. } = item;
        let mut arguments = elems.into_iter();
        let mut name_segments = match arguments
            .next()
            .expect("input must be a tuple of two elements")
        {
            syn::Expr::Path(expr_path) => expr_path.path.segments.into_iter(),
            _ => panic!("first element must be an ident"),
        };
        let name = name_segments
            .next()
            .expect("first element must be an ident");
        if name_segments.next().is_some() {
            panic!("first element must be an ident");
        }
        let closure = match arguments
            .next()
            .expect("input must be a tuple of two elements")
        {
            syn::Expr::Closure(closure) => closure,
            _ => panic!("second element must be a closure"),
        };

        let syn::ExprClosure { body, inputs, .. } = closure;
        let mut closure_idents = inputs
            .into_iter()
            .enumerate()
            .map(|(index, input)| match input {
                syn::Pat::Ident(pat_ident) => pat_ident.ident,
                _ => panic!("closure input at position '{}' was not an ident", index),
            });
        let ident_left = closure_idents
            .next()
            .expect("source closure must have four arguments");
        let ident_right = closure_idents
            .next()
            .expect("source closure must have four arguments");
        let ident_has_message = closure_idents
            .next()
            .expect("source closure must have four arguments");
        let ident_message = closure_idents
            .next()
            .expect("source closure must have four arguments");

        Self {
            attrs,
            body,
            name,
            ident_left,
            ident_right,
            ident_has_message,
            ident_message,
        }
    }
}

/// Derive a drop in replacement for `assert_eq`. When the equality check fails,
/// the given closure will be called.
///
/// ## Limitations
///
/// Derived macro definitions cannot be called within the same crate. You muct define your custom
/// macros in an external crate, and then import them before use in your tests.
///
/// If you call a derived macro within the same crate, you will see a compiler error such as:
///
/// ```log
///  error: macro-expanded `macro_export` macros from the current crate cannot be referred to by absolute paths
///   --> src/lib.rs:107:1
///    |
/// 4  | / pretty_assertions_derive::derive_assert_eq! {
/// 5  | |     (assert_eq_custom, |left, right, has_message, message| {
/// ...  |         // your custom macro definition here
/// 20 | |     }
/// 19 | | }
///    | |_^
/// 20 |
/// 21 |   assert_eq_custom!(3, 2);
///    |   ------------------------------ in this macro invocation
///    |
///    = note: `#[deny(macro_expanded_macro_exports_accessed_by_absolute_paths)]` on by default
///    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
///    = note: for more information, see issue #52234 <https://github.com/rust-lang/rust/issues/52234>
/// ```
///
/// ## Arguments
///
/// [`derive_assert_eq!`] accepts a single tuple with two items. Any annotations present on the tuple
/// (such as docstrings) will be transferred to the generated macro definiton.
///
/// ### Name
///
/// The first argument must be the name of the macro to produce.
///
/// ### Closure
///
/// The closure will be called when the equality check of the two values fails, and must accept
/// exactly four arguments itself. These are:
///
/// - `left: &T`, a reference to the left value
/// - `right: &U`, a reference to the right value
/// - `has_message: bool`, `true` if a custom panic message was given in the invocation
/// - `message: String`, the value of the custom message, or an empty `String`
///
/// ## Examples
///
/// Derive a macro named `assert_eq_emoji`, which adds emoji flair around each value.
///
/// ```rust
/// pretty_assertions_derive::derive_assert_eq! {
///     /// Emojified `assert_eq`.
///     (assert_eq_emoji, |left, right, has_message, message| {
///         ::std::panic!("😱 Values should be equal! 😱{}{}\
///            \n\
///            \n🎯 {} 🎯\
///            \n\
///            \n💥 {} 💥\
///            \n",
///            if has_message { "\n-> with custom message: " } else { "" },
///            message,
///            left,
///            right,
///         )
///     })
/// }
/// ```
///
/// A sample failure:
///
/// ```log
/// thread 'main' panicked at '😱 Values should be equal! 😱
///
/// 🎯 3 🎯
///
/// 💥 2 💥
/// ', pretty_assertions_derive_tests/examples/assert_emoji.rs:6:46
/// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
/// ```
#[proc_macro]
pub fn derive_assert_eq(item: TokenStream) -> TokenStream {
    let item_tuple = syn::parse_macro_input!(item as syn::ExprTuple);
    let ParsedItem {
        attrs,
        body,
        name,
        ident_left,
        ident_right,
        ident_has_message,
        ident_message,
    } = ParsedItem::from(item_tuple);

    let result = quote! {
        #(#attrs)*
        #[macro_export]
        macro_rules! #name {
            ($left:expr, $right:expr $(,)?) => ({
                $crate::#name!(@ $left, $right, false, "");
            });
            ($left:expr, $right:expr, $($arg:tt)*) => ({
                $crate::#name!(@ $left, $right, true, $($arg)+);
            });
            (@ $left:expr, $right:expr, $has_additional_args:expr, $($arg:tt)*) => ({
                match (&($left), &($right)) {
                    (__pretty_assertions_derive_left_val, __pretty_assertions_derive_right_val) => {
                        if !(*__pretty_assertions_derive_left_val == *__pretty_assertions_derive_right_val) {
                            let #ident_left = __pretty_assertions_derive_left_val;
                            let #ident_right = __pretty_assertions_derive_right_val;
                            let #ident_has_message = $has_additional_args;
                            let #ident_message = ::std::format!($($arg)*);
                            #body
                        }
                    }
                }
            });
        }
    };
    result.into()
}

/// Derive a drop in replacement for `assert_ne`. When the equality check passes,
/// the given closure will be called.
///
/// See [`derive_assert_eq!`] for further details, usage is identical.
///
/// ## Examples
///
/// Derive a macro named `assert_ne_emoji`, which adds emoji flair around each value.
///
/// ```rust
/// pretty_assertions_derive::derive_assert_ne! {
///     /// Emojified `assert_ne`.
///     (assert_ne_emoji, |left, right, has_message, message| {
///         ::std::panic!("😱 Values should not be equal! 😱{}{}\
///            \n\
///            \n🔥 {} 🔥\
///            \n\
///            \n🔥 {} 🔥\
///            \n",
///            if has_message { "\n-> with custom message: " } else { "" },
///            message,
///            left,
///            right,
///         )
///     })
/// }
/// ```
///
/// A sample failure:
///
/// ```log
/// thread 'main' panicked at '😱 Values should not be equal! 😱
/// -> with custom message: additional details
///
/// 🔥 3 🔥
///
/// 🔥 3 🔥
/// ', pretty_assertions_derive_tests/examples/assert_emoji.rs:12:46
/// ```
#[proc_macro]
pub fn derive_assert_ne(item: TokenStream) -> TokenStream {
    let item_tuple = syn::parse_macro_input!(item as syn::ExprTuple);
    let ParsedItem {
        attrs,
        body,
        name,
        ident_left,
        ident_right,
        ident_has_message,
        ident_message,
    } = ParsedItem::from(item_tuple);

    let result = quote! {
        #(#attrs)*
        #[macro_export]
        macro_rules! #name {
            ($left:expr, $right:expr $(,)?) => ({
                $crate::#name!(@ $left, $right, false, "");
            });
            ($left:expr, $right:expr, $($arg:tt)*) => ({
                $crate::#name!(@ $left, $right, true, $($arg)+);
            });
            (@ $left:expr, $right:expr, $has_additional_args:expr, $($arg:tt)*) => ({
                match (&($left), &($right)) {
                    (__pretty_assertions_derive_left_val, __pretty_assertions_derive_right_val) => {
                        if *__pretty_assertions_derive_left_val == *__pretty_assertions_derive_right_val {
                            let #ident_left = __pretty_assertions_derive_left_val;
                            let #ident_right = __pretty_assertions_derive_right_val;
                            let #ident_has_message = $has_additional_args;
                            let #ident_message = ::std::format!($($arg)*);
                            #body
                        }
                    }
                }
            });
        }
    };
    result.into()
}
