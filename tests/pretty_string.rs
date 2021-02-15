use std::fmt;

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
#[derive(PartialEq, Eq)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[test]
#[should_panic(expected = r#"assertion failed: `(PrettyString("") == PrettyString("foo"))`

[1mDiff[0m [31m< left[0m / [32mright >[0m :
[32m>foo[0m

"#)]
fn assert_eq_empty_first() {
    pretty_assertions::assert_eq!(PrettyString(""), PrettyString("foo"));
}
