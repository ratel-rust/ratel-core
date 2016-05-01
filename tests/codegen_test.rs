#[path="../src/grammar.rs"]
mod grammar;
#[path="../src/parser.rs"]
mod parser;
#[path="../src/codegen.rs"]
mod codegen;
#[path="../src/lexicon.rs"]
mod lexicon;
#[path="../src/tokenizer.rs"]
mod tokenizer;

fn output_program(input_program: &str) -> Result<String, Vec<String>> {
    let program = parser::parse(input_program.into());
    codegen::generate_code(program)
}

#[test]
fn convert_const_str_to_var() {
    assert_eq!(output_program("const foo = \"bar\";"), Ok("var foo = \"bar\";".into()));
}

#[test]
fn convert_var_str_to_var() {
    assert_eq!(output_program("var foo = \"bar\";"), Ok("var foo = \"bar\";".into()));
}

#[test]
fn convert_const_number_to_var() {
    assert_eq!(output_program("const foo = 435;"), Ok("var foo = 435;".into()));
}

#[test]
fn convert_const_true_to_var() {
    assert_eq!(output_program("const foo = true;"), Ok("var foo = true;".into()));
}

#[test]
fn convert_const_false_to_var() {
    assert_eq!(output_program("const foo = false;"), Ok("var foo = false;".into()));
}

#[test]
fn convert_const_null_to_var() {
    assert_eq!(output_program("const foo = null;"), Ok("var foo = null;".into()));
}

#[test]
fn convert_multiple_const_declarations() {
    let program = "const binary = 42, octal = 42, hexal = 42;";
    let expected = "var binary = 42, octal = 42, hexal = 42;";
    assert_eq!(output_program(program), Ok(expected.into()));
}
