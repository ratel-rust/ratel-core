/// Parses the given input string into an AST and compares it
/// with the given JSON input.

#[cfg(test)]
#[macro_export]
macro_rules! expect_parse {
    ($expr:expr, $expected:tt) => {{
        use $crate::parser::parse;
        use $crate::serde_json::to_value;

        let module = parse($expr).unwrap();
        let result = to_value(&module).unwrap();
        let expected = json!($expected);
        assert_eq!(result, expected);
    }};
}
