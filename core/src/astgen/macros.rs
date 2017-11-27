/// Parses the given input string into an AST and compares it
/// with the given JSON input.

#[macro_export]
macro_rules! expect_parse {
    ($expr:expr, $expected:tt) => ({
        let module = parse($expr).unwrap();
        let result = generate_ast(&module);
        let expected = json!($expected);
        assert_eq!(result, expected);
    })
}
