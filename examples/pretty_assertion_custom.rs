#[allow(unused_imports)]
use pretty_assertions;

macro_rules! assert_eq { ($($arg:tt)+) => ({
    pretty_assertions::with_config_assert_eq!(
        pretty_assertions::Config {
            auto_label: true,
            style: pretty_assertions::Color::Yellow.normal(),
            prefix: "<=>",
            prefix_left: "<<=", prefix_right: "=>>",
            ..Default::default()
            },
        $($arg)+
    )
})}

include!("standard_assertion.rs");
