use pretty_assertions_derive_tests::assert_eq_custom;

#[test]
fn assert_eq_custom_pass() {
    assert_eq_custom!(3, 1 + 2);
}

#[test]
#[should_panic(expected = r#"3 != 2: <no additional message>"#)]
fn assert_eq_custom_fail() {
    assert_eq_custom!(3, 2);
}

#[test]
#[should_panic(expected = r#"3 != 2: message with var: 71"#)]
fn assert_eq_custom_fail_message() {
    assert_eq_custom!(3, 2, "message with var: {}", 71);
}
