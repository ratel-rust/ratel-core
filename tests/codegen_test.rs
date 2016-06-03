// // extern crate badger;

// use super::badger::*;

// fn output_program(input_program: &str) -> String {
//     let mut ast = parser::parse(input_program.to_string());
//     transformer::transform(&mut ast, transformer::Settings::target_es5());
//     codegen::generate_code(ast, false)
// }

// macro_rules! assert_compile {
//     ($string:expr, $expect:expr) => {
//         assert_eq!(output_program($string), $expect.to_string());
//     }
// }

// #[test]
// fn convert_const_to_var_in_global_scope() {
//     assert_compile!("const pi = 3.14", "var pi = 3.14;\n");
// }

// #[test]
// fn convert_let_to_var_in_global_scope() {
//     assert_compile!("let pi = 3.14", "var pi = 3.14;\n");
// }

// #[test]
// fn dont_touch_var_in_global_scope() {
//     assert_compile!("var pi = 3.14", "var pi = 3.14;\n");
// }

// #[test]
// fn convert_let_to_var_in_block() {
//     let program = "if(true) {
//       let pi = 3.14;
//     }
//     ";

//     let expected = "if(true){var _pi = 3.14;}";

//     assert_compile!(program, expected);
// }
