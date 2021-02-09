use crate::{Color, Style};

/// Configuration structure for pretty assertions
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub auto_label: bool, // automatically label assertion arguments (when possible)

    pub default_label_left: &'static str, // default label for "left"
    pub default_label_right: &'static str, // default label for "right"

    pub prefix: &'static str, // prefix for lines which don't differ between the assert arguments
    pub prefix_left: &'static str, // prefix text for left/first (aka prior) argument differences
    pub prefix_right: &'static str, // prefix text for right/second (aka after) argument differences

    pub style: Style,            // style for baseline assertion output
    pub style_left: Style, // style for left/first (aka prior) argument (line-by-line) differences
    pub style_right: Style, // style for right/second (aka after) argument (line-by-line) differences
    pub style_left_diff: Style, // style for left/first (aka prior) argument intra-line differences
    pub style_right_diff: Style, // style for right/second (aka after) argument intra-line differences

    // "private"; but must be pub accessible for use in exported macros
    #[doc(hidden)]
    pub _maybe_label_left: Option<&'static str>, // left/first (aka prior) label, if available
    #[doc(hidden)]
    pub _maybe_label_right: Option<&'static str>, // right/second (aka after) label, if available
}

const PREFIX: &str = " ";

#[cfg(not(any(feature = "diffstyle_git")))]
const PREFIX_RIGHT: &str = ">"; // + > →
#[cfg(not(any(feature = "diffstyle_git")))]
const PREFIX_LEFT: &str = "<"; // - < ←

#[cfg(feature = "diffstyle_git")]
const PREFIX_RIGHT: &str = "+"; // + > →
#[cfg(feature = "diffstyle_git")]
const PREFIX_LEFT: &str = "-"; // - < ←

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_label: cfg!(feature = "labels"),
            default_label_left: "left",
            default_label_right: "right",
            //
            prefix: PREFIX,
            prefix_left: PREFIX_LEFT,
            prefix_right: PREFIX_RIGHT,
            //
            style: crate::Style::default(),
            style_left: Color::Red.normal(), // (dark) red ("Maroon"), aka Color::Fixed(1)
            style_right: Color::Green.normal(), // (dark) green ("Green"), aka Color::Fixed(2)
            style_left_diff: Color::Red.on(Color::Fixed(52)).bold(), // bold bright red ("Red") on "DarkRed"
            style_right_diff: Color::Green.on(Color::Fixed(22)).bold(), // bold bright green ("Lime") on "DarkGreen"
            // "private"
            _maybe_label_left: None,
            _maybe_label_right: None,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Config::default()
    }
}
