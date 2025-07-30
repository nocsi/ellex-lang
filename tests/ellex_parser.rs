#[test]
fn test_hello_world() {
    let input = r#"tell "Hello, {world}!" "#;
    assert!(parse(input).is_ok());
}
