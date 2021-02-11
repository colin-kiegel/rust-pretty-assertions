use ansi_term::{
    Colour::{Fixed, Green, Red},
    Style,
};
use similar::{ChangeTag, TextDiff};
use std::fmt;

macro_rules! paint {
    ($f:expr, $colour:expr, $fmt:expr, $($args:tt)*) => (
        write!($f, "{}", $colour.paint(format!($fmt, $($args)*)))
    )
}

const SIGN_RIGHT: &str = ">"; // + > →
const SIGN_LEFT: &str = "<"; // - < ←

/// Delay formatting this deleted chunk until later.
///
/// It can be formatted as a whole chunk by calling `flush`, or the inner value
/// obtained with `take` for further processing.
#[derive(Default)]
struct LatentDeletion<'a> {
    value: Option<&'a str>,
}

impl<'a> LatentDeletion<'a> {
    /// Set the chunk value.
    fn set(&mut self, value: &'a str) {
        self.value = Some(value);
    }

    /// Take the underlying chunk value.
    fn take(&mut self) -> Option<&'a str> {
        self.value.take()
    }

    /// If a value is set, print it as a whole chunk, using the given formatter.
    ///
    /// Resets the internal state to default.
    fn flush(&mut self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = self.value {
            paint!(f, Red, "{}{}", SIGN_LEFT, value)?;
        }
        self.value = None;
        Ok(())
    }
}

// Adapted from:
// https://github.com/johannhof/difference.rs/blob/c5749ad7d82aa3d480c15cb61af9f6baa08f116f/examples/github-style.rs
// Credits johannhof (MIT License)

/// Present the diff output for two mutliline strings in a pretty, colorised manner.
pub(crate) fn write_header(f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(
        f,
        "{} {} / {} :",
        Style::new().bold().paint("Diff"),
        Red.paint(format!("{} left", SIGN_LEFT)),
        Green.paint(format!("right {}", SIGN_RIGHT))
    )
}

/// Present the diff output for two mutliline strings in a pretty, colorised manner.
pub(crate) fn write_lines(f: &mut fmt::Formatter, left: &str, right: &str) -> fmt::Result {
    let diff = TextDiff::from_lines(left, right);

    // Keep track of if the previous chunk in the iteration was a deletion.
    //
    // We defer writing all deletions to the subsequent loop, to find out if
    // we need to write a character-level diff instead.
    let mut previous_deletion = LatentDeletion::default();

    for change in diff.iter_all_changes() {
        let tag = change.tag();
        match tag {
            ChangeTag::Equal => {
                // Handle the previous deletion, if it exists
                previous_deletion.flush(f)?;

                // Print this line with a space at the front to preserve indentation.
                write!(f, " {}", change.value())?;
            }
            ChangeTag::Insert => {
                let inserted = change.value();
                if let Some(deleted) = previous_deletion.take() {
                    // The insertion is preceded by an deletion.
                    //
                    // Let's highlight the character-differences in this replaced
                    // chunk. Note that this chunk can span over multiple lines.
                    write_inline_diff(f, deleted, inserted)?;
                } else {
                    paint!(f, Green, "{}{}", SIGN_RIGHT, inserted)?;
                }
            }
            ChangeTag::Delete => {
                // Handle the previous deletion, if it exists
                previous_deletion.flush(f)?;

                // If we get a deletion, defer writing it to the next loop
                // as we need to know what the next tag is.
                let deleted = change.value();
                previous_deletion.set(deleted);
            }
        }
    }

    // Handle the previous deletion, if it exists
    previous_deletion.flush(f)?;

    Ok(())
}

/// Group character styling for an inline diff, to prevent wrapping each single
/// character in terminal styling codes.
///
/// Styles are applied automatically each time a new style is given in `write_with_style`.
struct InlineWriter<'a, Writer> {
    f: &'a mut Writer,
    style: Style,
}

impl<'a, Writer> InlineWriter<'a, Writer>
where
    Writer: fmt::Write,
{
    fn new(f: &'a mut Writer) -> Self {
        InlineWriter {
            f,
            style: Style::new(),
        }
    }

    /// Push a new character into the buffer, specifying the style it should be written in.
    fn write_with_style(&mut self, c: &str, style: Style) -> fmt::Result {
        // If the style is the same as previously, just write character
        if style == self.style {
            write!(self.f, "{}", c)?;
        } else {
            // Close out previous style
            write!(self.f, "{}", self.style.suffix())?;

            // Store new style and start writing it
            write!(self.f, "{}{}", style.prefix(), c)?;
            self.style = style;
        }
        Ok(())
    }

    /// Finish any existing style and reset to default state.
    fn finish(&mut self) -> fmt::Result {
        // Close out previous style
        write!(self.f, "{}", self.style.suffix())?;
        self.style = Default::default();
        Ok(())
    }
}

