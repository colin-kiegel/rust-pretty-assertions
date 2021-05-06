//! Derives some custom macros for use in tests examples. Due to macro hoising/import limitations,
//! this must be done in a separate crate.

#![deny(clippy::all, missing_docs, unsafe_code)]

pretty_assertions_derive::derive_assert_eq!(
    /// A simple plain text assertion format for easy reading tests.
    (assert_eq_custom, |l, r, h, m| {
        if h {
            ::std::panic!("{:?} != {:?}: {}", l, r, m)
        } else {
            ::std::panic!("{:?} != {:?}: <no additional message>", l, r)
        }
    })
);

pretty_assertions_derive::derive_assert_eq! {
    /// Emojified `assert_eq`.
    (assert_eq_emoji, |left, right, has_message, message| {
        ::std::panic!("😱 Values should be equal! 😱{}{}\
           \n\
           \n🎯 {} 🎯\
           \n\
           \n💥 {} 💥\
           \n",
           if has_message { "\n-> with custom message: " } else { "" },
           message,
           left,
           right,
        )
    })
}

pretty_assertions_derive::derive_assert_ne! {
    /// Emojified `assert_ne`.
    (assert_ne_emoji, |left, right, has_message, message| {
        ::std::panic!("😱 Values should not be equal! 😱{}{}\
           \n\
           \n🔥 {} 🔥\
           \n\
           \n🔥 {} 🔥\
           \n",
           if has_message { "\n-> with custom message: " } else { "" },
           message,
           left,
           right,
        )
    })
}
