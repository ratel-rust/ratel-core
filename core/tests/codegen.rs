extern crate ratel;

pub use ratel::*;
pub use ratel::grammar::*;
pub use ratel::parser::parse;
pub use ratel::grammar::OperatorType::*;

fn output_program(input_program: &str) -> String {
    let mut ast = parser::parse(input_program.to_string()).expect("Must compile");
    transformer::transform(&mut ast, transformer::Settings::target_es5());
    codegen::generate_code(&ast, true)
}

macro_rules! assert_compile {
    ($string:expr, $expect:expr) => {
        assert_eq!(output_program($string), $expect.to_string());
    }
}

#[test]
fn convert_const_to_var_in_global_scope() {
    assert_compile!("const pi = 3.14;", "var pi=3.14;");
}

#[test]
fn convert_let_to_var_in_global_scope() {
    assert_compile!("let pi = 3.14;", "var pi=3.14;");
}

#[test]
fn dont_touch_var_in_global_scope() {
    assert_compile!("var pi = 3.14;", "var pi=3.14;");
}

#[test]
fn convert_let_to_var_in_block() {
    let program = "if(true) {
      let pi = 3.14;
    }";

    let expected = "if(!0){var pi=3.14;}";

    assert_compile!(program, expected);
}

