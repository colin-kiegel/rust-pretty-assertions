use difference::{Changeset, Difference};
use std::fmt::{self, Debug, Display};

use crate::config::Config;

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
        format_comparison(f, &self.changeset, &self.config)
    }
}

// Adapted from:
// https://github.com/johannhof/difference.rs/blob/c5749ad7d82aa3d480c15cb61af9f6baa08f116f/examples/github-style.rs
// Credits johannhof (MIT License)

macro_rules! paint {
    ($f:ident, $colour:expr, $fmt:expr, $($args:tt)*) => (
        write!($f, "{}", painted!($colour, format!($fmt, $($args)*)))
    )
}

macro_rules! painted {
    ($colour:expr, $formatted:expr) => {
        $colour.paint($formatted)
    };
}

pub fn format_comparison(
    f: &mut fmt::Formatter,
    changeset: &Changeset,
    config: &Config,
) -> fmt::Result {
    let diffs = &changeset.diffs;

    writeln!(
        f,
        "{} {} / {} :",
        config.style.bold().paint("Diff"),
        painted!(
            config.style_left,
            format!(
                "{} {}",
                config.prefix_left,
                config
                    ._maybe_label_left
                    .unwrap_or(config.default_label_left)
            )
        ),
        painted!(
            config.style_right,
            format!(
                "{} {}",
                config
                    ._maybe_label_right
                    .unwrap_or(config.default_label_right),
                config.prefix_right
            )
        )
    )?;
    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref same) => {
                // Have to split line by line in order to have the extra whitespace
                // at the beginning.
                for line in same.split('\n') {
                    writeln!(
                        f,
                        "{}{}",
                        painted!(config.style, config.prefix),
                        painted!(config.style, line)
                    )?;
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
                            paint!(f, config.style_right, "{}{}\n", config.prefix_right, line)?;
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
                            paint!(f, config.style_left, "{}{}\n", config.prefix_left, line)?;
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
    let mut removed_lines = removed.split('\n');
    let removed_line = removed_lines.next_back().unwrap();
    let mut added_lines = added.split('\n');
    let added_line = added_lines.next().unwrap();

    let Changeset { diffs, .. } = Changeset::new(removed_line, added_line, "");

    for line in removed_lines {
        paint!(f, config.style_left, "{}{}\n", config.prefix_left, line)?;
    }

    // LEFT side (==what's been)
    paint!(f, config.style_left, "{}", config.prefix_left)?;
    for c in &diffs {
        match *c {
            Difference::Same(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.style_left, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.style_left, "{}", config.prefix_left)?;
                });
            }
            Difference::Rem(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.style_left_diff, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.style_left, "{}", config.prefix_left)?;
                });
            }
            _ => (),
        }
    }
    writeln!(f, "{}", config.style.paint(""))?;

    // RIGHT side (==what's new)
    paint!(f, config.style_right, "{}", config.prefix_right)?;
    for c in &diffs {
        match *c {
            Difference::Same(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.style_right, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.style_right, "{}", config.prefix_right)?;
                });
            }
            Difference::Add(ref word_diff) => {
                join!(chunk in (word_diff.split('\n')) {
                    paint!(f, config.style_right_diff, "{}", chunk)?;
                } separated by {
                    writeln!(f)?;
                    paint!(f, config.style_right, "{}", config.prefix_right)?;
                });
            }
            _ => (),
        }
    }
    writeln!(f, "{}", painted!(config.style, ""))?;

    for line in added_lines {
        paint!(f, config.style_right, "{}{}\n", config.prefix_right, line)?;
    }

    Ok(())
}

#[test]
fn test_format_replacement() {
    let added = "    84,\
                 \n    248,";
    let removed = "    0,\
                   \n    0,\
                   \n    128,";

    let expect_template = "\u{1b}[31m{{<}}    0,\n\u{1b}[0m\u{1b}[31m{{<}}    0,\n\u{1b}[0m\u{1b}[31m{{<}}\u{1b}[0m\u{1b}[31m    \u{1b}[0m\u{1b}[1;48;5;52;31m12\u{1b}[0m\u{1b}[31m8\u{1b}[0m\u{1b}[31m,\u{1b}[0m\n\u{1b}[32m{{>}}\u{1b}[0m\u{1b}[32m    \u{1b}[0m\u{1b}[32m8\u{1b}[0m\u{1b}[1;48;5;22;32m4\u{1b}[0m\u{1b}[32m,\u{1b}[0m\n\u{1b}[32m{{>}}    248,\n\u{1b}[0m";

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "diffstyle_git")))]
    {
        expect = expect.replace("{{<}}", "<").replace("{{>}}", ">");
    }
    #[cfg(feature = "diffstyle_git")]
    {
        expect = expect.replace("{{<}}", "-").replace("{{>}}", "+");
    }

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

    crate::with_config_assert_eq!(
        Config {
            auto_label: true,
            ..Default::default()
        },
        actual,
        expect
    );
}
