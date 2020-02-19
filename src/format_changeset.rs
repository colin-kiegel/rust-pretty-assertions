use ansi_term::Colour::{Fixed, Green, Red};
use ansi_term::Style;
use difference::{Changeset, Difference};
use std::fmt::{self, Debug, Display};

#[doc(hidden)]
pub struct Comparison {
    changeset: Changeset,
    config: Config,
}

impl Comparison {
    pub fn new<TLeft: Debug, TRight: Debug>(
        config: Config,
        left: &TLeft,
        right: &TRight,
    ) -> Comparison {
        let left_dbg = format!("{:#?}", *left);
        let right_dbg = format!("{:#?}", *right);
        let changeset = Changeset::new(&left_dbg, &right_dbg, "\n");

        Comparison { changeset, config }
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_changeset(f, &self.changeset, &self.config)
    }
}

macro_rules! paint {
    ($f:ident, $colour:expr, $fmt:expr, $($args:tt)*) => (
        write!($f, "{}", $colour.paint(format!($fmt, $($args)*)))
    )
}

const PREFIX: &str = " ";
const PREFIX_RIGHT: &str = ">"; // + > →
const PREFIX_LEFT: &str = "<"; // - < ←

#[doc(hidden)]
pub struct Config {
    pub default_label_left: &'static str,
    pub default_label_right: &'static str,
    pub left_color: ansi_term::Colour,
    pub left_color_diff_bg: u8,
    pub maybe_label_left: Option<&'static str>,
    pub maybe_label_right: Option<&'static str>,
    pub right_color: ansi_term::Colour,
    pub right_color_diff_bg: u8,
    pub prefix: &'static str,
    pub prefix_left: &'static str,
    pub prefix_right: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_label_left: "left",
            default_label_right: "right",
            left_color: Red, // (dark) red
            left_color_diff_bg: 52,
            right_color: Green, // (dark) green
            right_color_diff_bg: 22,
            prefix: PREFIX,
            prefix_left: PREFIX_LEFT,
            prefix_right: PREFIX_RIGHT,
            maybe_label_left: None,
            maybe_label_right: None,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        Config {
            ..Default::default()
        }
    }
}

// Adapted from:
// https://github.com/johannhof/difference.rs/blob/c5749ad7d82aa3d480c15cb61af9f6baa08f116f/examples/github-style.rs
// Credits johannhof (MIT License)

pub fn format_changeset(
    f: &mut fmt::Formatter,
    changeset: &Changeset,
    config: &Config,
) -> fmt::Result {
    let diffs = &changeset.diffs;

    writeln!(
        f,
        "{} {} / {} :",
        Style::new().bold().paint("Diff"),
        config.left_color.paint(format!(
            "{} {}",
            config.prefix_left,
            config.maybe_label_left.unwrap_or(config.default_label_left)
        )),
        config.right_color.paint(format!(
            "{} {}",
            config
                .maybe_label_right
                .unwrap_or(config.default_label_right),
            config.prefix_right
        ))
    )?;
    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref same) => {
                // Have to split line by line in order to have the extra whitespace
                // at the beginning.
                for line in same.split('\n') {
                    writeln!(f, "{}{}", config.prefix, line)?;
                }
            }
            Difference::Add(ref added) => {
                let prev = i.checked_sub(1).and_then(|x| diffs.get(x));
                match prev {
                    Some(&Difference::Rem(ref removed)) => {
                        // The addition is preceded by an removal.
                        //
                        // Let's highlight the character-differences in this replaced
                        // chunk. Note that this chunk can span over multiple lines.
                        format_replacement(f, added, removed, &config)?;
                    }
                    _ => {
                        for line in added.split('\n') {
                            paint!(f, config.right_color, "{}{}\n", config.prefix_right, line)?;
                        }
                    }
                };
            }
            Difference::Rem(ref removed) => {
                let next = i.checked_add(1).and_then(|x| diffs.get(x));
                match next {
                    Some(&Difference::Add(_)) => {
                        // The removal is followed by an addition.
                        //
                        // ... we'll handle both in the next iteration.
                    }
                    _ => {
                        for line in removed.split('\n') {
                            paint!(f, config.left_color, "{}{}\n", config.prefix_left, line)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

macro_rules! join {
    (
        $elem:ident in ($iter:expr) {
            $( $body:tt )*
        } separated by {
            $( $separator:tt )*
        }
    ) => (
        let mut iter = $iter;

        if let Some($elem) = iter.next() {
            $( $body )*
        }

        for $elem in iter {
            $( $separator )*
            $( $body )*
        }
    )
}

pub fn format_replacement(
    f: &mut dyn fmt::Write,
    added: &str,
    removed: &str,
    config: &Config,
) -> fmt::Result {
    let Changeset { diffs, .. } = Changeset::new(removed, added, "");

    // LEFT side (==what's been)
    paint!(f, config.left_color, "{}", config.prefix_left)?;
    for c in &diffs {
        match *c {
            Difference::Same(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.left_color, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.left_color, "{}", config.prefix_left)?;
                });
            }
            Difference::Rem(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.left_color.on(Fixed(config.left_color_diff_bg)).bold(), "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.left_color.bold(), "{}", config.prefix_left)?;
                });
            }
            _ => (),
        }
    }
    writeln!(f)?;

    // RIGHT side (==what's new)
    paint!(f, config.right_color, "{}", config.prefix_right)?;
    for c in &diffs {
        match *c {
            Difference::Same(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.right_color, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.right_color, "{}", config.prefix_right)?;
                });
            }
            Difference::Add(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.right_color.on(Fixed(config.right_color_diff_bg)).bold(), "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.right_color.bold(), "{}", config.prefix_right)?;
                });
            }
            _ => (),
        }
    }

    writeln!(f)
}

#[test]
fn test_format_replacement() {
    let added = "    84,\
                 \n    248,";
    let removed = "    0,\
                   \n    0,\
                   \n    128,";

    let expect_template = "\u{1b}[31m{{<}}\u{1b}[0m\u{1b}[31m    \u{1b}[0m\u{1b}[1;48;5;52;31m0\u{1b}[0m\u{1b}[31m,\u{1b}[0m\n\u{1b}[31m{{<}}\u{1b}[0m\u{1b}[31m    \u{1b}[0m\u{1b}[1;48;5;52;31m0,\u{1b}[0m\n\u{1b}[1;31m{{<}}\u{1b}[0m\u{1b}[1;48;5;52;31m    1\u{1b}[0m\u{1b}[31m2\u{1b}[0m\u{1b}[31m8,\u{1b}[0m\n\u{1b}[32m{{>}}\u{1b}[0m\u{1b}[32m    \u{1b}[0m\u{1b}[1;48;5;22;32m84\u{1b}[0m\u{1b}[32m,\u{1b}[0m\n\u{1b}[32m{{>}}\u{1b}[0m\u{1b}[32m    \u{1b}[0m\u{1b}[32m2\u{1b}[0m\u{1b}[1;48;5;22;32m4\u{1b}[0m\u{1b}[32m8,\u{1b}[0m\n";

    let mut expect = expect_template.to_string();

    expect = expect.replace("{{<}}", "<").replace("{{>}}", ">");

    let mut actual = String::new();
    let config = Config::new();
    let _ = format_replacement(&mut actual, added, removed, &config);

    println!(
        "## removed ##\
         \n{}\
         \n## added ##\
         \n{}\
         \n## diff ##\
         \n{}",
        removed, added, actual
    );

    println!("actual={}", actual);
    println!("expect={}", expect);

    assert_eq!(actual, expect);
}
