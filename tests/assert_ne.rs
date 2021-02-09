#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne};

use pretty_assertions::{with_config_assert_eq, with_config_assert_ne, Config};

use maybe_unwind::maybe_unwind;

fn test_setup() {
    // set panic() hook for maybe_unwind
    static SET_PANIC_HOOK: std::sync::Once = std::sync::Once::new();
    SET_PANIC_HOOK.call_once(maybe_unwind::set_hook);
}

#[test]
fn assert_ne() {
    test_setup();

    #[derive(Debug, PartialEq)]
    struct Foo {
        lorem: &'static str,
        ipsum: u32,
        dolor: Result<String, String>,
    }

    let x = Some(Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok("hey".to_string()),
    });

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`

[1mBoth sides[0m:
Some(
    Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok(
            "hey",
        ),
    },
)

"#;

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "labels")))]
    {
        expect = expect
            .replace("{{left}}", "left")
            .replace("{{right}}", "right");
    }
    #[cfg(feature = "labels")]
    {
        expect = expect.replace("{{left}}", "x").replace("{{right}}", "x");
    }

    let result = maybe_unwind(|| {
        assert_ne!(x, x);
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        pretty_assertions::Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}

#[test]
fn with_labels_assert_ne() {
    test_setup();

    #[derive(Debug, PartialEq)]
    struct Foo {
        lorem: &'static str,
        ipsum: u32,
        dolor: Result<String, String>,
    }

    let x = Some(Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok("hey".to_string()),
    });

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`

[1mBoth sides[0m:
Some(
    Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok(
            "hey",
        ),
    },
)

"#;

    let mut expect = expect_template.to_string();

    expect = expect.replace("{{<}}", "<").replace("{{>}}", ">");
    expect = expect.replace("{{left}}", "x").replace("{{right}}", "x");

    let result = maybe_unwind(|| {
        with_config_assert_ne!(
            Config {
                auto_label: true,
                ..Default::default()
            },
            x,
            x
        );
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        pretty_assertions::Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}

#[test]
fn assert_ne_custom() {
    test_setup();

    #[derive(Debug, PartialEq)]
    struct Foo {
        lorem: &'static str,
        ipsum: u32,
        dolor: Result<String, String>,
    }

    let x = Some(Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok("hey".to_string()),
    });

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`: custom panic message

[1mBoth sides[0m:
Some(
    Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok(
            "hey",
        ),
    },
)

"#;

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "labels")))]
    {
        expect = expect
            .replace("{{left}}", "left")
            .replace("{{right}}", "right");
    }
    #[cfg(feature = "labels")]
    {
        expect = expect.replace("{{left}}", "x").replace("{{right}}", "x");
    }

    let result = maybe_unwind(|| {
        assert_ne!(x, x, "custom panic message");
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        pretty_assertions::Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}

#[test]
#[should_panic]
fn assert_ne_non_empty_return() {
    test_setup();

    fn not_zero(x: u32) -> u32 {
        assert_ne!(x, 0);
        x
    };
    not_zero(0);
}

#[test]
fn assert_ne_partial() {
    test_setup();

    // Workaround for https://github.com/rust-lang/rust/issues/47619
    // can be removed, when we require rust 1.25 or higher
    struct Foo(f32);

    use ::std::fmt;
    impl fmt::Debug for Foo {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:.1?}", self.0)
        }
    }

    impl PartialEq for Foo {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`

[1mDiff[0m [31m{{<}} {{left}}[0m / [32m{{right}} {{>}}[0m :
[31m{{<}}[0m[1;48;5;52;31m-[0m[31m0.0[0m
[32m{{>}}[0m[32m0.0[0m

[1;4mNote[0m: According to the `PartialEq` implementation, both of the values are partially equivalent, even if the `Debug` outputs differ.

"#;

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "diffstyle_git")))]
    {
        expect = expect.replace("{{<}}", "<").replace("{{>}}", ">");
    }
    #[cfg(feature = "diffstyle_git")]
    {
        expect = expect.replace("{{<}}", "-").replace("{{>}}", "+");
    }

    expect = expect
        .replace("{{left}}", "left")
        .replace("{{right}}", "right");

    let result = maybe_unwind(|| {
        assert_ne!(Foo(-0.0), Foo(0.0));
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}

#[test]
fn assert_ne_trailing_comma() {
    test_setup();

    #[derive(Debug, PartialEq)]
    struct Foo {
        lorem: &'static str,
        ipsum: u32,
        dolor: Result<String, String>,
    }

    let x = Some(Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok("hey".to_string()),
    });

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`

[1mBoth sides[0m:
Some(
    Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok(
            "hey",
        ),
    },
)

"#;

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "labels")))]
    {
        expect = expect
            .replace("{{left}}", "left")
            .replace("{{right}}", "right");
    }
    #[cfg(feature = "labels")]
    {
        expect = expect.replace("{{left}}", "x").replace("{{right}}", "x");
    }

    let result = maybe_unwind(|| {
        assert_ne!(x, x,);
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}

#[test]
fn assert_ne_custom_trailing_comma() {
    test_setup();

    #[derive(Debug, PartialEq)]
    struct Foo {
        lorem: &'static str,
        ipsum: u32,
        dolor: Result<String, String>,
    }

    let x = Some(Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok("hey".to_string()),
    });

    let expect_template = r#"assertion failed: `({{left}} != {{right}})`: custom panic message

[1mBoth sides[0m:
Some(
    Foo {
        lorem: "Hello World!",
        ipsum: 42,
        dolor: Ok(
            "hey",
        ),
    },
)

"#;

    let mut expect = expect_template.to_string();

    #[cfg(not(any(feature = "labels")))]
    {
        expect = expect
            .replace("{{left}}", "left")
            .replace("{{right}}", "right");
    }
    #[cfg(feature = "labels")]
    {
        expect = expect.replace("{{left}}", "x").replace("{{right}}", "x");
    }

    let result = maybe_unwind(|| {
        assert_ne!(x, x, "custom panic message",);
    });

    assert!(result.is_err());

    let result = result.unwrap_err().payload_str().to_owned();
    println!("expect={}", expect);
    println!("result={}", result);
    with_config_assert_eq!(
        Config {
            auto_label: true,
            ..Default::default()
        },
        expect,
        result
    );
}