/// Format a single line to show an inline diff of the two strings given.
///
/// The given strings should be the output of a line diff, including a trailing newline.
///
/// The output of this function will be two lines, each with a trailing newline.
fn write_inline_diff<TWrite: fmt::Write>(f: &mut TWrite, left: &str, right: &str) -> fmt::Result {
    let diff = TextDiff::from_chars(left, right);

    // LEFT side (==what's been)
    let light = Red.into();
    let heavy = Red.on(Fixed(52)).bold();
    let mut writer = InlineWriter::new(f);
    writer.write_with_style(SIGN_LEFT, light)?;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => writer.write_with_style(change.value(), light)?,
            ChangeTag::Delete => writer.write_with_style(change.value(), heavy)?,
            _ => (),
        }
    }

    // RIGHT side (==what's new)
    let light = Green.into();
    let heavy = Green.on(Fixed(22)).bold();
    writer.write_with_style(SIGN_RIGHT, light)?;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => writer.write_with_style(change.value(), light)?,
            ChangeTag::Insert => writer.write_with_style(change.value(), heavy)?,
            _ => (),
        }
    }
    writer.finish()
}

#[cfg(test)]
mod test {
    use super::*;

    const RED_LIGHT: &str = "\u{1b}[31m";
    const GREEN_LIGHT: &str = "\u{1b}[32m";
    const RED_HEAVY: &str = "\u{1b}[1;48;5;52;31m";
    const GREEN_HEAVY: &str = "\u{1b}[1;48;5;22;32m";
    const RESET: &str = "\u{1b}[0m";

    fn check_inline_diff(left: &str, right: &str, expected: &str) {
        let mut actual = String::new();
        write_inline_diff(&mut actual, left, right).unwrap();

        println!(
            "## left ##\n\
             {}\n\
             ## right ##\n\
             {}\n\
             ## actual diff ##\n\
             {}\n\
             ## expected diff ##\n\
             {}",
            left, right, actual, expected
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn write_inline_diff_newline_only() {
        let left = "\n";
        let right = "\n";
        let expected = format!(
            "{red_light}<\n{reset}\
             {green_light}>\n{reset}",
            red_light = RED_LIGHT,
            green_light = GREEN_LIGHT,
            reset = RESET,
        );

        check_inline_diff(left, right, &expected);
    }

    #[test]
    fn write_inline_diff_added() {
        let left = "\n";
        let right = "polymerase\n";
        let expected = format!(
            "{red_light}<\n{reset}\
             {green_light}>{reset}{green_heavy}polymerase{reset}{green_light}\n{reset}",
            red_light = RED_LIGHT,
            green_light = GREEN_LIGHT,
            green_heavy = GREEN_HEAVY,
            reset = RESET,
        );

        check_inline_diff(left, right, &expected);
    }

    #[test]
    fn write_inline_diff_removed() {
        let left = "polyacrylamide\n";
        let right = "\n";
        let expected = format!(
            "{red_light}<{reset}{red_heavy}polyacrylamide{reset}{red_light}\n{reset}\
             {green_light}>\n{reset}",
            red_light = RED_LIGHT,
            green_light = GREEN_LIGHT,
            red_heavy = RED_HEAVY,
            reset = RESET,
        );

        check_inline_diff(left, right, &expected);
    }

    #[test]
    fn write_inline_diff_changed() {
        let left = "polymerase\n";
        let right = "polyacrylamide\n";
        let expected = format!(
            "{red_light}<poly{reset}{red_heavy}me{reset}{red_light}ra{reset}{red_heavy}s{reset}{red_light}e\n{reset}\
             {green_light}>poly{reset}{green_heavy}ac{reset}{green_light}r{reset}{green_heavy}yl{reset}{green_light}a{reset}{green_heavy}mid{reset}{green_light}e\n{reset}",
            red_light = RED_LIGHT,
            green_light = GREEN_LIGHT,
            red_heavy = RED_HEAVY,
            green_heavy = GREEN_HEAVY,
            reset = RESET,
        );

        check_inline_diff(left, right, &expected);
    }
}
