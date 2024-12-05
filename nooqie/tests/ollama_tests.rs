#![cfg(test)]

use nooqie::commands::*;

#[test]
fn test_json_strip_escape_valid_input() {
    let _test_data = r#"nothing to strip"#;
    let result = ollama::json_strip_escape(&_test_data).unwrap();
    assert_eq!(result, r#"nothing to strip"#);
}

#[test]
fn test_json_strip_escape_invalid_input() {
    let _test_data = String::from(r#""every/thing" to \strip"#);
    let result = ollama::json_strip_escape(&_test_data).unwrap();
    assert_eq!(result, r#""every/thing" to \strip"#);
}
