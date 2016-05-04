extern crate badger;

use badger::*;

fn output_program(input_program: &str) -> Result<String, Vec<String>> {
    let program = parser::parse(input_program.into());
    let ast = transformer::traverse(program);
    codegen::generate_code(ast)
}

#[test]
fn convert_const_to_var_in_global_scope() {
    assert_eq!(output_program("const pi = 3.14"),
               Ok("var pi = 3.14;".into()));
}

#[test]
fn convert_let_to_var_in_global_scope() {
    assert_eq!(output_program("let pi = 3.14"),
               Ok("var pi = 3.14;".into()));
}

#[test]
fn dont_touch_var_in_global_scope() {
    assert_eq!(output_program("var pi = 3.14"),
               Ok("var pi = 3.14;".into()));
}

#[test]
fn convert_let_to_var_in_block() {
    let program = "if(true) {
      let pi = 3.14;
    }
    ";

    let expected = "if(true){var _pi = 3.14;}";

    assert_eq!(output_program(program), Ok(expected.into()));
}
